//! The shared `trailkit/` folder below the Nexus addons folder (ROADMAP R92).
//!
//! Every trailkit addon keeps its files in one place:
//!
//! ```text
//! addons/trailkit/
//! ├─ memo/     data of one addon (short name, see AddonName)
//! ├─ shared/   files every addon reads (theme, language; core plan C2)
//! └─ logs/     one log file per addon (crate trailkit-log)
//! ```
//!
//! Addons never build these paths from string literals; they ask
//! [`TrailkitDirectory`] for them.

use std::{fmt, path::PathBuf};

/// Name of the folder below the Nexus addons folder that holds every trailkit file.
pub const TRAILKIT_DIRECTORY_NAME: &str = "trailkit";

/// Name of the subfolder of [`TRAILKIT_DIRECTORY_NAME`] that holds the logs.
const LOGS_DIRECTORY_NAME: &str = "logs";

/// Name of the subfolder of [`TRAILKIT_DIRECTORY_NAME`] shared by every addon.
const SHARED_DIRECTORY_NAME: &str = "shared";

/// Short name of an addon as used in folder and file names: `memo` for
/// `trailkit-memo`.
///
/// Only lowercase ASCII letters, digits and inner hyphens are allowed, and the
/// names of the shared folders (`logs`, `shared`) are reserved, so an addon
/// folder can never collide with them. The check runs at compile time when
/// the name is a constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AddonName(&'static str);

impl AddonName {
    /// Creates an addon name.
    ///
    /// # Panics
    ///
    /// Panics (at compile time in a `const`) if `name` is empty, contains
    /// anything but lowercase ASCII letters, digits and inner hyphens, starts
    /// with `trailkit` or is a reserved folder name.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        assert!(is_valid_addon_name(name), "invalid addon name");
        Self(name)
    }

    /// The name as a string, e.g. `"memo"`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for AddonName {
    /// Writes the plain name, e.g. `memo`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

/// Whether `name` is a valid [`AddonName`] (see there for the rules).
const fn is_valid_addon_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() || bytes[0] == b'-' || bytes[bytes.len() - 1] == b'-' {
        return false;
    }
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-') {
            return false;
        }
        index += 1;
    }
    !starts_with(bytes, TRAILKIT_DIRECTORY_NAME.as_bytes())
        && !equals(bytes, LOGS_DIRECTORY_NAME.as_bytes())
        && !equals(bytes, SHARED_DIRECTORY_NAME.as_bytes())
}

/// `const` version of `<[u8]>::starts_with`.
const fn starts_with(bytes: &[u8], prefix: &[u8]) -> bool {
    if bytes.len() < prefix.len() {
        return false;
    }
    let mut index = 0;
    while index < prefix.len() {
        if bytes[index] != prefix[index] {
            return false;
        }
        index += 1;
    }
    true
}

/// `const` version of `==` for byte slices.
const fn equals(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len() && starts_with(left, right)
}

/// The `trailkit/` folder and the paths of its subfolders.
///
/// Creating the value touches no file; callers create folders on their worker
/// thread when they first write into them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrailkitDirectory {
    /// Absolute path of the `trailkit/` folder.
    root: PathBuf,
}

impl TrailkitDirectory {
    /// Wraps the path of the `trailkit/` folder, as Nexus returns it for
    /// [`TRAILKIT_DIRECTORY_NAME`].
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The `trailkit/` folder itself.
    #[must_use]
    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    /// The data folder of `addon`, e.g. `trailkit/memo/`.
    #[must_use]
    pub fn addon_data(&self, addon: AddonName) -> PathBuf {
        self.root.join(addon.as_str())
    }

    /// The folder holding every addon's log file, `trailkit/logs/`.
    #[must_use]
    pub fn logs(&self) -> PathBuf {
        self.root.join(LOGS_DIRECTORY_NAME)
    }

    /// The folder shared by every addon, `trailkit/shared/` (core plan C2).
    #[must_use]
    pub fn shared(&self) -> PathBuf {
        self.root.join(SHARED_DIRECTORY_NAME)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_every_subfolder_below_the_root() {
        let directory = TrailkitDirectory::new("addons/trailkit");
        let memo = AddonName::new("memo");

        assert_eq!(
            directory.addon_data(memo),
            PathBuf::from("addons/trailkit/memo")
        );
        assert_eq!(directory.logs(), PathBuf::from("addons/trailkit/logs"));
        assert_eq!(directory.shared(), PathBuf::from("addons/trailkit/shared"));
    }

    #[test]
    fn accepts_short_kebab_case_names() {
        for name in ["memo", "event", "core", "wvw-map", "tp2"] {
            assert!(is_valid_addon_name(name), "{name} should be valid");
        }
    }

    #[test]
    fn rejects_names_that_could_collide_or_escape() {
        for name in [
            "",
            "Memo",
            "trailkit-memo",
            "logs",
            "shared",
            "-memo",
            "memo-",
            "me mo",
            "../memo",
            "memo/notes",
        ] {
            assert!(!is_valid_addon_name(name), "{name:?} should be invalid");
        }
    }
}
