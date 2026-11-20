//! Semantic theme tokens and the built-in themes (trailkit-ui skill §2,
//! `references/tokens.md`; palette ROADMAP R71).
//!
//! Components never use raw colors or numbers; they read tokens from the
//! active [`Theme`]. Sizes are in pixels at 100 % UI scaling.

//!
//! The token types live in the submodules; every value is generated from
//! `design/tokens/` by `cargo xtask tokens` into `generated.rs`, which
//! assembles the themes from one module per token file in `generated/`
//! (ROADMAP R118, R133).

mod colors;
mod generated;
mod layout;
#[cfg(test)]
mod tests;
mod typography;

pub use colors::{
    AccentColors, BorderTokens, ChipCategoryColors, ChipColors, CoinColors, EventColors,
    HOVER_DARKENING, PRESSED_DARKENING, ShadowTokens, StatusColors, SurfaceColors, TextColors,
};
pub use layout::{ButtonLayout, ListLayout, MenuLayout, Radii, Spacing, TooltipLayout};
pub use typography::Typography;

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
    /// Colors of the chat-link chip categories (`chip.*`, R71).
    pub chip: ChipColors,
    /// Spacing scale (`space.*`).
    pub spacing: Spacing,
    /// List layout (`list.*`).
    pub list: ListLayout,
    /// Button layout (`button.*`, memo plan M131).
    pub button: ButtonLayout,
    /// Menu layout (`menu.*`, memo M177).
    pub menu: MenuLayout,
    /// Tooltip layout.
    pub tooltip: TooltipLayout,
    /// Corner radii (`radius.*`, R71).
    pub radius: Radii,
    /// Shadows (`shadow.*`, Q327): only tooltips have one.
    pub shadow: ShadowTokens,
    /// Font sizes and line height (`font.*`).
    pub typography: Typography,
}

impl Default for Theme {
    /// The default theme is [`Theme::DARK`].
    fn default() -> Self {
        Self::DARK
    }
}
