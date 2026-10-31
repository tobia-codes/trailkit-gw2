//! `cargo xtask deploy`: cross-compile an addon and copy it into the GW2
//! `addons` folder so Nexus hot-reloads it (ROADMAP R32).
//!
//! The target folder comes from the `TRAILKIT_GW2_ADDONS_DIR` environment
//! variable, else from `gw2_addons_dir` in the git-ignored `xtask.local.toml`
//! at the repository root. It is never hard-coded.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};

use crate::workspace::{self, Addon};

/// Windows target triple every addon is built for.
const TARGET: &str = "x86_64-pc-windows-msvc";

/// Environment variable that overrides the addons folder.
const ENV_ADDONS_DIR: &str = "TRAILKIT_GW2_ADDONS_DIR";

/// Local, git-ignored config file name at the repository root.
const LOCAL_CONFIG: &str = "xtask.local.toml";

/// Cargo build profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// `dev` profile (output in `target/<triple>/debug`).
    Debug,
    /// `release` profile (output in `target/<triple>/release`).
    Release,
}

impl Profile {
    /// Name of the output directory below `target/<triple>/`.
    const fn dir_name(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
        }
    }
}

/// Builds `addon_name` with cargo-xwin and copies the DLL into the addons folder.
///
/// # Errors
/// Returns an error if the addon does not exist, the addons folder is not
/// configured or missing, the build fails, or the copy fails (e.g. the DLL is
/// locked by the game).
pub fn run(addon_name: &str, profile: Profile) -> Result<()> {
    let addon = Addon::find(addon_name)?;
    let addons_dir_path = addons_dir()?;
    build(&addon, profile)?;

    let built_dll = workspace::root()
        .join("target")
        .join(TARGET)
        .join(profile.dir_name())
        .join(addon.built_dll_name());
    let deployed_dll = addons_dir_path.join(addon.shipped_dll_name());
    replace_file_atomically(&built_dll, &deployed_dll)?;
    println!("deployed {} → {}", addon.name(), deployed_dll.display());
    Ok(())
}

/// Resolves the GW2 addons folder from the environment or the local config.
///
/// # Errors
/// Returns an error if neither source is set or the folder does not exist.
fn addons_dir() -> Result<PathBuf> {
    let addons_dir = match std::env::var_os(ENV_ADDONS_DIR) {
        Some(value) => PathBuf::from(value),
        None => read_local_config()?,
    };
    if !addons_dir.is_dir() {
        bail!("GW2 addons folder {} does not exist", addons_dir.display());
    }
    Ok(addons_dir)
}

/// Reads `gw2_addons_dir` from `xtask.local.toml`.
///
/// # Errors
/// Returns an error if the file is missing, invalid, or lacks the key.
fn read_local_config() -> Result<PathBuf> {
    let path = workspace::root().join(LOCAL_CONFIG);
    let text = fs::read_to_string(&path).with_context(|| {
        format!(
            "GW2 addons folder not configured: set {ENV_ADDONS_DIR} or create {LOCAL_CONFIG} \
             (see xtask/README.md)"
        )
    })?;
    parse_addons_dir(&text).with_context(|| format!("in {}", path.display()))
}

/// Extracts `gw2_addons_dir = "<path>"` from the local config text.
///
/// # Errors
/// Returns an error if the TOML is invalid or the key is missing / not a string.
fn parse_addons_dir(text: &str) -> Result<PathBuf> {
    let config: toml_edit::DocumentMut = text.parse()?;
    config
        .get("gw2_addons_dir")
        .and_then(|item| item.as_str())
        .map(PathBuf::from)
        .context("missing string key `gw2_addons_dir`")
}

/// Runs `cargo xwin build` for the addon.
///
/// # Errors
/// Returns an error if cargo cannot be started or the build fails.
fn build(addon: &Addon, profile: Profile) -> Result<()> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command.current_dir(workspace::root()).args([
        "xwin",
        "build",
        "--package",
        addon.name(),
        "--target",
        TARGET,
    ]);
    if profile == Profile::Release {
        command.arg("--release");
    }
    let status = command
        .status()
        .context("failed to run `cargo xwin` (install with `cargo install --locked cargo-xwin`)")?;
    if !status.success() {
        bail!("build failed ({status})");
    }
    Ok(())
}

/// Copies `source` next to `destination` and renames it over `destination`, so Nexus never sees
/// a half-written DLL.
///
/// # Errors
/// Returns an error if the copy or rename fails.
fn replace_file_atomically(source: &Path, destination: &Path) -> Result<()> {
    let temporary = destination.with_extension("dll.tmp");
    fs::copy(source, &temporary)
        .with_context(|| format!("copying {} to {}", source.display(), temporary.display()))?;
    fs::rename(&temporary, destination).with_context(|| {
        format!(
            "replacing {} (is the DLL locked by the game?)",
            destination.display()
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_addons_dir() {
        let addons_dir =
            parse_addons_dir(r#"gw2_addons_dir = "/mnt/c/Guild Wars 2/addons""#).unwrap();
        assert_eq!(addons_dir, PathBuf::from("/mnt/c/Guild Wars 2/addons"));
    }

    #[test]
    fn rejects_missing_key() {
        assert!(parse_addons_dir("other = 1").is_err());
    }
}
