//! Dropdown (select box) shared by all addons (memo plan M87).
//!
//! ImGui's plain combo does not follow the theme (label centering, borders,
//! the chosen option marked by color only), so the selected content sits in
//! a themed button that opens the option list below it. The dropdown has the
//! width of its widest option, so it does not jump when the choice changes,
//! but at most `button.dropdown_max_width` and the room it has: a longer
//! option is cut with "…" in the field and wraps on two lines in the list
//! (R128).

use crate::controls::hover::is_item_hovered;
use crate::controls::tooltip::{draw_tooltip, draw_tooltip_with};
use crate::text::{TextStyle, draw_text, draw_text_wrapped};
use nexus::imgui::{ImColor32, Selectable, Style, StyleColor, Ui};

use crate::{
    color::Color,
    controls::button::draw_label,
    controls::menu::{
        draw_fitted_popup, draw_menu_row_highlight, draw_menu_row_label, menu_row_content_right,
        place_next_popup,
    },
    text::font_registry::FontScope,
    text::fonts::FontRole,
    text::glyph_metrics::icon_visible_box,
    text::icons::Icon,
    text::text_layout::{truncate_to_width, wrap_to_two_lines},
    theme::Theme,
};

/// The width of a dropdown with `options`: the widest option plus padding,
/// a `spacing.extra_large` gap and the chevron (roomy, not squeezed: R127),
/// at most `button.dropdown_max_width` (R128), e.g. to right-align it or to
/// decide whether it fits next to something else.
#[must_use]
pub fn dropdown_width(ui: &Ui, theme: &Theme, options: &[&str]) -> f32 {
    let style = ui.clone_style();
    let widest = options
        .iter()
        .map(|option| ui.calc_text_size(option)[0])
        .fold(0.0, f32::max);
    let natural =
        widest + 2.0 * style.frame_padding[0] + theme.spacing.extra_large + icon_width(ui);
    natural.min(theme.button.dropdown_max_width_for(ui.current_font_size()))
}

/// Draws a dropdown showing `options[selected]` and returns the index the
/// user picked, if it differs from `selected`. `id` must be unique in the
/// current ID scope; `tooltip` is shown while the field is hovered (the
/// options alone may not say what is chosen, `a11y-icon-tooltip`).
///
/// There is one dropdown for every place (R71): button height, filled like a
/// button (`bg.hover`) with a `border` frame. It never grows beyond the room
/// left in the current line (R128); a cut option shows in full in the
/// tooltip.
pub fn draw_dropdown(
    ui: &Ui,
    theme: &Theme,
    id: &str,
    tooltip: &str,
    options: &[&str],
    selected: usize,
) -> Option<usize> {
    let available = ui.content_region_avail()[0];
    let width = dropdown_width(ui, theme, options).min(available.max(icon_width(ui)));
    let style = ui.clone_style();
    let _id = ui.push_id(id);

    // Filled and hovered like a button (`StyleScope` colors).
    let clicked = ui.button_with_size("##selected_content", [width, ui.frame_height()]);
    let is_hovered = is_item_hovered(ui);
    let bounds = [ui.item_rect_min(), ui.item_rect_max()];
    if clicked {
        ui.open_popup(OPTION_LIST_ID);
    }
    let (picked, is_open) = draw_option_list(ui, theme, &style, options, selected, bounds);
    let label = options.get(selected).copied();
    let shown = draw_selected_content(ui, theme, &style, label, bounds, is_open);
    if is_hovered && !is_open {
        match label {
            // Cut in the field: the tooltip names the full option (R128).
            Some(label) if shown.len() < label.len() => draw_tooltip_with(ui, theme, || {
                // Wrapped at the field's maximum width, so it stays a block.
                let wrap = ui.cursor_pos()[0]
                    + theme.button.dropdown_max_width_for(ui.current_font_size());
                let _wrap = ui.push_text_wrap_pos_with_pos(wrap);
                draw_text(ui, theme, TextStyle::Body, tooltip);
                draw_text_wrapped(ui, theme, TextStyle::Body, label);
            }),
            _ => draw_tooltip(ui, theme, tooltip),
        }
    }
    picked.filter(|&index| index != selected)
}

/// ImGui ID of the popup that holds the option list.
const OPTION_LIST_ID: &str = "##options";

