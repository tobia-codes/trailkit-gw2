//! `cargo xtask sync-versions`: Changesets owns each addon's version in its
//! placeholder `package.json`; this command copies it into `Cargo.toml`
//! (ROADMAP R16/R17). With `--check` it only verifies (used by CI).

use std::fs;

use anyhow::{Context, Result, bail};
use toml_edit::{DocumentMut, value};

use crate::workspace::Addon;

/// Whether to write changes or only report mismatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Update `Cargo.toml` files in place.
    Write,
    /// Fail if any `Cargo.toml` version differs from its `package.json`.
    Check,
}

/// Syncs (or checks) the versions of every addon.
///
/// # Errors
/// Returns an error if a manifest cannot be read/parsed/written, or — in
/// [`Mode::Check`] — if any version is out of sync.
pub fn run(mode: Mode) -> Result<()> {
    let mut mismatches = Vec::new();
    for addon in Addon::all()? {
        let package_json_path = addon.dir().join("package.json");
        let cargo_toml_path = addon.dir().join("Cargo.toml");
        let package_json_version = parse_package_json_version(
            &fs::read_to_string(&package_json_path)
                .with_context(|| format!("reading {}", package_json_path.display()))?,
        )
        .with_context(|| format!("in {}", package_json_path.display()))?;
        let cargo_toml_text = fs::read_to_string(&cargo_toml_path)
            .with_context(|| format!("reading {}", cargo_toml_path.display()))?;
        let mut cargo_toml: DocumentMut = cargo_toml_text
            .parse()
            .with_context(|| format!("parsing {}", cargo_toml_path.display()))?;
        let cargo_toml_version = cargo_toml
            .get("package")
            .and_then(|package| package.get("version"))
            .and_then(|version| version.as_str())
            .unwrap_or_default()
            .to_owned();

        if cargo_toml_version == package_json_version {
            continue;
        }
        match mode {
            Mode::Check => mismatches.push(format!(
                "{}: Cargo.toml {cargo_toml_version} ≠ package.json {package_json_version}",
                addon.name()
            )),
            Mode::Write => {
                cargo_toml["package"]["version"] = value(package_json_version.as_str());
                fs::write(&cargo_toml_path, cargo_toml.to_string())
                    .with_context(|| format!("writing {}", cargo_toml_path.display()))?;
                println!(
                    "{}: {cargo_toml_version} → {package_json_version}",
                    addon.name()
                );
            }
        }
    }
    if !mismatches.is_empty() {
        bail!(
            "versions out of sync (run `cargo xtask sync-versions`):\n  {}",
            mismatches.join("\n  ")
        );
    }
    Ok(())
}

/// Extracts the `version` string from a `package.json` text.
///
/// # Errors
/// Returns an error if the JSON is invalid or has no string `version`.
fn parse_package_json_version(json: &str) -> Result<String> {
    let package_json: serde_json::Value = serde_json::from_str(json)?;
    package_json
        .get("version")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .context("missing string field `version`")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_package_version() {
        assert_eq!(
            parse_package_json_version(r#"{"name":"x","version":"1.2.3"}"#).unwrap(),
            "1.2.3"
        );
    }

    #[test]
    fn rejects_missing_version() {
        assert!(parse_package_json_version(r#"{"name":"x"}"#).is_err());
    }
}
