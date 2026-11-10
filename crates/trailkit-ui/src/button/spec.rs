//! One text button with a kind and a state (ROADMAP R93): every text button
//! of the design system is drawn here; `draw_button`, `draw_primary_button`
//! and the other functions of [`super`] are short forms of it.
//!
//! - [`ButtonKind`] is how the button looks when it can be used.
//! - [`ButtonState`] is what it can do now: [`ButtonState::Busy`] shows a
//!   spinner while its action runs (`ix-busy-buttons`, use
//!   [`crate::busy::BusyState`] for the minimum time);
//!   [`ButtonState::Disabled`] draws the label in `text.disabled`, without
//!   hover, and names the reason in a tooltip with a neutral `ban` icon.
//!   Neither reports clicks.

use nexus::imgui::{ImColor32, StyleColor, Ui};

use super::{
    button_width, draw_label, icon_label_button_width, icon_label_content_width, icon_label_gap,
};
use crate::{
    busy::BusyState,
    color::Color,
    dropdown::{draw_border, draw_icon_into, icon_width},
    icons::Icon,
    spinner::draw_spinner,
    theme::Theme,
    tooltip::draw_reason_tooltip,
};

/// How a text button looks while it can be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonKind {
    /// The current `Button` style colors with a `border` frame, e.g.
    /// "Cancel"; on a tinted error toast `bg.active` (memo M180).
    #[default]
    Regular,
    /// The one primary (brass) button of a view (`lay-one-primary`).
    Primary,
    /// The `danger` fill that confirms a destructive action (memo M146).
    DangerFill,
}

/// What a button can do now (R93). An enum, so a button cannot be busy and
/// disabled at once, and a disabled button always names its reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonState<'text> {
    /// It can be clicked.
    #[default]
    Ready,
    /// Its action runs: a spinner before the label, no clicks.
    Busy,
    /// It cannot be used now; `reason` says why (and how to change it),
    /// shown in a tooltip (trailkit-ui principle 4).
    Disabled {
        /// The reason, in the addon's language.
        reason: &'text str,
    },
}

impl ButtonState<'_> {
    /// Whether the button reports clicks.
    #[must_use]
    pub const fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }
}

impl BusyState {
    /// The button state this busy state stands for: [`ButtonState::Busy`]
    /// while the action runs and for at least `MINIMUM_BUSY`, else
    /// [`ButtonState::Ready`] (R93). The one place that turns the stored
    /// state into what a frame draws.
    #[must_use]
    pub fn button_state(&self) -> ButtonState<'static> {
        if self.is_busy() {
            ButtonState::Busy
        } else {
            ButtonState::Ready
        }
    }
}

/// A text button to draw: label, kind, state, an optional leading icon and
/// an optional width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Button<'text> {
    /// The label.
    pub label: &'text str,
    /// How it looks while it can be used.
    pub kind: ButtonKind,
    /// What it can do now.
    pub state: ButtonState<'text>,
    /// A Lucide icon before the label (a busy button shows its spinner
    /// there instead).
    pub icon: Option<Icon>,
    /// The width; `0.0` fits the content (at least `button.min_width`).
    pub width: f32,
}

impl<'text> Button<'text> {
    /// A regular, ready button with `label` that fits its content.
    #[must_use]
    pub const fn new(label: &'text str) -> Self {
        Self {
            label,
            kind: ButtonKind::Regular,
            state: ButtonState::Ready,
            icon: None,
            width: 0.0,
        }
    }

    /// The same button of `kind`.
    #[must_use]
    pub const fn kind(self, kind: ButtonKind) -> Self {
        Self { kind, ..self }
    }

    /// The same button in `state`.
    #[must_use]
    pub const fn state(self, state: ButtonState<'text>) -> Self {
        Self { state, ..self }
    }

    /// The same button with `icon` before its label.
    #[must_use]
    pub const fn icon(self, icon: Icon) -> Self {
        Self {
            icon: Some(icon),
            ..self
        }
    }

    /// The same button `width` wide (`0.0` fits the content).
    #[must_use]
    pub const fn width(self, width: f32) -> Self {
        Self { width, ..self }
    }

    /// Whether something (icon or spinner) stands before the label.
    const fn has_leading(&self) -> bool {
        self.icon.is_some() || matches!(self.state, ButtonState::Busy)
    }

    /// The width the button takes.
    #[must_use]
    pub fn measure(&self, ui: &Ui, theme: &Theme) -> f32 {
        if self.width > 0.0 {
            self.width
        } else if self.has_leading() {
            icon_label_button_width(ui, theme, self.label)
        } else {
            button_width(ui, theme, self.label)
        }
    }

