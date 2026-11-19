//! Low-level ImGui drawing support: the draw list, the theme style scope,
//! the mouse cursor extent and screen rectangles (ROADMAP R134).

#[cfg(windows)]
pub mod cursor_extent;
#[cfg(windows)]
pub mod draw_list;
pub mod screen_rect;
#[cfg(windows)]
pub mod style_scope;
