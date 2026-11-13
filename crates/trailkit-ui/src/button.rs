//! Button variants shared by all addons (trailkit-ui skill §3).
//!
//! `lay-one-primary`: at most one primary button per window or dialog.
//! Every text button is a [`Button`] with a [`ButtonKind`] and a
//! [`ButtonState`] (R93); the `draw_*` functions are short forms.

mod spec;

pub use self::spec::{Button, ButtonKind, ButtonState};

use crate::tooltip::draw_tooltip;
use nexus::imgui::{ImColor32, StyleColor, Ui};

use crate::{
    color::Color,
    dropdown::icon_width,
    font_registry::FontScope,
    fonts::FontRole,
    glyph_metrics::{capital_middle, icon_visible_box},
    icons::Icon,
    spinner::draw_spinner,
    theme::Theme,
    tooltip::draw_reason_tooltip,
};

/// Draws a button with `label` and returns whether it was clicked. Unlike
/// `ui.button` the label is centered by its capital letters, not by the font
/// line box, in which the text font sits low (memo plan M85). The button is
/// at least `button.min_width` wide (M131).
pub fn draw_button(ui: &Ui, theme: &Theme, label: &str) -> bool {
    draw_button_with_size(ui, theme, label, [0.0, 0.0])
}

/// Like [`draw_button`] with an explicit width (`size[0]`; `0.0` fits the
/// label but keeps `button.min_width`). The height is the frame height.
pub fn draw_button_with_size(ui: &Ui, theme: &Theme, label: &str, size: [f32; 2]) -> bool {
    Button::new(label).width(size[0]).draw(ui, theme)
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

/// Like [`draw_primary_button`] with an explicit width (`size[0]`; `0.0`
/// fits the label), e.g. full width in a stacked button column.
pub fn draw_primary_button_with_size(ui: &Ui, theme: &Theme, label: &str, size: [f32; 2]) -> bool {
    Button::new(label)
        .kind(ButtonKind::Primary)
        .width(size[0])
        .draw(ui, theme)
}

/// Draws a button for a destructive action that asks first (memo M146):
/// the regular button background with a `danger` label, e.g. "Empty trash".
pub fn draw_danger_button(ui: &Ui, theme: &Theme, label: &str) -> bool {
    let _text = ui.push_style_color(StyleColor::Text, theme.status.danger.to_array());
    draw_button(ui, theme, label)
}

/// Draws the confirming button of a destructive dialog (memo M146): a
/// `danger` fill with a label in the window color, `size[0]` wide (`0.0`
/// fits the label).
pub fn draw_danger_fill_button_with_size(
    ui: &Ui,
    theme: &Theme,
    label: &str,
    size: [f32; 2],
) -> bool {
    Button::new(label)
        .kind(ButtonKind::DangerFill)
        .width(size[0])
        .draw(ui, theme)
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
    Button::new(label)
        .kind(ButtonKind::Primary)
        .icon(icon)
        .draw(ui, theme)
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
pub(crate) fn icon_label_content_width(ui: &Ui, theme: &Theme, label: &str) -> f32 {
    icon_width(ui) + icon_label_gap(theme) + ui.calc_text_size(label)[0]
}

/// The minimum width of a button with a text label in the current font
/// (`button.min_width`, M131).
fn min_text_button_width(ui: &Ui, theme: &Theme) -> f32 {
    theme.button.min_width_for(ui.current_font_size())
}

/// The gap between an icon and its label inside a button.
pub(crate) fn icon_label_gap(theme: &Theme) -> f32 {
    theme.spacing.extra_small * 1.5
}

/// Draws a borderless icon button (`bg.hover` only while hovered) with
/// `tooltip` and returns whether it was clicked.
pub fn draw_icon_button(ui: &Ui, theme: &Theme, icon: Icon, tooltip: &str) -> bool {
    draw_icon_toggle(ui, theme, icon, tooltip, false)
}

/// Like [`draw_icon_button`] on a popup or toast surface, where `bg.hover`
/// hardly differs from the fill: hovered it sits on `bg.active` (memo M180,
/// a toast's close button).
pub fn draw_popup_icon_button(ui: &Ui, theme: &Theme, icon: Icon, tooltip: &str) -> bool {
    draw_icon_button_with_hover(ui, theme, icon, tooltip, false, theme.surface.active)
}

/// Like [`draw_icon_button`] in `state` (R93): disabled it shows the icon in
/// `text.disabled` without hover or pressed state and names the reason in
/// its tooltip (the
/// reason replaces `tooltip`); busy it shows a spinner instead of the icon.
/// Reports clicks only when ready.
pub fn draw_icon_button_with_state(
    ui: &Ui,
    theme: &Theme,
    icon: Icon,
    tooltip: &str,
    state: ButtonState<'_>,
) -> bool {
    match state {
        ButtonState::Ready => draw_icon_button(ui, theme, icon, tooltip),
        // Neither is an ImGui button: no hover, no pressed state, no click.
        ButtonState::Disabled { reason } => {
            ui.dummy([icon_button_width(ui, icon), ui.frame_height()]);
            let color = ImColor32::from(theme.text.disabled.to_array()).to_bits();
            draw_icon_centered(ui, icon, [ui.item_rect_min(), ui.item_rect_max()], color);
            if ui.is_item_hovered() {
                draw_reason_tooltip(ui, theme, reason);
            }
            false
        }
        ButtonState::Busy => {
            ui.dummy([icon_button_width(ui, icon), ui.frame_height()]);
            let [min, max] = [ui.item_rect_min(), ui.item_rect_max()];
            let color = Color::from_array(ui.style_color(StyleColor::Text));
            draw_spinner(
                ui,
                [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0],
                color,
            );
            if ui.is_item_hovered() {
                draw_tooltip(ui, tooltip);
            }
            false
        }
    }
}

/// Like [`draw_icon_button`], but a toggle that keeps a state: while
/// `active` it sits on `bg.active` with a `text.primary` icon, otherwise it
/// looks like a plain icon button. Toggles never use the brass accent
/// (ROADMAP R79, tokens.md §3). The icon is centered by its visible ink, so
/// it lines up with a label centered by its capitals (memo M170).
pub fn draw_icon_toggle(ui: &Ui, theme: &Theme, icon: Icon, tooltip: &str, active: bool) -> bool {
    draw_icon_button_with_hover(ui, theme, icon, tooltip, active, theme.surface.hover)
}

/// The icon button behind [`draw_icon_toggle`] and
/// [`draw_popup_icon_button`]: `hover` is the fill while hovered and not
/// `active`.
fn draw_icon_button_with_hover(
    ui: &Ui,
    theme: &Theme,
    icon: Icon,
    tooltip: &str,
    active: bool,
    hover: Color,
) -> bool {
    let _active_text =
        active.then(|| ui.push_style_color(StyleColor::Text, theme.text.primary.to_array()));
    let resting = if active {
        theme.surface.active.to_array()
    } else {
        [0.0; 4]
    };
    let color = ImColor32::from(ui.style_color(StyleColor::Text)).to_bits();
    let clicked = {
        let _colors = [
            ui.push_style_color(StyleColor::Text, [0.0; 4]),
            ui.push_style_color(StyleColor::Button, resting),
            ui.push_style_color(
                StyleColor::ButtonHovered,
                if active { resting } else { hover.to_array() },
            ),
            ui.push_style_color(StyleColor::ButtonActive, theme.surface.active.to_array()),
        ];
        let _icons = FontScope::push(ui, FontRole::Icons);
        let mut buffer = [0; 4];
        ui.button(icon.as_str(&mut buffer))
    };
    draw_icon_centered(ui, icon, [ui.item_rect_min(), ui.item_rect_max()], color);
    if !tooltip.is_empty() && ui.is_item_hovered() {
        draw_tooltip(ui, tooltip);
    }
    clicked
}

/// Draws `icon` with its visible pixels centered in `[min, max]`, on whole
/// pixels. Neither the font's line box nor the glyph's quad will do: how the
/// ink sits in them changes with the font size.
fn draw_icon_centered(ui: &Ui, icon: Icon, [min, max]: [[f32; 2]; 2], color: u32) {
    let _icons = FontScope::push(ui, FontRole::Icons);
    let line = ui.text_line_height();
    let ink = icon_visible_box(icon.glyph(), line);
    let position = nexus::imgui::sys::ImVec2 {
        x: ((min[0] + max[0] - ink[0] - ink[2]) / 2.0).round(),
        y: ((min[1] + max[1] - ink[1] - ink[3]) / 2.0).round(),
    };
    let mut buffer = [0; 4];
    let glyph = icon.as_str(&mut buffer);
    let begin = glyph.as_ptr().cast::<std::ffi::c_char>();
    // SAFETY: called on the render thread inside a frame, where ImGui has a
    // current window and returns its draw list; `begin..begin + len` is the
    // UTF-8 glyph, which ImGui only reads.
    unsafe {
        nexus::imgui::sys::ImDrawList_AddText_Vec2(
            nexus::imgui::sys::igGetWindowDrawList(),
            position,
            color,
            begin,
            begin.add(glyph.len()),
        );
    }
}

/// The width a text button with `label` takes (at least `button.min_width`,
/// M131), for right-aligning button rows.
#[must_use]
pub fn button_width(ui: &Ui, theme: &Theme, label: &str) -> f32 {
    let natural = ui.calc_text_size(label)[0] + 2.0 * ui.clone_style().frame_padding[0];
    natural.max(min_text_button_width(ui, theme))
}
