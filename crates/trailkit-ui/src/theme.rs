//! Semantic theme tokens and the built-in themes (trailkit-ui skill §2,
//! `references/tokens.md`; palette ROADMAP R71).
//!
//! Components never use raw colors or numbers; they read tokens from the
//! active [`Theme`]. Sizes are in pixels at 100 % UI scaling.

mod layout;
mod typography;

pub use layout::{ButtonLayout, ListLayout, MenuLayout, PaneLayout, Radii, Spacing};
pub use typography::Typography;

use crate::{chat_link::ChipCategory, color::Color};
use layout::{BUTTON_LAYOUT, LIST_LAYOUT, MENU_LAYOUT, PANE_LAYOUT, RADII, SPACING};
use typography::TYPOGRAPHY;

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

    /// Stable lowercase code of the theme for logs and files: `dark`, `light`.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

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
    /// Colors of the reset note kinds (`reset.*`, memo plan M104).
    pub reset: ResetColors,
    /// Colors of the chat-link chip categories (`chip.*`, R71).
    pub chip: ChipColors,
    /// Spacing scale (`space.*`).
    pub spacing: Spacing,
    /// List layout (`list.*`).
    pub list: ListLayout,
    /// Pane layout (`pane.*`).
    pub pane: PaneLayout,
    /// Button layout (`button.*`, memo plan M131).
    pub button: ButtonLayout,
    /// Menu layout (`menu.*`, memo M177).
    pub menu: MenuLayout,
    /// Corner radii (`radius.*`, R71).
    pub radius: Radii,
    /// Shadows (`shadow.*`, Q327): only tooltips have one.
    pub shadow: ShadowTokens,
    /// Font sizes and line height (`font.*`).
    pub typography: Typography,
}

/// Background colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceColors {
    /// `bg.window`: window background.
    pub window: Color,
    /// `bg.panel`: panels, title bars, child regions.
    pub panel: Color,
    /// `bg.input`: input fields.
    pub input: Color,
    /// `bg.hover`: hovered interactive elements.
    pub hover: Color,
    /// `bg.active`: pressed / selected elements.
    pub active: Color,
    /// `bg.pressed`: pressed regular buttons, dropdowns and menu rows (R85).
    pub pressed: Color,
    /// `bg.popup`: popups, tooltips, menus.
    pub popup: Color,
    /// `bg.popup_hover`: hovered and pressed rows inside popups, where
    /// `bg.hover` is almost invisible on `bg.popup` (ROADMAP R80).
    pub popup_hover: Color,
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

/// OKLab lightness step from a colored button's fill to its hover fill
/// (R107): 8 % darker in both themes.
pub const HOVER_DARKENING: f32 = 0.08;

/// OKLab lightness step from a colored button's fill to its pressed fill
/// (R107): 12 % darker in both themes.
pub const PRESSED_DARKENING: f32 = 0.12;

/// Accent colors: brass, spent only on the primary action, focus and
/// selection (R71).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccentColors {
    /// `accent`: fill of the primary button and the selection bar. A fill
    /// only: in the light theme it is too light for text or thin lines.
    pub base: Color,
    /// `accent.hover`: hovered primary button, `accent` darkened by
    /// [`HOVER_DARKENING`] (R107).
    pub hover: Color,
    /// `accent.active`: pressed primary button, `accent` darkened by
    /// [`PRESSED_DARKENING`] (R107).
    pub active: Color,
    /// `accent.text`: brass as text, icon or line on a surface (active
    /// toggles, check marks, drop line); ≥ 4.5:1 in both themes.
    pub text: Color,
    /// `accent.highlight`: translucent brass behind search hits (memo M122);
    /// text on it keeps ≥ 4.5:1 over every surface.
    pub highlight: Color,
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
    /// `danger.hover`: hovered delete button, `danger` darkened by
    /// [`HOVER_DARKENING`] (R105, R107).
    pub danger_hover: Color,
    /// `danger.active`: pressed delete button, `danger` darkened by
    /// [`PRESSED_DARKENING`] (R105, R107).
    pub danger_active: Color,
    /// `danger.subtle`: `danger` tinted into `bg.popup` (20 % Dark, 10 %
    /// Light), the fill of an error toast (memo M180, Q144 B) and of a
    /// `danger` status card (memo M184).
    pub danger_subtle: Color,
    /// `warning.subtle`: `warning` tinted into `bg.popup` (16 % Dark, 10 %
    /// Light), the fill of a `warning` status card (memo M184).
    pub warning_subtle: Color,
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
    /// `border.subtle`: the faint outline of windows, panes, popups and
    /// dialogs (memo M103).
    pub subtle: Color,
    /// `border.width`: border thickness in pixels.
    pub width: f32,
}

