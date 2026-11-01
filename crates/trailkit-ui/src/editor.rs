//! A text editor with word wrap for ImGui (memo plan M58–M60).
//!
//! ImGui's multiline input cannot wrap lines, so trailkit draws its own
//! editor. This module holds the platform-independent model: [`buffer`]
//! (text, cursor, selection, undo). Layout, highlighting and the ImGui view
//! follow.

pub mod buffer;
