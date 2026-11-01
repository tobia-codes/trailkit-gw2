//! Semantic theme tokens and the built-in themes (trailkit-ui skill §2).
//!
//! Components never use raw colors or numbers; they read tokens from the
//! active [`Theme`]. Sizes are in pixels at 100 % UI scaling.

use crate::color::Color;

/// The built-in themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeKind {
    /// Dark theme (default).
    #[default]
    Dark,
    /// Light theme.
    Light,
}

impl ThemeKind {
    /// Every built-in theme, in a stable order.
    pub const ALL: [Self; 2] = [Self::Dark, Self::Light];

    /// Returns the full token set of this built-in theme.
    #[must_use]
    pub const fn theme(self) -> Theme {
        match self {
            Self::Dark => Theme::DARK,
            Self::Light => Theme::LIGHT,
        }
    }
}

/// A complete set of design tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    /// Background colors (`bg.*`, `overlay.modal`).
    pub surface: SurfaceColors,
    /// Text colors (`text.*`).
    pub text: TextColors,
    /// Accent colors (`accent*`).
    pub accent: AccentColors,
    /// Status colors (`success`, `warning`, `danger`, `info`).
    pub status: StatusColors,
    /// Border tokens (`border*`).
    pub border: BorderTokens,
    /// Event timeline colors (`event.*`).
    pub event: EventColors,
    /// Coin colors of chat-link chips (memo plan M75).
    pub coin: CoinColors,
    /// Spacing scale (`space.*`).
    pub spacing: Spacing,
    /// List metrics (`list.*`).
    pub list: ListMetrics,
    /// Pane metrics (`pane.*`).
    pub pane: PaneMetrics,
    /// Corner radius (`radius`, default 0 — flat and square).
    pub radius: f32,
    /// Font sizes (`font.size.*`).
    pub typography: Typography,
}

/// Background colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceColors {
    /// `bg.window`: window background.
    pub window: Color,
    /// `bg.panel`: panels, title bars, child regions.
    pub panel: Color,
    /// `bg.input`: input fields and default buttons.
    pub input: Color,
    /// `bg.hover`: hovered interactive elements.
    pub hover: Color,
    /// `bg.active`: pressed / selected elements.
    pub active: Color,
    /// `bg.popup`: popups, tooltips, menus.
    pub popup: Color,
    /// `overlay.modal`: dimming behind modal dialogs.
    pub modal_overlay: Color,
}

/// Text colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextColors {
    /// `text.primary`: body text.
    pub primary: Color,
    /// `text.secondary`: help text, captions.
    pub secondary: Color,
    /// `text.disabled`: disabled controls (exempt from the contrast rule).
    pub disabled: Color,
    /// `text.link`: links.
    pub link: Color,
    /// `text.on_accent`: text on accent-colored backgrounds.
    pub on_accent: Color,
}

/// Accent colors (primary actions, focus, selection).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccentColors {
    /// `accent`: resting state.
    pub base: Color,
    /// `accent.hover`: hovered state.
    pub hover: Color,
    /// `accent.active`: pressed state.
    pub active: Color,
}

/// Status colors (always combined with an icon or text, never color alone).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusColors {
    /// `success`.
    pub success: Color,
    /// `warning`.
    pub warning: Color,
    /// `danger`.
    pub danger: Color,
    /// `info`.
    pub info: Color,
}

/// Border tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderTokens {
    /// `border`: default border color.
    pub color: Color,
    /// `border.focus`: focused element border.
    pub focus: Color,
    /// `border.width`: border thickness in pixels.
    pub width: f32,
}

/// Gold, silver and copper, as the game colors coin amounts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoinColors {
    /// `coin.gold`.
    pub gold: Color,
    /// `coin.silver`.
    pub silver: Color,
    /// `coin.copper`.
    pub copper: Color,
}

/// Event timeline colors (per-category overrides allowed later).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EventColors {
    /// `event.default`.
    pub default: Color,
    /// `event.highlight`.
    pub highlight: Color,
    /// `event.active`.
    pub active: Color,
    /// `event.upcoming`.
    pub upcoming: Color,
    /// `event.custom`.
    pub custom: Color,
}

/// Spacing scale in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spacing {
    /// `space.xs` (4).
    pub extra_small: f32,
    /// `space.sm` (8).
    pub small: f32,
    /// `space.md` (12).
    pub medium: f32,
    /// `space.lg` (16).
    pub large: f32,
    /// `space.xl` (24).
    pub extra_large: f32,
}

