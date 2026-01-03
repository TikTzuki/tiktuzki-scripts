#![allow(missing_docs)]

mod cargo_manager;
mod template;

use crate::cargo_manager::CargoManager;
use crate::template::render_template;
use anyhow::{Context, Result};
use clap::builder::TypedValueParser;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cargo-deps")]
#[command(about = "Modify workspace dependencies in Cargo.toml")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    RenderTemplate {
        template: String,
        #[arg(short, long, default_value = ".")]
        target: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::RenderTemplate { template, target } => {
            render_template(&template, &target)?;
            println!("Successfully rendered {} {}", template, target.display());
        }
    }

    Ok(())
}
