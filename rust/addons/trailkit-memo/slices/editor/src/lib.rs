//! Editor slice of trailkit-memo (ROADMAP R76, memo plan M51–M54, M58,
//! M60, M69, M72, M93, M94, M100, M102): the edit session with its markdown
//! formatting commands, and the editor that draws title input, body,
//! formatting toolbar and preview.
//!
//! Depends only on `trailkit-memo-shared`, never on another slice. The app
//! keeps the end of the note toolbar (Edit / Cancel / Save) and the
//! unsaved-changes dialog; it hands in the preview layout and the search,
//! and turns the editor's actions into its own. `note_editing` is
//! platform-independent (R23); the drawing compiles for Windows.

pub mod i18n;
pub mod note_editing;

#[cfg(windows)]
pub mod editor;
