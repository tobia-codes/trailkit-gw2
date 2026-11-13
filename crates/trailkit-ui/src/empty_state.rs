//! The one empty-state layout of every addon (ROADMAP R96, Q283 b,
//! `tokens.md` §7.1 rule 3): an optional icon in `text.disabled`, a title in
//! [`TextStyle::Body`], an optional hint in [`TextStyle::Secondary`] and
//! optional buttons with one primary, centered as one block.

use nexus::imgui::{StyleVar, Ui};

use crate::{
    button::{button_width, draw_button, draw_primary_button},
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    text::{TextStyle, draw_icon},
    text_layout::{centered_text, indent_to_center},
    theme::Theme,
};

/// What an empty state shows.
#[derive(Debug, Clone, Copy)]
pub struct EmptyState<'text> {
    /// Icon above the title, if any.
    pub icon: Option<Icon>,
    /// What is empty, e.g. "No notes yet".
    pub title: &'text str,
    /// One sentence below the title, if any.
    pub hint: Option<&'text str>,
    /// Buttons below the texts, if any.
    pub buttons: Option<EmptyStateButtons<'text>>,
}

/// The buttons of an empty state: always one primary (`lay-one-primary`)
/// and at most one regular button left of it; if both do not fit side by
/// side, the primary one stacks on top (R103).
#[derive(Debug, Clone, Copy)]
pub struct EmptyStateButtons<'text> {
    /// Label of the primary button.
    pub primary: &'text str,
    /// Label of the regular button, if any.
    pub secondary: Option<&'text str>,
}

/// Which button of an empty state was clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyStateAction {
    /// The primary button.
    Primary,
    /// The regular button.
    Secondary,
}

/// Draws `state` centered in the remaining content region and returns the
/// clicked button, if any.
///
/// The block height is estimated before drawing (Q283): the icon and the
/// title count one line each, the hint two lines (it may wrap once in a
/// narrow pane) and the buttons two frames (they may stack). Wrapped lines
/// of one text sit at the line height; only blocks are apart by the item
/// spacing (`tokens.md` §7.1 rule 4).
pub fn draw_empty_state(
    ui: &Ui,
    theme: &Theme,
    state: &EmptyState<'_>,
) -> Option<EmptyStateAction> {
    let available = ui.content_region_avail();
    let [left, top] = ui.cursor_screen_pos();
    let height = block_height(ui, theme, state);
    ui.set_cursor_screen_pos([left, top + ((available[1] - height) / 2.0).max(0.0)]);
    if let Some(icon) = state.icon {
        let width = {
            let _icons = FontScope::push(ui, FontRole::Icons);
            let mut buffer = [0; 4];
            ui.calc_text_size(icon.as_str(&mut buffer))[0]
        };
        indent_to_center(ui, width, available[0]);
        draw_icon(ui, icon, theme.text.disabled);
    }
    draw_centered_paragraph(ui, theme, TextStyle::Body, state.title, available[0]);
    if let Some(hint) = state.hint {
        draw_centered_paragraph(ui, theme, TextStyle::Secondary, hint, available[0]);
    }
    state.buttons.and_then(|buttons| {
        ui.dummy([0.0, theme.spacing.small]);
        draw_buttons(ui, theme, buttons, available[0])
    })
}

/// The estimated height of `state`'s block (see [`draw_empty_state`]).
fn block_height(ui: &Ui, theme: &Theme, state: &EmptyState<'_>) -> f32 {
    let line = ui.text_line_height();
    let line_with_spacing = ui.text_line_height_with_spacing();
    let spacing = line_with_spacing - line;
    let icon = if state.icon.is_some() {
        line_with_spacing
    } else {
        0.0
    };
    let hint = if state.hint.is_some() {
        2.0 * line + spacing
    } else {
        0.0
    };
    let buttons = if state.buttons.is_some() {
        theme.spacing.small + 2.0 * ui.frame_height_with_spacing()
    } else {
        0.0
    };
    icon + line_with_spacing + hint + buttons
}

/// Draws `text` in `style`, each wrapped line centered in `width` at the
/// line height, followed by the item spacing as one block.
fn draw_centered_paragraph(ui: &Ui, theme: &Theme, style: TextStyle, text: &str, width: f32) {
    let spacing = ui.clone_style().item_spacing;
    {
        let _style = style.push(ui, theme);
        let _tight = ui.push_style_var(StyleVar::ItemSpacing([spacing[0], 0.0]));
        centered_text(ui, text, width);
    }
    let [x, y] = ui.cursor_pos();
    ui.set_cursor_pos([x, y + spacing[1]]);
}

/// Draws `buttons` centered in `width`, side by side with the primary one
/// on the right if they fit, else stacked with the primary one on top
/// (R103); returns the clicked one.
fn draw_buttons(
    ui: &Ui,
    theme: &Theme,
    buttons: EmptyStateButtons<'_>,
    width: f32,
) -> Option<EmptyStateAction> {
    let primary_width = button_width(ui, theme, buttons.primary);
    let spacing = theme.spacing.small;
    let secondary_width = buttons
        .secondary
        .map(|label| button_width(ui, theme, label));
    let fits_one_row =
        secondary_width.is_none_or(|secondary| primary_width + spacing + secondary <= width);
    let row_width = match secondary_width {
        Some(secondary) if fits_one_row => primary_width + spacing + secondary,
        _ => primary_width,
    };
    indent_to_center(ui, row_width, width);
    let secondary = buttons.secondary.zip(secondary_width);
    let mut action = None;
    if fits_one_row && let Some((label, _)) = secondary {
        if draw_button(ui, theme, label) {
            action = Some(EmptyStateAction::Secondary);
        }
        ui.same_line_with_spacing(0.0, spacing);
    }
    if draw_primary_button(ui, theme, buttons.primary) {
        action = Some(EmptyStateAction::Primary);
    }
    if !fits_one_row && let Some((label, secondary)) = secondary {
        indent_to_center(ui, secondary, width);
        if draw_button(ui, theme, label) {
            action = Some(EmptyStateAction::Secondary);
        }
    }
    action
}
