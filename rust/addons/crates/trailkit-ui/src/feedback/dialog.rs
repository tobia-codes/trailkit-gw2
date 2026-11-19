//! A dialog (`overlay-dialog-layout`, `overlay-dialog-scope`): a card on a
//! [`Backdrop`] over one window, centered in it, `bg.panel` with a
//! `border.subtle` outline, `radius.lg` and `space.lg` padding. The card
//! starts with the title in `Heading3` and, unless turned off, the close icon
//! at its top right (R141); the dialog draws its content below.
//!
//! While the close icon is on, the icon, Escape and a click on the backdrop
//! all close the dialog. While it is off, none of them does, so such a
//! dialog must end by itself (a running action) or always offer an active
//! button that leads out (R145).

use nexus::imgui::{Key, Ui};

use crate::{
    controls::button::{draw_icon_button, icon_button_width},
    controls::text_field::move_below_last_item,
    drawing::draw_list::window_draw_list,
    drawing::screen_rect::ScreenRect,
    feedback::backdrop::{Backdrop, BackdropArea},
    text::font_registry::FontScope,
    text::icons::Icon,
    text::{TextStyle, draw_text_wrapped},
    theme::Theme,
};

/// The height of a dialog's card measured in the last frame, so the next
/// frame can center it. The dialog's owner keeps one per open dialog and
/// resets it to the default when the dialog closes.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DialogHeight(Option<f32>);

/// What the content of a dialog returned, and whether the dialog was closed.
#[derive(Debug, Clone, PartialEq)]
pub struct DialogResponse<R> {
    /// The value the content returned.
    pub content: R,
    /// Whether the close icon, Escape or a click on the backdrop closed the
    /// dialog in this frame. Never `true` while the close icon is off.
    pub is_closed: bool,
}

/// A dialog, configured and drawn in one expression.
#[derive(Debug, Clone, Copy)]
pub struct Dialog<'a> {
    /// ImGui id of the dialog's backdrop window, unique in the addon.
    id: &'a str,
    /// The heading.
    title: &'a str,
    /// The tooltip of the close icon.
    close_tooltip: &'a str,
    /// Whether the close icon is shown.
    is_closable: bool,
    /// The window the dialog dims and is centered in.
    window: ScreenRect,
    /// Whether the dialog takes the focus in this frame.
    is_focused: bool,
}

impl<'a> Dialog<'a> {
    /// A dialog titled `title` over `window`, with the close icon and its
    /// `close_tooltip`.
    #[must_use]
    pub const fn new(
        id: &'a str,
        title: &'a str,
        close_tooltip: &'a str,
        window: ScreenRect,
    ) -> Self {
        Self {
            id,
            title,
            close_tooltip,
            is_closable: true,
            window,
            is_focused: false,
        }
    }

    /// Whether the close icon is shown (default: yes). Turning it off also
    /// keeps Escape and a click on the backdrop from closing the dialog.
    #[must_use]
    pub const fn closable(mut self, is_closable: bool) -> Self {
        self.is_closable = is_closable;
        self
    }

    /// Whether the dialog takes the focus in this frame, e.g. after the
    /// window below was clicked.
    #[must_use]
    pub const fn focused(mut self, is_focused: bool) -> Self {
        self.is_focused = is_focused;
        self
    }

    /// Draws the dialog and returns what `content` returned, or `None` when
    /// ImGui skipped the backdrop window. `content_width` gets the width of
    /// the window and returns the width of the content; `content` draws the
    /// content below the title, that wide, starting at the cursor. `height`
    /// is kept by the caller between frames.
    pub fn draw<R>(
        &self,
        ui: &Ui,
        theme: &Theme,
        height: &mut DialogHeight,
        content_width: impl FnOnce(f32) -> f32,
        content: impl FnOnce(f32) -> R,
    ) -> Option<DialogResponse<R>> {
        let mut drawn = None;
        let is_backdrop_clicked = Backdrop::new(
            self.id,
            BackdropArea::Window {
                window: self.window,
                is_focused: self.is_focused,
            },
        )
        .clickable(self.is_closable)
        .draw(ui, theme, || {
            let (card, value, is_close_clicked) =
                self.draw_card(ui, theme, height, content_width, content);
            drawn = Some((value, is_close_clicked));
            Some(card)
        });
        let (content, is_close_clicked) = drawn?;
        let is_escaped = self.is_closable && ui.is_key_pressed(Key::Escape);
        Some(DialogResponse {
            content,
            is_closed: is_close_clicked || is_escaped || is_backdrop_clicked,
        })
    }

