//! Menu entries for context menus shared by all addons (memo plan M98, M99).
//!
//! ImGui draws its own check mark and submenu arrow; these entries use the
//! Lucide `check` and `chevron-right` icons like the dropdown, and submenus
//! open flush beside their parent instead of overlapping it.

use crate::tooltip::draw_tooltip;
use nexus::imgui::{ItemHoveredFlags, MenuItem, MenuToken, StyleColor, StyleVar, Ui};

use crate::{
    color::Color,
    dropdown::{draw_icon_into, icon_width, packed},
    icons::Icon,
    selection_bar::hide_square_row_highlight,
    theme::Theme,
};

/// Draws the popup `id` if it is open, with `content` inside it. Every popup
/// goes through here: ImGui's own square row highlight is hidden, and the
/// entries of this module draw a rounded one in `bg.popup_hover` instead
/// (ROADMAP R80, R81).
pub fn draw_popup(ui: &Ui, id: &str, content: impl FnOnce()) {
    ui.popup(id, || {
        let _square = hide_square_row_highlight(ui);
        content();
    });
}

/// Places the next popup with its top at `position` (screen). `pivot_x` is the
/// share of the popup's width left of `position`: 0 lines up its left edge
/// (a dropdown), 1 its right edge (a menu at the end of a toolbar). Call it
/// right before [`draw_popup`], every frame.
pub fn place_next_popup(position: [f32; 2], pivot_x: f32) {
    // SAFETY: a plain ImGui call on the render thread inside a frame; it only
    // sets the position of the next window.
    unsafe {
        nexus::imgui::sys::igSetNextWindowPos(
            nexus::imgui::sys::ImVec2 {
                x: position[0],
                y: position[1],
            },
            0,
            nexus::imgui::sys::ImVec2 { x: pivot_x, y: 0.0 },
        );
    }
}

/// Fills the menu row `[min, max]` (screen) in `color` with `radius.md`
/// corners into `draw_list`.
///
/// # Safety
/// `draw_list` must be a draw list of the current frame, used on the render
/// thread.
unsafe fn fill_menu_row(
    draw_list: *mut nexus::imgui::sys::ImDrawList,
    theme: &Theme,
    [min, max]: [[f32; 2]; 2],
    color: Color,
) {
    // SAFETY: guaranteed by the caller; the call only adds a rectangle.
    unsafe {
        nexus::imgui::sys::ImDrawList_AddRectFilled(
            draw_list,
            nexus::imgui::sys::ImVec2 {
                x: min[0],
                y: min[1],
            },
            nexus::imgui::sys::ImVec2 {
                x: max[0],
                y: max[1],
            },
            packed(color),
            theme.radius.medium,
            nexus::imgui::sys::ImDrawCornerFlags_All as i32,
        );
    }
}

/// Draws the rounded highlight of the menu row that was just drawn:
/// `bg.pressed` while pressed, `bg.popup_hover` while hovered, else
/// `bg.active` while `selected` (R80, R81, R85). Call it right after the item,
/// before its label.
pub(crate) fn draw_menu_row_highlight(ui: &Ui, theme: &Theme, selected: bool) {
    let color = if ui.is_item_active() {
        theme.surface.pressed
    } else if ui.is_item_hovered() {
        theme.surface.popup_hover
    } else if selected {
        theme.surface.active
    } else {
        return;
    };
    // SAFETY: called on the render thread inside a frame, so the current
    // window's draw list is valid.
    unsafe {
        fill_menu_row(
            nexus::imgui::sys::igGetWindowDrawList(),
            theme,
            [ui.item_rect_min(), ui.item_rect_max()],
            color,
        );
    }
}

/// Draws the label of a menu row at `origin` (screen). ImGui's own label is
/// hidden because it would lie below the rounded highlight.
pub(crate) fn draw_menu_row_label(origin: [f32; 2], color: Color, label: &str) {
    let begin = label.as_ptr().cast::<std::ffi::c_char>();
    // SAFETY: called on the render thread inside a frame, so the current
    // window's draw list is valid; `begin..begin + len` is the UTF-8 label,
    // which ImGui only reads.
    unsafe {
        nexus::imgui::sys::ImDrawList_AddText_Vec2(
            nexus::imgui::sys::igGetWindowDrawList(),
            nexus::imgui::sys::ImVec2 {
                x: origin[0],
                y: origin[1],
            },
            packed(color),
            begin,
            begin.add(label.len()),
        );
    }
}

/// Draws a menu item and returns whether it was clicked. A `checked` item
/// shows a check mark in the accent color at its right end.
pub fn draw_menu_item(ui: &Ui, theme: &Theme, label: &str, checked: bool) -> bool {
    let origin = ui.cursor_screen_pos();
    let clicked = {
        let _hidden = ui.push_style_color(StyleColor::Text, [0.0; 4]);
        MenuItem::new(label).build(ui)
    };
    draw_menu_row_highlight(ui, theme, false);
    draw_menu_row_label(origin, theme.text.primary, label);
    if checked {
        let [min, max] = [ui.item_rect_min(), ui.item_rect_max()];
        // SAFETY: called on the render thread inside a frame, so the
        // current window's draw list is valid.
        let draw_list = unsafe { nexus::imgui::sys::igGetWindowDrawList() };
        draw_icon_into(
            draw_list,
            ui,
            Icon::CHECK,
            max[0] - icon_width(ui),
            [min[1], max[1]],
            packed(theme.accent.text),
        );
    }
    clicked
}