/// Colors of the reset note kinds: badge and kind chip share them (memo
/// plan M104). Text colors, so they keep ≥ 4.5:1 contrast.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResetColors {
    /// `reset.daily`: daily reset notes (gold, like the accent).
    pub daily: Color,
    /// `reset.weekly`: weekly reset notes (muted violet, apart from the
    /// gold accent and the blue links).
    pub weekly: Color,
}

/// Text and background color of one chat-link chip category.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChipCategoryColors {
    /// Color of the type icon and label (≥ 4.5:1 on `background`).
    pub text: Color,
    /// Fill of the chip.
    pub background: Color,
}

/// The four chat-link chip categories (R71); `ChatLinkKind::category`
/// assigns every link kind to one of them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChipColors {
    /// `chip.place`: map point, WvW objective (copper).
    pub place: ChipCategoryColors,
    /// `chip.character`: skill, trait, build (blue).
    pub character: ChipCategoryColors,
    /// `chip.item`: item, recipe, skin, outfit, coins (jade).
    pub item: ChipCategoryColors,
    /// `chip.progress`: achievement, NPC text, player (violet).
    pub progress: ChipCategoryColors,
}

impl ChipColors {
    /// The colors of `category`.
    #[must_use]
    pub const fn for_category(&self, category: ChipCategory) -> ChipCategoryColors {
        match category {
            ChipCategory::Place => self.place,
            ChipCategory::Character => self.character,
            ChipCategory::Item => self.item,
            ChipCategory::Progress => self.progress,
        }
    }
}

/// Shadow tokens (Q327, Q328): the one exception to "no shadows" (R71) is
/// the tooltip, which floats over panes and rows and was hard to see. ImGui
/// cannot blur, so a shadow is the same shape drawn offset and darker.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowTokens {
    /// `shadow.popup`: the color of a tooltip's shadow.
    pub popup: Color,
    /// `shadow.popup.offset`: how far the shadow reaches below the tooltip,
    /// in pixels.
    pub popup_offset: f32,
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

