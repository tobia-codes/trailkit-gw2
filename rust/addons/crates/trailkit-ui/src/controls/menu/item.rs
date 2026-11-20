//! What a menu entry shows (ROADMAP R155): one description for every entry,
//! drawn by [`super::draw_menu_item`] in a popup and by
//! [`super::draw_menu_panel`] in a menu drawn as a panel (R155-1).

use crate::{color::Color, text::icons::Icon, theme::Theme};

/// How a menu entry's label and icon are colored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuItemTone {
    /// `text.primary` label and icon.
    #[default]
    Regular,
    /// `danger` label and icon, for a destructive entry that asks first
    /// (memo M146).
    Danger,
}

/// One menu entry: its label and what else it shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuItem<'text> {
    /// The label; also the entry's ImGui ID within its menu.
    pub label: &'text str,
    /// A Lucide icon before the label (memo M133).
    pub icon: Option<Icon>,
    /// A shortcut in `text.secondary` at the row's right end (memo M198).
    pub shortcut: Option<&'text str>,
    /// Whether a check mark in the accent color sits at the row's right end,
    /// e.g. the current choice of "Move to".
    pub checked: bool,
    /// How label and icon are colored.
    pub tone: MenuItemTone,
    /// Whether the entry names what is shown now, e.g. "Notes" in memo's
    /// window menu: it sits on `bg.active` (with the selection bar in a
    /// panel).
    pub selected: bool,
    /// Why the entry cannot be used now; `None` while it can. A disabled
    /// entry is drawn in `text.disabled`, reports no click and names the
    /// reason in a tooltip (memo M141, R93).
    pub disabled_reason: Option<&'text str>,
}

impl<'text> MenuItem<'text> {
    /// A regular, usable entry with `label` and nothing else.
    #[must_use]
    pub const fn new(label: &'text str) -> Self {
        Self {
            label,
            icon: None,
            shortcut: None,
            checked: false,
            tone: MenuItemTone::Regular,
            selected: false,
            disabled_reason: None,
        }
    }

    /// The same entry with `icon` before its label.
    #[must_use]
    pub const fn icon(self, icon: Icon) -> Self {
        Self {
            icon: Some(icon),
            ..self
        }
    }

    /// The same entry with `shortcut` at its right end.
    #[must_use]
    pub const fn shortcut(self, shortcut: &'text str) -> Self {
        Self {
            shortcut: Some(shortcut),
            ..self
        }
    }

    /// The same entry with a check mark while `checked`.
    #[must_use]
    pub const fn checked(self, checked: bool) -> Self {
        Self { checked, ..self }
    }

    /// The same entry in `tone`.
    #[must_use]
    pub const fn tone(self, tone: MenuItemTone) -> Self {
        Self { tone, ..self }
    }

    /// The same entry, selected while `selected`.
    #[must_use]
    pub const fn selected(self, selected: bool) -> Self {
        Self { selected, ..self }
    }

    /// The same entry, disabled for `reason` when it is `Some`.
    #[must_use]
    pub const fn disabled_for(self, reason: Option<&'text str>) -> Self {
        Self {
            disabled_reason: reason,
            ..self
        }
    }

    /// Whether the entry can be used now.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.disabled_reason.is_none()
    }

    /// The color of the label and the icon.
    pub(super) const fn color(&self, theme: &Theme) -> Color {
        match (self.is_enabled(), self.tone) {
            (false, _) => theme.text.disabled,
            (true, MenuItemTone::Regular) => theme.text.primary,
            (true, MenuItemTone::Danger) => theme.status.danger,
        }
    }
}