/// Draws the open option list below the dropdown at `bounds` (its min and
/// max corner), if it is open: every option as a menu row, the chosen one
/// highlighted and check-marked. Returns the option the user clicked, if
/// any, and whether the list is open this frame.
fn draw_option_list(
    ui: &Ui,
    theme: &Theme,
    style: &Style,
    options: &[&str],
    selected: usize,
    [min, max]: [[f32; 2]; 2],
) -> (Option<usize>, bool) {
    let width = max[0] - min[0];
    // The rows need room for the widest option, a gap and the check mark
    // inside the popup's own padding, plus the row padding the check mark
    // keeps from the row's end like in every menu (M177).
    let list_width = width
        .max(width - 2.0 * style.frame_padding[0] + 2.0 * style.window_padding[0])
        + theme.menu.row_padding_x;
    // The list opens `space.xs` below its field, so the two borders do not
    // overlap (R148).
    place_next_popup([min[0], max[1] + theme.spacing.extra_small], 0.0);
    // SAFETY: a plain ImGui call on the render thread inside a frame; it only
    // sets the size limits of the next window (the popup).
    unsafe {
        nexus::imgui::sys::igSetNextWindowSizeConstraints(
            nexus::imgui::sys::ImVec2 {
                x: list_width,
                y: 0.0,
            },
            // Exactly as wide as the field allows: the hidden full labels of
            // long options would otherwise widen it (R128).
            nexus::imgui::sys::ImVec2 {
                x: list_width,
                y: f32::MAX,
            },
            None,
            std::ptr::null_mut(),
        );
    }
    let mut picked = None;
    let mut is_open = false;
    draw_fitted_popup(ui, theme, OPTION_LIST_ID, || {
        is_open = true;
        // Room for the text: the row without the check mark and a gap. The
        // row width is explicit, so a hidden long label cannot widen it.
        let row_width = ui.content_region_avail()[0];
        let text_width = row_width - icon_width(ui) - theme.spacing.medium;
        let line_height = ui.text_line_height();
        for (index, option) in options.iter().enumerate() {
            let is_selected = index == selected;
            let _row = ui.push_id(i32::try_from(index).unwrap_or(i32::MAX));
            let origin = ui.cursor_screen_pos();
            // A long option wraps on a second line, then ends with "…" (R128).
            let (first, second) = wrap_to_two_lines(ui, option, text_width);
            let extra = if second.is_some() { line_height } else { 0.0 };
            let clicked = {
                let _hidden = ui.push_style_color(StyleColor::Text, [0.0; 4]);
                Selectable::new(option)
                    .selected(is_selected)
                    .size([row_width, line_height + extra])
                    .build(ui)
            };
            if clicked {
                picked = Some(index);
            }
            draw_menu_row_highlight(ui, theme, is_selected);
            draw_menu_row_label(origin, theme.text.primary, &first);
            if let Some(second) = &second {
                draw_menu_row_label(
                    [origin[0], origin[1] + line_height],
                    theme.text.primary,
                    second,
                );
            }
            // The chosen option also gets a check mark, not only a color
            // (`a11y-color-alone`), beside its first line.
            if is_selected {
                let [row_min, row_max] = [ui.item_rect_min(), ui.item_rect_max()];
                draw_icon(
                    ui,
                    Icon::CHECK,
                    menu_row_content_right(ui, theme) - icon_width(ui),
                    [row_min[1], row_max[1] - extra],
                    packed(theme.accent.text),
                );
            }
        }
    });
    (picked, is_open)
}

/// Draws what the closed dropdown at `bounds` (its min and max corner)
/// shows: its frame (focus color while the list is open), the selected
/// option's `label`, cut with "…" if it does not fit (R128), and the
/// chevron. Returns the label as shown.
fn draw_selected_content<'label>(
    ui: &Ui,
    theme: &Theme,
    style: &Style,
    label: Option<&'label str>,
    [min, max]: [[f32; 2]; 2],
    is_open: bool,
) -> std::borrow::Cow<'label, str> {
    let frame_padding = style.frame_padding[0];
    let border = if is_open {
        theme.border.focus
    } else {
        theme.border.color
    };
    draw_border(
        min,
        max,
        border,
        theme.border.width.max(1.0),
        theme.radius.medium,
    );
    let chevron_x = max[0] - frame_padding - icon_width(ui);
    let room = chevron_x - theme.spacing.small - (min[0] + frame_padding);
    let shown = truncate_to_width(ui, label.unwrap_or_default(), room);
    if !shown.is_empty() {
        draw_label(
            &shown,
            min[0] + frame_padding,
            [min[1], max[1]],
            packed(theme.text.primary),
        );
    }
    draw_icon(
        ui,
        Icon::CHEVRON_DOWN,
        chevron_x,
        [min[1], max[1]],
        packed(theme.text.secondary),
    );
    shown
}

/// The width of one icon glyph at the current font size.
pub(crate) fn icon_width(ui: &Ui) -> f32 {
    let _icons = FontScope::push(ui, FontRole::Icons);
    let mut buffer = [0; 4];
    ui.calc_text_size(Icon::CHEVRON_DOWN.as_str(&mut buffer))[0]
}

/// Draws `icon` at `x`, centered between `top` and `bottom` by its visible
/// pixels (memo M170, M177), into the current window's draw list.
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
    let line = ui.text_line_height();
    let ink = icon_visible_box(icon.glyph(), line);
    let y = ((top + bottom - ink[1] - ink[3]) / 2.0).round();
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
pub(crate) fn draw_border(
    min: [f32; 2],
    max: [f32; 2],
    color: Color,
    thickness: f32,
    rounding: f32,
) {
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
            rounding,
            nexus::imgui::sys::ImDrawCornerFlags_All as i32,
            thickness,
        );
    }
}

/// `color` as packed `ImColor32` bits for raw draw list calls.
pub(crate) fn packed(color: Color) -> u32 {
    ImColor32::from(color.to_array()).to_bits()
}