    /// Draws the button at the cursor and returns whether it was clicked
    /// (never while busy or disabled). The label is centered by its capital
    /// letters (memo M85).
    pub fn draw(&self, ui: &Ui, theme: &Theme) -> bool {
        let colors = ButtonColors::of(ui, theme, self.kind, self.state);
        let size = [self.measure(ui, theme), ui.frame_height()];
        let clicked = {
            let _colors = [
                ui.push_style_color(StyleColor::Button, colors.fill.to_array()),
                ui.push_style_color(StyleColor::ButtonHovered, colors.hovered.to_array()),
                ui.push_style_color(StyleColor::ButtonActive, colors.pressed.to_array()),
            ];
            // The label is the ID only; ImGui draws the frame, the label follows.
            let _id = ui.push_id(self.label);
            ui.button_with_size("##button", size)
        };
        let [min, max] = [ui.item_rect_min(), ui.item_rect_max()];
        if colors.bordered {
            // The fill alone can vanish on `bg.window` (R85), like on a dropdown.
            draw_border(
                min,
                max,
                theme.border.color,
                theme.border.width,
                theme.radius.medium,
            );
        }
        self.draw_content(ui, theme, [min, max], colors.label);
        if let ButtonState::Disabled { reason } = self.state
            && ui.is_item_hovered()
        {
            draw_reason_tooltip(ui, theme, reason);
        }
        clicked && self.state.is_ready()
    }

    /// Draws the spinner or icon and the label inside `[min, max]`.
    fn draw_content(&self, ui: &Ui, theme: &Theme, [min, max]: [[f32; 2]; 2], label: Color) {
        let packed = ImColor32::from(label.to_array()).to_bits();
        let span = [min[1], max[1]];
        if !self.has_leading() {
            let padding = ui.clone_style().frame_padding[0];
            let text_width = ui.calc_text_size(self.label)[0];
            let x = min[0] + ((max[0] - min[0] - text_width) / 2.0).max(padding);
            draw_label(self.label, x, span, packed);
            return;
        }
        // Leading part and label are centered as one group when the width
        // is larger than the content (M131).
        let content_width = icon_label_content_width(ui, theme, self.label);
        let x = (min[0] + (max[0] - min[0] - content_width) / 2.0).round();
        let leading = icon_width(ui);
        match (self.state, self.icon) {
            (ButtonState::Busy, _) => {
                draw_spinner(ui, [x + leading / 2.0, (min[1] + max[1]) / 2.0], label);
            }
            (_, Some(icon)) => {
                // SAFETY: called on the render thread inside a frame, where
                // ImGui has a current window and returns its draw list.
                let draw_list = unsafe { nexus::imgui::sys::igGetWindowDrawList() };
                draw_icon_into(draw_list, ui, icon, x, span, packed);
            }
            (_, None) => {}
        }
        draw_label(
            self.label,
            x + leading + icon_label_gap(theme),
            span,
            packed,
        );
    }
}

/// The colors of one button in one frame.
#[derive(Clone, Copy)]
struct ButtonColors {
    /// The fill at rest.
    fill: Color,
    /// The fill while hovered.
    hovered: Color,
    /// The fill while held.
    pressed: Color,
    /// The label (and icon or spinner) color.
    label: Color,
    /// Whether a `border` frame is drawn.
    bordered: bool,
}

impl ButtonColors {
    /// The colors of a `kind` button in `state`. Regular buttons take the
    /// current style colors, so a surface may tint them (e.g. a toast).
    fn of(ui: &Ui, theme: &Theme, kind: ButtonKind, state: ButtonState<'_>) -> Self {
        let style = |color| Color::from_array(ui.style_color(color));
        let regular = Self {
            fill: style(StyleColor::Button),
            hovered: style(StyleColor::ButtonHovered),
            pressed: style(StyleColor::ButtonActive),
            label: style(StyleColor::Text),
            bordered: true,
        };
        let ready = match kind {
            ButtonKind::Regular => regular,
            ButtonKind::Primary => Self {
                fill: theme.accent.base,
                hovered: theme.accent.hover,
                pressed: theme.accent.active,
                label: theme.text.on_accent,
                bordered: false,
            },
            ButtonKind::DangerFill => Self {
                fill: theme.status.danger,
                hovered: theme.status.danger.with_alpha(0.85),
                pressed: theme.status.danger.with_alpha(0.7),
                label: theme.surface.window,
                bordered: false,
            },
        };
        match state {
            ButtonState::Ready => ready,
            // Busy keeps its look but does not react to the mouse.
            ButtonState::Busy => Self {
                hovered: ready.fill,
                pressed: ready.fill,
                ..ready
            },
            // A disabled button never shines brass or red: the regular fill
            // with a `text.disabled` label, no hover.
            ButtonState::Disabled { .. } => Self {
                hovered: regular.fill,
                pressed: regular.fill,
                label: theme.text.disabled,
                ..regular
            },
        }
    }
}
