//! Wires the note-list slice into the memo main window (ROADMAP R76,
//! questions 48–53): the app draws the search field (while shown, M138),
//! hands the search and the reset-schedule extras to the list and maps its
//! actions.

use jiff::Timestamp;
use nexus::imgui::{ChildWindow, Ui};
use trailkit_ui::{
    menu::{begin_submenu, draw_menu_item},
    theme::Theme,
};

use trailkit_memo_note_list::note_list::{
    ListSearch, ListSearchTexts, NoteList, NoteListAction, NoteListInput, NoteRowExtras,
};
use trailkit_memo_reset_schedule::{
    i18n::ResetText,
    note_kind::{KIND_OPTIONS, reset_badge, reset_tooltip_lines},
};
use trailkit_memo_search::{
    i18n::SearchText, no_search_results::draw_no_search_results, search_field::SearchFieldTexts,
};
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText},
    note_tree::Note,
};

use super::{MemoWindow, WindowAction};

impl From<NoteListAction> for WindowAction {
    /// Maps an action of the note list to the window's own action.
    fn from(action: NoteListAction) -> Self {
        match action {
            NoteListAction::Select(node) => Self::Select(node),
            NoteListAction::SelectForContextMenu(node) => Self::SelectForContextMenu(node),
            NoteListAction::SetExpanded(group, expanded) => Self::SetExpanded(group, expanded),
            NoteListAction::Rename(node, title) => Self::Rename(node, title),
            NoteListAction::NewNote(point) => Self::NewNote(point),
            NoteListAction::NewGroup(point) => Self::NewGroup(point),
            NoteListAction::Move(node, point) => Self::Move(node, point),
            NoteListAction::SetCollapsedInSearch(group, collapsed) => {
                Self::SetCollapsedInSearch(group, collapsed)
            }
            NoteListAction::Delete(node) => Self::Delete(node),
        }
    }
}

impl MemoWindow {
    /// Draws the fixed header (search field while shown, M138; toolbar) and
    /// below it the scrolling tree (M129).
    pub(super) fn draw_tree_pane(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        // The trash tool shows the trash in place of the notes (M136).
        if self.trash.visible {
            self.draw_trash_column(ui, language, theme);
            return;
        }
        if self.ui_state.search_visible {
            let search_texts = SearchFieldTexts {
                hint: SearchText::SearchHint.get(language),
                clear: SearchText::ClearSearch.get(language),
            };
            if self.search_field.draw(ui, theme, search_texts) {
                // Apply the new text in this frame already.
                self.update_search();
            }
        }
        if let Some(action) = NoteList::draw_header(ui, language, theme) {
            self.actions.push(action.into());
        }
        // Search field and buttons stay put; only the tree below the
        // separator scrolls, with its own scrollbar (M129).
        ChildWindow::new("note_list")
            .size([0.0, 0.0])
            .draw_background(false)
            .build(ui, || self.draw_note_list(ui, language, theme));
    }

    /// Draws the scrolling part of the note list: the empty search result
    /// (M126) or the list with the search and the reset badge and menu.
    fn draw_note_list(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        if let Some(current) = &self.search.current
            && !current.results.has_hits()
        {
            if draw_no_search_results(ui, language, theme, &current.text) {
                self.actions.push(WindowAction::ClearSearch);
            }
            return;
        }
        let search = self.search.current.as_ref().map(|current| ListSearch {
            query: &current.query,
            filter: &current.results,
            collapsed: &self.collapsed_in_search,
            texts: ListSearchTexts {
                content_hits: SearchText::ContentHits.get(language),
                clear_search_to_move: SearchText::ClearSearchToMove.get(language),
            },
        });
        let badge = |note: &Note| reset_badge(note.kind, language, theme);
        // The same kind and countdown as the note toolbar (M95), formatted
        // only while the badge is hovered.
        let mut badge_tooltip = |ui: &Ui, note: &Note| {
            let Some(period) = note.kind.period() else {
                return;
            };
            let lines = reset_tooltip_lines(period, language, Timestamp::now());
            ui.tooltip(|| {
                for line in &lines {
                    ui.text(line);
                }
            });
        };
        // The reset type submenu with the current kind checked (M86).
        let window_actions = &mut self.actions;
        let mut context_menu = |ui: &Ui, note: &Note| {
            draw_reset_type_submenu(ui, theme, language, note, window_actions);
        };
        let output = self.note_list.draw(
            ui,
            language,
            theme,
            NoteListInput {
                document: &self.document,
                selected: self.ui_state.selected,
                edited: self.editor.as_ref().map(|editor| editor.session().note),
                search,
                extras: NoteRowExtras {
                    badge: &badge,
                    badge_tooltip: &mut badge_tooltip,
                    context_menu: &mut context_menu,
                },
            },
        );
        self.actions
            .extend(output.actions.into_iter().map(WindowAction::from));
    }
}

/// Draws the reset type submenu of `note` with its current kind checked (M86),
/// used by the row's context menu and the note pane's `⋯` menu (M160).
pub(super) fn draw_reset_type_submenu(
    ui: &Ui,
    theme: &Theme,
    language: Language,
    note: &Note,
    actions: &mut Vec<WindowAction>,
) {
    let Some(_submenu) = begin_submenu(ui, theme, ResetText::ResetType.get(language)) else {
        return;
    };
    for (period, text) in KIND_OPTIONS {
        let is_current = note.kind.period() == period;
        if draw_menu_item(ui, theme, text.get(language), is_current) && !is_current {
            actions.push(WindowAction::SetResetPeriod(note.id, period));
        }
    }
}
