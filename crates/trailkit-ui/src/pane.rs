//! Panes: content regions inside a window (trailkit-ui skill §3,
//! `lay-pane-padding`).
//!
//! A pane is an ImGui child window with the `bg.panel` background,
//! `pane.padding` on every side and a `border.highlight` line along its top
//! inner edge (R71). ImGui gives borderless child windows no
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
        // The faint `border.subtle` outline with rounded corners (memo M103).
        .border(true)
        .begin(ui);
    padding.pop();
    token.map(|_pane| {
        draw_top_highlight(ui, theme);
        content()
    })
}

/// Draws the 1 px `border.highlight` line along the top inner edge of the
/// current pane, between its rounded corners.
fn draw_top_highlight(ui: &Ui, theme: &Theme) {
    let [left, top] = ui.window_pos();
    let width = ui.window_size()[0];
    let corner = theme.radius.large;
    let y = top + theme.border.width + 0.5;
    ui.get_window_draw_list()
        .add_line(
            [left + corner, y],
            [left + width - corner, y],
            theme.border.highlight.to_array(),
        )
        .thickness(1.0)
        .build();
}
