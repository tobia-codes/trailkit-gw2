//! Panes: content regions inside a window (trailkit-ui skill §3,
//! `lay-pane-padding`).
//!
//! A pane is an ImGui child window with the `bg.panel` background and
//! `pane.padding` on every side. ImGui gives borderless child windows no
//! padding at all, so every sub-region of a trailkit window is drawn through
//! [`draw_pane`] instead of a bare `ChildWindow`.

use nexus::imgui::{ChildWindow, StyleVar, Ui};

use crate::theme::Theme;

/// Draws a pane of `size` (ImGui child-window sizing: `0.0` fills the
/// remaining space, negative values leave that much space free) and runs
/// `content` inside it.
///
/// Returns the result of `content`, or `None` if the pane is fully clipped
/// and `content` was not called.
pub fn draw_pane<R>(
    ui: &Ui,
    theme: &Theme,
    id: &str,
    size: [f32; 2],
    content: impl FnOnce() -> R,
) -> Option<R> {
    // ImGui reads the padding once in `BeginChild`; popping right after it
    // keeps the window padding for popups and nested windows unchanged.
    let padding = ui.push_style_var(StyleVar::WindowPadding([
        theme.pane.padding,
        theme.pane.padding,
    ]));
    let token = ChildWindow::new(id)
        .size(size)
        .always_use_window_padding(true)
        .begin(ui);
    padding.pop();
    token.map(|_pane| content())
}
