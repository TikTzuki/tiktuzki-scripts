#![allow(missing_docs)]
use anyhow::{Context, Result};
use aws_config::{BehaviorVersion, Region, defaults};
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::{ByteStream, SdkBody};
use chrono::Utc;
use clap::Parser;
use devopsk::process_reader::ProgressBody;
use log::{error, info};
use std::env::set_var;
use std::fs::create_dir_all;
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::{process::Command, sync::watch, time};

const COMPRESS_SUFFIX: &'static str = ".zst";

#[derive(Parser, Debug)]
#[command(about = "Manage tigerbeele: run, stream logs, stop for backup and restart")]
struct Args {
    #[arg(long, default_value = "tigerbeetle")]
    exe: String,

    #[arg(
        long,
        short,
        default_value_t = 0,
        help = "Backup interval in seconds (0 to disable)"
    )]
    interval_secs: u64,

    #[arg(
        long,
        short = 'f',
        default_value = "./backups/0_0.tigerbeetle",
        help = "Path to tigerbeetle data file to backup"
    )]
    backup_file: PathBuf,

    #[arg(
        long,
        short = 'b',
        default_value = "backup",
        help = "S3 bucket to upload backups to"
    )]
    bucket: String,

    #[arg(last = true)]
    child_args: Vec<String>,
}

pub async fn compress_file(src: &PathBuf) -> Result<PathBuf> {
    let filename = src
        .file_name()
        .context("failed to get file name")?
        .to_str()
        .context("invalid backup file path")?;
    let dst_path = PathBuf::from(format!("/tmp/{}{}", filename, COMPRESS_SUFFIX));
    let mut encoder = {
        let target = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&dst_path)
            .context("create temp file for compression")?;
        zstd::Encoder::new(target, 1)?
    };
    let mut src = std::fs::File::open(&src).context("open src file")?;
    std::io::copy(&mut src, &mut encoder)?;
    encoder.finish()?;
    info!("File compressed to {:?}", dst_path);
    Ok(PathBuf::from(dst_path))
}

pub trait BackupStrategy {
    async fn upload_backup(&self, bucket: &str, path: &PathBuf) -> Result<()>;
}

impl BackupStrategy for Client {
    async fn upload_backup(&self, bucket: &str, path: &PathBuf) -> Result<()> {
        let meta = tokio::fs::metadata(path).await?;
        info!("Starting backup: {:?}, {} bytes", &path, meta.len());

        let compressed_path = compress_file(path).await?;
        let body = ByteStream::read_from()
            .path(&compressed_path)
            .buffer_size((meta.len() / 100) as usize)
            .build()
            .await?;
        let compressed_file = compressed_path
            .file_name()
            .expect("invalid backup file path")
            .to_str()
            .expect("invalid backup file path");

        let request = self
            .put_object()
            .bucket(bucket)
            .key(format!(
                "backup/{}/{}",
                Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
                compressed_file
            ))
            .body(body);
        let meta = tokio::fs::metadata(&compressed_path).await?;
        info!(
            "Uploading compressed backup: {:?}, {} bytes",
            &compressed_path,
            meta.len()
        );
        let customized = request
            .customize()
            .map_request(ProgressBody::<SdkBody>::replace);
        let out = customized.send().await?;
        info!("Backup upload completed: {:?}", out);
        Ok(())
    }
}

