//! A toast: a short message with optional actions at the bottom centre of
//! the current window (memo M142, M148, M158, M159, `ix-toasts`).
//!
//! One toast at a time; a new one replaces the old. It stays
//! [`TOAST_SECONDS`] and its timer pauses while the mouse is over it. It is
//! drawn as a child window, so it lies above the window's content and takes
//! the clicks there.

use crate::tooltip::draw_tooltip;
use nexus::imgui::{ChildWindow, Ui, WindowHoveredFlags};

use crate::{
    button::{button_width, draw_button},
    theme::Theme,
};

/// How long a toast stays while the mouse is not over it (M148).
pub const TOAST_SECONDS: f32 = 5.0;

/// A shown toast and how long it has been visible.
#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    /// The message, in the addon's language.
    message: String,
    /// Seconds the toast was visible while not hovered.
    visible_seconds: f32,
    /// Whether the mouse was over the toast in the last frame.
    hovered: bool,
}

/// An action button of a toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToastAction<'text> {
    /// The button's label.
    pub label: &'text str,
    /// The tooltip, e.g. naming the action's shortcut (M162).
    pub tooltip: Option<&'text str>,
}

/// How a toast arranges its message and action buttons (M159).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToastLayout {
    /// All in one row.
    Inline,
    /// The buttons in one row below the message, at its right end.
    ActionsBelow,
    /// Every button in its own row below the message.
    ActionsStacked,
}

impl ToastLayout {
    /// The layout for a message of `text_width` and buttons that together
    /// take `actions_width` (with their gaps), within `content_width`.
    fn choose(text_width: f32, actions_width: f32, gap: f32, content_width: f32) -> Self {
        if actions_width <= 0.0 || text_width + gap + actions_width <= content_width {
            Self::Inline
        } else if actions_width <= content_width {
            Self::ActionsBelow
        } else {
            Self::ActionsStacked
        }
    }
}

/// What happened to the toast in this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToastOutcome {
    /// The index of the action button that was clicked.
    pub clicked_action: Option<usize>,
    /// The toast's time is up; the caller drops it.
    pub expired: bool,
}

impl Toast {
    /// A new toast showing `message`.
    #[must_use]
    pub const fn new(message: String) -> Self {
        Self {
            message,
            visible_seconds: 0.0,
            hovered: false,
        }
    }

    /// The message shown.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Counts `seconds` of visibility unless the mouse is over the toast;
    /// returns whether its time is up.
    fn advance(&mut self, seconds: f32) -> bool {
        if !self.hovered {
            self.visible_seconds += seconds;
        }
        self.visible_seconds >= TOAST_SECONDS
    }
}

/// Where the parts of a toast go, measured once per frame.
#[derive(Debug, Clone, Copy)]
struct ToastGeometry {
    /// How message and buttons are arranged.
    layout: ToastLayout,
    /// Width of the message.
    text_width: f32,
    /// Width of all buttons side by side.
    actions_width: f32,
    /// Width of the card.
    width: f32,
    /// Height of the card.
    height: f32,
    /// Distance between the tops of two rows.
    row_step: f32,
}

