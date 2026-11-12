//! The window menu of the memo main window (M166): the title bar's menu button
//! opens a panel over the full content height with a dimmed backdrop. It holds
//! the ways to the notes, the search and the trash, and hides the note list.

use nexus::imgui::{ChildWindow, MouseButton, StyleVar, Ui};
use trailkit_ui::{
    font_registry::FontScope,
    fonts::FontRole,
    glyph_metrics::{capital_middle, glyph_icon_ink, glyph_ink_box},
    icons::Icon,
    pane::draw_pane,
    selection_bar::draw_row_state,
    separator::draw_compact_separator,
    theme::Theme,
};

use trailkit_memo_shared::i18n::{Language, LocalizedText, Text};
use trailkit_memo_trash::i18n::TrashText;

use super::{MemoWindow, WindowAction};

/// Largest share of the window the menu may cover, so a strip of the backdrop
/// stays visible to click away (`lay-drawer`).
const WINDOW_MENU_MAXIMUM_FRACTION: f32 = 0.85;
/// Narrowest the menu gets in average characters, so it reads as a panel and
/// not as a list tailored to its longest label (M166).
const WINDOW_MENU_MINIMUM_CHARACTERS: f32 = 27.0;

/// One row of the window menu.
#[derive(Debug, Clone)]
struct WindowMenuRow {
    /// The Lucide icon before the label.
    icon: Icon,
    /// The localized label; also the row's ImGui id.
    label: &'static str,
    /// The shortcut at the row's right end.
    shortcut: Option<&'static str>,
    /// Whether the row names the area the window shows.
    selected: bool,
    /// Whether a separator sets the row apart from the rows above.
    separated: bool,
    /// What choosing the row does.
    action: WindowAction,
}

impl MemoWindow {
    /// The rows the menu offers now. Without notes there is nothing to search
    /// or hide; the editor hides the note list by itself (M167).
    fn window_menu_rows(&self, language: Language) -> Vec<WindowMenuRow> {
        let has_notes = !self.document.root.is_empty();
        let mut rows = vec![WindowMenuRow {
            icon: Icon::FILE_TEXT,
            label: Text::MenuNotes.get(language),
            shortcut: None,
            selected: !self.trash.visible,
            separated: false,
            action: WindowAction::ShowNotes,
        }];
        if has_notes {
            rows.push(WindowMenuRow {
                icon: Icon::SEARCH,
                label: Text::MenuSearch.get(language),
                shortcut: Some(Text::SearchShortcut.get(language)),
                selected: false,
                separated: false,
                action: WindowAction::ShowSearch,
            });
        }
        rows.push(WindowMenuRow {
            icon: Icon::TRASH_2,
            label: TrashText::Trash.get(language),
            shortcut: None,
            selected: self.trash.visible,
            separated: false,
            action: WindowAction::OpenTrash,
        });
        if has_notes && self.editor.is_none() {
            rows.push(WindowMenuRow {
                icon: Icon::PANEL_LEFT,
                label: if self.ui_state.note_list_hidden {
                    Text::ShowNoteList.get(language)
                } else {
                    Text::HideNoteList.get(language)
                },
                shortcut: None,
                selected: false,
                separated: true,
                action: WindowAction::ToggleNoteList,
            });
        }
        rows
    }

