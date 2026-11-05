//! Button variants shared by all addons (trailkit-ui skill §3).
//!
//! `lay-one-primary`: at most one primary button per window or dialog.

use nexus::imgui::{ImColor32, StyleColor, Ui};

use crate::{
    dropdown::{draw_border, draw_icon_into, icon_width},
    font_registry::FontScope,
    fonts::FontRole,
    glyph_metrics::capital_middle,
    icons::Icon,
    theme::Theme,
};

/// Draws a button with `label` and returns whether it was clicked. Unlike
/// `ui.button` the label is centered by its capital letters, not by the font
/// line box, in which the text font sits low (memo plan M85). The button is
/// at least `button.min_width` wide (M131).
pub fn draw_button(ui: &Ui, theme: &Theme, label: &str) -> bool {
    draw_button_with_size(ui, theme, label, [0.0, 0.0])
}

/// Like [`draw_button`] with an explicit size (ImGui sizing: `0.0` fits the
/// label but keeps `button.min_width`, negative values fill the available
/// space).
pub fn draw_button_with_size(ui: &Ui, theme: &Theme, label: &str, size: [f32; 2]) -> bool {
    let clicked = draw_filled_button(ui, theme, label, size);
    // The fill alone can vanish on `bg.window` (R85), like on a dropdown.
    draw_border(
        ui.item_rect_min(),
        ui.item_rect_max(),
        theme.border.color,
        theme.border.width,
        theme.radius.medium,
    );
    clicked
}

/// Draws a button without a frame, in the current `Button*` colors, for
/// [`draw_button_with_size`] and the fills that stand out on their own.
fn draw_filled_button(ui: &Ui, theme: &Theme, label: &str, size: [f32; 2]) -> bool {
    let padding = ui.clone_style().frame_padding;
    let text_size = ui.calc_text_size(label);
    let size = [
        if size[0] == 0.0 {
            button_width(ui, theme, label)
        } else {
            size[0]
        },
        if size[1] == 0.0 {
            text_size[1] + 2.0 * padding[1]
        } else {
            size[1]
        },
    ];
    // The label is the ID only; ImGui draws the frame, the label follows.
    let _id = ui.push_id(label);
    let clicked = ui.button_with_size("##button", size);
    let [min, max] = [ui.item_rect_min(), ui.item_rect_max()];
    let x = min[0] + ((max[0] - min[0] - text_size[0]) / 2.0).max(padding[0]);
    let color = ImColor32::from(ui.style_color(StyleColor::Text)).to_bits();
    draw_label(label, x, [min[1], max[1]], color);
    clicked
}

/// Draws `label` starting at `x`, vertically centered by its capital letters
/// between `top` and `bottom` (memo plan M85), in the current font and
/// `color` (packed `ImColor32` bits). Works while a caller holds the window's
/// `DrawListMut` (imgui-rs allows only one at a time, e.g. in a dialog card).
pub(crate) fn draw_label(label: &str, x: f32, [top, bottom]: [f32; 2], color: u32) {
    // SAFETY: only reads ImGui's current font size on the render thread.
    let line_height = unsafe { nexus::imgui::sys::igGetFontSize() };
    let middle = capital_middle().unwrap_or(line_height / 2.0);
    let y = ((top + bottom) / 2.0 - middle).round();
    let begin = label.as_ptr().cast::<std::ffi::c_char>();
    // SAFETY: called on the render thread inside a frame, so the current
    // window's draw list is valid; `begin..begin + len` is the UTF-8 label,
    // which ImGui only reads.
    unsafe {
        nexus::imgui::sys::ImDrawList_AddText_Vec2(
            nexus::imgui::sys::igGetWindowDrawList(),
            nexus::imgui::sys::ImVec2 { x, y },
            color,
            begin,
            begin.add(label.len()),
        );
    }
}

/// Draws a primary button (accent background, `text.on_accent` label) and
/// returns whether it was clicked.
pub fn draw_primary_button(ui: &Ui, theme: &Theme, label: &str) -> bool {
    draw_primary_button_with_size(ui, theme, label, [0.0, 0.0])
}

/// Like [`draw_primary_button`] with an explicit size (ImGui sizing: `0.0`
/// fits the label), e.g. full width in a stacked button column.
pub fn draw_primary_button_with_size(ui: &Ui, theme: &Theme, label: &str, size: [f32; 2]) -> bool {
    let _colors = [
        ui.push_style_color(StyleColor::Button, theme.accent.base.to_array()),
        ui.push_style_color(StyleColor::ButtonHovered, theme.accent.hover.to_array()),
        ui.push_style_color(StyleColor::ButtonActive, theme.accent.active.to_array()),
        ui.push_style_color(StyleColor::Text, theme.text.on_accent.to_array()),
    ];
    draw_filled_button(ui, theme, label, size)
}

/// Draws a button for a destructive action that asks first (memo M146):
/// the regular button background with a `danger` label, e.g. "Empty trash".
pub fn draw_danger_button(ui: &Ui, theme: &Theme, label: &str) -> bool {
    let _text = ui.push_style_color(StyleColor::Text, theme.status.danger.to_array());
    draw_button(ui, theme, label)
}

/// Draws the confirming button of a destructive dialog (memo M146): a
/// `danger` fill with a label in the window color, at `size` (ImGui sizing,
/// `0.0` fits the label).
pub fn draw_danger_fill_button_with_size(
    ui: &Ui,
    theme: &Theme,
    label: &str,
    size: [f32; 2],
) -> bool {
    let danger = theme.status.danger;
    let _colors = [
        ui.push_style_color(StyleColor::Button, danger.to_array()),
        ui.push_style_color(
            StyleColor::ButtonHovered,
            danger.with_alpha(0.85).to_array(),
        ),
        ui.push_style_color(StyleColor::ButtonActive, danger.with_alpha(0.7).to_array()),
        ui.push_style_color(StyleColor::Text, theme.surface.window.to_array()),
    ];
    draw_filled_button(ui, theme, label, size)
}

