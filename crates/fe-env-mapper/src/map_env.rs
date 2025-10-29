use crate::filter_env::FilterConfig;
use crate::replace_env::Replacer;
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use dotenvy::dotenv;
use env_file_reader::read_file;
use fs_extra::dir::{CopyOptions, copy};
use futures::stream;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Placeholder {
    Underscore,  // `__KEY__` — double underscores surrounding the variable name
    DoubleCurly, // `{{KEY}}` — double curly braces
    DollarCurly, // `${KEY}` — dollar sign with single curly braces
    DollarBrace, // `${{KEY}}` — dollar sign with double curly braces (e.g. used by some templating styles)
}
impl Placeholder {
    pub fn regex_pattern(&self) -> &str {
        match self {
            Placeholder::Underscore => r"__([^_]+(?:_[^_]+)*)__",
            Placeholder::DoubleCurly => r"\{\{([^}]+)\}\}",
            Placeholder::DollarCurly => r"\$\{([^}]+)\}",
            Placeholder::DollarBrace => r"\$\{\{([^}]+)\}\}",
        }
    }

    pub fn format_template(&self, key: &str) -> String {
        match self {
            Placeholder::Underscore => format!("__{}__", key),
            Placeholder::DoubleCurly => format!("{{{{{}}}}}", key), // yields `{{KEY}}`
            Placeholder::DollarCurly => format!("${{{}}}", key),    // yields `${KEY}`
            Placeholder::DollarBrace => format!("${{{{{}}}}}", key), // yields `${{KEY}}`
        }
    }
}
impl From<u8> for Placeholder {
    fn from(value: u8) -> Self {
        match value {
            1 => Placeholder::Underscore,
            2 => Placeholder::DoubleCurly,
            3 => Placeholder::DollarCurly,
            4 => Placeholder::DollarBrace,
            _ => Placeholder::Underscore,
        }
    }
}

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
        placeholder: Placeholder,
        filter_config: FilterConfig,
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
        filter_config: FilterConfig,
        placeholder: Placeholder,
        source_dir: &Path,
        output_dir: Option<String>,
    ) -> anyhow::Result<()> {
        if !file_path.is_file() {
            return Ok(());
        }
        let original = tokio::fs::read_to_string(&file_path).await?;

        let replacer = Replacer::new()?;

        // Filter env values based on filter config
        let filtered_replacements: HashMap<String, String> = replacements
            .iter()
            .filter(|(key, _)| filter_config.should_process(key))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Replace placeholders
        let updated = replacer.replace_all(&original, &filtered_replacements, &placeholder)?;

        // Print stats if verbose or has changes
        if updated != original {
            println!("📝 Processing: {}", file_path.display());
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
        placeholder: Placeholder,
        filter_config: FilterConfig,
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

        if !filter_config.is_empty() {
            println!("Filter Configuration:");
            println!(
                "  Include rules: {} rules",
                filter_config.include_rules.len()
            );
            println!(
                "  Exclude rules: {} rules",
                filter_config.exclude_rules.len()
            );

            // Show which keys will be processed
            let filtered_keys: Vec<&String> = replacements
                .keys()
                .filter(|k| filter_config.should_process(k))
                .collect();
            println!("  Keys to process: {:?}", filtered_keys);

            let skipped_keys: Vec<&String> = replacements
                .keys()
                .filter(|k| !filter_config.should_process(k))
                .collect();
            if !skipped_keys.is_empty() {
                println!("  Keys to skip: {:?}", skipped_keys);
            }
        }

        stream::iter(targets)
            .map(|file_path| async {
                if let Err(e) = Self::process(
                    file_path,
                    &replacements,
                    filter_config.clone(),
                    placeholder.clone(),
                    &source_dir,
                    output_dir.clone(),
                )
                .await
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
