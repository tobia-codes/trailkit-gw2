//! RAII guard that applies a [`Theme`] to the ImGui context shared by all
//! Nexus addons (nexus-addon-dev skill §1, trailkit-ui skill §2).
//!
//! The global ImGui style is never modified: [`StyleScope::push`] pushes every
//! themed color and style variable onto ImGui's stacks, and dropping the scope
//! pops exactly those entries again. The pushed values live in fixed-size
//! arrays, so applying a theme allocates nothing per frame.

use nexus::imgui::{ColorStackToken, StyleColor, StyleStackToken, StyleVar, Ui};

use crate::{color::Color, theme::Theme};

/// Reads one color token from a theme.
type ColorToken = fn(&Theme) -> Color;

/// Maps every themed ImGui color slot to the theme token it uses.
const COLOR_SLOTS: [(StyleColor, ColorToken); 44] = [
    (StyleColor::Text, |theme| theme.text.primary),
    (StyleColor::TextDisabled, |theme| theme.text.disabled),
    (StyleColor::WindowBg, |theme| theme.surface.window),
    (StyleColor::ChildBg, |theme| theme.surface.panel),
    (StyleColor::PopupBg, |theme| theme.surface.popup),
    (StyleColor::Border, |theme| theme.border.color),
    (StyleColor::BorderShadow, |_| Color::TRANSPARENT),
    (StyleColor::FrameBg, |theme| theme.surface.input),
    (StyleColor::FrameBgHovered, |theme| theme.surface.hover),
    (StyleColor::FrameBgActive, |theme| theme.surface.active),
    (StyleColor::TitleBg, |theme| theme.surface.panel),
    (StyleColor::TitleBgActive, |theme| theme.surface.active),
    (StyleColor::TitleBgCollapsed, |theme| theme.surface.panel),
    (StyleColor::MenuBarBg, |theme| theme.surface.panel),
    (StyleColor::ScrollbarBg, |theme| theme.surface.window),
    (StyleColor::ScrollbarGrab, |theme| theme.surface.active),
    (StyleColor::ScrollbarGrabHovered, |theme| {
        theme.text.disabled
    }),
    (StyleColor::ScrollbarGrabActive, |theme| {
        theme.text.secondary
    }),
    (StyleColor::CheckMark, |theme| theme.accent.base),
    (StyleColor::SliderGrab, |theme| theme.accent.base),
    (StyleColor::SliderGrabActive, |theme| theme.accent.active),
    (StyleColor::Button, |theme| theme.surface.input),
    (StyleColor::ButtonHovered, |theme| theme.surface.hover),
    (StyleColor::ButtonActive, |theme| theme.surface.active),
    (StyleColor::Header, |theme| theme.surface.hover),
    (StyleColor::HeaderHovered, |theme| theme.surface.hover),
    (StyleColor::HeaderActive, |theme| theme.surface.active),
    (StyleColor::Separator, |theme| theme.border.color),
    (StyleColor::SeparatorHovered, |theme| theme.border.focus),
    (StyleColor::SeparatorActive, |theme| theme.border.focus),
    (StyleColor::ResizeGrip, |theme| theme.border.color),
    (StyleColor::ResizeGripHovered, |theme| theme.accent.hover),
    (StyleColor::ResizeGripActive, |theme| theme.accent.active),
    (StyleColor::Tab, |theme| theme.surface.panel),
    (StyleColor::TabHovered, |theme| theme.surface.hover),
    (StyleColor::TabActive, |theme| theme.surface.active),
    (StyleColor::TabUnfocused, |theme| theme.surface.panel),
    (StyleColor::TabUnfocusedActive, |theme| theme.surface.active),
    (StyleColor::TableHeaderBg, |theme| theme.surface.panel),
    (StyleColor::TableBorderStrong, |theme| theme.border.color),
    (StyleColor::TableBorderLight, |theme| theme.border.color),
    (StyleColor::TextSelectedBg, |theme| {
        theme.accent.base.with_alpha(0.35)
    }),
    (StyleColor::NavHighlight, |theme| theme.border.focus),
    (StyleColor::ModalWindowDimBg, |theme| {
        theme.surface.modal_overlay
    }),
];

/// Maps every themed ImGui style variable to the theme tokens it uses.
const STYLE_VARS: [fn(&Theme) -> StyleVar; 14] = [
    |theme| StyleVar::WindowRounding(theme.radius),
    |theme| StyleVar::ChildRounding(theme.radius),
    |theme| StyleVar::PopupRounding(theme.radius),
    |theme| StyleVar::FrameRounding(theme.radius),
    |theme| StyleVar::ScrollbarRounding(theme.radius),
    |theme| StyleVar::GrabRounding(theme.radius),
    |theme| StyleVar::TabRounding(theme.radius),
    |theme| StyleVar::WindowBorderSize(theme.border.width),
    |theme| StyleVar::PopupBorderSize(theme.border.width),
    |theme| StyleVar::ChildBorderSize(theme.border.width),
    |_| StyleVar::FrameBorderSize(0.0),
    |theme| StyleVar::WindowPadding([theme.spacing.medium, theme.spacing.medium]),
    |theme| StyleVar::FramePadding([theme.spacing.small, theme.spacing.extra_small]),
    |theme| StyleVar::ItemSpacing([theme.spacing.small, theme.spacing.small]),
];

/// A theme applied to the shared ImGui context; restores the previous style
/// when dropped. Create it right before drawing an addon window and keep it
/// alive until the window is finished.
#[must_use = "the theme is popped again as soon as the scope is dropped"]
pub struct StyleScope<'ui> {
    /// Pushed colors; each token pops one color when dropped.
    _colors: [ColorStackToken<'ui>; COLOR_SLOTS.len()],
    /// Pushed style variables; each token pops one variable when dropped.
    _style_vars: [StyleStackToken<'ui>; STYLE_VARS.len()],
}

impl<'ui> StyleScope<'ui> {
    /// Pushes all colors and style variables of `theme` onto ImGui's stacks.
    pub fn push(ui: &'ui Ui, theme: &Theme) -> Self {
        Self {
            _colors: COLOR_SLOTS
                .map(|(slot, token)| ui.push_style_color(slot, token(theme).to_array())),
            _style_vars: STYLE_VARS.map(|style_var| ui.push_style_var(style_var(theme))),
        }
    }
}