/// Like [`draw_icon_button`] with a `danger` icon, for a destructive action
/// that asks first (memo M145, "Delete permanently").
pub fn draw_danger_icon_button(ui: &Ui, theme: &Theme, icon: Icon, tooltip: &str) -> bool {
    let _text = ui.push_style_color(StyleColor::Text, theme.status.danger.to_array());
    draw_icon_button(ui, theme, icon, tooltip)
}

/// Draws a primary button with a Lucide `icon` before `label` (accent
/// background, `text.on_accent` icon and label; R71) and returns whether it
/// was clicked.
pub fn draw_primary_button_with_icon(ui: &Ui, theme: &Theme, icon: Icon, label: &str) -> bool {
    let _colors = [
        ui.push_style_color(StyleColor::Button, theme.accent.base.to_array()),
        ui.push_style_color(StyleColor::ButtonHovered, theme.accent.hover.to_array()),
        ui.push_style_color(StyleColor::ButtonActive, theme.accent.active.to_array()),
        ui.push_style_color(StyleColor::Text, theme.text.on_accent.to_array()),
    ];
    let gap = icon_label_gap(theme);
    let content_width = icon_label_content_width(ui, theme, label);
    let size = [icon_label_button_width(ui, theme, label), ui.frame_height()];
    let _id = ui.push_id(label);
    let clicked = ui.button_with_size("##button", size);
    let [min, max] = [ui.item_rect_min(), ui.item_rect_max()];
    let color = ImColor32::from(ui.style_color(StyleColor::Text)).to_bits();
    // SAFETY: called on the render thread inside a frame, where ImGui has a
    // current window and returns its draw list.
    let draw_list = unsafe { nexus::imgui::sys::igGetWindowDrawList() };
    // Icon and label are centered as one group when the minimum width makes
    // the button wider than its content (M131).
    let icon_x = (min[0] + (max[0] - min[0] - content_width) / 2.0).round();
    draw_icon_into(draw_list, ui, icon, icon_x, [min[1], max[1]], color);
    draw_label(
        label,
        icon_x + icon_width(ui) + gap,
        [min[1], max[1]],
        color,
    );
    clicked
}

/// The width of an icon button showing `icon`.
#[must_use]
pub fn icon_button_width(ui: &Ui, icon: Icon) -> f32 {
    let _icons = FontScope::push(ui, FontRole::Icons);
    let mut buffer = [0; 4];
    ui.calc_text_size(icon.as_str(&mut buffer))[0] + 2.0 * ui.clone_style().frame_padding[0]
}

/// The width of a button with an icon and `label` (at least
/// `button.min_width`, M131), for right-aligning it.
#[must_use]
pub fn icon_label_button_width(ui: &Ui, theme: &Theme, label: &str) -> f32 {
    let padding = ui.clone_style().frame_padding[0];
    (2.0 * padding + icon_label_content_width(ui, theme, label))
        .max(min_text_button_width(ui, theme))
}

/// The width of icon, gap and `label` inside a button, without padding.
fn icon_label_content_width(ui: &Ui, theme: &Theme, label: &str) -> f32 {
    icon_width(ui) + icon_label_gap(theme) + ui.calc_text_size(label)[0]
}

/// The minimum width of a button with a text label in the current font
/// (`button.min_width`, M131).
fn min_text_button_width(ui: &Ui, theme: &Theme) -> f32 {
    theme.button.min_width_for(ui.current_font_size())
}

/// The gap between an icon and its label inside a button.
fn icon_label_gap(theme: &Theme) -> f32 {
    theme.spacing.extra_small * 1.5
}

/// Draws a borderless icon button (`bg.hover` only while hovered) with
/// `tooltip` and returns whether it was clicked.
pub fn draw_icon_button(ui: &Ui, theme: &Theme, icon: Icon, tooltip: &str) -> bool {
    draw_icon_toggle(ui, theme, icon, tooltip, false)
}

/// Like [`draw_icon_button`], but a toggle that keeps a state: while
/// `active` it sits on `bg.active` with a `text.primary` icon, otherwise it
/// looks like a plain icon button. Toggles never use the brass accent
/// (ROADMAP R79, tokens.md §3).
pub fn draw_icon_toggle(ui: &Ui, theme: &Theme, icon: Icon, tooltip: &str, active: bool) -> bool {
    let _active_text =
        active.then(|| ui.push_style_color(StyleColor::Text, theme.text.primary.to_array()));
    let resting = if active {
        theme.surface.active.to_array()
    } else {
        [0.0; 4]
    };
    let clicked = {
        let _colors = [
            ui.push_style_color(StyleColor::Button, resting),
            ui.push_style_color(
                StyleColor::ButtonHovered,
                if active {
                    resting
                } else {
                    theme.surface.hover.to_array()
                },
            ),
            ui.push_style_color(StyleColor::ButtonActive, theme.surface.active.to_array()),
        ];
        let _icons = FontScope::push(ui, FontRole::Icons);
        let mut buffer = [0; 4];
        ui.button(icon.as_str(&mut buffer))
    };
    if ui.is_item_hovered() {
        ui.tooltip_text(tooltip);
    }
    clicked
}

/// The width a text button with `label` takes (at least `button.min_width`,
/// M131), for right-aligning button rows.
#[must_use]
pub fn button_width(ui: &Ui, theme: &Theme, label: &str) -> f32 {
    let natural = ui.calc_text_size(label)[0] + 2.0 * ui.clone_style().frame_padding[0];
    natural.max(min_text_button_width(ui, theme))
}
