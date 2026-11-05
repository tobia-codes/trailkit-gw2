//! Menu entries for context menus shared by all addons (memo plan M98, M99).
//!
//! ImGui draws its own check mark and submenu arrow; these entries use the
//! Lucide `check` and `chevron-right` icons like the dropdown, and submenus
//! open flush beside their parent instead of overlapping it.

use nexus::imgui::{ItemHoveredFlags, MenuItem, MenuToken, StyleColor, StyleVar, Ui};

use crate::{
    dropdown::{draw_icon_into, icon_width, packed},
    icons::Icon,
    theme::Theme,
};

/// Draws a menu item and returns whether it was clicked. A `checked` item
/// shows a check mark in the accent color at its right end.
pub fn draw_menu_item(ui: &Ui, theme: &Theme, label: &str, checked: bool) -> bool {
    let clicked = MenuItem::new(label).build(ui);
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
        let _hidden = ui.push_style_color(StyleColor::Text, [0.0; 4]);
        MenuItem::new(label).enabled(enabled).build(ui)
    };
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
    let begin = label.as_ptr().cast::<std::ffi::c_char>();
    // SAFETY: render thread inside a frame; `draw_list` is the current
    // window's; `begin..begin + len` is the UTF-8 label, which ImGui only
    // reads.
    unsafe {
        nexus::imgui::sys::ImDrawList_AddText_Vec2(
            draw_list,
            nexus::imgui::sys::ImVec2 {
                x: label_x,
                y: origin[1],
            },
            packed(color),
            begin,
            begin.add(label.len()),
        );
    }
    if let Some(tooltip) = disabled_tooltip
        && ui.is_item_hovered_with_flags(ItemHoveredFlags::ALLOW_WHEN_DISABLED)
    {
        ui.tooltip_text(tooltip);
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
