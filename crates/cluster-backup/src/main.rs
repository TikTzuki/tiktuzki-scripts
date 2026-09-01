//! CLI for shipping a backup run to S3.

use anyhow::{Context, Result, bail};
use chrono::{Datelike, Local};
use clap::{Args, Parser, Subcommand};
use cluster_backup::archive::{self, DEFAULT_LEVEL};
use cluster_backup::crypt::Recipient;
use cluster_backup::plan::{Half, Plan, is_complete};
use cluster_backup::s3::{Uploader, parse_storage_class};
use log::{info, warn};
use std::path::{Path, PathBuf};

/// Archive, encrypt and upload a microk8s backup run.
#[derive(Parser, Debug)]
#[command(name = "cluster-backup", about, version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Show what would be uploaded, without touching S3.
    Plan(RunArgs),
    /// Archive, encrypt and upload a run.
    Upload(UploadArgs),
}

/// How to locate the run directory to work on.
#[derive(Args, Debug, Clone)]
struct RunArgs {
    /// Directory holding timestamped runs.
    #[arg(long, default_value = "/srv/k8s-volumes/backups")]
    backup_dir: PathBuf,

    /// A specific run directory. Defaults to the newest under --backup-dir.
    #[arg(long)]
    run: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct UploadArgs {
    #[command(flatten)]
    run: RunArgs,

    /// Destination bucket.
    #[arg(long, env = "S3_BUCKET")]
    bucket: String,

    /// Key prefix within the bucket.
    #[arg(long, env = "S3_PREFIX", default_value = "node1")]
    prefix: String,

    /// AWS region. Falls back to the ambient AWS configuration.
    #[arg(long, env = "AWS_REGION")]
    region: Option<String>,

    /// age public key ("age1…") to encrypt to. The private half must NOT live on this host.
    #[arg(long, env = "AGE_RECIPIENT")]
    recipient: String,

    /// Storage class for the small, restore-blocking half.
    #[arg(long, env = "CRITICAL_CLASS", default_value = "STANDARD_IA")]
    critical_class: String,

    /// Storage class for database dumps and volume tarballs.
    #[arg(long, env = "BULK_CLASS", default_value = "DEEP_ARCHIVE")]
    bulk_class: String,

    /// ISO weekday to upload the bulk half on (1=Mon … 7=Sun, 0=never).
    ///
    /// Deep Archive bills a 180-day minimum per object regardless of when you delete it,
    /// so uploading gigabytes daily bills roughly 180 copies at steady state.
    #[arg(long, env = "UPLOAD_BULK_DOW", default_value_t = 7)]
    bulk_dow: u32,

    /// Upload the bulk half even if today is not its scheduled day.
    #[arg(long, env = "FORCE_BULK")]
    force_bulk: bool,

    /// zstd compression level.
    #[arg(long, default_value_t = DEFAULT_LEVEL)]
    level: i32,
}

/// Newest `20*-*` directory under `dir`.
///
/// Run names are `YYYYmmdd-HHMMSS`, so lexical order is chronological order.
fn newest_run(dir: &Path) -> Result<PathBuf> {
    let mut runs: Vec<PathBuf> = std::fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_dir()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("20") && n.contains('-'))
        })
        .collect();
    runs.sort();
    runs.pop()
        .with_context(|| format!("no backup runs found under {}", dir.display()))
}

fn resolve_run(args: &RunArgs) -> Result<PathBuf> {
    let run = match &args.run {
        Some(r) => r.clone(),
        None => newest_run(&args.backup_dir)?,
    };
    if !run.is_dir() {
        bail!("not a directory: {}", run.display());
    }
    // MANIFEST.txt is written last by the collection step. Without it the run is the
    // wreckage of a failed collection, and shipping it would create something that looks
    // like a backup and is not one.
    if !is_complete(&run) {
        bail!(
            "{} has no MANIFEST.txt — incomplete run, refusing to upload",
            run.display()
        );
    }
    Ok(run)
}

fn stamp_of(run: &Path) -> Result<String> {
    run.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .with_context(|| format!("cannot derive a run name from {}", run.display()))
}

fn should_upload_bulk(args: &UploadArgs) -> bool {
    if args.force_bulk {
        return true;
    }
    if args.bulk_dow == 0 {
        return false;
    }
    Local::now().weekday().number_from_monday() == args.bulk_dow
}