    /// Draws the card with the title, `content` and the close icon into the
    /// current window and returns the card, the content's value and whether
    /// the close icon was clicked.
    fn draw_card<R>(
        &self,
        ui: &Ui,
        theme: &Theme,
        height: &mut DialogHeight,
        content_width: impl FnOnce(f32) -> f32,
        content: impl FnOnce(f32) -> R,
    ) -> (ScreenRect, R, bool) {
        let padding = theme.spacing.large;
        let window_size = ui.window_size();
        let width = content_width(window_size[0]).max(1.0);
        let card_height = height.0.unwrap_or(window_size[1] / 3.0);
        let card = draw_card_background(ui, theme, [width + 2.0 * padding, card_height]);
        ui.set_cursor_screen_pos([card.min[0] + padding, card.min[1] + padding]);
        let value = ui.group(|| {
            {
                let close_width = if self.is_closable {
                    icon_button_width(ui, Icon::X) + theme.spacing.small
                } else {
                    0.0
                };
                let title_width = (width - close_width).max(1.0);
                let _wrap = ui.push_text_wrap_pos_with_pos(ui.cursor_pos()[0] + title_width);
                draw_text_wrapped(ui, theme, TextStyle::Heading3, self.title);
            }
            move_below_last_item(ui, theme.spacing.medium);
            content(width)
        });
        *height = DialogHeight(Some(ui.item_rect_max()[1] - card.min[1] + padding));
        let is_close_clicked = self.is_closable && self.draw_close(ui, theme, card, padding);
        (card, value, is_close_clicked)
    }

    /// Draws the close icon at the top right of `card`, inside its `padding`
    /// and centered on the first line of the title, and returns whether it
    /// was clicked. The cursor stays where it was.
    fn draw_close(&self, ui: &Ui, theme: &Theme, card: ScreenRect, padding: f32) -> bool {
        let cursor = ui.cursor_screen_pos();
        let title_height = {
            let _title = FontScope::push(ui, TextStyle::Heading3.font());
            ui.text_line_height()
        };
        ui.set_cursor_screen_pos([
            card.max[0] - padding - icon_button_width(ui, Icon::X),
            card.min[1] + padding + (title_height - ui.frame_height()) / 2.0,
        ]);
        let is_clicked = draw_icon_button(ui, theme, Icon::X, self.close_tooltip);
        ui.set_cursor_screen_pos(cursor);
        is_clicked
    }
}

/// Draws an empty card of `size` centered in the current window and returns
/// where it was drawn.
fn draw_card_background(ui: &Ui, theme: &Theme, size: [f32; 2]) -> ScreenRect {
    let position = ui.window_pos();
    let window = ui.window_size();
    let min = [
        position[0] + ((window[0] - size[0]) / 2.0).max(0.0),
        position[1] + ((window[1] - size[1]) / 2.0).max(0.0),
    ];
    let card = ScreenRect::from_position_size(min, size);
    let draw_list = window_draw_list(ui);
    draw_list
        .add_rect(card.min, card.max, theme.surface.panel.to_array())
        .filled(true)
        .rounding(theme.radius.large)
        .build();
    draw_list
        .add_rect(card.min, card.max, theme.border.subtle.to_array())
        .thickness(theme.border.width.max(1.0))
        .rounding(theme.radius.large)
        .build();
    card
}
