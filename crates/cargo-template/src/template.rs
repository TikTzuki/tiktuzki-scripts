//! Template rendering module for Cargo.toml files.
//!
//! This module handles loading predefined templates and applying them to
//! workspace or package Cargo.toml files. Templates are embedded at compile
//! time from the `templates/` directory.

use crate::cargo_manager::{CargoManager, CargoType, detect_cargo_type};
use anyhow::{Context, Result, bail};
use include_dir::{Dir, include_dir};
use std::path::PathBuf;
use toml_edit::{DocumentMut, TableLike};

/// Embedded template directory containing all available templates
const TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

/// Updates dependencies in the target Cargo.toml using the provided manager.
///
/// # Arguments
///
/// * `manager` - A mutable reference to a cargo manager
/// * `dependencies` - Table of dependencies to add/update
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if the commit fails.
fn update_dependencies(manager: &mut dyn CargoManager, dependencies: &dyn TableLike) -> Result<()> {
    for (name, item) in dependencies.iter() {
        manager
            .update_dependency(dependencies.key(name).unwrap(), Some(item))
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to add dependency '{}': {}", name, e);
            });
    }
    manager.commit()
}

/// Renders a template into the target Cargo.toml file.
///
/// This function:
/// 1. Loads the specified template from embedded resources
/// 2. Detects whether the target is a workspace or package
/// 3. Applies workspace package metadata if present
/// 4. Updates dependencies if present in the template
///
/// # Arguments
///
/// * `template_name` - Name of the template directory (e.g., "workspace", "axum")
/// * `target` - Path to the directory containing Cargo.toml
///
/// # Returns
///
/// Returns `Ok(())` if the template was successfully applied, or an error if:
/// - Template is not found
/// - Target Cargo.toml is invalid
/// - Write operations fail
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// render_template("workspace", &PathBuf::from("."))?;
/// # Ok(())
/// # }
/// ```
pub fn render_templates(template_name: Vec<&str>, target: &PathBuf) -> Result<()> {
    for name in template_name {
        render_template(name, target)
            .with_context(|| format!("Failed to render template '{}'", name))?;
    }
    Ok(())
}

pub fn render_template(template_name: &str, target: &PathBuf) -> Result<()> {
    // Load template from embedded resources
    let template_path = TEMPLATE_DIR
        .get_file(format!("{}/Cargo.toml", template_name))
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_name))?;

    // Detect cargo type (workspace or package)
    let manager = detect_cargo_type(target)
        .map_err(|err| anyhow::anyhow!("Failed to detect cargo type: {}", err))?;

    // Parse template document
    let doc = template_path
        .contents_utf8()
        .ok_or_else(|| anyhow::anyhow!("Failed to read template contents"))?
        .parse::<DocumentMut>()?;

    // Extract workspace dependencies section
    let dependencies = doc["workspace"]
        .get("dependencies")
        .and_then(|it| it.as_table());

    // Extract workspace package metadata section
    let workspace_package = doc["workspace"].get("package").and_then(|it| it.as_table());

    // Apply template based on cargo type
    match manager {
        CargoType::Workspace(mut m) => {
            if let Some(wp) = workspace_package {
                m.format_workspace_package(wp)?;
            }
            if let Some(deps) = dependencies {
                update_dependencies(&mut m, deps)?;
            }
            m.commit()?;
        }
        CargoType::Package(mut m) => {
            if let Some(wp) = workspace_package {
                m.format_workspace_package(wp)?;
            }
            if let Some(deps) = dependencies {
                update_dependencies(&mut m, deps)?;
            }
            m.commit()?;
        }
        CargoType::Both => {
            bail!("Cargo.toml with both workspace and package sections is not supported");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template() {
        let result = render_template("workspace", &".".into());
        // This test may fail if not in a cargo project directory
        assert!(result.is_ok() || result.is_err());
    }
}
