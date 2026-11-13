//! The status card (memo M184, ROADMAP R96, R103): a status line on a fill
//! tinted with the status color, framed in that color, with an optional
//! detail line hanging under the status text, buttons at the card's left edge
//! and an optional close button. It shows a state that needs attention in
//! place, above or within the content it concerns; it is not a toast.
//!
//! Only the status sentence is in the status color (Q332). The buttons keep
//! the two button styles of R103 on the tinted fill.

use nexus::imgui::{Ui, sys};

use crate::{
    button::{Button, ButtonKind, draw_icon_button, icon_button_width, icon_label_gap},
    color::Color,
    dropdown::packed,
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    status_line::{Status, draw_status_line},
    text::{TextStyle, draw_text_wrapped},
    theme::Theme,
};

/// The status of a card. Only statuses with a tinted fill exist, so a card
/// cannot be drawn without one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardStatus {
    /// Something needs attention: `warning` on `warning.subtle`.
    Warning,
    /// Something failed: `danger` on `danger.subtle`.
    Danger,
}

impl CardStatus {
    /// The status line's status.
    #[must_use]
    pub const fn status(self) -> Status {
        match self {
            Self::Warning => Status::Warning,
            Self::Danger => Status::Danger,
        }
    }

    /// The card's fill and frame.
    const fn colors(self, theme: &Theme) -> (Color, Color) {
        match self {
            Self::Warning => (theme.status.warning_subtle, theme.status.warning),
            Self::Danger => (theme.status.danger_subtle, theme.status.danger),
        }
    }
}

/// A line under the status text in a normal text role, e.g. a file name.
#[derive(Debug, Clone, Copy)]
pub struct CardDetail<'text> {
    /// The text, wrapped under the status text.
    pub text: &'text str,
    /// Its role; never a status color (Q332).
    pub style: TextStyle,
}

/// What a status card shows.
#[derive(Debug, Clone, Copy)]
pub struct StatusCard<'text> {
    /// Its status: icon, colors and fill.
    pub status: CardStatus,
    /// The status sentence.
    pub text: &'text str,
    /// An optional line under it.
    pub detail: Option<CardDetail<'text>>,
    /// The buttons in a row at the left edge, the brass one last; when the
    /// row does not fit they stack, the brass one first (R103, Q349).
    pub buttons: &'text [Button<'text>],
    /// The close button's tooltip, or `None` for a card that stays while its
    /// state lasts (Q336).
    pub close_tooltip: Option<&'text str>,
}

/// What the user did with a status card this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StatusCardOutcome {
    /// The index of the clicked button in [`StatusCard::buttons`].
    pub clicked_button: Option<usize>,
    /// The close button was clicked.
    pub closed: bool,
}

/// Draws `card` at the cursor across the available width and moves the
/// cursor `spacing.medium` below it.
///
/// The fill goes into a lower draw-list channel after the content is laid
/// out, so the card has its final height in the first frame. Must not be
/// called while the window's draw list is already split into channels.
pub fn draw_status_card(ui: &Ui, theme: &Theme, card: &StatusCard<'_>) -> StatusCardOutcome {
    // Icon buttons take their icon as ImGui id: without its own id scope the
    // card's `x` would clash with the title bar's, and ImGui drops the click.
    let _id = ui.push_id(card.text);
    let mut outcome = StatusCardOutcome::default();
    let window_left = ui.window_pos()[0];
    let [left, top] = ui.cursor_screen_pos();
    let right = left + ui.content_region_avail()[0];
    let padding = theme.spacing.small;
    let [inner_left, inner_right] = [left + padding, right - padding];
    let close_width = close_width(ui, theme, card.close_tooltip.is_some());
    // SAFETY: called on the render thread inside a frame, so the current
    // window's draw list is valid; the split is merged below before return.
    let draw_list = unsafe {
        let draw_list = sys::igGetWindowDrawList();
        sys::ImDrawList_ChannelsSplit(draw_list, 2);
        sys::ImDrawList_ChannelsSetCurrent(draw_list, 1);
        draw_list
    };
    ui.set_cursor_screen_pos([inner_left, top + padding]);
    let text_left = {
        let _wrap = ui.push_text_wrap_pos_with_pos(inner_right - close_width - window_left);
        draw_status_line(ui, theme, card.status.status(), card.text, TextStyle::Body)
    };
    let mut bottom = ui.item_rect_max()[1];
    if let Some(detail) = card.detail {
        draw_detail(ui, theme, detail, text_left, inner_right);
        bottom = ui.item_rect_max()[1];
    }
    if let Some(clicked) = draw_buttons(
        ui,
        theme,
        card.buttons,
        [inner_left, inner_right],
        &mut bottom,
    ) {
        outcome.clicked_button = Some(clicked);
    }
    if let Some(tooltip) = card.close_tooltip {
        ui.set_cursor_screen_pos([inner_right - icon_button_width(ui, Icon::X), top + padding]);
        outcome.closed = draw_icon_button(ui, theme, Icon::X, tooltip);
        bottom = bottom.max(ui.item_rect_max()[1]);
    }
    let card_bottom = bottom + padding;
    let (fill, frame) = card.status.colors(theme);
    // SAFETY: as above; channel 0 lies below the content, then the channels
    // are merged into the window's draw list again.
    unsafe {
        sys::ImDrawList_ChannelsSetCurrent(draw_list, 0);
        let (min, max) = (
            sys::ImVec2 { x: left, y: top },
            sys::ImVec2 {
                x: right,
                y: card_bottom,
            },
        );
        let corners = sys::ImDrawCornerFlags_All as i32;
        sys::ImDrawList_AddRectFilled(
            draw_list,
            min,
            max,
            packed(fill),
            theme.radius.medium,
            corners,
        );
        sys::ImDrawList_AddRect(
            draw_list,
            min,
            max,
            packed(frame),
            theme.radius.medium,
            corners,
            theme.border.width.max(1.0),
        );
        sys::ImDrawList_ChannelsMerge(draw_list);
    }
    // One item over the whole card, so the layout continues below it,
    // `spacing.medium` away like below the separator the card replaces.
    let gap_below = theme.spacing.medium - ui.clone_style().item_spacing[1];
    ui.set_cursor_screen_pos([left, top]);
    ui.dummy([right - left, card_bottom - top + gap_below.max(0.0)]);
    outcome
}

