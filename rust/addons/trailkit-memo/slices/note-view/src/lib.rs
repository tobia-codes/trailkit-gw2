//! Note-view slice of trailkit-memo (ROADMAP R76, memo plan M41, M49, M81,
//! M122): the selected note's title with an optional widget behind it and
//! its rendered markdown in a scroll region below.
//!
//! Depends only on `trailkit-memo-shared`, never on another slice: what
//! other slices add (the kind dropdown, the draft hint) the app draws or
//! hands in as a neutral [`trailkit_ui::page::TitleTrailing`]. The view compiles for
//! Windows.

pub mod i18n;

#[cfg(windows)]
pub mod note_view;
