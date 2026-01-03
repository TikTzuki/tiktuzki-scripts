#![allow(missing_docs)]

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "installer")]
#[command(about = "Tik's script installer")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands
#[derive(Subcommand)]
enum Commands {
    /// Render a template into a target Cargo.toml file
    Install {
        /// Name of the template to apply (e.g., "workspace", "axum", "sqlx_postgres")
        #[arg(short, long)]
        tool: String,

        #[arg(short, long, default_value = "latest")]
        version: String,
    },
}

const BANNER: &str = include_str!("../banner.txt");

const MAP_ENV_SERVICE: &str = "https://github.com/TikTzuki/tiktuzuki-scripts";

pub fn install_tool(tool: String, version: String) -> anyhow::Result<()> {
    println!("{}", BANNER);
    println!("Now attempting installation...\n");
    // Local variables
    let tool_dir_key = format!("{}_DIR", tool.to_uppercase().replace("-", "_"));
    let home_dir = env::var("HOME").expect("HOME not set");
    let tool_dir = env::var(&tool_dir_key).unwrap_or(format!("{}/{}", home_dir, tool));

    let map_env_bin_dir = format!("{}/bin", tool_dir);
    let tool_bash_profile = format!("{}/.bash_profile", home_dir);
    let tool_bashrc = format!("{}/.bashrc", home_dir);
    let tool_zshrc = format!("{}/.zshrc", env::var("ZDOTDIR").unwrap_or(home_dir.clone()));

    let map_env_init_snippet = format!(
        r#"# {tool}
export {tool_dir_key}="{tool_dir}"
export PATH="${tool_dir_key}/bin:$PATH""#,
    );

    // Check for existing installation
    println!("Looking for a previous installation of map_env...");
    if PathBuf::from(&tool_dir).exists() {
        println!("map_env found at {}", tool_dir);
        println!(
            "Please remove existing installation first: rm -rf {}",
            tool_dir
        );
        return Ok(());
    }

    // Create directories
    println!("Creating directories...");
    fs::create_dir_all(&map_env_bin_dir).context("Failed to create bin directory")?;

    // Detect platform
    println!("Prime platform file...");
    let platform = infer_platform();
    let binary_url = format!("{MAP_ENV_SERVICE}/releases/{version}/download/{tool}-{platform}");

    println!("Downloading from {}", binary_url);
    let tool_file_path = format!("{map_env_bin_dir}/{tool}");

    // Download binary
    let status = Command::new("curl")
        .args(["-L", &binary_url, "-o", &tool_file_path])
        .status()
        .context("Failed to download binary")?;
    if !status.success() {
        bail!("Failed to download binary from {}", binary_url);
    }

    println!("{tool} installed at {tool_file_path}");

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tool_file_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tool_file_path, perms)?;
    }

    // Update shell profiles
    let is_darwin = cfg!(target_os = "macos") || env::consts::OS == "macos";

    if is_darwin {
        println!("Attempt to update of login bash profile on OSX...");
        update_shell_profile(&tool_bash_profile, &map_env_init_snippet)?;
    } else {
        println!("Attempt update of interactive bash profile on regular UNIX...");
        update_shell_profile(&tool_bashrc, &map_env_init_snippet)?;
    }

    println!("Attempt update of zsh profile...");
    update_shell_profile(&tool_zshrc, &map_env_init_snippet)?;

    println!(
        r#"
    Installation complete!
    Please restart your terminal, or run:
    source ~/.bashrc  # or ~/.zshrc
    Then try: 
    ${tool} --help
    Enjoy!!!
    "#
    );

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Install { tool, version } => {
            // Check dependencies
            println!("Checking dependencies...");
            for cmd in ["curl", "tar"] {
                if !is_cli_exists(cmd) {
                    bail!("Error: {} is required but not installed.", cmd);
                }
            }
            install_tool(tool, version)?;
        }
    }

    Ok(())
}

fn is_cli_exists(cmd: &str) -> bool {
    Command::new("command")
        .args(["-v", cmd])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn infer_platform() -> &'static str {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match (os, arch) {
        ("linux", "x86") => "linuxx32",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "arm") => "linuxarm32hf",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("windows", "x86_64") => "windowsx64",
        _ => "exotic",
    }
}

fn update_shell_profile(profile_path: &str, snippet: &str) -> Result<()> {
    // Create file if it doesn't exist
    if !PathBuf::from(profile_path).exists() {
        fs::File::create(profile_path)?;
    }

    // Check if already configured
    let content = fs::read_to_string(profile_path)?;
    if content.contains("MAP_ENV_DIR") {
        return Ok(());
    }

    // Append snippet
    let mut file = fs::OpenOptions::new().append(true).open(profile_path)?;

    writeln!(file, "\n{}", snippet)?;
    println!("Added map_env to {}", profile_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        install_tool("map_env".to_string(), "v0.1.0".to_string())?;
        Ok(())
    }
}
