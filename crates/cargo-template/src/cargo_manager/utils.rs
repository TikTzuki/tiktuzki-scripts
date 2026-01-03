//! Utility functions for Cargo.toml manipulation.

use anyhow::Context;
use std::process::Command;
use toml_edit::{Item, Table};

/// Retrieves the Git remote URL for the "origin" remote.
///
/// This function executes `git remote get-url origin` and returns the URL.
/// The URL is automatically inserted into the repository field of workspace packages.
///
/// # Returns
///
/// Returns the remote URL as a string, or an error if:
/// - Git is not installed
/// - Not in a Git repository
/// - No "origin" remote exists
///
/// # Examples
///
/// ```no_run
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// let url = get_git_remote_url()?;
/// println!("Repository URL: {}", url);
/// # Ok(())
/// # }
/// ```
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

/// Inserts a key-value pair into a table if the key doesn't exist or is empty.
///
/// This function checks if the key exists in the template. If it does, it uses
/// the template value; otherwise, it calls the default value function.
///
/// # Arguments
///
/// * `table` - The target table to insert into
/// * `template` - The template table to check for values
/// * `key` - The key to insert
/// * `default_value_fn` - Function that returns a default value if template doesn't provide one
pub fn insert_if_missing(
    table: &mut Table,
    template: &Table,
    key: &str,
    default_value_fn: fn() -> Item,
) {
    if !table.contains_key(key) || table.get(key).unwrap().as_str() == Some("") {
        let item = template.get(key).cloned().unwrap_or_else(default_value_fn);
        table.insert(key, item);
    }
}

/// Inserts a key-value pair into a table only if the template provides a non-empty value.
///
/// This function prioritizes template values and only falls back to defaults if the
/// template doesn't specify the key and the target table also doesn't have it.
///
/// # Arguments
///
/// * `table` - The target table to insert into
/// * `template` - The template table to check for values
/// * `key` - The key to insert
/// * `default_value_fn` - Function that returns a default value if neither template nor table have the key
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
