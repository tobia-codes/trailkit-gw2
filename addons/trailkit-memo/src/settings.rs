//! User settings of trailkit-memo and the visibility rules derived from them.
//!
//! Phase 0 (ROADMAP R60): only the defaults exist; persistence and the
//! settings window follow in phase 1. Platform-independent so the rules can be
//! unit-tested on any host.

/// Whether the addon adds its icon to the Nexus quick access bar (R55).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuickAccessIcon {
    /// The icon is shown (default).
    #[default]
    Shown,
    /// No icon; the window opens via keybind only.
    Hidden,
}

/// What an open window does while the game is not in gameplay — loading
/// screens, cutscenes, character select (R56, R61).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutsideGameplay {
    /// The window is not drawn but stays open and reappears afterwards (default).
    #[default]
    Hide,
    /// The window is drawn as usual.
    Show,
}

/// All user settings of trailkit-memo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Settings {
    /// Quick access icon visibility.
    pub quick_access_icon: QuickAccessIcon,
    /// Window behavior outside gameplay.
    pub outside_gameplay: OutsideGameplay,
}

impl Settings {
    /// The default settings, usable in `const`/`static` contexts.
    pub const DEFAULT: Self = Self {
        quick_access_icon: QuickAccessIcon::Shown,
        outside_gameplay: OutsideGameplay::Hide,
    };

    /// Whether an open window may be drawn in the current game state.
    #[must_use]
    pub const fn allows_drawing(&self, is_gameplay: bool) -> bool {
        is_gameplay || matches!(self.outside_gameplay, OutsideGameplay::Show)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn const_default_matches_derived_default() {
        assert_eq!(Settings::DEFAULT, Settings::default());
    }

    #[test]
    fn default_hides_outside_gameplay() {
        assert!(Settings::DEFAULT.allows_drawing(true));
        assert!(!Settings::DEFAULT.allows_drawing(false));
    }

    #[test]
    fn show_outside_gameplay_always_draws() {
        let settings = Settings {
            outside_gameplay: OutsideGameplay::Show,
            ..Settings::DEFAULT
        };
        assert!(settings.allows_drawing(false));
    }
}
