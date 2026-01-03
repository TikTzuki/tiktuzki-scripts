use crate::cargo_manager::CargoManager;
use crate::cargo_manager::utils::*;
use anyhow::Context;
use std::path::PathBuf;
use toml_edit::{DocumentMut, InlineTable, Item, Key, Table};
pub struct PackageCargo {
    pub path: PathBuf,
    pub doc: DocumentMut,
}

impl PackageCargo {
    pub fn new(path: PathBuf, doc: DocumentMut) -> Self {
        Self { path, doc }
    }

    fn depend_on_workspace() -> Item {
        let mut table = InlineTable::new();
        table.set_dotted(true);
        table.insert("workspace", true.into());
        toml_edit::value(table)
    }
}
impl CargoManager for PackageCargo {
    fn format_workspace_package(&mut self, template: &Table) -> anyhow::Result<()> {
        let wp = self.doc["package"]
            .as_table_mut()
            .context("workspace.package section not found")?;

        insert_if_missing(wp, template, "name", || "".into());
        insert_if_missing(wp, template, "description", || "".into());

        wp.insert("version", Self::depend_on_workspace());
        wp.insert("edition", Self::depend_on_workspace());
        wp.insert("rust-version", Self::depend_on_workspace());
        wp.insert("authors", Self::depend_on_workspace());
        wp.insert("license", Self::depend_on_workspace());
        wp.insert("homepage", Self::depend_on_workspace());
        wp.insert("repository", Self::depend_on_workspace());
        wp.insert("exclude", Self::depend_on_workspace());
        Ok(())
    }

    fn get_dependency(&self, name: &Key) -> anyhow::Result<&Item> {
        let deps = self.doc["dependencies"]
            .as_table()
            .context("dependencies section not found")?;

        let value = deps
            .get(name)
            .context(format!("Dependency '{}' not found", name))?;
        Ok(value)
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
        let mut value = spec.map_or(InlineTable::new(), |s| {
            InlineTable::from(
                s.as_inline_table()
                    .expect("spec is not an inline table")
                    .clone(),
            )
        });
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
