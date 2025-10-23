use anyhow::{Context, anyhow};
use async_trait::async_trait;
use dotenvy::dotenv;
use env_file_reader::read_file;
use fs_extra::dir::{CopyOptions, copy};
use futures::stream::StreamExt;
use futures::{stream};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

#[async_trait]
pub trait EnvMapper {
    async fn map_env(
        &self,
        source_code: String,
        pattern_file: String,
        dynamic_file: Option<String>,
        output_dir: Option<String>,
        suffixes: Vec<String>,
        worker: u8,
    ) -> anyhow::Result<()>;
}

pub struct VITEnvMapper {}

impl VITEnvMapper {
    pub fn copy_to_output(source: String, dest: String) -> anyhow::Result<()> {
        let mut options = CopyOptions::new();
        options.overwrite = true; // overwrite existing files
        options.copy_inside = true; // copy contents inside source_dir

        copy(&source, &dest, &options)
            .context(format!("copy from {} to {} error", &source, &dest))?;
        Ok(())
    }

    async fn process(
        file_path: PathBuf,
        replacements: &HashMap<String, String>,
        source_dir: &Path,
        output_dir: Option<String>,
    ) -> anyhow::Result<()> {
        if !file_path.is_file() {
            return Ok(());
        }
        let original = tokio::fs::read_to_string(&file_path).await?;
        // let original = fs::read_to_string(&file_path)?;
        let mut updated = original.clone();

        // apply replacements; more specific: escaped form first (\${VAR})
        for (env_key, env_val) in replacements {
            let pattern = format!("${{{}}}", env_key); // matches: ${VAR}
            // replace escaped form first
            updated = updated.replace(&pattern, env_val);
        }

        if let Some(output_dir) = &output_dir {
            let relative_path = file_path
                .strip_prefix(source_dir)
                .context("Failed to get relative path")?;
            let output_path = Path::new(output_dir).join(relative_path);
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&output_path, updated.clone())?;
            println!("Written to {}", output_path.display());
        } else {
            if updated != original {
                fs::write(&file_path, updated)?;
                println!("Updated {}", file_path.display());
            }
        }
        Ok(())
    }
}

#[async_trait]
impl EnvMapper for VITEnvMapper {
    async fn map_env(
        &self,
        source_code: String,
        pattern_file: String,
        dynamic_file: Option<String>,
        output_dir: Option<String>,
        suffixes: Vec<String>,
        worker: u8,
    ) -> anyhow::Result<()> {
        let start = std::time::Instant::now();
        println!("Processing {}", pattern_file);
        // Load .env file from the current directory
        dotenv().ok();
        if !Path::new(&pattern_file).exists() {
            return Err(anyhow!("Pattern file '{}' not found", pattern_file));
        }

        let mut replacements: HashMap<String, String> =
            read_file(&pattern_file).context(format!("Failed to read '{}'", &pattern_file))?;
        let mut full_envs: HashMap<String, String> = env::vars().collect();
        if let Some(dynamic_file) = dynamic_file {
            let override_env =
                read_file(&dynamic_file).context(format!("Failed to read '{}'", &dynamic_file))?;
            full_envs.extend(override_env);
        }
        for (key, val) in replacements.iter_mut() {
            if let Some(env_val) = full_envs.get(key) {
                *val = env_val.to_string();
            }
        }

        println!("ENVS: {:?}", serde_json::to_string_pretty(&replacements)?);

        let source_dir = Path::new(&source_code);
        let mut targets = Vec::new();
        for entry in walkdir::WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                for suffix in &suffixes {
                    if let Some(ex) = e.path().extension() {
                        if suffix.as_str().eq(ex) {
                            return Some(e);
                        }
                    }
                }
                None
            })
        {
            println!("Processing {}", entry.path().display());
            targets.push(entry.path().to_path_buf());
        }
        println!("Targets files: {:?}", targets.len());

        if let Some(output_dir) = &output_dir {
            VITEnvMapper::copy_to_output(source_code.clone(), output_dir.clone())
                .context("Failed to copy source code to output dir")?;
        }

        stream::iter(targets)
            .map(|file_path| async {
                if let Err(e) =
                    Self::process(file_path, &replacements, &source_dir, output_dir.clone()).await
                {
                    eprintln!("Error processing file: {}", e);
                    return Err(e);
                }
                Ok(())
            })
            .buffer_unordered(worker as usize)
            .collect::<Vec<_>>()
            .await;
        println!("map_env execution time: {:?}", start.elapsed());
        Ok(())
    }
}
