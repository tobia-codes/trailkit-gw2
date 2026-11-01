//! A text editor with word wrap for ImGui (memo plan M58–M60).
//!
//! ImGui's multiline input cannot wrap lines, so trailkit draws its own
//! editor. The platform-independent model: [`buffer`] (text, cursor,
//! selection, undo), [`layout`] (word wrap, position mapping) and
//! [`highlight`] (light markdown highlighting); `view` (Windows only) draws
//! it with ImGui and handles input.

pub mod buffer;
pub mod highlight;
pub mod layout;
#[cfg(windows)]
pub mod view;
