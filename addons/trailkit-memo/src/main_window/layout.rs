//! Layout of the memo main window: the two panes (M72, M74) and the empty
//! state (M42).

use nexus::imgui::Ui;
use trailkit_ui::{
    button::{button_width, draw_button, draw_primary_button},
    pane::draw_pane,
    splitter::draw_vertical_splitter,
    text_layout::{centered_text, indent_to_center},
    theme::Theme,
};

use trailkit_memo_shared::i18n::LocalizedText;
use trailkit_memo_shared::i18n::{Language, Text};

use super::{MemoWindow, PaneWidths, WindowAction};

impl MemoWindow {
    /// Draws tree and note as two panes with a draggable splitter in the gap
    /// between them (M21). The splitter shows an accent line only while it
    /// is hovered or dragged.
    pub(super) fn draw_two_panes(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        widths: PaneWidths,
    ) {
        let available = ui.content_region_avail();
        let gap = theme.pane.gap;
        let usable = available[0] - gap;
        let tree_maximum = (usable - widths.note_minimum).max(widths.tree_minimum);
        // A fixed width (M74): a wider window gives the extra room to the
        // note; a narrower one squeezes the list only while it lacks space.
        let character_width = ui.calc_text_size("n")[0];
        let padding = 2.0 * theme.pane.padding;
        let tree_width = (self.ui_state.valid_note_list_characters() * character_width + padding)
            .clamp(widths.tree_minimum, tree_maximum);

        draw_pane(ui, theme, "tree", [tree_width, 0.0], || {
            self.draw_tree_pane(ui, language, theme);
        });

        ui.same_line_with_spacing(0.0, 0.0);
        let splitter = draw_vertical_splitter(
            ui,
            theme,
            "splitter",
            tree_width,
            (widths.tree_minimum, tree_maximum),
            available[1],
        );
        if let Some(width) = splitter.left_width {
            self.ui_state.note_list_characters = (width - padding) / character_width;
        }
        if splitter.released {
            self.saver.save_ui_state(self.ui_state);
        }

        ui.same_line_with_spacing(0.0, 0.0);
        draw_pane(ui, theme, "note", [0.0, 0.0], || {
            self.draw_note_pane(ui, language, theme);
        });
    }
}

/// Draws the empty state (M42): headline, primary and secondary button.
pub(super) fn draw_empty_state(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    actions: &mut Vec<WindowAction>,
) {
    let available = ui.content_region_avail();
    let block_height = ui.text_line_height_with_spacing() + 2.0 * ui.frame_height_with_spacing();
    let start = ui.cursor_pos();
    ui.set_cursor_pos([
        start[0],
        start[1] + ((available[1] - block_height) / 2.0).max(0.0),
    ]);

    centered_text(ui, Text::NoNotesYet.get(language), available[0]);
    ui.dummy([0.0, theme.spacing.small]);

    let primary = Text::CreateFirstNote.get(language);
    let secondary = Text::NewGroup.get(language);
    let primary_width = button_width(ui, theme, primary);
    let secondary_width = button_width(ui, theme, secondary);
    let spacing = theme.spacing.small;
    let fits_one_row = primary_width + spacing + secondary_width <= available[0];

    // Primary action uses the accent (`lay-one-primary`).
    let row_width = if fits_one_row {
        primary_width + spacing + secondary_width
    } else {
        primary_width
    };
    indent_to_center(ui, row_width, available[0]);
    if draw_primary_button(ui, theme, primary) {
        actions.push(WindowAction::NewNote(None));
    }
    if fits_one_row {
        ui.same_line_with_spacing(0.0, spacing);
    } else {
        indent_to_center(ui, secondary_width, available[0]);
    }
    if draw_button(ui, theme, secondary) {
        actions.push(WindowAction::NewGroup(None));
    }
}
