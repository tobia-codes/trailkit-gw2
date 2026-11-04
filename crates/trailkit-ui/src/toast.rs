//! A toast: a short message with an optional action at the bottom centre of
//! the current window (memo M142, M148, `ix-toasts`).
//!
//! One toast at a time; a new one replaces the old. It stays
//! [`TOAST_SECONDS`] and its timer pauses while the mouse is over it. It is
//! drawn as a child window, so it lies above the window's content and takes
//! the clicks there.

use nexus::imgui::{ChildWindow, StyleColor, StyleVar, Ui, WindowHoveredFlags};

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

/// What happened to the toast in this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToastOutcome {
    /// The action button was clicked.
    pub action_clicked: bool,
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

/// Draws `toast` at the bottom centre of the current window with an
/// `action` button (a regular button, e.g. "Undo") and advances its timer.
/// Call it after the window's content, so it lies on top.
pub fn draw_toast(ui: &Ui, theme: &Theme, toast: &mut Toast, action: Option<&str>) -> ToastOutcome {
    let mut outcome = ToastOutcome {
        expired: toast.advance(ui.io().delta_time),
        ..ToastOutcome::default()
    };
    let padding = theme.spacing.small;
    let gap = theme.spacing.medium;
    let text_width = ui.calc_text_size(&toast.message)[0];
    let action_width = action.map_or(0.0, |label| gap + button_width(ui, theme, label));
    let window_size = ui.window_size();
    let maximum_width = (window_size[0] - 2.0 * theme.spacing.large).max(1.0);
    let width = (2.0 * padding + text_width + action_width).min(maximum_width);
    let height = ui.frame_height() + 2.0 * padding;
    let window_position = ui.window_pos();
    let bottom_gap = ui.clone_style().window_padding[1] + theme.spacing.large;
    ui.set_cursor_screen_pos([
        window_position[0] + ((window_size[0] - width) / 2.0).max(0.0),
        window_position[1] + window_size[1] - bottom_gap - height,
    ]);
    let _colors = [
        ui.push_style_color(StyleColor::ChildBg, theme.surface.popup.to_array()),
        ui.push_style_color(StyleColor::Border, theme.border.color.to_array()),
    ];
    let _rounding = ui.push_style_var(StyleVar::ChildRounding(theme.radius.medium));
    let window_padding = ui.push_style_var(StyleVar::WindowPadding([padding, padding]));
    let token = ChildWindow::new("##toast")
        .size([width, height])
        .border(true)
        .scroll_bar(false)
        .scrollable(false)
        .begin(ui);
    window_padding.pop();
    let Some(_toast) = token else {
        return outcome;
    };
    toast.hovered = ui.is_window_hovered_with_flags(WindowHoveredFlags::CHILD_WINDOWS);
    let top = ui.cursor_screen_pos()[1];
    let line_offset = (ui.frame_height() - ui.text_line_height()) / 2.0;
    ui.set_cursor_screen_pos([ui.cursor_screen_pos()[0], top + line_offset]);
    ui.text(&toast.message);
    if let Some(label) = action {
        ui.same_line_with_spacing(0.0, gap);
        ui.set_cursor_screen_pos([ui.cursor_screen_pos()[0], top]);
        outcome.action_clicked = draw_button(ui, theme, label);
    }
    outcome
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
    fn hovering_pauses_the_timer() {
        let mut toast = Toast::new("Deleted".to_owned());
        toast.hovered = true;
        assert!(!toast.advance(TOAST_SECONDS * 2.0));
        toast.hovered = false;
        assert!(!toast.advance(1.0));
    }
}
