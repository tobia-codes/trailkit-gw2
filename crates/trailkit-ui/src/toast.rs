//! A toast: a short message with optional actions at the bottom centre of
//! the current window (memo M142, M148, M158, M159, M180, `ix-toasts`).
//!
//! One toast at a time; a new one replaces the old. A regular or success
//! toast stays [`TOAST_SECONDS`] and its timer pauses while the mouse is over
//! it; an error toast has no timer (memo Q145). An error toast is tinted
//! (`danger.subtle`, `danger` frame and `circle-alert`), may show a counter,
//! a spinner and a close button. It is drawn as a child window, so it lies
//! above the window's content and takes the clicks there.

use crate::draw_list::window_draw_list;
use crate::tooltip::draw_tooltip;
use nexus::imgui::{ChildWindow, ImColor32, StyleColor, Ui, WindowHoveredFlags};

use crate::{
    button::{Button, ButtonState, draw_popup_icon_button, icon_button_width},
    color::Color,
    dropdown::{draw_icon_into, icon_width},
    icons::Icon,
    spinner::{draw_spinner, spinner_width},
    theme::Theme,
};

/// How long a toast stays while the mouse is not over it (M148).
pub const TOAST_SECONDS: f32 = 5.0;

/// The look and lifetime of a toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastTone {
    /// A plain message, e.g. "Moved to the trash".
    Neutral,
    /// Something succeeded after an error was shown (`circle-check` in
    /// `success`).
    Success,
    /// An error that stays until it is resolved or closed (no timer).
    Error,
}

/// A shown toast and how long it has been visible.
#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    /// The message, in the addon's language.
    message: String,
    /// The look and lifetime.
    tone: ToastTone,
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
    /// What the button can do now: busy shows a spinner (memo M180,
    /// "Saving…"); neither busy nor disabled report clicks (R93).
    pub state: ButtonState<'text>,
}

/// What a toast shows besides its message and actions (memo M180).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToastExtras<'text> {
    /// A short counter after the message in `text.secondary`, e.g. "2/3".
    pub counter: Option<&'text str>,
    /// Whether a spinner after the message shows that something runs.
    pub spinner: bool,
    /// The tooltip of a close button (`x`, top right); `None` for none.
    pub close_tooltip: Option<&'text str>,
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
    /// The close button was clicked; the caller hides the toast.
    pub closed: bool,
}

impl Toast {
    /// A new regular toast showing `message`.
    #[must_use]
    pub const fn new(message: String) -> Self {
        Self::with_tone(message, ToastTone::Neutral)
    }

    /// A new toast showing `message` in `tone`.
    #[must_use]
    pub const fn with_tone(message: String, tone: ToastTone) -> Self {
        Self {
            message,
            tone,
            visible_seconds: 0.0,
            hovered: false,
        }
    }

    /// The message shown.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The look and lifetime.
    #[must_use]
    pub const fn tone(&self) -> ToastTone {
        self.tone
    }

    /// Counts `seconds` of visibility unless the mouse is over the toast;
    /// returns whether its time is up. An error toast never expires.
    fn advance(&mut self, seconds: f32) -> bool {
        if self.tone == ToastTone::Error {
            return false;
        }
        if !self.hovered {
            self.visible_seconds += seconds;
        }
        self.visible_seconds >= TOAST_SECONDS
    }
}

impl ToastTone {
    /// The icon before the message and its color, if the tone has one.
    fn icon(self, theme: &Theme) -> Option<(Icon, Color)> {
        match self {
            Self::Neutral => None,
            Self::Success => Some((Icon::CIRCLE_CHECK, theme.status.success)),
            Self::Error => Some((Icon::CIRCLE_ALERT, theme.status.danger)),
        }
    }

    /// The card's fill and frame.
    const fn card_colors(self, theme: &Theme) -> (Color, Color) {
        match self {
            Self::Neutral | Self::Success => (theme.surface.popup, theme.border.color),
            Self::Error => (theme.status.danger_subtle, theme.status.danger),
        }
    }
}

/// The button of an action.
const fn action_button<'text>(action: &ToastAction<'text>) -> Button<'text> {
    Button::new(action.label).state(action.state)
}

/// The width of an action button.
fn action_width(ui: &Ui, theme: &Theme, action: &ToastAction<'_>) -> f32 {
    action_button(action).measure(ui, theme)
}

