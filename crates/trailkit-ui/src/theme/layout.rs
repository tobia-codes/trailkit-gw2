//! Layout tokens: spacing, corner radii and the sizes of lists, panes,
//! buttons and menus, in pixels at 100 % UI scaling. They are the same
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

/// List layout in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListLayout {
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

/// Pane layout in pixels (content regions inside a window).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaneLayout {
    /// `pane.padding`: inner padding on every side of a pane; no content
    /// touches a pane edge (`lay-pane-padding`).
    pub padding: f32,
    /// `pane.gap`: space between two neighboring panes (holds the splitter).
    pub gap: f32,
}

/// Button layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonLayout {
    /// `button.min_width`: minimum width of a button with a text label, as a
    /// multiple of the current font size, so it scales with the text size
    /// (memo plan M131). Icon-only buttons stay square.
    pub min_width: f32,
}

impl ButtonLayout {
    /// The minimum width of a text button in pixels for `font_size`.
    #[must_use]
    pub fn min_width_for(&self, font_size: f32) -> f32 {
        (font_size * self.min_width).round()
    }
}

/// Menu layout (memo M177).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuLayout {
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

/// Tokens shared by both built-in themes (sizes, not colors).
pub(super) const SPACING: Spacing = Spacing {
    extra_small: 4.0,
    small: 8.0,
    medium: 12.0,
    large: 16.0,
    extra_large: 24.0,
};

/// List layout shared by both built-in themes.
pub(super) const LIST_LAYOUT: ListLayout = ListLayout {
    item_spacing: 4.0,
    indent: 16.0,
    selection_bar_width: 3.0,
    selection_bar_inset: 6.0,
};

/// Corner radii shared by both built-in themes.
pub(super) const RADII: Radii = Radii {
    small: 4.0,
    medium: 6.0,
    large: 10.0,
};

/// Pane layout shared by both built-in themes (`space.sm`).
pub(super) const PANE_LAYOUT: PaneLayout = PaneLayout {
    padding: SPACING.small,
    gap: SPACING.small,
};

/// Button layout shared by both built-in themes.
pub(super) const BUTTON_LAYOUT: ButtonLayout = ButtonLayout { min_width: 4.0 };

/// Menu layout shared by both built-in themes.
pub(super) const MENU_LAYOUT: MenuLayout = MenuLayout {
    padding: 6.0,
    row_padding_x: 10.0,
    row_padding_y: 7.0,
    row_gap: 2.0,
    min_width: 14.0,
};
