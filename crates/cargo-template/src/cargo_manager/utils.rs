use anyhow::Context;
use std::process::Command;
use toml_edit::{Item, Table};

pub fn get_git_remote_url() -> anyhow::Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        anyhow::bail!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let url = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git output")?
        .trim()
        .to_string();

    Ok(url)
}
pub fn insert_if_missing(
    table: &mut Table,
    template: &Table,
    key: &str,
    default_value_fn: fn() -> Item,
) {
    if !table.contains_key(key) || table.get(key).unwrap().as_str() == Some("") {
        let item = template
            .get(key)
            .map(|it| it.clone())
            .unwrap_or_else(|| default_value_fn());
        table.insert(key, item);
    }
}
pub fn insert_if_template_not_empty(
    table: &mut Table,
    template: &Table,
    key: &str,
    default_value_fn: fn() -> Item,
) {
    if let Some(template_value) = template.get(key) {
        table.insert(key, template_value.clone());
    } else if !table.contains_key(key) {
        table.insert(key, default_value_fn());
    }
}
