//! Layout tokens: spacing, corner radii and the sizes of lists, buttons
//! and menus, in pixels at 100 % UI scaling. They are the same
//! in every built-in theme.

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

/// List layout in pixels; gaps, indents and insets come from the spacing
/// scale (R148).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListLayout {
    /// `list.selection_bar_width`: width of the accent bar on the left edge
    /// of the selected row (R71).
    pub selection_bar_width: f32,
}

/// Button layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonLayout {
    /// `button.min_width`: minimum width of a button with a text label, as a
    /// multiple of the current font size, so it scales with the text size
    /// (memo plan M131). Icon-only buttons stay square.
    pub min_width: f32,
    /// `button.dropdown_max_width`: maximum width of a dropdown field, as a
    /// multiple of the current font size; longer options are cut with "…"
    /// in the field and wrap in the list (R128).
    pub dropdown_max_width: f32,
}

impl ButtonLayout {
    /// The minimum width of a text button in pixels for `font_size`.
    #[must_use]
    pub fn min_width_for(&self, font_size: f32) -> f32 {
        (font_size * self.min_width).round()
    }

    /// The maximum width of a dropdown field in pixels for `font_size`.
    #[must_use]
    pub fn dropdown_max_width_for(&self, font_size: f32) -> f32 {
        (font_size * self.dropdown_max_width).round()
    }
}

/// Tooltip layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TooltipLayout {
    /// `tooltip.max_width`: where a tooltip's text wraps, in font sizes.
    pub max_width: f32,
}

impl TooltipLayout {
    /// The width at which a tooltip's text wraps, in pixels for `font_size`.
    #[must_use]
    pub fn max_width_for(&self, font_size: f32) -> f32 {
        (font_size * self.max_width).round()
    }
}

/// Menu layout (memo M177); row paddings and gaps come from the spacing
/// scale (R148).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuLayout {
    /// `menu.min_width`: minimum width of a command menu, as a multiple of the
    /// current font size, so it scales with the text size.
    pub min_width: f32,
}

impl MenuLayout {
    /// The minimum width of a command menu in pixels for `font_size`.
    #[must_use]
    pub fn min_width_for(&self, font_size: f32) -> f32 {
        (font_size * self.min_width).round()
    }
}

/// Corner radii in pixels (R71).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Radii {
    /// `radius.sm` (4): chips, badges, scrollbar grab, tooltips (Q328).
    pub small: f32,
    /// `radius.md` (6): buttons, inputs, dropdowns, rows, menu items.
    pub medium: f32,
    /// `radius.lg` (10): windows, panes, popups, dialogs.
    pub large: f32,
}
