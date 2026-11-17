//! Color token types (trailkit-ui skill §2, ROADMAP R118): the colors of a
//! theme grouped by role, and the darkening steps of colored buttons (R107).
//! The values are generated into `generated/colors.rs` from
//! `design/tokens/colors.toml` (R133).

use crate::{chat_link::ChipCategory, color::Color};

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
    /// `text.on_danger`: label of a delete button on `danger.fill` (R125).
    pub on_danger: Color,
}

/// OKLab lightness step from a colored button's fill to its hover fill
/// (R107, Q446): 4 % darker in both themes.
pub const HOVER_DARKENING: f32 = 0.04;

/// OKLab lightness step from a colored button's fill to its pressed fill
/// (R107, Q446): 6 % darker in both themes.
pub const PRESSED_DARKENING: f32 = 0.06;

/// Accent colors: violet (R124, before brass), spent only on the primary
/// action, focus and selection (R71).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccentColors {
    /// `accent`: fill of the primary button. A fill only: text and thin
    /// lines use `accent.text`.
    pub base: Color,
    /// `accent.hover`: hovered primary button, `accent` darkened by
    /// [`HOVER_DARKENING`] (R107).
    pub hover: Color,
    /// `accent.active`: pressed primary button, `accent` darkened by
    /// [`PRESSED_DARKENING`] (R107).
    pub active: Color,
    /// `accent.text`: the accent as text, icon or line on a surface (active
    /// toggles, check marks, drop line); ≥ 4.5:1 in both themes.
    pub text: Color,
    /// `accent.highlight`: opaque accent behind search hits, the hit drawn on
    /// it in `text.on_accent` like a text marker (memo M122, M197).
    pub highlight: Color,
}

/// Status colors (always combined with an icon or text, never color alone).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusColors {
    /// `success`.
    pub success: Color,
    /// `warning`.
    pub warning: Color,
    /// `danger`: red as text or icon on a surface (menu entry, toast icon).
    pub danger: Color,
    /// `danger.fill`: fill of a delete button (R105, R125) and frame of an
    /// error toast or `danger` status card; darker than `danger` in Dark.
    pub danger_fill: Color,
    /// `danger.fill_hover`: hovered delete button, `danger.fill` darkened by
    /// [`HOVER_DARKENING`] (R107).
    pub danger_fill_hover: Color,
    /// `danger.fill_active`: pressed delete button, `danger.fill` darkened by
    /// [`PRESSED_DARKENING`] (R107).
    pub danger_fill_active: Color,
    /// `danger.subtle`: `danger.fill` tinted into `bg.popup` (20 % Dark,
    /// 10 % Light), the fill of an error toast (memo M180, Q144 B) and of a
    /// `danger` status card (memo M184, R125).
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

/// Text and background color of one chat-link chip category.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChipCategoryColors {
    /// Color of the type icon and label (≥ 4.5:1 on `background`).
    pub text: Color,
    /// Fill of the chip.
    pub background: Color,
}

/// The five chat-link chip categories (R71, R126, Q459);
/// `ChatLinkKind::category` assigns every link kind to one of them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChipColors {
    /// `chip.map_point`: waypoint, point of interest, vista (blue).
    pub map_point: ChipCategoryColors,
    /// `chip.wvw_objective`: WvW objective (blue like map points for now).
    pub wvw_objective: ChipCategoryColors,
    /// `chip.character`: skill, trait, build (copper).
    pub character: ChipCategoryColors,
    /// `chip.item`: item, recipe, skin, outfit, coins (jade).
    pub item: ChipCategoryColors,
    /// `chip.progress`: achievement, NPC text, player (rosé).
    pub progress: ChipCategoryColors,
}

impl ChipColors {
    /// The colors of `category`.
    #[must_use]
    pub const fn for_category(&self, category: ChipCategory) -> ChipCategoryColors {
        match category {
            ChipCategory::MapPoint => self.map_point,
            ChipCategory::WvwObjective => self.wvw_objective,
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
