//! Window state that survives restarts, stored in `ui_state.json` (memo plan
//! M39, `nav-state-preservation`).
//!
//! Kept apart from `settings.json` so a settings export holds no window state
//! and a broken state file is harmless. Window position and size are kept by
//! ImGui itself.

use serde::{Deserialize, Serialize};
use trailkit_storage::{SchemaVersion, VersionedDocument};

use crate::note_tree::NodeId;

/// Default share of the window width taken by the note list.
pub const DEFAULT_SPLITTER_FRACTION: f32 = 0.35;

/// The restorable state of the main window.
///
/// A field missing from the file falls back to its default.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiState {
    /// The selected note or group.
    pub selected: Option<NodeId>,
    /// Share of the window width taken by the note list (two-pane layout).
    pub splitter_fraction: f32,
    /// Vertical scroll position of the note view.
    pub note_scroll: f32,
    /// Whether the user hid the note list column of the two-pane layout (M48).
    pub note_list_hidden: bool,
}

impl Default for UiState {
    /// Nothing selected, default splitter, top of the note, note list shown.
    fn default() -> Self {
        Self {
            selected: None,
            splitter_fraction: DEFAULT_SPLITTER_FRACTION,
            note_scroll: 0.0,
            note_list_hidden: false,
        }
    }
}

impl VersionedDocument for UiState {
    const FILE_STEM: &'static str = "ui_state";
    const CURRENT_SCHEMA_VERSION: SchemaVersion = SchemaVersion::FIRST;
}

impl UiState {
    /// The splitter fraction, clamped to a usable range (protects against a
    /// hand-edited or corrupted file).
    #[must_use]
    pub fn clamped_splitter_fraction(&self) -> f32 {
        if self.splitter_fraction.is_finite() {
            self.splitter_fraction.clamp(0.1, 0.9)
        } else {
            DEFAULT_SPLITTER_FRACTION
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_use_defaults() -> serde_json::Result<()> {
        let state: UiState = serde_json::from_str(r#"{"note_scroll": 12.5}"#)?;
        assert_eq!(state.selected, None);
        assert!((state.splitter_fraction - DEFAULT_SPLITTER_FRACTION).abs() < f32::EPSILON);
        assert!((state.note_scroll - 12.5).abs() < f32::EPSILON);
        assert!(!state.note_list_hidden);
        Ok(())
    }

    #[test]
    fn splitter_fraction_is_clamped() {
        let state = UiState {
            splitter_fraction: 5.0,
            ..UiState::default()
        };
        assert!((state.clamped_splitter_fraction() - 0.9).abs() < f32::EPSILON);
        let broken = UiState {
            splitter_fraction: f32::NAN,
            ..UiState::default()
        };
        assert!(
            (broken.clamped_splitter_fraction() - DEFAULT_SPLITTER_FRACTION).abs() < f32::EPSILON
        );
    }
}