    /// Draws the open menu over the content area from `start` with `size`: it
    /// appears instantly (`overlay-motion`) and closes on a click on the
    /// backdrop or when a row is chosen.
    pub(super) fn draw_window_menu(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        start: [f32; 2],
        size: [f32; 2],
    ) {
        if !self.window_menu_open {
            return;
        }
        ui.set_cursor_screen_pos(start);
        // The backdrop takes the clicks. ImGui renders a child's background
        // into the parent's draw list, below the panes drawn before, so the
        // dimming is a rectangle in the backdrop's own list.
        let Some(_backdrop) = ChildWindow::new("window_menu_backdrop")
            .size(size)
            .scrollable(false)
            .scroll_bar(false)
            .movable(false)
            .draw_background(false)
            .begin(ui)
        else {
            return;
        };
        let backdrop_start = ui.window_pos();
        ui.get_window_draw_list()
            .add_rect(
                backdrop_start,
                [backdrop_start[0] + size[0], backdrop_start[1] + size[1]],
                theme.surface.modal_overlay.to_array(),
            )
            .filled(true)
            .rounding(theme.radius.large)
            .build();

        let rows = self.window_menu_rows(language);
        let width = window_menu_width(ui, theme, &rows).min(size[0] * WINDOW_MENU_MAXIMUM_FRACTION);
        let mut chosen = None;
        draw_pane(ui, theme, "window_menu", [width, 0.0], || {
            let _spacing = ui.push_style_var(StyleVar::ItemSpacing([0.0, theme.list.item_spacing]));
            for row in &rows {
                if row.separated {
                    draw_compact_separator(ui, theme);
                }
                if draw_window_menu_row(ui, theme, row) {
                    chosen = Some(row.action.clone());
                }
            }
        });
        if let Some(action) = chosen {
            self.window_menu_open = false;
            self.actions.push(action);
        } else if ui.is_window_hovered() && ui.is_mouse_clicked(MouseButton::Left) {
            self.window_menu_open = false;
        }
    }
}

/// The side of the square the rows' icons are centered in.
fn icon_square_side(ui: &Ui) -> f32 {
    let _icons = FontScope::push(ui, FontRole::Icons);
    ui.text_line_height()
}

/// The width of the panel: its minimum, or the widest row with its padding.
fn window_menu_width(ui: &Ui, theme: &Theme, rows: &[WindowMenuRow]) -> f32 {
    let gap = theme.spacing.small;
    let widest_row = rows
        .iter()
        .map(|row| {
            let shortcut = row
                .shortcut
                .map_or(0.0, |text| 2.0 * gap + ui.calc_text_size(text)[0]);
            ui.calc_text_size(row.label)[0] + shortcut
        })
        .fold(0.0, f32::max);
    let minimum = ui.calc_text_size("n")[0] * WINDOW_MENU_MINIMUM_CHARACTERS;
    (widest_row + icon_square_side(ui) + 3.0 * gap + 2.0 * theme.pane.padding).max(minimum)
}

/// Draws one row over the available width: icon, label and the shortcut at its
/// right end; a selected row looks like a selected list row. Returns whether
/// it was clicked.
fn draw_window_menu_row(ui: &Ui, theme: &Theme, row: &WindowMenuRow) -> bool {
    let width = ui.content_region_avail()[0];
    let height = ui.frame_height() + theme.spacing.extra_small;
    let clicked = ui.invisible_button(row.label, [width, height]);
    draw_row_state(ui, theme, row.selected);
    let [left, top] = ui.item_rect_min();
    let right = ui.item_rect_max()[0];
    let middle = top + height / 2.0;
    let gap = theme.spacing.small;
    let square = icon_square_side(ui);
    {
        let _icons = FontScope::push(ui, FontRole::Icons);
        let ink = glyph_icon_ink(row.icon.glyph())
            .or_else(|| glyph_ink_box(row.icon.glyph()))
            .unwrap_or([0.0, 0.0, square, square]);
        let position = [
            left + gap + (square - (ink[2] - ink[0])) / 2.0 - ink[0],
            middle - (ink[1] + ink[3]) / 2.0,
        ];
        let mut buffer = [0; 4];
        ui.get_window_draw_list().add_text(
            position,
            theme.text.primary.to_array(),
            row.icon.as_str(&mut buffer),
        );
    }
    let text_y = middle - capital_middle().unwrap_or(ui.text_line_height() / 2.0);
    let draw_list = ui.get_window_draw_list();
    draw_list.add_text(
        [left + 2.0 * gap + square, text_y],
        theme.text.primary.to_array(),
        row.label,
    );
    if let Some(shortcut) = row.shortcut {
        draw_list.add_text(
            [right - gap - ui.calc_text_size(shortcut)[0], text_y],
            theme.text.secondary.to_array(),
            shortcut,
        );
    }
    clicked
}