/// List metrics in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListMetrics {
    /// `list.item_spacing`: vertical gap between rows.
    pub item_spacing: f32,
    /// `list.indent`: indentation per tree level.
    pub indent: f32,
}

/// Pane metrics in pixels (content regions inside a window).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaneMetrics {
    /// `pane.padding`: inner padding on every side of a pane; no content
    /// touches a pane edge (`lay-pane-padding`).
    pub padding: f32,
    /// `pane.gap`: space between two neighboring panes (holds the splitter).
    pub gap: f32,
}

/// Font sizes in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Typography {
    /// `font.size.small`.
    pub small: f32,
    /// `font.size.body`.
    pub body: f32,
    /// `font.size.heading`.
    pub heading: f32,
}

/// Tokens shared by both built-in themes (sizes, not colors).
const SPACING: Spacing = Spacing {
    extra_small: 4.0,
    small: 8.0,
    medium: 12.0,
    large: 16.0,
    extra_large: 24.0,
};

/// List metrics shared by both built-in themes.
const LIST_METRICS: ListMetrics = ListMetrics {
    item_spacing: 4.0,
    indent: 16.0,
};

/// Pane metrics shared by both built-in themes (`space.sm`).
const PANE_METRICS: PaneMetrics = PaneMetrics {
    padding: SPACING.small,
    gap: SPACING.small,
};

/// Font sizes shared by both built-in themes.
const TYPOGRAPHY: Typography = Typography {
    small: 12.0,
    body: 14.0,
    heading: 18.0,
};

impl Theme {
    /// The built-in dark theme (default).
    pub const DARK: Self = Self {
        surface: SurfaceColors {
            window: Color::from_rgb_hex(0x1B1D21),
            panel: Color::from_rgb_hex(0x23262B),
            input: Color::from_rgb_hex(0x2C3036),
            hover: Color::from_rgb_hex(0x353A42),
            active: Color::from_rgb_hex(0x40464F),
            popup: Color::from_rgb_hex(0x202327),
            modal_overlay: Color::from_rgba_hex(0x0000_00B3),
        },
        text: TextColors {
            primary: Color::from_rgb_hex(0xE8E9EB),
            secondary: Color::from_rgb_hex(0xA9AEB6),
            disabled: Color::from_rgb_hex(0x6B7079),
            link: Color::from_rgb_hex(0x8AB8FF),
            on_accent: Color::from_rgb_hex(0x16181B),
        },
        accent: AccentColors {
            base: Color::from_rgb_hex(0xD9A441),
            hover: Color::from_rgb_hex(0xE6B65C),
            active: Color::from_rgb_hex(0xC2902F),
        },
        status: StatusColors {
            success: Color::from_rgb_hex(0x5CC27A),
            warning: Color::from_rgb_hex(0xE5B94C),
            danger: Color::from_rgb_hex(0xF48A8A),
            info: Color::from_rgb_hex(0x6AAEF2),
        },
        border: BorderTokens {
            color: Color::from_rgb_hex(0x3A3F47),
            focus: Color::from_rgb_hex(0xD9A441),
            width: 1.0,
        },
        coin: CoinColors {
            gold: Color::from_rgba_hex(0xE8C1_4AFF),
            silver: Color::from_rgba_hex(0xC4C8_CEFF),
            copper: Color::from_rgba_hex(0xC87F_4AFF),
        },
        event: EventColors {
            default: Color::from_rgb_hex(0x8A9099),
            highlight: Color::from_rgb_hex(0xF2C14E),
            active: Color::from_rgb_hex(0x5CC27A),
            upcoming: Color::from_rgb_hex(0x6AAEF2),
            custom: Color::from_rgb_hex(0xB48EF0),
        },
        spacing: SPACING,
        list: LIST_METRICS,
        pane: PANE_METRICS,
        radius: 0.0,
        typography: TYPOGRAPHY,
    };

