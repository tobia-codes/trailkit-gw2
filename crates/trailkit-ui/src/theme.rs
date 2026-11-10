//! Semantic theme tokens and the built-in themes (trailkit-ui skill §2,
//! `references/tokens.md`; palette ROADMAP R71).
//!
//! Components never use raw colors or numbers; they read tokens from the
//! active [`Theme`]. Sizes are in pixels at 100 % UI scaling.

use crate::{chat_link::ChipCategory, color::Color};

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
    /// List metrics (`list.*`).
    pub list: ListMetrics,
    /// Pane metrics (`pane.*`).
    pub pane: PaneMetrics,
    /// Button metrics (`button.*`, memo plan M131).
    pub button: ButtonMetrics,
    /// Menu metrics (`menu.*`, memo M177).
    pub menu: MenuMetrics,
    /// Corner radii (`radius.*`, R71).
    pub radius: Radii,
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

/// Accent colors: brass, spent only on the primary action, focus and
/// selection (R71).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccentColors {
    /// `accent`: fill of the primary button and the selection bar. A fill
    /// only: in the light theme it is too light for text or thin lines.
    pub base: Color,
    /// `accent.hover`: hovered primary button.
    pub hover: Color,
    /// `accent.active`: pressed primary button.
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
    /// `danger.subtle`: `danger` tinted into `bg.popup` (20 % Dark, 10 %
    /// Light), the fill of an error toast (memo M180, Q144 B).
    pub danger_subtle: Color,
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

/// Corner radii in pixels (R71).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Radii {
    /// `radius.sm` (4): chips, badges, scrollbar grab.
    pub small: f32,
    /// `radius.md` (6): buttons, inputs, dropdowns, rows, menu items.
    pub medium: f32,
    /// `radius.lg` (10): windows, panes, popups, dialogs.
    pub large: f32,
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
    /// `list.selection_bar_width`: width of the brass bar on the left edge
    /// of the selected row (R71).
    pub selection_bar_width: f32,
    /// `list.selection_bar_inset`: gap between the bar and the top and
    /// bottom of its row.
    pub selection_bar_inset: f32,
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

/// Button metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonMetrics {
    /// `button.min_width`: minimum width of a button with a text label, as a
    /// multiple of the current font size, so it scales with the text size
    /// (memo plan M131). Icon-only buttons stay square.
    pub min_width: f32,
}

impl ButtonMetrics {
    /// The minimum width of a text button in pixels for `font_size`.
    #[must_use]
    pub fn min_width_for(&self, font_size: f32) -> f32 {
        (font_size * self.min_width).round()
    }
}

/// Menu metrics (memo M177).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuMetrics {
    /// `menu.padding`: gap between a menu's edge and its rows.
    pub padding: f32,
    /// `menu.row_padding_x`: gap between a row's edge and its content.
    pub row_padding_x: f32,
    /// `menu.row_padding_y`: gap between a row's edge and its text line.
    pub row_padding_y: f32,
    /// `menu.row_gap`: gap between two rows.
    pub row_gap: f32,
    /// `menu.min_width`: minimum width of a command menu, as a multiple of the
    /// current font size, so it scales with the text size.
    pub min_width: f32,
}

impl MenuMetrics {
    /// The minimum width of a command menu in pixels for `font_size`.
    #[must_use]
    pub fn min_width_for(&self, font_size: f32) -> f32 {
        (font_size * self.min_width).round()
    }
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
    /// `font.line_height`: line height of running text (notes, editor) as a
    /// multiple of the font size; roomy enough for chat-link chips (M77).
    pub line_height: f32,
}

impl Typography {
    /// The line height of running text in `font_size`, in whole pixels.
    #[must_use]
    pub fn line_height_for(&self, font_size: f32) -> f32 {
        (font_size * self.line_height).round()
    }

    /// The space above the text in such a line (half the extra height), so
    /// text sits in the middle of its line.
    #[must_use]
    pub fn leading_for(&self, font_size: f32) -> f32 {
        ((self.line_height_for(font_size) - font_size) / 2.0).floor()
    }
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
    selection_bar_width: 3.0,
    selection_bar_inset: 6.0,
};