pub async fn start_main(
    exe: String,
    child_args: Vec<String>,
    shutdown_rx: watch::Receiver<bool>,
) -> Result<Child> {
    info!("Spawning child: {} {:?}", exe, child_args);
    let mut main_process = Command::new(&exe)
        .args(&child_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to start '{}'", exe))?;

    // attach stdout/stderr readers
    let stdout = main_process.stdout.take().expect("child stdout missing");
    let mut lines = BufReader::new(stdout).lines();
    let shutdown = shutdown_rx.clone();
    tokio::spawn(async move {
        loop {
            if *shutdown.borrow() {
                break;
            }

            match lines.next_line().await {
                Ok(Some(line)) => info!("{}", line),
                Ok(None) => break,
                Err(e) => {
                    info!("error reading child stdout: {}", e);
                    break;
                }
            }
        }
    });

    let stderr = main_process.stderr.take().expect("child stderr missing");
    let mut lines = BufReader::new(stderr).lines();
    let shutdown = shutdown_rx.clone();
    tokio::spawn(async move {
        loop {
            if *shutdown.borrow() {
                break;
            }

            match lines.next_line().await {
                Ok(Some(line)) => info!("{}", line),
                Ok(None) => break,
                Err(e) => {
                    error!("error reading child stderr: {}", e);
                    break;
                }
            }
        }
    });
    Ok(main_process)
}

//  cargo run --bin=backup_tigerbee -- --interval-secs=0 -b=tigerbeetle-backup -f=/Users/tiktuzki/Desktop/repos/personal/tik_scripts/crates/devopsk/src/bin/0.tigerbeetle -- start --addresses=0.0.0.0:3000 /Users/tiktuzki/Desktop/repos/personal/tik_scripts/crates/devopsk/src/bin/0.tigerbeetle
#[tokio::main]
async fn main() -> Result<()> {
    unsafe {
        set_var("RUST_LOG", "debug");
    }
    let _ = dyson_log::init_log();
    let args = Args::parse();

    info!(
        "Starting manager for '{}' (interval {}s, backups -> {:?})",
        args.exe, args.interval_secs, args.backup_file
    );

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        shutdown_tx.send(true).expect("shutdown notify failed");
    });

    // create backup interval if enabled
    if args.interval_secs > 0 {
        // validate backup file
        if !&args.backup_file.is_file() {
            return Err(anyhow::anyhow!(
                "Backup file {:?} does not exist or is not a file",
                args.backup_file
            ));
        }
        // let region = std::env::var("AWS_DEFAULT_REGION").unwrap_or("ap-southeast-1".to_string());
        let bucket = args.bucket.as_str();
        let config = defaults(BehaviorVersion::v2025_08_07())
            // .region(Region::new(region.to_owned()))
            .load()
            .await;
        let client = Client::new(&config);

        info!("Start backup loop");
        let mut interval = time::interval(Duration::from_secs(args.interval_secs));
        loop {
            let mut child = start_main(
                args.exe.clone(),
                args.child_args.clone(),
                shutdown_rx.clone(),
            )
            .await?;
            tokio::select! {
                _ = interval.tick() => {
                    let _ = kill_and_wait(&mut child).await;
                    if let Err(e) = client.upload_backup(bucket, &args.backup_file).await {
                        error!("backup failed: {:#}", e);
                    } else {
                        info!("backup completed");
                    }
                }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!("Shutdown requested: killing child and exiting");
                        break;
                    }
                }
                _ = child.wait() => {
                    info!("Child process stopped");
                    break;
                }
            }
        }
    } else {
        let mut child = start_main(
            args.exe.clone(),
            args.child_args.clone(),
            shutdown_rx.clone(),
        )
        .await?;
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!("Shutdown requested: killing child and exiting");
                        break;
                    }
                }
                _ = child.wait() => {
                    info!("Child process stopped");
                    break;
                }
            }
        }
    }
    info!("Manager process exiting");
    Ok(())
}

async fn kill_and_wait(child: &mut Child) -> Result<()> {
    // Try to kill the child; if already exited, ignore errors
    if let Some(id) = child.id() {
        info!("Killing child pid {}", id);
    }
    match child.kill().await {
        Ok(()) => {
            let _ = child.wait().await;
        }
        Err(e) => {
            // if process already dead, kill may error; attempt to wait anyway
            error!("kill() error (maybe already exited): {}", e);
            let _ = child.wait().await;
        }
    }
    Ok(())
}
