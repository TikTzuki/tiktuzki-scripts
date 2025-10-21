use anyhow::anyhow;
use env_file_reader::read_file;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub trait EnvMapper {
    fn map_env(
        &self,
        source_code: String,
        pattern_file: String,
        suffixes: Vec<String>,
        worker: u8,
    ) -> anyhow::Result<()>;
}

pub struct VITEnvMapper {}

impl EnvMapper for VITEnvMapper {
    fn map_env(
        &self,
        source_code: String,
        pattern_file: String,
        suffixes: Vec<String>,
        worker: u8,
    ) -> anyhow::Result<()> {
        println!("Processing {}", pattern_file);
        if !Path::new(&pattern_file).exists() {
            return Err(anyhow!("Pattern file '{}' not found", pattern_file));
        }

        let replacements: HashMap<String, String> = read_file(pattern_file)?;
        println!("ENVS: {:?}", serde_json::to_string_pretty(&replacements)?);

        let source_dir = Path::new(&source_code);
        let mut targets = Vec::new();
        for (ex, entry) in walkdir::WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                for suffix in &suffixes {
                    if let Some(ex) = e.path().extension() {
                        if suffix.as_str().eq(ex) {
                            return Some((suffix, e));
                        }
                    }
                }
                None
            })
        {
            println!("Processing {}", entry.path().display());
            targets.push((ex, entry.path().to_path_buf()));
        }
        println!("Targets files {:?}", targets);

        for (ex, file_path) in targets {
            if !file_path.is_file() {
                continue;
            }
            let original = fs::read_to_string(&file_path)?;
            let mut updated = original.clone();

            // apply replacements; more specific: escaped form first (\${VAR})
            match ex.as_str() {
                "html" => {
                    for (env_key, env_val) in &replacements {
                        let pattern = format!("${{{}}}", env_key); // matches: ${VAR}
                        // replace escaped form first
                        updated = updated.replace(&pattern, env_val);
                    }
                }
                "js" => {
                    for (env_key, env_val) in &replacements {
                        let pattern = format!("${{{}}}", env_key); // matches: ${VAR}
                        // replace escaped form first
                        updated = updated.replace(&pattern, env_val);
                    }
                }
                _ => {
                    eprintln!("Unsupported {}", original);
                }
            };

            if updated != original {
                fs::write(&file_path, updated)?;
                println!("Updated {}", file_path.display());
            }
        }

        Ok(())
    }
}
