use crate::cargo_manager::{CargoManager, CargoType, detect_cargo_type};
use anyhow::{Context, Result, bail};
use include_dir::{Dir, include_dir};
use std::path::PathBuf;
use toml_edit::{DocumentMut, TableLike};

const TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

fn update_dependencies(manager: &mut dyn CargoManager, dependencies: &dyn TableLike) -> Result<()> {
    for (name, item) in dependencies.iter() {
        manager
            .update_dependency(dependencies.key(name).unwrap(), Some(item))
            .unwrap_or_else(|e| {
                eprintln!("Failed to add dependency '{}': {}", name, e);
            });
    }
    manager.commit()
}

pub fn render_template(template_name: &str, target: &PathBuf) -> Result<()> {
    let template_path = TEMPLATE_DIR
        .get_file(format!("{}/Cargo.toml", template_name))
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_name))?;
    let manager = detect_cargo_type(target)
        .map_err(|err| anyhow::anyhow!("Failed to detect cargo type: {}", err))?;

    let doc = template_path
        .contents_utf8()
        .ok_or_else(|| anyhow::anyhow!("Failed to read template contents"))?
        .parse::<DocumentMut>()?;

    let dependencies = doc["workspace"]
        .get("dependencies")
        .map_or(None, |it| it.as_table());

    let workspace_package = doc["workspace"]
        .get("package")
        .map_or(None, |it| it.as_table());

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
        _ => bail!("Unsupported cargo type for template rendering"),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::template::render_template;
    use std::env;

    #[test]
    fn test_render_template() {
        unsafe {
            env::set_var("TARGET_CARGO", "./tmp");
        }
        let result = render_template("workspace", &".".into());
        if let Err(e) = result {
            panic!("Failed to render template: {}", e);
        }
    }
}
