//! Tooltips with their text centered by its capital letters (memo M171).
//!
//! ImGui pads a tooltip evenly around the font's line box, in which Open Sans
//! sits low, so its text looks about a pixel too deep. Every tooltip goes through
//! this module; clippy bans the plain ImGui calls (see `clippy.toml`).

use nexus::imgui::Ui;

use crate::{glyph_metrics::capital_middle, icons::Icon, text::draw_icon, theme::Theme};

/// Shows `text` as a tooltip at the mouse.
pub fn draw_tooltip(ui: &Ui, text: impl AsRef<str>) {
    draw_tooltip_with(ui, || ui.text(text.as_ref()));
}

/// Shows why a control is disabled (R93): a neutral Lucide `ban` icon in
/// `text.secondary` (not `danger`: it is no error) and `reason` in
/// `text.primary`, readable even inside a disabled control's colors.
pub fn draw_reason_tooltip(ui: &Ui, theme: &Theme, reason: &str) {
    draw_tooltip_with(ui, || {
        draw_icon(ui, Icon::BAN, theme.text.secondary);
        ui.same_line_with_spacing(0.0, theme.spacing.small);
        ui.text_colored(theme.text.primary.to_array(), reason);
    });
}

/// Shows a tooltip at the mouse with the items `content` draws.
#[allow(
    clippy::disallowed_methods,
    reason = "this is the one place that may open a plain ImGui tooltip"
)]
pub fn draw_tooltip_with(ui: &Ui, content: impl FnOnce()) {
    ui.tooltip(|| {
        let shift = line_box_excess(ui);
        let [left, top] = ui.cursor_screen_pos();
        ui.set_cursor_screen_pos([left, top - shift]);
        content();
        // The content moved up, so the tooltip must not end higher with it:
        // an empty item reaches as far down as the content did before moving.
        let bottom = ui.item_rect_max()[1] + shift;
        ui.set_cursor_screen_pos([left, bottom]);
        ui.dummy([0.0, 0.0]);
    });
}

/// How many whole pixels the capitals of the current font sit below the middle
/// of its line box.
fn line_box_excess(ui: &Ui) -> f32 {
    capital_middle().map_or(0.0, |middle| (middle - ui.text_line_height() / 2.0).round())
}
