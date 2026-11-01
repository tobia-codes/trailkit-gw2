//! User settings of trailkit-memo and the visibility rules derived from them.
//!
//! Persisted as `settings.json` through `trailkit-storage` (memo plan M17);
//! the settings window follows in phase 1 step 8. Platform-independent so the
//! rules can be unit-tested on any host.

use serde::{Deserialize, Serialize};
use trailkit_storage::{DocumentFields, MigrationError, SchemaVersion, VersionedDocument};

/// Whether the addon adds its icon to the Nexus quick access bar (R55).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuickAccessIcon {
    /// The icon is shown (default).
    #[default]
    Shown,
    /// No icon; the window opens via keybind only.
    Hidden,
}

/// What an open window does while the game is not in gameplay — loading
/// screens, cutscenes, character select (R56, R61).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutsideGameplay {
    /// The window is not drawn but stays open and reappears afterwards (default).
    #[default]
    Hide,
    /// The window is drawn as usual.
    Show,
}

/// What happens when the user leaves the editor with unsaved changes by
/// closing the window or navigating (memo plan M57).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnsavedChanges {
    /// Ask "Save / Discard / Keep editing" (default).
    #[default]
    Ask,
    /// Save without asking; "Cancel" discards without asking.
    SaveAutomatically,
}

/// All user settings of trailkit-memo.
///
/// A field missing from `settings.json` falls back to its default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Quick access icon visibility.
    pub quick_access_icon: QuickAccessIcon,
    /// Window behavior outside gameplay.
    pub outside_gameplay: OutsideGameplay,
    /// Leaving the editor with unsaved changes (M57).
    pub unsaved_changes: UnsavedChanges,
}

impl Settings {
    /// The default settings, usable in `const`/`static` contexts.
    pub const DEFAULT: Self = Self {
        quick_access_icon: QuickAccessIcon::Shown,
        outside_gameplay: OutsideGameplay::Hide,
        unsaved_changes: UnsavedChanges::Ask,
    };

    /// Whether an open window may be drawn in the current game state.
    #[must_use]
    pub const fn allows_drawing(&self, is_gameplay: bool) -> bool {
        is_gameplay || matches!(self.outside_gameplay, OutsideGameplay::Show)
    }
}

impl VersionedDocument for Settings {
    const FILE_STEM: &'static str = "settings";
    /// Version 2 added `unsaved_changes` (M57).
    const CURRENT_SCHEMA_VERSION: SchemaVersion = SchemaVersion(2);

    /// Version 1 → 2 only added a field; missing fields take their default
    /// when reading, so the fields stay as they are.
    fn migrate_one_step(
        from: SchemaVersion,
        fields: DocumentFields,
    ) -> Result<DocumentFields, MigrationError> {
        if from == SchemaVersion::FIRST {
            Ok(fields)
        } else {
            Err(MigrationError::NoMigrationPath { from })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn const_default_matches_derived_default() {
        assert_eq!(Settings::DEFAULT, Settings::default());
    }

    #[test]
    fn default_hides_outside_gameplay() {
        assert!(Settings::DEFAULT.allows_drawing(true));
        assert!(!Settings::DEFAULT.allows_drawing(false));
    }

    #[test]
    fn show_outside_gameplay_always_draws() {
        let settings = Settings {
            outside_gameplay: OutsideGameplay::Show,
            ..Settings::DEFAULT
        };
        assert!(settings.allows_drawing(false));
    }

    #[test]
    fn json_uses_snake_case_and_fills_missing_fields() -> serde_json::Result<()> {
        let value = serde_json::to_value(Settings::DEFAULT)?;
        assert_eq!(value["quick_access_icon"], "shown");
        assert_eq!(value["outside_gameplay"], "hide");

        let partial: Settings = serde_json::from_str(r#"{"quick_access_icon": "hidden"}"#)?;
        assert_eq!(partial.quick_access_icon, QuickAccessIcon::Hidden);
        assert_eq!(partial.outside_gameplay, OutsideGameplay::Hide);
        assert_eq!(partial.unsaved_changes, UnsavedChanges::Ask);
        assert_eq!(value["unsaved_changes"], "ask");
        Ok(())
    }
}
