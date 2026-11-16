//! Window state that survives restarts, stored in `ui_state.json` (memo plan
//! M39, `nav-state-preservation`).
//!
//! Kept apart from `settings.json` so a settings export holds no window state
//! and a broken state file is harmless. Window position and size are kept by
//! ImGui itself.

use serde::{Deserialize, Serialize};
use trailkit_storage::{SchemaVersion, VersionedDocument};

use crate::note_tree::{NodeId, NoteId};

/// Default content width of the note list in average characters (`n`),
/// the regular pane minimum (M40, M74).
pub const DEFAULT_NOTE_LIST_CHARACTERS: f32 = 22.0;

/// Default content width of the help window's topic list in average body
/// characters (ROADMAP R101 Q315).
pub const DEFAULT_HELP_TOPIC_LIST_CHARACTERS: f32 = 14.0;

/// The tab the help window shows (ROADMAP R101 Q318).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelpWindowTab {
    /// The help topics.
    #[default]
    Help,
    /// Saving the log and reporting a problem.
    ReportProblem,
}

/// The restorable state of the main window and the help window.
///
/// A field missing from the file falls back to its default.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiState {
    /// The selected note or group.
    pub selected: Option<NodeId>,
    /// The note in the note pane. It stays while a group is selected (M173).
    pub shown_note: Option<NoteId>,
    /// Content width of the note list in average characters (two-pane
    /// layout, M74): a fixed width, so a wider window gives the note the
    /// extra room; measured in characters so it follows the font size.
    pub note_list_characters: f32,
    /// Vertical scroll position of the note view.
    pub note_scroll: f32,
    /// Whether the user hid the note list column of the two-pane layout
    /// (M48, toggled from the title bar since M137).
    pub note_list_hidden: bool,
    /// Whether the search field above the note list is shown (M138); it
    /// stays until the user hides it again.
    pub search_visible: bool,
    /// Whether the note view is compact: no toolbar and no kind dropdown
    /// (M139).
    pub compact_note_view: bool,
    /// Whether the editor shows the markdown preview (M60).
    pub preview_visible: bool,
    /// Share of the editor width taken by the text when the preview is shown
    /// side by side (M60).
    pub preview_split_fraction: f32,
    /// The help window's shown tab (R101 Q318).
    pub help_tab: HelpWindowTab,
    /// The position of the help window's selected topic (R101 Q318).
    pub help_topic: usize,
    /// Content width of the help window's topic list in average body
    /// characters (R101 Q314).
    pub help_topic_list_characters: f32,
}

impl Default for UiState {
    /// Nothing selected, default splitter, top of the note, note list shown,
    /// search field hidden, regular note view, no preview; the help window
    /// on its Help tab with the first topic.
    fn default() -> Self {
        Self {
            selected: None,
            shown_note: None,
            note_list_characters: DEFAULT_NOTE_LIST_CHARACTERS,
            note_scroll: 0.0,
            note_list_hidden: false,
            search_visible: false,
            compact_note_view: false,
            preview_visible: false,
            preview_split_fraction: 0.5,
            help_tab: HelpWindowTab::Help,
            help_topic: 0,
            help_topic_list_characters: DEFAULT_HELP_TOPIC_LIST_CHARACTERS,
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
        assert!(!state.search_visible);
        assert!(!state.compact_note_view);
        assert_eq!(state.help_tab, HelpWindowTab::Help);
        assert_eq!(state.help_topic, 0);
        Ok(())
    }

    #[test]
    fn the_help_window_state_round_trips() -> serde_json::Result<()> {
        let state = UiState {
            help_tab: HelpWindowTab::ReportProblem,
            help_topic: 3,
            help_topic_list_characters: 20.5,
            ..UiState::default()
        };
        let json = serde_json::to_string(&state)?;
        assert!(json.contains(r#""help_tab":"report_problem""#));
        assert_eq!(serde_json::from_str::<UiState>(&json)?, state);
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
