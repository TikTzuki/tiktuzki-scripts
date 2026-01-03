//! Package Cargo.toml management.
//!
//! This module provides functionality for managing package-level Cargo.toml files
//! within a workspace.

use crate::cargo_manager::CargoManager;
use crate::cargo_manager::utils::*;
use anyhow::Context;
use std::path::PathBuf;
use toml_edit::{DocumentMut, InlineTable, Item, Key, Table};

/// Manager for package-level Cargo.toml files.
///
/// This struct handles Cargo.toml files that contain a `[package]` section,
/// typically found in workspace members. It configures packages to inherit
/// settings from the workspace using `workspace = true`.
pub struct PackageCargo {
    /// Path to the Cargo.toml file
    pub path: PathBuf,

    /// Parsed TOML document
    pub doc: DocumentMut,
}

impl PackageCargo {
    /// Creates a new PackageCargo instance.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Cargo.toml file
    /// * `doc` - Parsed TOML document
    pub fn new(path: PathBuf, doc: DocumentMut) -> Self {
        Self { path, doc }
    }

    /// Creates an inline table with `workspace = true`.
    ///
    /// This is used to configure package fields to inherit from the workspace.
    fn depend_on_workspace() -> Item {
        let mut table = InlineTable::new();
        table.set_dotted(true);
        table.insert("workspace", true.into());
        toml_edit::value(table)
    }
}

impl CargoManager for PackageCargo {
    fn format_workspace_package(&mut self, template: &Table) -> anyhow::Result<()> {
        let package = self.doc["package"]
            .as_table_mut()
            .context("package section not found")?;

        // Insert package-specific fields
        insert_if_missing(package, template, "name", || "".into());
        insert_if_missing(package, template, "description", || "".into());

        // Configure package to inherit workspace metadata
        package.insert("version", Self::depend_on_workspace());
        package.insert("edition", Self::depend_on_workspace());
        package.insert("rust-version", Self::depend_on_workspace());
        package.insert("authors", Self::depend_on_workspace());
        package.insert("license", Self::depend_on_workspace());
        package.insert("homepage", Self::depend_on_workspace());
        package.insert("repository", Self::depend_on_workspace());
        package.insert("exclude", Self::depend_on_workspace());

        Ok(())
    }

    fn get_dependency(&self, name: &Key) -> anyhow::Result<&Item> {
        let deps = self.doc["dependencies"]
            .as_table()
            .context("dependencies section not found")?;

        deps.get(name)
            .context(format!("Dependency '{}' not found", name))
    }

    fn remove_dependency(&mut self, name: &Key) -> anyhow::Result<()> {
        let deps = self.doc["dependencies"]
            .as_table_mut()
            .context("dependencies section not found")?;

        deps.remove(name);
        Ok(())
    }

    fn update_dependency(&mut self, key: &Key, spec: Option<&Item>) -> anyhow::Result<()> {
        let deps = self.doc["dependencies"]
            .as_table_mut()
            .context("dependencies section not found")?;

        let mut value = spec.map_or_else(
            || InlineTable::new(),
            |s| {
                InlineTable::from(
                    s.as_inline_table()
                        .expect("spec is not an inline table")
                        .clone(),
                )
            },
        );

        // Remove version and set workspace = true for workspace dependencies
        value.remove("version");
        value.insert("workspace", true.into());

        deps.insert_formatted(key, toml_edit::value(value));
        Ok(())
    }

    fn file(&self) -> &PathBuf {
        &self.path
    }

    fn doc(&self) -> &DocumentMut {
        &self.doc
    }
}
