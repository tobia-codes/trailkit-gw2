//! Layout of the memo main window: the two panes (M72, M74, the shared
//! resizable pane columns of R101) and the empty state (M42).

use nexus::imgui::Ui;
use trailkit_ui::{
    empty_state::{self, EmptyState, EmptyStateAction, EmptyStateButtons},
    icons::Icon,
    pane_columns::{FillColumn, FixedColumn, PaneColumns, draw_resizable_pane_columns},
    theme::Theme,
};

use trailkit_memo_shared::i18n::LocalizedText;
use trailkit_memo_shared::i18n::{Language, Text};

use super::{MINIMUM_PANE_CHARACTERS, MINIMUM_TREE_CHARACTERS, MemoWindow, WindowAction};

impl MemoWindow {
    /// Draws tree and note as two panes with a draggable splitter in the gap
    /// between them (M21, the shared resizable pane columns of R101). The
    /// note list keeps a fixed width (M74); the drag ends with a save.
    pub(super) fn draw_two_panes(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let note_list = [FixedColumn {
            id: "tree",
            characters: self.ui_state.valid_note_list_characters(),
            minimum_characters: MINIMUM_TREE_CHARACTERS,
        }];
        let columns = PaneColumns {
            fixed: &note_list,
            fill: FillColumn {
                id: "note",
                minimum_characters: MINIMUM_PANE_CHARACTERS,
            },
        };
        let response = draw_resizable_pane_columns(ui, theme, &columns, |column| {
            if column == 0 {
                self.draw_tree_pane(ui, language, theme);
            } else {
                self.draw_note_pane(ui, language, theme);
            }
        });
        if let Some((_, characters)) = response.resized {
            self.ui_state.note_list_characters = characters;
        }
        if response.released {
            self.saver.save_ui_state(self.ui_state);
        }
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
