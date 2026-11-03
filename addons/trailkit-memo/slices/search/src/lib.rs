//! Search slice of trailkit-memo (ROADMAP R76, memo plan M118–M129): the
//! search index and results over the note tree, the search field above the
//! note list and the texts both show.
//!
//! Depends only on `trailkit-memo-shared`, never on another slice. The
//! ImGui-free logic is tested on any host; the field compiles for Windows.

pub mod i18n;
pub mod note_search;

#[cfg(windows)]
pub mod search_field;
