//! Dropdown (select box) shared by all addons (memo plan M87).
//!
//! ImGui's plain combo does not follow the theme (label centering, borders,
//! the chosen option marked by color only), so the field is a themed button
//! that opens a popup list below it. The field has the width of its widest
//! option, so it does not jump when the choice changes.

use nexus::imgui::{ImColor32, Selectable, StyleColor, Ui};

use crate::{
    button::draw_label, color::Color, font_registry::FontScope, fonts::FontRole, icons::Icon,
    theme::Theme,
};

/// The width of a dropdown with `options`: the widest option plus padding,
/// a gap and the chevron, e.g. to right-align it or to decide whether it
/// fits next to something else.
#[must_use]
pub fn dropdown_width(ui: &Ui, options: &[&str]) -> f32 {
    let style = ui.clone_style();
    let widest = options
        .iter()
        .map(|option| ui.calc_text_size(option)[0])
        .fold(0.0, f32::max);
    widest + 2.0 * style.frame_padding[0] + style.item_spacing[0] + icon_width(ui)
}

/// Draws a dropdown showing `options[selected]` and returns the index the
/// user picked, if it differs from `selected`. `id` must be unique in the
/// current ID scope; `tooltip` is shown while the field is hovered (the
/// options alone may not say what is chosen, `a11y-icon-tooltip`).
pub fn draw_dropdown(
    ui: &Ui,
    theme: &Theme,
    id: &str,
    tooltip: &str,
    options: &[&str],
    selected: usize,
) -> Option<usize> {
    let width = dropdown_width(ui, options);
    let style = ui.clone_style();
    let _id = ui.push_id(id);
    let popup_id = "##options";

    let clicked = {
        let _colors = [
            ui.push_style_color(StyleColor::Button, theme.surface.input.to_array()),
            ui.push_style_color(StyleColor::ButtonHovered, theme.surface.hover.to_array()),
            ui.push_style_color(StyleColor::ButtonActive, theme.surface.active.to_array()),
        ];
        ui.button_with_size("##field", [width, ui.frame_height()])
    };
    let is_hovered = ui.is_item_hovered();
    let [min, max] = [ui.item_rect_min(), ui.item_rect_max()];
    if clicked {
        ui.open_popup(popup_id);
    }

    // The popup rows need room for the widest option, a gap and the check
    // mark inside the popup's own padding.
    let popup_width =
        width.max(width - 2.0 * style.frame_padding[0] + 2.0 * style.window_padding[0]);
    // SAFETY: plain ImGui calls on the render thread inside a frame; they
    // only set the position and minimum size of the next window (the popup).
    unsafe {
        nexus::imgui::sys::igSetNextWindowPos(
            nexus::imgui::sys::ImVec2 {
                x: min[0],
                y: max[1],
            },
            0,
            nexus::imgui::sys::ImVec2 { x: 0.0, y: 0.0 },
        );
        nexus::imgui::sys::igSetNextWindowSizeConstraints(
            nexus::imgui::sys::ImVec2 {
                x: popup_width,
                y: 0.0,
            },
            nexus::imgui::sys::ImVec2 {
                x: f32::MAX,
                y: f32::MAX,
            },
            None,
            std::ptr::null_mut(),
        );
    }
    let mut picked = None;
    let mut is_open = false;
    ui.popup(popup_id, || {
        is_open = true;
        for (index, option) in options.iter().enumerate() {
            let is_selected = index == selected;
            let _row = ui.push_id(i32::try_from(index).unwrap_or(i32::MAX));
            if Selectable::new(option).selected(is_selected).build(ui) {
                picked = Some(index);
            }
            // The chosen option also gets a check mark, not only a color
            // (`a11y-color-alone`).
            if is_selected {
                let [row_min, row_max] = [ui.item_rect_min(), ui.item_rect_max()];
                draw_icon(
                    ui,
                    Icon::CHECK,
                    row_max[0] - icon_width(ui),
                    [row_min[1], row_max[1]],
                    packed(theme.accent.base),
                );
            }
        }
    });
    if is_hovered && !is_open {
        ui.tooltip_text(tooltip);
    }
    let border = if is_open {
        theme.border.focus
    } else {
        theme.border.color
    };
    draw_border(min, max, border, theme.border.width.max(1.0));
    if let Some(label) = options.get(selected) {
        draw_label(
            label,
            min[0] + style.frame_padding[0],
            [min[1], max[1]],
            packed(theme.text.primary),
        );
    }
    draw_icon(
        ui,
        Icon::CHEVRON_DOWN,
        max[0] - style.frame_padding[0] - icon_width(ui),
        [min[1], max[1]],
        packed(theme.text.secondary),
    );
    picked.filter(|&index| index != selected)
}

/// The width of one icon glyph at the current font size.
pub(crate) fn icon_width(ui: &Ui) -> f32 {
    let _icons = FontScope::push(ui, FontRole::Icons);
    let mut buffer = [0; 4];
    ui.calc_text_size(Icon::CHEVRON_DOWN.as_str(&mut buffer))[0]
}

/// Draws `icon` at `x`, centered between `top` and `bottom` by its line box
/// (Lucide glyphs fill the whole line box, memo plan M85), into the current
/// window's draw list.
fn draw_icon(ui: &Ui, icon: Icon, x: f32, vertical_span: [f32; 2], color: u32) {
    // SAFETY: called on the render thread inside a frame, so the current
    // window's draw list is valid.
    let draw_list = unsafe { nexus::imgui::sys::igGetWindowDrawList() };
    draw_icon_into(draw_list, ui, icon, x, vertical_span, color);
}

/// Like [`draw_icon`], into `draw_list` (e.g. a parent menu's list while a
/// submenu window is current).
pub(crate) fn draw_icon_into(
    draw_list: *mut nexus::imgui::sys::ImDrawList,
    ui: &Ui,
    icon: Icon,
    x: f32,
    [top, bottom]: [f32; 2],
    color: u32,
) {
    let _icons = FontScope::push(ui, FontRole::Icons);
    let mut buffer = [0; 4];
    let glyph = icon.as_str(&mut buffer);
    let height = ui.text_line_height();
    let y = ((top + bottom - height) / 2.0).round();
    let begin = glyph.as_ptr().cast::<std::ffi::c_char>();
    // SAFETY: called on the render thread inside a frame with a draw list
    // of this frame; `begin..begin + len` is the UTF-8 glyph, which ImGui
    // only reads.
    unsafe {
        nexus::imgui::sys::ImDrawList_AddText_Vec2(
            draw_list,
            nexus::imgui::sys::ImVec2 { x, y },
            color,
            begin,
            begin.add(glyph.len()),
        );
    }
}

/// Draws a rectangle outline from `min` to `max` (the field's border).
fn draw_border(min: [f32; 2], max: [f32; 2], color: Color, thickness: f32) {
    // SAFETY: called on the render thread inside a frame, so the current
    // window's draw list is valid; the call only adds a rectangle.
    unsafe {
        nexus::imgui::sys::ImDrawList_AddRect(
            nexus::imgui::sys::igGetWindowDrawList(),
            nexus::imgui::sys::ImVec2 {
                x: min[0],
                y: min[1],
            },
            nexus::imgui::sys::ImVec2 {
                x: max[0],
                y: max[1],
            },
            packed(color),
            0.0,
            0,
            thickness,
        );
    }
}

/// `color` as packed `ImColor32` bits for raw draw list calls.
pub(crate) fn packed(color: Color) -> u32 {
    ImColor32::from(color.to_array()).to_bits()
}
