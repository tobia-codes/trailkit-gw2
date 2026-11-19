//! The backdrop behind a dialog or a panel (`overlay-dialog-scope`): it dims
//! an area in `surface.modal_overlay` and, when it is clickable, reports a
//! click beside the content drawn on it. Every addon dims through here, so
//! all backdrops look and react the same.

use nexus::imgui::{
    ChildWindow, Condition, MouseButton, StyleColor, StyleVar, Ui, Window, WindowFlags,
};

use crate::{
    drawing::draw_list::window_draw_list, text::font_registry::FontScope, text::fonts::FontRole,
    theme::Theme,
};

/// Where a backdrop dims.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackdropArea {
    /// Its own borderless window exactly over another window, title bar
    /// included, so it dims only that window while the game and other
    /// windows stay usable.
    Window {
        /// Top-left corner of the window to dim, in screen coordinates.
        position: [f32; 2],
        /// Width and height of the window to dim.
        size: [f32; 2],
        /// Whether the backdrop takes the focus in this frame, e.g. after
        /// the window below was clicked.
        is_focused: bool,
    },
    /// A region of `size` at the cursor of the current window, e.g. its
    /// content area, with the window's rounded corners.
    Region {
        /// Width and height of the region.
        size: [f32; 2],
    },
}

/// A backdrop, configured and drawn in one expression.
#[derive(Debug, Clone, Copy)]
pub struct Backdrop<'a> {
    /// ImGui id of the backdrop's window, unique in the addon.
    id: &'a str,
    /// Where it dims.
    area: BackdropArea,
    /// Whether a click on it is reported.
    is_clickable: bool,
}

impl<'a> Backdrop<'a> {
    /// A backdrop with the id `id` over `area` that ignores clicks.
    #[must_use]
    pub const fn new(id: &'a str, area: BackdropArea) -> Self {
        Self {
            id,
            area,
            is_clickable: false,
        }
    }

    /// Whether a click on the backdrop is reported (default: no).
    #[must_use]
    pub const fn clickable(mut self, is_clickable: bool) -> Self {
        self.is_clickable = is_clickable;
        self
    }

    /// Dims the area, draws `content` on it and returns whether the backdrop
    /// was clicked: a left click on the area beside everything `content`
    /// drew. Always `false` unless the backdrop is
    /// [`clickable`](Self::clickable).
    pub fn draw(&self, ui: &Ui, theme: &Theme, content: impl FnOnce()) -> bool {
        let draw_and_check = || {
            // The group's box is the union of everything the content drew.
            ui.group(content);
            let (min, max) = (ui.item_rect_min(), ui.item_rect_max());
            let mouse = ui.io().mouse_pos;
            let is_on_content =
                (min[0]..=max[0]).contains(&mouse[0]) && (min[1]..=max[1]).contains(&mouse[1]);
            self.is_clickable
                && !is_on_content
                && ui.is_window_hovered()
                && ui.is_mouse_clicked(MouseButton::Left)
        };
        match self.area {
            BackdropArea::Window {
                position,
                size,
                is_focused,
            } => {
                let _body_font = FontScope::push(ui, FontRole::Body);
                let _background = ui
                    .push_style_color(StyleColor::WindowBg, theme.surface.modal_overlay.to_array());
                let _border = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
                Window::new(self.id)
                    .position(position, Condition::Always)
                    .size(size, Condition::Always)
                    .flags(
                        WindowFlags::NO_DECORATION
                            | WindowFlags::NO_MOVE
                            | WindowFlags::NO_SAVED_SETTINGS
                            | WindowFlags::NO_NAV,
                    )
                    .focused(is_focused)
                    .build(ui, draw_and_check)
                    .unwrap_or(false)
            }
            BackdropArea::Region { size } => {
                // ImGui renders a child's background into the parent's draw
                // list, below the panes drawn before, so the dimming is a
                // rectangle in the backdrop's own list.
                let Some(_region) = ChildWindow::new(self.id)
                    .size(size)
                    .scrollable(false)
                    .scroll_bar(false)
                    .movable(false)
                    .draw_background(false)
                    .begin(ui)
                else {
                    return false;
                };
                let start = ui.window_pos();
                window_draw_list(ui)
                    .add_rect(
                        start,
                        [start[0] + size[0], start[1] + size[1]],
                        theme.surface.modal_overlay.to_array(),
                    )
                    .filled(true)
                    .rounding(theme.radius.large)
                    .build();
                draw_and_check()
            }
        }
    }
}