async fn cmd_plan(args: &RunArgs) -> Result<()> {
    let run = resolve_run(args)?;
    println!("run: {}", run.display());
    for half in [Half::Critical, Half::Bulk] {
        let plan = Plan::resolve(&run, half);
        if plan.is_empty() {
            println!("  {:<9} (nothing present)", half.as_str());
        } else {
            println!("  {:<9} {}", half.as_str(), plan.members.join(", "));
        }
    }
    Ok(())
}

async fn cmd_upload(args: &UploadArgs) -> Result<()> {
    let run = resolve_run(&args.run)?;
    let stamp = stamp_of(&run)?;

    // Validate everything cheap before spending CPU on gigabytes of compression.
    let recipient = Recipient::parse(&args.recipient)?;
    let critical_class = parse_storage_class(&args.critical_class)?;
    let bulk_class = parse_storage_class(&args.bulk_class)?;
    if args.bulk_dow > 7 {
        bail!("--bulk-dow must be 0-7, got {}", args.bulk_dow);
    }

    let uploader = Uploader::new(args.bucket.clone(), args.region.clone()).await?;

    // Stage archives beside the runs: a bulk archive can be many gigabytes, and the root
    // filesystem is both smaller and shared with the OS and container images.
    let staging = tempfile::Builder::new()
        .prefix(".upload-")
        .tempdir_in(&args.run.backup_dir)
        .with_context(|| format!("creating staging dir in {}", args.run.backup_dir.display()))?;

    let mut halves = vec![(Half::Critical, critical_class)];
    if should_upload_bulk(args) {
        halves.push((Half::Bulk, bulk_class));
    } else {
        info!(
            "bulk: skipped — uploads on ISO weekday {}, today is {} (--force-bulk to override)",
            args.bulk_dow,
            Local::now().weekday().number_from_monday()
        );
    }

    for (half, class) in halves {
        let plan = Plan::resolve(&run, half);
        if plan.is_empty() {
            warn!("{}: nothing present in this run, skipping", half.as_str());
            continue;
        }

        let dest = staging
            .path()
            .join(format!("{stamp}-{}.tar.zst.age", half.as_str()));
        info!("{}: archiving {}", half.as_str(), plan.members.join(", "));
        let stats = archive::build(&run, &plan, &recipient, &dest, args.level)?;
        info!(
            "{}: {} entries -> {} bytes encrypted",
            half.as_str(),
            stats.members,
            stats.bytes
        );

        let key = format!("{}/{}/{stamp}.tar.zst.age", args.prefix, half.as_str());
        let stored = uploader.upload_verified(&key, &dest, &class).await?;
        info!(
            "{}: verified s3://{}/{key} — {} bytes, {}",
            half.as_str(),
            args.bucket,
            stored.bytes,
            stored.storage_class
        );

        // Free the staging copy as we go; both halves at once could be twice the run size.
        let _ = std::fs::remove_file(&dest);
    }

    info!(
        "done: s3://{}/{}/*/{stamp}.tar.zst.age",
        args.bucket, args.prefix
    );
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dyson_log::init_log();
    let cli = Cli::parse();
    match &cli.command {
        Command::Plan(args) => cmd_plan(args).await,
        Command::Upload(args) => cmd_upload(args).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn newest_run_picks_the_latest_and_ignores_other_dirs() {
        let d = tempfile::tempdir().unwrap();
        for name in ["20260825-010000", "20260901-093329", "20260830-235959"] {
            fs::create_dir(d.path().join(name)).unwrap();
        }
        fs::create_dir(d.path().join("lost+found")).unwrap();
        fs::create_dir(d.path().join(".upload-abc")).unwrap();

        let newest = newest_run(d.path()).unwrap();
        assert_eq!(newest.file_name().unwrap(), "20260901-093329");
    }

    #[test]
    fn newest_run_errors_on_an_empty_dir() {
        let d = tempfile::tempdir().unwrap();
        assert!(newest_run(d.path()).is_err());
    }

    #[test]
    fn incomplete_runs_are_refused() {
        let d = tempfile::tempdir().unwrap();
        let run = d.path().join("20260901-000000");
        fs::create_dir(&run).unwrap();
        let args = RunArgs {
            backup_dir: d.path().to_path_buf(),
            run: None,
        };
        let err = resolve_run(&args).unwrap_err().to_string();
        assert!(err.contains("MANIFEST.txt"), "unexpected error: {err}");

        fs::write(run.join("MANIFEST.txt"), b"ok").unwrap();
        assert!(resolve_run(&args).is_ok());
    }

    #[test]
    fn stamp_comes_from_the_directory_name() {
        assert_eq!(
            stamp_of(Path::new("/srv/k8s-volumes/backups/20260901-093329")).unwrap(),
            "20260901-093329"
        );
    }
}
