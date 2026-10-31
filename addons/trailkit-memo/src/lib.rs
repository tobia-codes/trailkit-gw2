//! trailkit-memo — a notes addon for Guild Wars 2, loaded by Raidcore Nexus.
//!
//! Phase 0 skeleton: validates build → deploy → hot reload. It registers a
//! main window ("Hello") toggled by a keybind that is unbound by default
//! (memo plan M14).
//!
//! Layout follows ROADMAP R23: platform-independent logic (e.g. [`i18n`])
//! lives in modules without ImGui/Nexus so it can be unit-tested on any host;
//! the Nexus glue in `addon` only compiles for Windows.

pub mod i18n;

#[cfg(windows)]
mod addon;