/// How a menu item with an icon looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItemTone {
    /// `text.primary` label and icon.
    Regular,
    /// `danger` label and icon, for a destructive item that asks first
    /// (memo M146).
    Danger,
}

/// Draws a menu item with a Lucide `icon` before `label` and returns
/// whether it was clicked (memo M133). A disabled item is drawn in
/// `text.disabled` and shows `disabled_tooltip` while hovered (M141).
pub fn draw_menu_item_with_icon(
    ui: &Ui,
    theme: &Theme,
    icon: Icon,
    label: &str,
    tone: MenuItemTone,
    disabled_tooltip: Option<&str>,
) -> bool {
    let enabled = disabled_tooltip.is_none();
    // ImGui draws the label at the cursor; the item rect is no substitute
    // because it reaches half the item spacing further up and left.
    let origin = ui.cursor_screen_pos();
    // ImGui keeps room for a check mark after the label; the label moves
    // right by the icon, which fits into that room.
    let clicked = {
        // A disabled item draws its label in `TextDisabled`, not `Text`.
        let _hidden = [
            ui.push_style_color(StyleColor::Text, [0.0; 4]),
            ui.push_style_color(StyleColor::TextDisabled, [0.0; 4]),
        ];
        MenuItem::new(label).enabled(enabled).build(ui)
    };
    // ImGui reports the hover of a disabled item once the item has ended.
    if enabled {
        draw_menu_row_highlight(ui, theme, false);
    }
    let color = match (enabled, tone) {
        (false, _) => theme.text.disabled,
        (true, MenuItemTone::Regular) => theme.text.primary,
        (true, MenuItemTone::Danger) => theme.status.danger,
    };
    // SAFETY: called on the render thread inside a frame, so the current
    // window's draw list is valid.
    let draw_list = unsafe { nexus::imgui::sys::igGetWindowDrawList() };
    let line = [origin[1], origin[1] + ui.text_line_height()];
    draw_icon_into(draw_list, ui, icon, origin[0], line, packed(color));
    let label_x = origin[0] + icon_width(ui) + theme.spacing.extra_small * 1.5;
    draw_menu_row_label([label_x, origin[1]], color, label);
    if let Some(tooltip) = disabled_tooltip
        && ui.is_item_hovered_with_flags(ItemHoveredFlags::ALLOW_WHEN_DISABLED)
    {
        draw_tooltip(ui, tooltip);
    }
    clicked && enabled
}

/// Begins a submenu entry labeled `label` with a chevron at its right end;
/// returns a token while the submenu is open (draw its items, then drop the
/// token). The submenu starts at the parent's right edge (memo M98).
pub fn begin_submenu<'ui>(ui: &'ui Ui, theme: &Theme, label: &str) -> Option<MenuToken<'ui>> {
    let style = ui.clone_style();
    let [x, y] = ui.cursor_screen_pos();
    let right_edge = ui.window_content_region_max()[0] + ui.window_pos()[0];
    let bottom = y + ui.text_line_height();
    // SAFETY: called on the render thread inside a frame, so the current
    // (parent menu) window's draw list is valid for this frame.
    let draw_list = unsafe { nexus::imgui::sys::igGetWindowDrawList() };
    let token = {
        // ImGui lets a submenu overlap its parent by `ItemInnerSpacing.x`;
        // only `BeginMenu` reads it. ImGui's own label and arrow are drawn
        // transparent and replaced below.
        let _flush = ui.push_style_var(StyleVar::ItemInnerSpacing([
            0.0,
            style.item_inner_spacing[1],
        ]));
        let _hidden = ui.push_style_color(StyleColor::Text, [0.0; 4]);
        ui.begin_menu(label)
    };
    // While the submenu is open the current window is the submenu, so the
    // row's rect cannot be read from the last item: it is the same rect
    // ImGui gives a selectable, reaching half the item spacing outwards.
    if token.is_some() || ui.is_item_hovered() {
        let half_spacing = [
            (style.item_spacing[0] * 0.5).floor(),
            (style.item_spacing[1] * 0.5).floor(),
        ];
        let row = [
            [x - half_spacing[0], y - half_spacing[1]],
            [
                right_edge + style.item_spacing[0] - half_spacing[0],
                bottom + style.item_spacing[1] - half_spacing[1],
            ],
        ];
        // SAFETY: render thread inside a frame; `draw_list` belongs to the
        // parent menu of this frame.
        unsafe { fill_menu_row(draw_list, theme, row, theme.surface.popup_hover) };
    }
    let begin = label.as_ptr().cast::<std::ffi::c_char>();
    // SAFETY: render thread inside a frame; `draw_list` belongs to the
    // parent menu of this frame; `begin..begin + len` is the UTF-8 label,
    // which ImGui only reads.
    unsafe {
        nexus::imgui::sys::ImDrawList_AddText_Vec2(
            draw_list,
            nexus::imgui::sys::ImVec2 { x, y },
            packed(theme.text.primary),
            begin,
            begin.add(label.len()),
        );
    }
    draw_icon_into(
        draw_list,
        ui,
        Icon::CHEVRON_RIGHT,
        right_edge - icon_width(ui),
        [y, bottom],
        packed(theme.text.secondary),
    );
    token
}
