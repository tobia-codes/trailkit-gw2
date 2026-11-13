//! Tooltips with their text centered by its capital letters (memo M171).
//!
//! ImGui pads a tooltip evenly around the font's line box, in which Open Sans
//! sits low, so its text looks about a pixel too deep. Tooltips have a `border`
//! outline, small corners and the design's one shadow (Q327). Every tooltip goes
//! through this module; clippy bans the plain ImGui calls (see `clippy.toml`).

use nexus::imgui::{StyleColor, StyleVar, Ui};

use crate::{
    color::Color,
    glyph_metrics::capital_middle,
    icons::Icon,
    text::{IconText, TextFlow, TextStyle, draw_icon_text},
    theme::Theme,
};

/// Shows `text` as a tooltip at the mouse.
pub fn draw_tooltip(ui: &Ui, theme: &Theme, text: impl AsRef<str>) {
    draw_tooltip_with(ui, theme, || ui.text(text.as_ref()));
}

/// Shows why a control is disabled (R93): a neutral Lucide `ban` icon in
/// `text.secondary` (not `danger`: it is no error) and `reason` in
/// `text.primary`, readable even inside a disabled control's colors.
pub fn draw_reason_tooltip(ui: &Ui, theme: &Theme, reason: &str) {
    draw_tooltip_with(ui, theme, || {
        draw_icon_text(
            ui,
            theme,
            &IconText {
                icon: Icon::BAN,
                icon_color: theme.text.secondary,
                text: reason,
                style: TextStyle::Body,
                text_color: None,
                flow: TextFlow::SingleLine,
            },
        );
    });
}

/// Shows a tooltip at the mouse with the items `content` draws (Q327): on
/// `bg.popup` with a `border` outline, `radius.sm` corners (not like a
/// button) and a hard `shadow.popup` below it.
pub fn draw_tooltip_with(ui: &Ui, theme: &Theme, content: impl FnOnce()) {
    draw_tooltip_with_border(ui, theme, theme.border.color, content);
}

/// Like [`draw_tooltip_with`] with a `border` color of its own, e.g. memo's
/// drag label, whose gold or red outline ties it to the drop target (M116).
#[allow(
    clippy::disallowed_methods,
    reason = "this is the one place that may open a plain ImGui tooltip"
)]
pub fn draw_tooltip_with_border(ui: &Ui, theme: &Theme, border: Color, content: impl FnOnce()) {
    // ImGui reads both when the tooltip window begins (and draws its frame
    // there), so they are popped right after.
    let rounding = ui.push_style_var(StyleVar::WindowRounding(theme.radius.small));
    let border = ui.push_style_color(StyleColor::Border, border.to_array());
    let mut frame = None;
    ui.tooltip(|| {
        rounding.pop();
        border.pop();
        // SAFETY: inside the tooltip window on the render thread; only reads
        // whether the window was hidden in the last frame.
        let appearing = unsafe { nexus::imgui::sys::igIsWindowAppearing() };
        // A tooltip that just appears has no size yet: no shadow this frame.
        if !appearing {
            let position = ui.window_pos();
            let size = ui.window_size();
            frame = Some([position, [position[0] + size[0], position[1] + size[1]]]);
        }
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
    if let Some(frame) = frame {
        draw_shadow(ui, theme, frame);
    }
}

/// Draws the tooltip's shadow: its shape `shadow.popup.offset` lower, on the
/// window below the tooltip, which ImGui draws first, so only the part below
/// the tooltip shows. The clip rectangle is widened, as the tooltip may
/// reach beyond that window.
fn draw_shadow(ui: &Ui, theme: &Theme, [min, max]: [[f32; 2]; 2]) {
    let offset = theme.shadow.popup_offset;
    let draw_list = ui.get_window_draw_list();
    draw_list.with_clip_rect([0.0, 0.0], ui.io().display_size, || {
        draw_list
            .add_rect(
                [min[0], min[1] + offset],
                [max[0], max[1] + offset],
                theme.shadow.popup.to_array(),
            )
            .filled(true)
            .rounding(theme.radius.small)
            .build();
    });
}

/// How many whole pixels the capitals of the current font sit below the middle
/// of its line box.
fn line_box_excess(ui: &Ui) -> f32 {
    capital_middle().map_or(0.0, |middle| (middle - ui.text_line_height() / 2.0).round())
}