    /// The built-in light theme.
    pub const LIGHT: Self = Self {
        surface: SurfaceColors {
            window: Color::from_rgb_hex(0xF4F5F7),
            panel: Color::from_rgb_hex(0xFFFFFF),
            input: Color::from_rgb_hex(0xFFFFFF),
            hover: Color::from_rgb_hex(0xE6E8EC),
            active: Color::from_rgb_hex(0xD6DAE0),
            popup: Color::from_rgb_hex(0xFFFFFF),
            modal_overlay: Color::from_rgba_hex(0x0000_0080),
        },
        text: TextColors {
            primary: Color::from_rgb_hex(0x1B1D21),
            secondary: Color::from_rgb_hex(0x4F5560),
            disabled: Color::from_rgb_hex(0x9AA0A9),
            link: Color::from_rgb_hex(0x1F5FBF),
            on_accent: Color::from_rgb_hex(0xFFFFFF),
        },
        accent: AccentColors {
            base: Color::from_rgb_hex(0x8C5E0A),
            hover: Color::from_rgb_hex(0x7A5208),
            active: Color::from_rgb_hex(0x664406),
        },
        status: StatusColors {
            success: Color::from_rgb_hex(0x1A7034),
            warning: Color::from_rgb_hex(0x8A5A00),
            danger: Color::from_rgb_hex(0xB42318),
            info: Color::from_rgb_hex(0x1F5FBF),
        },
        border: BorderTokens {
            color: Color::from_rgb_hex(0xC9CED6),
            focus: Color::from_rgb_hex(0x8C5E0A),
            width: 1.0,
        },
        coin: CoinColors {
            gold: Color::from_rgba_hex(0xA67C_00FF),
            silver: Color::from_rgba_hex(0x7A80_88FF),
            copper: Color::from_rgba_hex(0x9C5A_2AFF),
        },
        event: EventColors {
            default: Color::from_rgb_hex(0x5F6670),
            highlight: Color::from_rgb_hex(0x9A6B00),
            active: Color::from_rgb_hex(0x1A7034),
            upcoming: Color::from_rgb_hex(0x1F5FBF),
            custom: Color::from_rgb_hex(0x6B3FC0),
        },
        spacing: SPACING,
        list: LIST_METRICS,
        pane: PANE_METRICS,
        radius: 0.0,
        typography: TYPOGRAPHY,
    };
}

impl Default for Theme {
    /// The default theme is [`Theme::DARK`].
    fn default() -> Self {
        Self::DARK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimum WCAG contrast for text (trailkit-ui skill §1.5).
    const MINIMUM_TEXT_CONTRAST: f32 = 4.5;

    /// Asserts `foreground` on `background` meets the text contrast minimum.
    fn assert_readable(kind: ThemeKind, name: &str, foreground: Color, background: Color) {
        let ratio = foreground.contrast_ratio(background);
        assert!(
            ratio >= MINIMUM_TEXT_CONTRAST,
            "{kind:?}: {name} has contrast {ratio:.2}, needs {MINIMUM_TEXT_CONTRAST}"
        );
    }

    #[test]
    fn text_meets_contrast_minimum_in_every_theme() {
        for kind in ThemeKind::ALL {
            let theme = kind.theme();
            let backgrounds = [
                ("bg.window", theme.surface.window),
                ("bg.panel", theme.surface.panel),
                ("bg.input", theme.surface.input),
                ("bg.popup", theme.surface.popup),
                ("bg.hover", theme.surface.hover),
            ];
            let texts = [
                ("text.primary", theme.text.primary),
                ("text.secondary", theme.text.secondary),
                ("text.link", theme.text.link),
                ("success", theme.status.success),
                ("warning", theme.status.warning),
                ("danger", theme.status.danger),
                ("info", theme.status.info),
            ];
            for (background_name, background) in backgrounds {
                for (text_name, text) in texts {
                    assert_readable(
                        kind,
                        &format!("{text_name} on {background_name}"),
                        text,
                        background,
                    );
                }
            }
            for (accent_name, accent) in [
                ("accent", theme.accent.base),
                ("accent.hover", theme.accent.hover),
                ("accent.active", theme.accent.active),
            ] {
                assert_readable(
                    kind,
                    &format!("text.on_accent on {accent_name}"),
                    theme.text.on_accent,
                    accent,
                );
            }
        }
    }

    #[test]
    fn built_in_themes_are_square() {
        for kind in ThemeKind::ALL {
            assert_eq!(kind.theme().radius, 0.0);
        }
    }

    #[test]
    fn dark_is_default() {
        assert_eq!(ThemeKind::default(), ThemeKind::Dark);
        assert_eq!(Theme::default(), Theme::DARK);
    }
}
