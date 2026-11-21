//! Wires the note-list slice into the memo main window (ROADMAP R76,
//! questions 48–53): the app draws the search field (while shown, M138),
//! hands the search and the reset-schedule extras to the list and maps its
//! actions.

use jiff::Timestamp;
use nexus::imgui::{ChildWindow, Ui};
use trailkit_ui::controls::tooltip::draw_tooltip_with;
use trailkit_ui::{
    controls::menu::{MenuItem, begin_submenu, draw_menu_item},
    controls::search::{
        bar::{CountPlacement, SearchBarTexts, SearchStatus, draw_search_bar},
        field::SearchFieldTexts,
    },
    layout::stack::Stack,
    text::icons::Icon,
    text::{TextStyle, draw_text},
    theme::Theme,
};

use trailkit_memo_drafts::i18n::DraftText;
use trailkit_memo_note_list::note_list::{
    ListSearch, ListSearchTexts, NoteList, NoteListAction, NoteListInput, NoteRowExtras,
    UnsavedChanges,
};
use trailkit_memo_reset_schedule::{
    i18n::ResetText,
    note_kind::{KIND_OPTIONS, reset_badge, reset_tooltip_lines},
};
use trailkit_memo_search::{i18n::SearchText, no_search_results::draw_no_search_results};
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText},
    note_tree::Note,
};

use crate::main_window::{MemoWindow, WindowAction};

impl From<NoteListAction> for WindowAction {
    /// Maps an action of the note list to the window's own action.
    fn from(action: NoteListAction) -> Self {
        match action {
            NoteListAction::Select(node) => Self::Select(node),
            NoteListAction::ClearSelection => Self::ClearSelection,
            NoteListAction::ClickGroup(group) => Self::ClickGroup(group),
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
    /// Draws the fixed header and below it the scrolling tree (M129), as one
    /// stack (R148). While the search is shown (M138) the divider sits under
    /// it, above the toolbar (mock v9); otherwise under the toolbar.
    pub(in crate::main_window) fn draw_tree_pane(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
    ) {
        // The trash takes the place of the notes (M136).
        if self.trash.visible {
            self.draw_trash_column(ui, language, theme);
            return;
        }
        Stack::vertical(theme).build(ui, |stack| {
            let search_visible = self.ui_state.search_visible;
            if search_visible {
                stack.add(|| self.draw_list_search(ui, language, theme));
                stack.add_separator(theme);
            }
            if let Some(action) = stack.add(|| NoteList::draw_header(ui, language, theme)) {
                self.actions.push(action.into());
            }
            if !search_visible {
                stack.add_separator(theme);
            }
            // Search field and buttons stay put; only the tree below the
            // divider scrolls, with its own scrollbar (M129).
            stack.add(|| {
                ChildWindow::new("note_list")
                    .size([0.0, 0.0])
                    .draw_background(false)
                    .build(ui, || self.draw_note_list(ui, language, theme));
            });
        });
    }

    /// Draws the search above the list: the closable field and, while
    /// notes match, "2 of 4" with the arrows below it (mock v9). Typing, the
    /// `x`, the arrows and Enter take effect in this frame.
    fn draw_list_search(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let position = self.hit_position();
        let count = position.map_or_else(String::new, |position| {
            let current = position
                .current
                .map_or_else(|| "–".to_owned(), |current| current.to_string());
            SearchText::FindCount
                .get(language)
                .replace("{current}", &current)
                .replace("{total}", &position.total.to_string())
        });
        // Without hits the list itself says so (M126), so no row shows.
        let status = if position.is_some() {
            SearchStatus::Hits
        } else {
            SearchStatus::Empty
        };
        let texts = SearchBarTexts {
            field: SearchFieldTexts {
                hint: SearchText::SearchHint.get(language),
                clear: SearchText::ClearSearch.get(language),
                close: Some(SearchText::CloseSearch.get(language)),
            },
            count: &count,
            no_results: SearchText::FindNoResults.get(language),
            previous: SearchText::FindPrevious.get(language),
            next: SearchText::FindNext.get(language),
        };
        let response = draw_search_bar(
            ui,
            theme,
            &mut self.search_field,
            status,
            CountPlacement::Below,
            texts,
        );
        if response.text_changed {
            // Apply the new text in this frame already.
            self.update_search();
        }
        if response.close_clicked {
            self.actions.push(WindowAction::HideSearch);
        }
        if response.previous {
            self.step_list_search(false);
        }
        if response.next {
            self.step_list_search(true);
        }
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
        let badge = |note: &Note| reset_badge(note.reset_schedule);
        // The same kind and countdown as the note toolbar (M95), formatted
        // only while the badge is hovered.
        let mut badge_tooltip = |ui: &Ui, note: &Note| {
            let Some(period) = note.reset_schedule.period() else {
                return;
            };
            let lines = reset_tooltip_lines(period, language, Timestamp::now());
            draw_tooltip_with(ui, theme, || {
                for line in &lines {
                    draw_text(ui, theme, TextStyle::Body, line);
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
                collapsed_groups: &self.ui_state.collapsed_groups,
                edited: self.editor.as_ref().map(|editor| editor.session().note),
                search,
                extras: NoteRowExtras {
                    badge: &badge,
                    badge_tooltip: &mut badge_tooltip,
                    context_menu: &mut context_menu,
                    unsaved_changes: UnsavedChanges {
                        notes: &self.unsaved_notes,
                        tooltip: DraftText::UnsavedChangesMark.get(language),
                    },
                },
            },
        );
        self.actions
            .extend(output.actions.into_iter().map(WindowAction::from));
        self.note_list_takes_delete_key = output.takes_delete_key;
    }
}

/// Draws the reset type submenu of `note` with its current kind checked (M86),
/// used by the row's context menu and the note pane's `⋯` menu (M160).
pub(in crate::main_window) fn draw_reset_type_submenu(
    ui: &Ui,
    theme: &Theme,
    language: Language,
    note: &Note,
    actions: &mut Vec<WindowAction>,
) {
    let label = ResetText::ResetType.get(language);
    let Some(_submenu) = begin_submenu(ui, theme, Icon::ROTATE_CCW, label) else {
        return;
    };
    for (period, text) in KIND_OPTIONS {
        let is_current = note.reset_schedule.period() == period;
        let option = MenuItem::new(text.get(language)).checked(is_current);
        if draw_menu_item(ui, theme, &option) && !is_current {
            actions.push(WindowAction::SetResetPeriod(note.id, period));
        }
    }
}
