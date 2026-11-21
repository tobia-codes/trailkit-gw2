//! Search slice of trailkit-memo (ROADMAP R76, memo plan M118–M129): the
//! search index and results over the note tree, the empty search result and
//! the texts the searches show; the note list reads the results through the
//! shared `ListFilter` trait. The search field and the find bar themselves
//! are the shared search of `trailkit-ui` (BL30, Q581).
//!
//! Depends only on `trailkit-memo-shared`, never on another slice. The
//! ImGui-free logic is tested on any host; the drawing compiles for Windows.

pub mod i18n;
pub mod note_search;

#[cfg(windows)]
pub mod no_search_results;