/// Where the parts of a toast go, measured once per frame.
#[derive(Debug, Clone, Copy)]
struct ToastGeometry {
    /// How message and buttons are arranged.
    layout: ToastLayout,
    /// Width of the first row's text group: icon, message, counter, spinner.
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
    /// Measures `toast` with `actions` and `extras` for the current window.
    fn measure(
        ui: &Ui,
        theme: &Theme,
        toast: &Toast,
        actions: &[ToastAction<'_>],
        extras: ToastExtras<'_>,
    ) -> Self {
        let padding = theme.spacing.small;
        let gap = theme.spacing.medium;
        let small_gap = theme.spacing.small;
        let mut text_width = ui.calc_text_size(&toast.message)[0];
        if toast.tone.icon(theme).is_some() {
            text_width += icon_width(ui) + small_gap;
        }
        if let Some(counter) = extras.counter {
            text_width += small_gap + ui.calc_text_size(counter)[0];
        }
        if extras.spinner {
            text_width += small_gap + spinner_width(ui);
        }
        // The close button stays in the first row's top right corner.
        let close_width = extras
            .close_tooltip
            .map_or(0.0, |_| gap + icon_button_width(ui, Icon::X));
        let widest_action = actions
            .iter()
            .map(|action| action_width(ui, theme, action))
            .fold(0.0, f32::max);
        let actions_width = actions
            .iter()
            .map(|action| action_width(ui, theme, action))
            .sum::<f32>()
            + gap * actions.len().saturating_sub(1) as f32;
        let maximum_width = (ui.window_size()[0] - 2.0 * theme.spacing.large).max(1.0);
        let first_row = text_width + close_width;
        let layout =
            ToastLayout::choose(first_row, actions_width, gap, maximum_width - 2.0 * padding);
        let row_height = ui.frame_height();
        let row_step = row_height + theme.spacing.small;
        let (content_width, rows) = match layout {
            ToastLayout::Inline if actions.is_empty() => (first_row, 1),
            ToastLayout::Inline => (first_row + gap + actions_width, 1),
            ToastLayout::ActionsBelow => (first_row.max(actions_width), 2),
            ToastLayout::ActionsStacked => (first_row.max(widest_action), 1 + actions.len()),
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
    draw_toast_with_extras(ui, theme, toast, actions, ToastExtras::default())
}

/// Like [`draw_toast`], with a counter, a spinner or a close button
/// (memo M180).
pub fn draw_toast_with_extras(
    ui: &Ui,
    theme: &Theme,
    toast: &mut Toast,
    actions: &[ToastAction<'_>],
    extras: ToastExtras<'_>,
) -> ToastOutcome {
    let mut outcome = ToastOutcome {
        expired: toast.advance(ui.io().delta_time),
        ..ToastOutcome::default()
    };
    let geometry = ToastGeometry::measure(ui, theme, toast, actions, extras);
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
    let (fill, frame) = toast.tone.card_colors(theme);
    {
        let draw_list = window_draw_list(ui);
        draw_list
            .add_rect(card_start, card_end, fill.to_array())
            .filled(true)
            .rounding(theme.radius.medium)
            .build();
        draw_list
            .add_rect(card_start, card_end, frame.to_array())
            .rounding(theme.radius.medium)
            .build();
    }
    toast.hovered = ui.is_window_hovered_with_flags(WindowHoveredFlags::CHILD_WINDOWS);
    let padding = theme.spacing.small;
    let [left, top] = [card_start[0] + padding, card_start[1] + padding];
    draw_toast_text(ui, theme, toast, extras, [left, top]);
    // Buttons on the tinted error fill use `bg.active`, so they stand out.
    let _buttons = (toast.tone == ToastTone::Error).then(|| {
        [
            ui.push_style_color(StyleColor::Button, theme.surface.active.to_array()),
            ui.push_style_color(StyleColor::ButtonHovered, theme.surface.pressed.to_array()),
        ]
    });
    outcome.clicked_action = draw_toast_actions(ui, theme, actions, geometry, [left, top]);
    if let Some(tooltip) = extras.close_tooltip {
        let right = card_end[0] - padding;
        ui.set_cursor_screen_pos([right - icon_button_width(ui, Icon::X), top]);
        outcome.closed = draw_popup_icon_button(ui, theme, Icon::X, tooltip);
    }
    outcome
}

/// Draws the first row's text group starting at `[left, top]`: the tone's
/// icon, the message, the counter and the spinner.
fn draw_toast_text(
    ui: &Ui,
    theme: &Theme,
    toast: &Toast,
    extras: ToastExtras<'_>,
    [left, top]: [f32; 2],
) {
    let small_gap = theme.spacing.small;
    let row = [top, top + ui.frame_height()];
    let line_offset = (ui.frame_height() - ui.text_line_height()) / 2.0;
    let mut x = left;
    if let Some((icon, color)) = toast.tone.icon(theme) {
        // SAFETY: called on the render thread inside a frame, so the current
        // window's draw list is valid.
        let draw_list = unsafe { nexus::imgui::sys::igGetWindowDrawList() };
        let packed = ImColor32::from(color.to_array()).to_bits();
        draw_icon_into(draw_list, ui, icon, x, row, packed);
        x += icon_width(ui) + small_gap;
    }
    ui.set_cursor_screen_pos([x, top + line_offset]);
    ui.text(&toast.message);
    x += ui.calc_text_size(&toast.message)[0];
    if let Some(counter) = extras.counter {
        x += small_gap;
        ui.set_cursor_screen_pos([x, top + line_offset]);
        ui.text_colored(theme.text.secondary.to_array(), counter);
        x += ui.calc_text_size(counter)[0];
    }
    if extras.spinner {
        x += small_gap;
        let color = toast
            .tone
            .icon(theme)
            .map_or(theme.text.secondary, |(_, color)| color);
        draw_spinner(
            ui,
            [x + spinner_width(ui) / 2.0, (row[0] + row[1]) / 2.0],
            color,
        );
    }
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
        let button = action_width(ui, theme, action);
        if layout == ToastLayout::ActionsStacked {
            position[0] = right - button;
        }
        ui.set_cursor_screen_pos(position);
        if action_button(action).draw(ui, theme) {
            clicked = Some(index);
        }
        if let Some(tooltip) = action.tooltip
            && ui.is_item_hovered()
        {
            draw_tooltip(ui, theme, tooltip);
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
    fn an_error_toast_never_expires() {
        let mut toast = Toast::with_tone("Saving failed".to_owned(), ToastTone::Error);
        assert!(!toast.advance(TOAST_SECONDS * 10.0));
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
