//! trailkit-memo — a notes addon for Guild Wars 2, loaded by Raidcore Nexus.
//!
//! Phase 0 skeleton: validates build → deploy → hot reload. It registers a
//! main window ("Hello") toggled by a keybind that is unbound by default
//! (memo plan M14) or by its quick access icon (R55); the window is hidden
//! outside gameplay (R56).
//!
//! Layout follows ROADMAP R23: platform-independent logic (e.g. [`i18n`])
//! lives in modules without ImGui/Nexus so it can be unit-tested on any host;
//! the Nexus glue in `addon` only compiles for Windows.

pub mod drafts;
pub mod i18n;
pub mod note_editing;
pub mod note_tree;
pub mod settings;
pub mod storage_worker;
pub mod ui_state;

#[cfg(windows)]
mod addon;
#[cfg(windows)]
mod main_window;
