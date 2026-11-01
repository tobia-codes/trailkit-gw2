//! Window state that survives restarts, stored in `ui_state.json` (memo plan
//! M39, `nav-state-preservation`).
//!
//! Kept apart from `settings.json` so a settings export holds no window state
//! and a broken state file is harmless. Window position and size are kept by
//! ImGui itself.

use serde::{Deserialize, Serialize};
use trailkit_storage::{SchemaVersion, VersionedDocument};

use crate::note_tree::NodeId;

/// Default content width of the note list in average characters (`n`),
/// the regular pane minimum (M40, M74).
pub const DEFAULT_NOTE_LIST_CHARACTERS: f32 = 22.0;

/// The restorable state of the main window.
///
/// A field missing from the file falls back to its default.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiState {
    /// The selected note or group.
    pub selected: Option<NodeId>,
    /// Content width of the note list in average characters (two-pane
    /// layout, M74): a fixed width, so a wider window gives the note the
    /// extra room; measured in characters so it follows the font size.
    pub note_list_characters: f32,
    /// Vertical scroll position of the note view.
    pub note_scroll: f32,
    /// Whether the user hid the note list column of the two-pane layout (M48).
    pub note_list_hidden: bool,
    /// Whether the editor shows the markdown preview (M60).
    pub preview_visible: bool,
    /// Share of the editor width taken by the text when the preview is shown
    /// side by side (M60).
    pub preview_split_fraction: f32,
}

impl Default for UiState {
    /// Nothing selected, default splitter, top of the note, note list shown,
    /// no preview.
    fn default() -> Self {
        Self {
            selected: None,
            note_list_characters: DEFAULT_NOTE_LIST_CHARACTERS,
            note_scroll: 0.0,
            note_list_hidden: false,
            preview_visible: false,
            preview_split_fraction: 0.5,
        }
    }
}

impl VersionedDocument for UiState {
    const FILE_STEM: &'static str = "ui_state";
    const CURRENT_SCHEMA_VERSION: SchemaVersion = SchemaVersion::FIRST;
}

impl UiState {
    /// The note list width in characters, or the default if the stored value
    /// is broken (protects against a hand-edited or corrupted file). The
    /// window clamps it further to the space it has.
    #[must_use]
    pub fn valid_note_list_characters(&self) -> f32 {
        if self.note_list_characters.is_finite() && self.note_list_characters > 0.0 {
            self.note_list_characters
        } else {
            DEFAULT_NOTE_LIST_CHARACTERS
        }
    }

    /// The editor/preview split, clamped to a usable range (protects against
    /// a hand-edited or corrupted file).
    #[must_use]
    pub fn clamped_preview_split_fraction(&self) -> f32 {
        clamp_fraction(self.preview_split_fraction, 0.5)
    }
}

/// Clamps a stored fraction to a usable range, or `default` if it is broken.
fn clamp_fraction(fraction: f32, default: f32) -> f32 {
    if fraction.is_finite() {
        fraction.clamp(0.1, 0.9)
    } else {
        default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_use_defaults() -> serde_json::Result<()> {
        let state: UiState = serde_json::from_str(r#"{"note_scroll": 12.5}"#)?;
        assert_eq!(state.selected, None);
        assert!((state.note_list_characters - DEFAULT_NOTE_LIST_CHARACTERS).abs() < f32::EPSILON);
        assert!((state.note_scroll - 12.5).abs() < f32::EPSILON);
        assert!(!state.note_list_hidden);
        Ok(())
    }

    #[test]
    fn broken_note_list_width_falls_back_to_the_default() {
        for broken in [f32::NAN, f32::INFINITY, -3.0, 0.0] {
            let state = UiState {
                note_list_characters: broken,
                ..UiState::default()
            };
            assert!(
                (state.valid_note_list_characters() - DEFAULT_NOTE_LIST_CHARACTERS).abs()
                    < f32::EPSILON
            );
        }
    }

    #[test]
    fn old_splitter_fraction_is_ignored() -> serde_json::Result<()> {
        let state: UiState = serde_json::from_str(r#"{"splitter_fraction": 0.35}"#)?;
        assert!((state.note_list_characters - DEFAULT_NOTE_LIST_CHARACTERS).abs() < f32::EPSILON);
        Ok(())
    }
}
