//! Layout of the memo main window: the two panes (M72, M74) and the empty
//! state (M42).

use nexus::imgui::Ui;
use trailkit_ui::{
    empty_state::{self, EmptyState, EmptyStateAction, EmptyStateButtons},
    icons::Icon,
    pane::draw_pane,
    splitter::draw_vertical_splitter,
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

/// Draws the empty state (M42, Q283 b): the `file-text` icon, "No notes
/// yet", the primary "Create your first note" and "New group".
pub(super) fn draw_empty_state(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    actions: &mut Vec<WindowAction>,
) {
    let state = EmptyState {
        icon: Some(Icon::FILE_TEXT),
        title: Text::NoNotesYet.get(language),
        hint: None,
        buttons: Some(EmptyStateButtons {
            primary: Text::CreateFirstNote.get(language),
            secondary: Some(Text::NewGroup.get(language)),
        }),
    };
    match empty_state::draw_empty_state(ui, theme, &state) {
        Some(EmptyStateAction::Primary) => actions.push(WindowAction::NewNote(None)),
        Some(EmptyStateAction::Secondary) => actions.push(WindowAction::NewGroup(None)),
        None => {}
    }
}
