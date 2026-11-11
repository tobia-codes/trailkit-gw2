//! Text roles and the one way addons draw text and icons (ROADMAP R96,
//! `tokens.md` §7.1).
//!
//! A [`TextStyle`] fixes the font and the color of a text role. Addon code
//! never pushes a font or a text color by hand: it calls [`draw_text`],
//! [`draw_text_wrapped`] or [`draw_truncated_text`] with a style, and
//! [`TextStyle::push`] only where it has to measure, center or style an
//! input (Q281). Icons go through [`draw_icon`].
//!
//! The roles and their colors are platform-independent; drawing is Windows
//! only, like all ImGui code (ROADMAP R39).

#[cfg(windows)]
use nexus::imgui::{ColorStackToken, StyleColor, Ui};

use crate::{color::Color, fonts::FontRole, theme::Theme};
#[cfg(windows)]
use crate::{
    font_registry::FontScope, icons::Icon, text_layout::truncate_to_width, tooltip::draw_tooltip,
};

/// A text role: font and color of one kind of text (`tokens.md` §7.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextStyle {
    /// Object or page title (bold, 1.5×, `text.primary`).
    Heading1,
    /// Section (bold, 1.3×, `text.primary`).
    Heading2,
    /// Subsection or dialog title (bold, 1.15×, `text.primary`).
    Heading3,
    /// Content, labels and empty-state titles (`text.primary`).
    Body,
    /// Emphasis, e.g. a group name in a list (bold, `text.primary`).
    BodyStrong,
    /// Info, metadata, hints and placeholders (`text.secondary`).
    Secondary,
    /// Fine print (0.85×, `text.secondary`).
    Small,
}

impl TextStyle {
    /// Every style, from the largest to the smallest.
    pub const ALL: [Self; 7] = [
        Self::Heading1,
        Self::Heading2,
        Self::Heading3,
        Self::Body,
        Self::BodyStrong,
        Self::Secondary,
        Self::Small,
    ];

    /// The font this style draws with.
    #[must_use]
    pub const fn font(self) -> FontRole {
        match self {
            Self::Heading1 => FontRole::Heading1,
            Self::Heading2 => FontRole::Heading2,
            Self::Heading3 => FontRole::Heading3,
            Self::Body | Self::Secondary => FontRole::Body,
            Self::BodyStrong => FontRole::BodyBold,
            Self::Small => FontRole::Small,
        }
    }

    /// The text color of this style in `theme`.
    #[must_use]
    pub const fn color(self, theme: &Theme) -> Color {
        match self {
            Self::Heading1 | Self::Heading2 | Self::Heading3 | Self::Body | Self::BodyStrong => {
                theme.text.primary
            }
            Self::Secondary | Self::Small => theme.text.secondary,
        }
    }

    /// The style for a markdown heading level (1 = largest, Q276).
    #[must_use]
    pub const fn heading(level: u8) -> Self {
        match level {
            0 | 1 => Self::Heading1,
            2 => Self::Heading2,
            _ => Self::Heading3,
        }
    }

    /// Pushes this style's font and color until the returned scope is
    /// dropped. Only for measuring, centering and inputs (Q281); drawing goes
    /// through [`draw_text`] and its variants.
    #[cfg(windows)]
    pub fn push<'ui>(self, ui: &'ui Ui, theme: &Theme) -> TextStyleScope<'ui> {
        TextStyleScope {
            _font: FontScope::push(ui, self.font()),
            _color: ui.push_style_color(StyleColor::Text, self.color(theme).to_array()),
        }
    }
}

/// The font and text color of a [`TextStyle`], pushed by
/// [`TextStyle::push`]; both are popped on drop.
#[cfg(windows)]
#[must_use = "the style is popped again as soon as the scope is dropped"]
pub struct TextStyleScope<'ui> {
    /// The pushed font.
    _font: FontScope<'ui>,
    /// The pushed text color.
    _color: ColorStackToken<'ui>,
}

/// Draws `text` on one line in `style`.
#[cfg(windows)]
pub fn draw_text(ui: &Ui, theme: &Theme, style: TextStyle, text: &str) {
    let _style = style.push(ui, theme);
    ui.text(text);
}

/// Draws `text` in `style`, wrapped at the current wrap position (the
/// window's right edge unless one is pushed).
#[cfg(windows)]
pub fn draw_text_wrapped(ui: &Ui, theme: &Theme, style: TextStyle, text: &str) {
    let _style = style.push(ui, theme);
    ui.text_wrapped(text);
}

/// Draws `text` in `style` on one line of at most `width`, shortened with
/// "…" if it does not fit (`lay-truncation`); a shortened text shows the
/// full text as a tooltip while hovered (Q282). Returns whether it was
/// shortened.
#[cfg(windows)]
pub fn draw_truncated_text(
    ui: &Ui,
    theme: &Theme,
    style: TextStyle,
    text: &str,
    width: f32,
) -> bool {
    let truncated = {
        let _style = style.push(ui, theme);
        let shown = truncate_to_width(ui, text, width);
        ui.text(&shown);
        matches!(shown, std::borrow::Cow::Owned(_))
    };
    // The tooltip uses the regular tooltip font, not the text's style.
    if truncated && ui.is_item_hovered() {
        draw_tooltip(ui, text);
    }
    truncated
}

/// Draws `icon` at body size in `color` as one item (R96, Q268). The color
/// follows the icon's role (`tokens.md` §7.1 rule 2).
#[cfg(windows)]
pub fn draw_icon(ui: &Ui, icon: Icon, color: Color) {
    let _icons = FontScope::push(ui, FontRole::Icons);
    let mut buffer = [0; 4];
    ui.text_colored(color.to_array(), icon.as_str(&mut buffer));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_follow_the_text_roles() {
        let theme = Theme::DARK;
        for style in TextStyle::ALL {
            let expected = match style {
                TextStyle::Secondary | TextStyle::Small => theme.text.secondary,
                _ => theme.text.primary,
            };
            assert_eq!(style.color(&theme), expected, "{style:?}");
        }
        assert_eq!(TextStyle::BodyStrong.font(), FontRole::BodyBold);
        assert_eq!(TextStyle::Small.font(), FontRole::Small);
        assert_eq!(TextStyle::heading(1), TextStyle::Heading1);
        assert_eq!(TextStyle::heading(6), TextStyle::Heading3);
    }
}
