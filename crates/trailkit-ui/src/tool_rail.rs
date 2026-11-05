//! The tool rail: a narrow vertical strip of icon toggles on the left edge
//! of a window, like the activity bar of VS Code (memo M136, ROADMAP R78).
//!
//! The rail never wraps: more tools stack downwards. Tools sit at the top,
//! window-wide entries such as settings at the bottom. Its toggles use the
//! neutral toggle style, never the brass accent (R79). Whether the rail is
//! shown is decided by the addon (a title bar toggle in memo).

use nexus::imgui::{ChildWindow, StyleColor, StyleVar, Ui};

use crate::{
    button::{draw_icon_toggle, icon_button_width},
    icons::Icon,
    theme::Theme,
};

/// One toggle of the rail.
#[derive(Debug, Clone, Copy)]
pub struct ToolRailItem<'texts> {
    /// The Lucide icon.
    pub icon: Icon,
    /// Tooltip naming the tool.
    pub tooltip: &'texts str,
    /// Whether the tool is on (e.g. the search field is shown).
    pub active: bool,
}

/// Which rail item was clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolRailClick {
    /// The item at this index of the top group.
    Top(usize),
    /// The item at this index of the bottom group.
    Bottom(usize),
}

/// The width of the rail: one icon button.
#[must_use]
pub fn tool_rail_width(ui: &Ui) -> f32 {
    icon_button_width(ui, Icon::SEARCH)
}

/// Draws the rail with `top` items stacked from the top and `bottom` items
/// stacked against the bottom, over the full available height, and returns
/// the clicked item. Leaves the cursor on the same line, right of the rail.
pub fn draw_tool_rail(
    ui: &Ui,
    theme: &Theme,
    top: &[ToolRailItem<'_>],
    bottom: &[ToolRailItem<'_>],
) -> Option<ToolRailClick> {
    let width = tool_rail_width(ui);
    let height = ui.content_region_avail()[1];
    let mut clicked = None;
    ChildWindow::new("tool_rail")
        .size([width, height])
        .draw_background(false)
        .scrollable(false)
        .scroll_bar(false)
        .movable(false)
        .build(ui, || {
            // The same gap as the title bar toggles above the rail.
            let _spacing = ui.push_style_var(StyleVar::ItemSpacing([0.0, theme.spacing.small]));
            let _muted = ui.push_style_color(StyleColor::Text, theme.text.secondary.to_array());
            for (index, item) in top.iter().enumerate() {
                if draw_icon_toggle(ui, theme, item.icon, item.tooltip, item.active) {
                    clicked = Some(ToolRailClick::Top(index));
                }
            }
            if bottom.is_empty() {
                return;
            }
            let bottom_height = bottom.len() as f32 * ui.frame_height()
                + (bottom.len() - 1) as f32 * theme.spacing.small;
            let bottom_start = (height - bottom_height).max(ui.cursor_pos()[1]);
            ui.set_cursor_pos([ui.cursor_pos()[0], bottom_start]);
            for (index, item) in bottom.iter().enumerate() {
                if draw_icon_toggle(ui, theme, item.icon, item.tooltip, item.active) {
                    clicked = Some(ToolRailClick::Bottom(index));
                }
            }
        });
    clicked
}
