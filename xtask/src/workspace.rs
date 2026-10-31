//! Workspace paths and addon discovery shared by the xtask commands.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// The repository root (parent of the `xtask` crate directory).
#[must_use]
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// An addon crate that exists under `addons/<name>/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Addon {
    /// Crate / directory name, e.g. `trailkit-memo`.
    name: String,
}

impl Addon {
    /// Looks up an addon by name.
    ///
    /// # Errors
    /// Returns an error if `addons/<name>/Cargo.toml` does not exist or the name
    /// contains path separators.
    pub fn find(name: &str) -> Result<Self> {
        if name.is_empty() || name.contains(['/', '\\', '.']) {
            bail!("invalid addon name {name:?}");
        }
        let addon = Self {
            name: name.to_owned(),
        };
        if !addon.dir().join("Cargo.toml").is_file() {
            bail!("no addon named {name:?} under addons/");
        }
        Ok(addon)
    }

    /// Lists every addon under `addons/`, sorted by name.
    ///
    /// # Errors
    /// Returns an error if the `addons/` directory cannot be read.
    pub fn all() -> Result<Vec<Self>> {
        let dir = root().join("addons");
        let mut addons = Vec::new();
        for entry in
            std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))?
        {
            let entry = entry?;
            if entry.path().join("Cargo.toml").is_file() {
                addons.push(Self {
                    name: entry.file_name().to_string_lossy().into_owned(),
                });
            }
        }
        addons.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(addons)
    }

    /// The crate / directory name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The addon directory `addons/<name>`.
    #[must_use]
    pub fn dir(&self) -> PathBuf {
        root().join("addons").join(&self.name)
    }

    /// File name of the built DLL (`-` replaced by `_`, as cargo does).
    #[must_use]
    pub fn dll_name(&self) -> String {
        format!("{}.dll", self.name.replace('-', "_"))
    }
}
