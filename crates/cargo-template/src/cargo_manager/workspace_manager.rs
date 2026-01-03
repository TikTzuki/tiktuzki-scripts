use crate::cargo_manager::CargoManager;
use crate::cargo_manager::utils::*;
use anyhow::Context;
use std::path::PathBuf;
use toml_edit::{Array, DocumentMut, InlineTable, Item, Key, Table};

pub struct WorkspaceCargo {
    pub path: PathBuf,
    pub doc: DocumentMut,
}

impl WorkspaceCargo {
    pub fn new(path: PathBuf, doc: DocumentMut) -> Self {
        Self { path, doc }
    }
}

impl CargoManager for WorkspaceCargo {
    fn format_workspace_package(&mut self, template: &Table) -> anyhow::Result<()> {
        let wp = self.doc["workspace"]["package"]
            .as_table_mut()
            .context("workspace.package section not found")?;

        insert_if_missing(wp, template, "description", || "".into());

        insert_if_template_not_empty(wp, template, "version", || "0.0.1".into());
        insert_if_template_not_empty(wp, template, "edition", || "2024".into());
        insert_if_template_not_empty(wp, template, "rust-version", || "1.90".into());
        insert_if_missing(wp, template, "authors", || Array::default().into());
        insert_if_missing(wp, template, "license", || "".into());
        insert_if_missing(wp, template, "homepage", || "".into());
        wp.insert("repository", get_git_remote_url()?.into());
        insert_if_template_not_empty(wp, template, "exclude", || Array::default().into());
        Ok(())
    }

    fn get_dependency(&self, name: &Key) -> anyhow::Result<&Item> {
        let deps = self.doc["workspace"]["dependencies"]
            .as_table()
            .context("dependencies section not found")?;

        let value = deps
            .get(name)
            .context(format!("Dependency '{}' not found", name))?;
        Ok(value)
    }

    fn remove_dependency(&mut self, name: &Key) -> anyhow::Result<()> {
        let deps = self.doc["workspace"]["dependencies"]
            .as_table_mut()
            .context("workspace.dependencies not found")?;

        deps.remove(name);
        Ok(())
    }

    fn update_dependency(&mut self, key: &Key, spec: Option<&Item>) -> anyhow::Result<()> {
        let deps = self.doc["workspace"]["dependencies"]
            .as_table_mut()
            .context("workspace.dependencies not found")?;
        let value = spec.map_or(InlineTable::new(), |s| {
            InlineTable::from(
                s.as_inline_table()
                    .expect("spec is not an inline table")
                    .clone(),
            )
        });
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
