//! # Cargo Template CLI
//!
//! A command-line tool for applying predefined templates to Cargo projects.
//! This tool helps standardize workspace and package configurations by rendering
//! template configurations into existing Cargo.toml files.
//!
//! ## Features
//!
//! - Apply workspace configuration templates
//! - Update package metadata and dependencies
//! - Support for both workspace and package Cargo.toml files
//! - Automatic detection of Cargo project type
//!
//! ## Usage
//!
//! ```bash
//! cargo-template render-template <template-name> --target <path>
//! ```

mod cargo_manager;
mod template;

use crate::template::render_template;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Command-line interface for cargo-template
#[derive(Parser)]
#[command(name = "cargo-template")]
#[command(about = "Apply templates to Cargo.toml files")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands
#[derive(Subcommand)]
enum Commands {
    /// Render a template into a target Cargo.toml file
    RenderTemplate {
        /// Name of the template to apply (e.g., "workspace", "axum", "sqlx_postgres")
        template: String,

        /// Target directory containing Cargo.toml (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        target: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::RenderTemplate { template, target } => {
            render_template(&template, &target)?;
            println!(
                "✓ Successfully rendered template '{}' to {}",
                template,
                target.display()
            );
        }
    }

    Ok(())
}