impl ToastGeometry {
    /// Measures `message` and `actions` for the current window.
    fn measure(ui: &Ui, theme: &Theme, message: &str, actions: &[ToastAction<'_>]) -> Self {
        let padding = theme.spacing.small;
        let gap = theme.spacing.medium;
        let text_width = ui.calc_text_size(message)[0];
        let widest_action = actions
            .iter()
            .map(|action| button_width(ui, theme, action.label))
            .fold(0.0, f32::max);
        let actions_width = actions
            .iter()
            .map(|action| button_width(ui, theme, action.label))
            .sum::<f32>()
            + gap * actions.len().saturating_sub(1) as f32;
        let maximum_width = (ui.window_size()[0] - 2.0 * theme.spacing.large).max(1.0);
        let layout = ToastLayout::choose(
            text_width,
            actions_width,
            gap,
            maximum_width - 2.0 * padding,
        );
        let row_height = ui.frame_height();
        let row_step = row_height + theme.spacing.small;
        let (content_width, rows) = match layout {
            ToastLayout::Inline if actions.is_empty() => (text_width, 1),
            ToastLayout::Inline => (text_width + gap + actions_width, 1),
            ToastLayout::ActionsBelow => (text_width.max(actions_width), 2),
            ToastLayout::ActionsStacked => (text_width.max(widest_action), 1 + actions.len()),
        };
        Self {
            layout,
            text_width,
            actions_width,
            width: (2.0 * padding + content_width).min(maximum_width),
            height: row_height + row_step * (rows - 1) as f32 + 2.0 * padding,
            row_step,
        }
    }
}

/// Draws `toast` at the bottom centre of the current window with its
/// `actions` (regular buttons, e.g. "Undo") and advances its timer. The buttons
/// move below the message when the window is too narrow (M159). Call it after
/// the window's content, so it lies on top.
pub fn draw_toast(
    ui: &Ui,
    theme: &Theme,
    toast: &mut Toast,
    actions: &[ToastAction<'_>],
) -> ToastOutcome {
    let mut outcome = ToastOutcome {
        expired: toast.advance(ui.io().delta_time),
        ..ToastOutcome::default()
    };
    let geometry = ToastGeometry::measure(ui, theme, &toast.message, actions);
    let (width, height) = (geometry.width, geometry.height);
    let window_size = ui.window_size();
    let window_position = ui.window_pos();
    let bottom_gap = ui.clone_style().window_padding[1] + theme.spacing.large;
    ui.set_cursor_screen_pos([
        window_position[0] + ((window_size[0] - width) / 2.0).max(0.0),
        window_position[1] + window_size[1] - bottom_gap - height,
    ]);
    // A child without a border has no window padding. ImGui renders a child's
    // background into the parent's draw list, below everything drawn before
    // (the window menu, a note's text), so the card goes into the toast's own list.
    let Some(_toast) = ChildWindow::new("##toast")
        .size([width, height])
        .draw_background(false)
        .scroll_bar(false)
        .scrollable(false)
        .begin(ui)
    else {
        return outcome;
    };
    let card_start = ui.window_pos();
    let card_end = [card_start[0] + width, card_start[1] + height];
    {
        let draw_list = ui.get_window_draw_list();
        draw_list
            .add_rect(card_start, card_end, theme.surface.popup.to_array())
            .filled(true)
            .rounding(theme.radius.medium)
            .build();
        draw_list
            .add_rect(card_start, card_end, theme.border.color.to_array())
            .rounding(theme.radius.medium)
            .build();
    }
    toast.hovered = ui.is_window_hovered_with_flags(WindowHoveredFlags::CHILD_WINDOWS);
    let padding = theme.spacing.small;
    let [left, top] = [card_start[0] + padding, card_start[1] + padding];
    let line_offset = (ui.frame_height() - ui.text_line_height()) / 2.0;
    ui.set_cursor_screen_pos([left, top + line_offset]);
    ui.text(&toast.message);
    outcome.clicked_action = draw_toast_actions(ui, theme, actions, geometry, [left, top]);
    outcome
}

/// Draws the buttons of a toast whose content starts at `[left, top]` and
/// returns the index of the clicked one.
fn draw_toast_actions(
    ui: &Ui,
    theme: &Theme,
    actions: &[ToastAction<'_>],
    geometry: ToastGeometry,
    [left, top]: [f32; 2],
) -> Option<usize> {
    let gap = theme.spacing.medium;
    let layout = geometry.layout;
    let right = left + geometry.width - 2.0 * theme.spacing.small;
    let mut position = match layout {
        ToastLayout::Inline => [left + geometry.text_width + gap, top],
        ToastLayout::ActionsBelow | ToastLayout::ActionsStacked => {
            [right - geometry.actions_width, top + geometry.row_step]
        }
    };
    let mut clicked = None;
    for (index, action) in actions.iter().enumerate() {
        let button = button_width(ui, theme, action.label);
        if layout == ToastLayout::ActionsStacked {
            position[0] = right - button;
        }
        ui.set_cursor_screen_pos(position);
        if draw_button(ui, theme, action.label) {
            clicked = Some(index);
        }
        if let Some(tooltip) = action.tooltip
            && ui.is_item_hovered()
        {
            draw_tooltip(ui, tooltip);
        }
        if layout == ToastLayout::ActionsStacked {
            position[1] += geometry.row_step;
        } else {
            position[0] += button + gap;
        }
    }
    clicked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_toast_expires_after_its_time() {
        let mut toast = Toast::new("Deleted".to_owned());
        assert!(!toast.advance(TOAST_SECONDS - 1.0));
        assert!(toast.advance(1.0));
    }

    #[test]
    fn actions_move_below_the_message_when_the_window_is_narrow() {
        assert_eq!(
            ToastLayout::choose(100.0, 80.0, 10.0, 190.0),
            ToastLayout::Inline
        );
        assert_eq!(
            ToastLayout::choose(100.0, 80.0, 10.0, 189.0),
            ToastLayout::ActionsBelow
        );
        assert_eq!(
            ToastLayout::choose(100.0, 80.0, 10.0, 79.0),
            ToastLayout::ActionsStacked
        );
    }

    #[test]
    fn a_message_without_actions_stays_in_one_row() {
        assert_eq!(
            ToastLayout::choose(100.0, 0.0, 10.0, 50.0),
            ToastLayout::Inline
        );
    }

    #[test]
    fn hovering_pauses_the_timer() {
        let mut toast = Toast::new("Deleted".to_owned());
        toast.hovered = true;
        assert!(!toast.advance(TOAST_SECONDS * 2.0));
        toast.hovered = false;
        assert!(!toast.advance(1.0));
    }
}
