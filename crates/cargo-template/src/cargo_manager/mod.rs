mod package_manager;
mod utils;
mod workspace_manager;

use crate::cargo_manager::package_manager::PackageCargo;
use crate::cargo_manager::workspace_manager::WorkspaceCargo;
use anyhow::{Context, Result};
use clap::Parser;
use clap::builder::TypedValueParser;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use toml_edit::{DocumentMut, Item, Key, Table, TableLike};

pub enum CargoType {
    Workspace(WorkspaceCargo),
    Package(PackageCargo),
    Both, // A package that also defines a workspace
}

pub fn detect_cargo_type(target: &PathBuf) -> Result<CargoType> {
    let path = target.join("Cargo.toml");
    let doc = fs::read_to_string(&path)
        .context(format!("Failed to read Cargo.toml at {:?}", path))?
        .parse::<DocumentMut>()
        .context("Failed to parse Cargo.toml")?;

    let has_workspace = doc.get("workspace").is_some();
    let has_package = doc.get("package").is_some();

    match (has_workspace, has_package) {
        (true, true) => Ok((CargoType::Both)),
        (true, false) => Ok(CargoType::Workspace(WorkspaceCargo::new(path, doc))),
        (false, true) => Ok(CargoType::Package(PackageCargo::new(path, doc))),
        (false, false) => anyhow::bail!("Invalid Cargo.toml: no workspace or package section"),
    }
}

pub trait CargoManager {
    fn format_workspace_package(&mut self, table: &Table) -> Result<()>;
    fn get_dependency(&self, name: &Key) -> Result<&Item>;
    fn remove_dependency(&mut self, name: &Key) -> Result<()>;
    fn update_dependency(&mut self, name: &Key, spec: Option<&Item>) -> Result<()>;
    fn file(&self) -> &PathBuf;
    fn doc(&self) -> &DocumentMut;
    fn commit(&self) -> Result<()> {
        fs::write(self.file(), self.doc().to_string()).context("Failed to write Cargo.toml")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