/// Whether `text` fits on the first line of a card drawn at the cursor with
/// `status`, with or without a close button. A caller can then choose where
/// a text that does not fit breaks, e.g. before a date (Q339).
pub fn fits_status_card_line(
    ui: &Ui,
    theme: &Theme,
    status: CardStatus,
    closable: bool,
    text: &str,
) -> bool {
    let icon_width = {
        let _icons = FontScope::push(ui, FontRole::Icons);
        let mut buffer = [0; 4];
        ui.calc_text_size(status.status().icon().as_str(&mut buffer))[0]
    };
    let available = ui.content_region_avail()[0]
        - 2.0 * theme.spacing.small
        - close_width(ui, theme, closable)
        - icon_width
        - icon_label_gap(theme);
    let _font = FontScope::push(ui, TextStyle::Body.font());
    ui.calc_text_size(text)[0] <= available
}

/// The width the close button takes from the status text, or `0.0`.
fn close_width(ui: &Ui, theme: &Theme, closable: bool) -> f32 {
    if closable {
        icon_button_width(ui, Icon::X) + theme.spacing.extra_small
    } else {
        0.0
    }
}

/// Draws `detail` wrapped at `right`, every line starting at `text_left`
/// under the status text.
fn draw_detail(ui: &Ui, theme: &Theme, detail: CardDetail<'_>, text_left: f32, right: f32) {
    let window_left = ui.window_pos()[0];
    // Wrapped lines start at the window's content edge; indent them to the
    // status text.
    let hang = text_left - (window_left + ui.window_content_region_min()[0]);
    let _wrap = ui.push_text_wrap_pos_with_pos(right - window_left);
    ui.indent_by(hang);
    ui.set_cursor_screen_pos([text_left, ui.cursor_screen_pos()[1]]);
    draw_text_wrapped(ui, theme, detail.style, detail.text);
    ui.unindent_by(hang);
}

/// Draws `buttons` in one row from `left` in their order, the brass button
/// last (R103, Q349). When the row does not fit before `right`, they stack,
/// the brass button first. Lowers `bottom` to the last button's bottom and
/// returns the index of the clicked one.
fn draw_buttons(
    ui: &Ui,
    theme: &Theme,
    buttons: &[Button<'_>],
    [left, right]: [f32; 2],
    bottom: &mut f32,
) -> Option<usize> {
    let spacing = ui.clone_style().item_spacing;
    let row_width = buttons
        .iter()
        .map(|button| button.measure(ui, theme) + spacing[0])
        .sum::<f32>()
        - spacing[0];
    let stacked = row_width > right - left;
    let is_primary = |button: &Button<'_>| button.kind == ButtonKind::Primary;
    // Stacked: the brass button first, then the others in their order.
    let order = buttons
        .iter()
        .enumerate()
        .filter(|(_, button)| !stacked || is_primary(button))
        .chain(
            buttons
                .iter()
                .enumerate()
                .filter(|(_, button)| stacked && !is_primary(button)),
        );
    let mut clicked = None;
    let mut x = left;
    for (index, button) in order {
        let y = if stacked || x == left {
            *bottom + spacing[1]
        } else {
            ui.item_rect_min()[1]
        };
        ui.set_cursor_screen_pos([if stacked { left } else { x }, y]);
        if button.draw(ui, theme) {
            clicked = Some(index);
        }
        *bottom = ui.item_rect_max()[1];
        x = ui.item_rect_max()[0] + spacing[0];
    }
    clicked
}
