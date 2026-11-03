//! trailkit-memo — a notes addon for Guild Wars 2, loaded by Raidcore Nexus.
//!
//! This crate is the app (ROADMAP R76): it wires the slices into the main
//! window toggled by a keybind (unbound by default, memo plan M14) or the
//! quick access icon (R55) and builds the DLL. What slices share lives in
//! `trailkit-memo-shared`. Platform-independent logic stays testable on any
//! host (R23); the Nexus glue in `addon` only compiles for Windows.

pub mod note_editing;
pub mod note_search;
pub mod tree_drag;

#[cfg(windows)]
mod addon;
#[cfg(windows)]
mod main_window;
#[cfg(windows)]
mod search_field;