impl Theme {
    /// The built-in dark theme (default): ink-tinted surfaces, brass accent.
    pub const DARK: Self = Self {
        surface: SurfaceColors {
            window: Color::from_rgb_hex(0x0F1318),
            panel: Color::from_rgb_hex(0x151A21),
            input: Color::from_rgb_hex(0x0C1015),
            hover: Color::from_rgb_hex(0x1D232C),
            active: Color::from_rgb_hex(0x232A35),
            pressed: Color::from_rgb_hex(0x323B49),
            popup: Color::from_rgb_hex(0x1A2029),
            popup_hover: Color::from_rgb_hex(0x2A323E),
            modal_overlay: Color::from_rgba_hex(0x0507_0AB8),
        },
        text: TextColors {
            primary: Color::from_rgb_hex(0xE9ECF1),
            secondary: Color::from_rgb_hex(0x9AA4B2),
            disabled: Color::from_rgb_hex(0x5E6876),
            link: Color::from_rgb_hex(0x8DB8F0),
            on_accent: Color::from_rgb_hex(0x17130A),
        },
        accent: AccentColors {
            base: Color::from_rgb_hex(0xE2A740),
            hover: Color::from_rgb_hex(0xC88E1E),
            active: Color::from_rgb_hex(0xBB8200),
            text: Color::from_rgb_hex(0xE8B055),
            highlight: Color::from_rgb_hex(0xE2A740).with_alpha(0.24),
        },
        status: StatusColors {
            success: Color::from_rgb_hex(0x5CC48A),
            warning: Color::from_rgb_hex(0xE9B44C),
            danger: Color::from_rgb_hex(0xF08A84),
            danger_hover: Color::from_rgb_hex(0xD4716C),
            danger_active: Color::from_rgb_hex(0xC76561),
            // 20 % `danger` over `bg.popup` (#1A2029); 22 % would leave
            // `text.secondary` below 4.5:1.
            danger_subtle: Color::from_rgb_hex(0x45353B),
            // 16 % `warning` over `bg.popup` (#1A2029); 18 % would leave
            // `text.secondary` below 4.5:1.
            warning_subtle: Color::from_rgb_hex(0x3B382F),
            info: Color::from_rgb_hex(0x7AB0F0),
        },
        border: BorderTokens {
            color: Color::from_rgb_hex(0x2A323E),
            focus: Color::from_rgb_hex(0xE8B055),
            // About 6 % white over the dark surfaces.
            subtle: Color::from_rgb_hex(0xFFFFFF).with_alpha(0.06),
            width: 1.0,
        },
        shadow: ShadowTokens {
            popup: Color::from_rgb_hex(0x000000).with_alpha(0.55),
            popup_offset: 2.0,
        },
        reset: ResetColors {
            daily: Color::from_rgb_hex(0xE8B055),
            weekly: Color::from_rgb_hex(0xB8A6F0),
        },
        chip: ChipColors {
            place: ChipCategoryColors {
                text: Color::from_rgb_hex(0xE89A6B),
                background: Color::from_rgb_hex(0x2E292A),
            },
            character: ChipCategoryColors {
                text: Color::from_rgb_hex(0x8DB8F0),
                background: Color::from_rgb_hex(0x232D3A),
            },
            item: ChipCategoryColors {
                text: Color::from_rgb_hex(0x6CCDB3),
                background: Color::from_rgb_hex(0x1F2E31),
            },
            progress: ChipCategoryColors {
                text: Color::from_rgb_hex(0xB8A6F0),
                background: Color::from_rgb_hex(0x2A2C3C),
            },
        },
        coin: CoinColors {
            gold: Color::from_rgba_hex(0xE8C1_4AFF),
            silver: Color::from_rgba_hex(0xC4C8_CEFF),
            copper: Color::from_rgba_hex(0xC87F_4AFF),
        },
        event: EventColors {
            default: Color::from_rgb_hex(0x8E98A6),
            highlight: Color::from_rgb_hex(0xE8B055),
            active: Color::from_rgb_hex(0x5CC48A),
            upcoming: Color::from_rgb_hex(0x7AB0F0),
            custom: Color::from_rgb_hex(0xB8A6F0),
        },
        spacing: SPACING,
        list: LIST_LAYOUT,
        pane: PANE_LAYOUT,
        button: BUTTON_LAYOUT,
        menu: MENU_LAYOUT,
        radius: RADII,
        typography: TYPOGRAPHY,
    };

