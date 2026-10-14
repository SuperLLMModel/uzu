use std::fs;

use anyhow::{Context, Result};
use toml_edit::{DocumentMut, value};

use crate::configs::Paths;

pub fn bump_workspace_version(version: &str) -> Result<()> {
    let path = Paths::new()?.workspace_manifest_path();
    let body = fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let mut document: DocumentMut = body.parse()?;
    update_version(&mut document, version)?;
    fs::write(&path, document.to_string())?;
    Ok(())
}

fn update_version(
    document: &mut DocumentMut,
    version: &str,
) -> Result<()> {
    document["workspace"]["package"]["version"] = value(version);
    for (_, dependency) in
        document["workspace"]["dependencies"].as_table_mut().context("Missing workspace dependencies")?.iter_mut()
    {
        if dependency.get("path").is_some() && dependency.get("version").is_some() {
            dependency["version"] = value(version);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use toml_edit::DocumentMut;

    use super::update_version;

    #[test]
    fn bumps_publishable_path_dependencies_only() {
        let mut document: DocumentMut = r#"
[workspace.package]
version = "0.6.7"
[workspace.dependencies]
local = { path = "crates/local", version = "0.6.7" }
private = { path = "crates/private" }
external = "1.0"
"#
        .parse()
        .unwrap();

        update_version(&mut document, "0.6.8").unwrap();

        assert_eq!(document["workspace"]["package"]["version"].as_str(), Some("0.6.8"));
        let dependencies = &document["workspace"]["dependencies"];
        assert_eq!(dependencies["local"]["version"].as_str(), Some("0.6.8"));
        assert!(dependencies["private"].get("version").is_none());
        assert_eq!(dependencies["external"].as_str(), Some("1.0"));
    }
}
