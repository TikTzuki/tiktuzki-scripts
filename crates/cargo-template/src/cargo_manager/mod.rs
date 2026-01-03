//! Cargo.toml management module.
//!
//! This module provides abstractions for reading and modifying Cargo.toml files,
//! supporting both workspace and package configurations.

mod package_manager;
mod utils;
mod workspace_manager;

use crate::cargo_manager::package_manager::PackageCargo;
use crate::cargo_manager::workspace_manager::WorkspaceCargo;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use toml_edit::{DocumentMut, Item, Key, Table};

/// Represents the type of Cargo.toml file
pub enum CargoType {
    /// A workspace-only Cargo.toml (contains `[workspace]` section)
    Workspace(WorkspaceCargo),

    /// A package-only Cargo.toml (contains `[package]` section)
    Package(PackageCargo),

    /// A Cargo.toml with both workspace and package sections (currently unsupported)
    Both,
}

/// Detects the type of Cargo.toml at the given path.
///
/// # Arguments
///
/// * `target` - Path to the directory containing Cargo.toml
///
/// # Returns
///
/// Returns the detected `CargoType` or an error if:
/// - Cargo.toml doesn't exist at the path
/// - The file is invalid TOML
/// - Neither workspace nor package section exists
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// let cargo_type = detect_cargo_type(&PathBuf::from("."))?;
/// # Ok(())
/// # }
/// ```
pub fn detect_cargo_type(target: &PathBuf) -> Result<CargoType> {
    let path = target.join("Cargo.toml");
    let doc = fs::read_to_string(&path)
        .context(format!("Failed to read Cargo.toml at {:?}", path))?
        .parse::<DocumentMut>()
        .context("Failed to parse Cargo.toml")?;

    let has_workspace = doc.get("workspace").is_some();
    let has_package = doc.get("package").is_some();

    match (has_workspace, has_package) {
        (true, true) => Ok(CargoType::Both),
        (true, false) => Ok(CargoType::Workspace(WorkspaceCargo::new(path, doc))),
        (false, true) => Ok(CargoType::Package(PackageCargo::new(path, doc))),
        (false, false) => anyhow::bail!("Invalid Cargo.toml: no workspace or package section"),
    }
}

/// Trait for managing Cargo.toml files.
///
/// This trait provides a common interface for reading and modifying both
/// workspace and package Cargo.toml files.
pub trait CargoManager {
    /// Formats the workspace package metadata section.
    ///
    /// This method updates fields like version, authors, license, etc.
    /// based on the provided template table.
    fn format_workspace_package(&mut self, table: &Table) -> Result<()>;

    /// Retrieves a dependency by name.
    fn get_dependency(&self, name: &Key) -> Result<&Item>;

    /// Removes a dependency by name.
    fn remove_dependency(&mut self, name: &Key) -> Result<()>;

    /// Updates or adds a dependency.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name
    /// * `spec` - The dependency specification (version, features, etc.)
    fn update_dependency(&mut self, name: &Key, spec: Option<&Item>) -> Result<()>;

    /// Returns the path to the Cargo.toml file.
    fn file(&self) -> &PathBuf;

    /// Returns the parsed TOML document.
    fn doc(&self) -> &DocumentMut;

    /// Writes the modified document back to the file.
    ///
    /// This method is provided with a default implementation that writes
    /// the document to the path returned by `file()`.
    fn commit(&self) -> Result<()> {
        fs::write(self.file(), self.doc().to_string()).context("Failed to write Cargo.toml")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