    /// The built-in light theme: parchment-warm surfaces, brass accent.
    pub const LIGHT: Self = Self {
        surface: SurfaceColors {
            window: Color::from_rgb_hex(0xEFEDE8),
            panel: Color::from_rgb_hex(0xFAF9F6),
            input: Color::from_rgb_hex(0xFFFFFF),
            hover: Color::from_rgb_hex(0xF0EEE9),
            active: Color::from_rgb_hex(0xE7E3DB),
            pressed: Color::from_rgb_hex(0xDDD8CE),
            popup: Color::from_rgb_hex(0xFFFFFF),
            popup_hover: Color::from_rgb_hex(0xE7E3DB),
            modal_overlay: Color::from_rgba_hex(0x1410_0873),
        },
        text: TextColors {
            primary: Color::from_rgb_hex(0x1B1E23),
            secondary: Color::from_rgb_hex(0x555C66),
            disabled: Color::from_rgb_hex(0x9AA0A8),
            link: Color::from_rgb_hex(0x1F5DB0),
            on_accent: Color::from_rgb_hex(0x17130A),
        },
        accent: AccentColors {
            base: Color::from_rgb_hex(0xE2A740),
            hover: Color::from_rgb_hex(0xC88E1E),
            active: Color::from_rgb_hex(0xBB8200),
            text: Color::from_rgb_hex(0x8A5A10),
            highlight: Color::from_rgb_hex(0xE2A740).with_alpha(0.42),
        },
        status: StatusColors {
            success: Color::from_rgb_hex(0x176B3C),
            warning: Color::from_rgb_hex(0x8A5A00),
            danger: Color::from_rgb_hex(0xB3261E),
            danger_hover: Color::from_rgb_hex(0x970000),
            danger_active: Color::from_rgb_hex(0x890000),
            // 10 % `danger` over `bg.popup` (#FFFFFF).
            danger_subtle: Color::from_rgb_hex(0xF7E9E8),
            // 10 % `warning` over `bg.popup` (#FFFFFF).
            warning_subtle: Color::from_rgb_hex(0xF3EEE5),
            info: Color::from_rgb_hex(0x1F5DB0),
        },
        border: BorderTokens {
            color: Color::from_rgb_hex(0xD6D1C7),
            focus: Color::from_rgb_hex(0x8A5A10),
            // About 8 % warm black over the light surfaces.
            subtle: Color::from_rgb_hex(0x1E180C).with_alpha(0.08),
            width: 1.0,
        },
        shadow: ShadowTokens {
            popup: Color::from_rgb_hex(0x1E180C).with_alpha(0.18),
            popup_offset: 2.0,
        },
        reset: ResetColors {
            daily: Color::from_rgb_hex(0x8A5A10),
            weekly: Color::from_rgb_hex(0x6141B5),
        },
        chip: ChipColors {
            place: ChipCategoryColors {
                text: Color::from_rgb_hex(0xA2471A),
                background: Color::from_rgb_hex(0xFBE9DF),
            },
            character: ChipCategoryColors {
                text: Color::from_rgb_hex(0x1F5DB0),
                background: Color::from_rgb_hex(0xE5EEFA),
            },
            item: ChipCategoryColors {
                text: Color::from_rgb_hex(0x0E6E58),
                background: Color::from_rgb_hex(0xDDF2EB),
            },
            progress: ChipCategoryColors {
                text: Color::from_rgb_hex(0x6141B5),
                background: Color::from_rgb_hex(0xEDE8FA),
            },
        },
        coin: CoinColors {
            gold: Color::from_rgba_hex(0xA67C_00FF),
            silver: Color::from_rgba_hex(0x7A80_88FF),
            copper: Color::from_rgba_hex(0x9C5A_2AFF),
        },
        event: EventColors {
            default: Color::from_rgb_hex(0x5F6670),
            highlight: Color::from_rgb_hex(0x8A5A10),
            active: Color::from_rgb_hex(0x176B3C),
            upcoming: Color::from_rgb_hex(0x1F5DB0),
            custom: Color::from_rgb_hex(0x6141B5),
        },
        spacing: SPACING,
        list: LIST_LAYOUT,
        pane: PANE_LAYOUT,
        button: BUTTON_LAYOUT,
        menu: MENU_LAYOUT,
        radius: RADII,
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
                ("bg.popup_hover", theme.surface.popup_hover),
                ("bg.hover", theme.surface.hover),
                ("bg.active", theme.surface.active),
            ];
            let texts = [
                ("text.primary", theme.text.primary),
                ("text.secondary", theme.text.secondary),
                ("text.link", theme.text.link),
                ("accent.text", theme.accent.text),
                ("success", theme.status.success),
                ("warning", theme.status.warning),
                ("danger", theme.status.danger),
                ("info", theme.status.info),
                ("reset.daily", theme.reset.daily),
                ("reset.weekly", theme.reset.weekly),
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
            // A delete button's label is `bg.window` (R105).
            for (danger_name, danger) in [
                ("danger", theme.status.danger),
                ("danger.hover", theme.status.danger_hover),
                ("danger.active", theme.status.danger_active),
            ] {
                assert_readable(
                    kind,
                    &format!("bg.window on {danger_name}"),
                    theme.surface.window,
                    danger,
                );
            }
        }
    }

    /// Hover and pressed fills of colored buttons follow one rule (R107):
    /// the rest fill darkened by a fixed OKLab lightness step. The theme
    /// stores hex values; they may differ from the formula by rounding only.
    #[test]
    fn button_state_fills_follow_the_darkening_rule() {
        for kind in ThemeKind::ALL {
            let theme = kind.theme();
            for (name, base, stored, step) in [
                (
                    "accent.hover",
                    theme.accent.base,
                    theme.accent.hover,
                    HOVER_DARKENING,
                ),
                (
                    "accent.active",
                    theme.accent.base,
                    theme.accent.active,
                    PRESSED_DARKENING,
                ),
                (
                    "danger.hover",
                    theme.status.danger,
                    theme.status.danger_hover,
                    HOVER_DARKENING,
                ),
                (
                    "danger.active",
                    theme.status.danger,
                    theme.status.danger_active,
                    PRESSED_DARKENING,
                ),
            ] {
                let expected = base.darkened(step).to_array();
                for (channel, (stored, expected)) in
                    stored.to_array().iter().zip(expected).take(3).enumerate()
                {
                    assert!(
                        (stored - expected).abs() <= 1.0 / 255.0,
                        "{kind:?}: {name} channel {channel} is {stored:.4}, rule gives {expected:.4}"
                    );
                }
            }
        }
    }

    /// The error toast's message, counter and icon sit on `danger.subtle`;
    /// its buttons use `bg.active` (memo M180). A status card puts the
    /// status color, primary and secondary text on its tinted fill (M184).
    #[test]
    fn tinted_status_fills_meet_contrast_minimum() {
        for kind in ThemeKind::ALL {
            let theme = kind.theme();
            for (fill_name, fill, status) in [
                (
                    "danger.subtle",
                    theme.status.danger_subtle,
                    theme.status.danger,
                ),
                (
                    "warning.subtle",
                    theme.status.warning_subtle,
                    theme.status.warning,
                ),
            ] {
                for (name, text) in [
                    ("text.primary", theme.text.primary),
                    ("text.secondary", theme.text.secondary),
                    ("status", status),
                ] {
                    assert_readable(kind, &format!("{name} on {fill_name}"), text, fill);
                }
            }
        }
    }

    #[test]
    fn text_stays_readable_on_search_highlights() {
        for kind in ThemeKind::ALL {
            let theme = kind.theme();
            for (surface_name, surface) in [
                ("bg.panel", theme.surface.panel),
                ("bg.hover", theme.surface.hover),
                ("bg.active", theme.surface.active),
            ] {
                let highlight = theme.accent.highlight.over(surface);
                // Titles (text.primary) sit on list rows; links only occur
                // in the note view on bg.panel.
                let link = (surface_name == "bg.panel").then_some(("text.link", theme.text.link));
                for (text_name, text) in [("text.primary", theme.text.primary)]
                    .into_iter()
                    .chain(link)
                {
                    assert_readable(
                        kind,
                        &format!("{text_name} on accent.highlight over {surface_name}"),
                        text,
                        highlight,
                    );
                }
            }
        }
    }

    /// `bg.pressed` is shown only while a control is held and carries only
    /// button and menu labels, so only those need the minimum on it (R85).
    #[test]
    fn labels_meet_contrast_minimum_on_the_pressed_fill() {
        for kind in ThemeKind::ALL {
            let theme = kind.theme();
            for (name, label) in [
                ("text.primary", theme.text.primary),
                ("danger", theme.status.danger),
            ] {
                let contrast = label.contrast_ratio(theme.surface.pressed);
                assert!(
                    contrast >= 4.5,
                    "{kind:?}: {name} has contrast {contrast:.2}"
                );
            }
        }
    }

    #[test]
    fn chip_text_meets_contrast_minimum_on_its_fill() {
        for kind in ThemeKind::ALL {
            let theme = kind.theme();
            let chip = theme.chip;
            for (name, colors) in [
                ("chip.place", chip.place),
                ("chip.character", chip.character),
                ("chip.item", chip.item),
                ("chip.progress", chip.progress),
            ] {
                assert_readable(kind, name, colors.text, colors.background);
                assert_readable(
                    kind,
                    &format!("text.secondary on {name}"),
                    theme.text.secondary,
                    colors.background,
                );
                assert_readable(
                    kind,
                    &format!("text.primary on {name}"),
                    theme.text.primary,
                    colors.background,
                );
            }
        }
    }

    #[test]
    fn radii_grow_from_chips_to_windows() {
        for kind in ThemeKind::ALL {
            let radius = kind.theme().radius;
            assert_eq!(
                (radius.small, radius.medium, radius.large),
                (4.0, 6.0, 10.0)
            );
        }
    }

    #[test]
    fn dark_is_default() {
        assert_eq!(ThemeKind::default(), ThemeKind::Dark);
        assert_eq!(Theme::default(), Theme::DARK);
    }
}