/// Corner radii shared by both built-in themes.
const RADII: Radii = Radii {
    small: 4.0,
    medium: 6.0,
    large: 10.0,
};

/// Pane metrics shared by both built-in themes (`space.sm`).
const PANE_METRICS: PaneMetrics = PaneMetrics {
    padding: SPACING.small,
    gap: SPACING.small,
};

/// Button metrics shared by both built-in themes.
const BUTTON_METRICS: ButtonMetrics = ButtonMetrics { min_width: 4.0 };

/// Menu metrics shared by both built-in themes.
const MENU_METRICS: MenuMetrics = MenuMetrics {
    padding: 6.0,
    row_padding_x: 10.0,
    row_padding_y: 7.0,
    row_gap: 2.0,
    min_width: 14.0,
};

/// Font sizes shared by both built-in themes.
const TYPOGRAPHY: Typography = Typography {
    small: 12.0,
    body: 14.0,
    heading: 18.0,
    line_height: 1.5,
};

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
            hover: Color::from_rgb_hex(0xEDB65A),
            active: Color::from_rgb_hex(0xC98F2A),
            text: Color::from_rgb_hex(0xE8B055),
            highlight: Color::from_rgb_hex(0xE2A740).with_alpha(0.24),
        },
        status: StatusColors {
            success: Color::from_rgb_hex(0x5CC48A),
            warning: Color::from_rgb_hex(0xE9B44C),
            danger: Color::from_rgb_hex(0xF08A84),
            // 20 % `danger` over `bg.popup` (#1A2029); 22 % would leave
            // `text.secondary` below 4.5:1.
            danger_subtle: Color::from_rgb_hex(0x45353B),
            info: Color::from_rgb_hex(0x7AB0F0),
        },
        border: BorderTokens {
            color: Color::from_rgb_hex(0x2A323E),
            focus: Color::from_rgb_hex(0xE8B055),
            // About 6 % white over the dark surfaces.
            subtle: Color::from_rgb_hex(0xFFFFFF).with_alpha(0.06),
            width: 1.0,
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
        list: LIST_METRICS,
        pane: PANE_METRICS,
        button: BUTTON_METRICS,
        menu: MENU_METRICS,
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
            hover: Color::from_rgb_hex(0xD69A32),
            active: Color::from_rgb_hex(0xC48A24),
            text: Color::from_rgb_hex(0x8A5A10),
            highlight: Color::from_rgb_hex(0xE2A740).with_alpha(0.42),
        },
        status: StatusColors {
            success: Color::from_rgb_hex(0x176B3C),
            warning: Color::from_rgb_hex(0x8A5A00),
            danger: Color::from_rgb_hex(0xB3261E),
            // 10 % `danger` over `bg.popup` (#FFFFFF).
            danger_subtle: Color::from_rgb_hex(0xF7E9E8),
            info: Color::from_rgb_hex(0x1F5DB0),
        },
        border: BorderTokens {
            color: Color::from_rgb_hex(0xD6D1C7),
            focus: Color::from_rgb_hex(0x8A5A10),
            // About 8 % warm black over the light surfaces.
            subtle: Color::from_rgb_hex(0x1E180C).with_alpha(0.08),
            width: 1.0,
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
        list: LIST_METRICS,
        pane: PANE_METRICS,
        button: BUTTON_METRICS,
        menu: MENU_METRICS,
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
        }
    }

    /// The error toast's message, counter and icon sit on `danger.subtle`;
    /// its buttons use `bg.active` (memo M180).
    #[test]
    fn error_toast_content_meets_contrast_minimum() {
        for kind in ThemeKind::ALL {
            let theme = kind.theme();
            for (name, text) in [
                ("text.primary", theme.text.primary),
                ("text.secondary", theme.text.secondary),
                ("danger", theme.status.danger),
            ] {
                assert_readable(
                    kind,
                    &format!("{name} on danger.subtle"),
                    text,
                    theme.status.danger_subtle,
                );
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
