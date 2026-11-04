//! Note-list slice of trailkit-memo (ROADMAP R76, memo plan M43–M50,
//! M96–M117): the "New note / New group" header, the tree rows with inline
//! rename, context menus, "Move to…", search hits and drag & drop.
//!
//! Depends only on `trailkit-memo-shared`, never on another slice: the app
//! hands the search in through the shared `ListFilter` trait
//! ([`note_list::ListSearch`]) and the reset badge and menu as closures
//! ([`note_list::NoteRowExtras`]). The drop zones are ImGui-free and tested
//! on any host; the list compiles for Windows.

pub mod badge_geometry;
pub mod i18n;
pub mod tree_drag;

#[cfg(windows)]
pub mod note_list;
