//! Drawing the trash view and the undo toast in the memo main window (memo
//! M136, M145, M148): the trash column with its search (BL31) in place of
//! the note list, the read-only preview in the note pane and the toast at
//! the bottom.

use jiff::Timestamp;
use nexus::imgui::{ChildWindow, Ui};
use trailkit_ui::{
    controls::button::ButtonState,
    controls::search::{
        bar::{CountPlacement, SearchBarTexts, SearchStatus, draw_search_bar},
        field::SearchFieldTexts,
    },
    feedback::toast::{ToastAction, draw_toast},
    layout::stack::Stack,
    markdown::markdown_view::{MarkdownAction, open_link},
    text::{TextStyle, draw_text_wrapped},
    theme::Theme,
};

use trailkit_memo_note_view::note_view::{MarkedHits, NoteViewAction, NoteViewInput, ReadOnly};
use trailkit_memo_search::{
    i18n::SearchText,
    list_search::{HitStep, SearchUpdate},
    no_search_results::draw_no_search_results,
};
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText, chip_context},
    note_tree::{NodeId, NoteId, TreeNode},
};
use trailkit_memo_trash::{
    i18n::TrashText,
    labels,
    trash_view::{
        TrashAction, TrashListInput, TrashSearch, draw_group_preview, draw_preview_toolbar,
        draw_trash_entries, draw_trash_header,
    },
};

use crate::main_window::{MemoWindow, WindowAction, WindowToastKind};

impl MemoWindow {
    /// Draws the trash column in place of the note list (M136): header with
    /// "Empty", the trash's own search while it holds entries (BL31, under
    /// the header, not closable) and the scrolling entries.
    pub(in crate::main_window) fn draw_trash_column(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
    ) {
        let has_entries = !self.document.trash.is_empty();
        self.update_trash_search();
        let now = Timestamp::now();
        let mut clicked = None;
        let mut clear_search = false;
        Stack::vertical(theme).build(ui, |stack| {
            if let Some(action) = stack.add(|| draw_trash_header(ui, language, theme, has_entries))
            {
                self.actions.push(WindowAction::Trash(action));
            }
            if has_entries {
                stack.add(|| self.draw_trash_search(ui, language, theme));
            }
            stack.add_separator(theme);
            stack.add(|| {
                ChildWindow::new("trash_list")
                    .size([0.0, 0.0])
                    .draw_background(false)
                    .build(ui, || {
                        let current = self.trash.search.current();
                        if let Some(current) = current
                            && !current.results.has_hits()
                        {
                            clear_search =
                                draw_no_search_results(ui, language, theme, &current.text);
                            return;
                        }
                        let input = TrashListInput {
                            document: &self.document,
                            selected: self.trash.selected,
                            open_groups: &self.trash.open_groups,
                            search: current.map(|current| TrashSearch {
                                filter: &current.results,
                                query: &current.query,
                                collapsed: &self.trash.collapsed_in_search,
                            }),
                        };
                        clicked = draw_trash_entries(ui, language, theme, input, now);
                    });
            });
        });
        if clear_search {
            self.trash.search_field.clear(false);
        }
        if let Some(action) = clicked {
            self.actions.push(WindowAction::Trash(action));
        }
    }

    /// Brings the trash search up to date with its field and the trash;
    /// a new query opens every group again (M121).
    fn update_trash_search(&mut self) {
        let shown = self.trash_preview_note();
        let update = self.trash.search.update(
            &self.trash.search_field.text,
            &self.document,
            self.document_revision,
            shown,
        );
        if matches!(update, SearchUpdate::NewQuery | SearchUpdate::Ended) {
            self.trash.collapsed_in_search.clear();
            self.note_view.jump_to_first_hit();
        }
    }

    /// The trashed note shown in the preview, if the selection is a note.
    pub(in crate::main_window) fn trash_preview_note(&self) -> Option<NoteId> {
        match self.trash.selected? {
            NodeId::Note(note) => self
                .document
                .find_in_trash(NodeId::Note(note))
                .map(|_| note),
            NodeId::Group(_) => None,
        }
    }

    /// Draws the trash's search: the field without closing, and "1 of 3"
    /// with the arrows below it while entries match (mock v9).
    fn draw_trash_search(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let position = self.trash.search.position(self.trash_preview_note());
        let count = position.map_or_else(String::new, |position| {
            let current = position
                .current
                .map_or_else(|| "–".to_owned(), |current| current.to_string());
            SearchText::FindCount
                .get(language)
                .replace("{current}", &current)
                .replace("{total}", &position.total.to_string())
        });
        let status = if position.is_some() {
            SearchStatus::Hits
        } else {
            SearchStatus::Empty
        };
        let texts = SearchBarTexts {
            field: SearchFieldTexts {
                hint: TrashText::SearchTrash.get(language),
                clear: SearchText::ClearSearch.get(language),
                close: None,
            },
            count: &count,
            no_results: SearchText::FindNoResults.get(language),
            previous: SearchText::FindPrevious.get(language),
            next: SearchText::FindNext.get(language),
        };
        let response = draw_search_bar(
            ui,
            theme,
            &mut self.trash.search_field,
            status,
            CountPlacement::Below,
            texts,
        );
        if response.text_changed {
            self.update_trash_search();
        }
        for (pressed, forward) in [(response.previous, false), (response.next, true)] {
            if pressed {
                self.step_trash_search(forward);
            }
        }
    }

    /// Goes to the next or previous hit of the trash search (BL31): inside
    /// the previewed note, else the next matching note is selected, its
    /// groups open for this search.
    fn step_trash_search(&mut self, forward: bool) {
        let shown = self.trash_preview_note();
        match self.trash.search.step(shown, forward) {
            Some(HitStep::InsideNote) => self.note_view.reveal_current_hit(),
            Some(HitStep::ToNote { note, .. }) => {
                let node = NodeId::Note(note);
                let mut parent = self.document.trashed_parent(node).map(|group| group.id);
                while let Some(group) = parent {
                    self.trash.collapsed_in_search.remove(&group);
                    parent = self
                        .document
                        .trashed_parent(NodeId::Group(group))
                        .map(|group| group.id);
                }
                self.actions
                    .push(WindowAction::Trash(TrashAction::Select(node)));
                self.note_view.reveal_current_hit();
            }
            None => {}
        }
    }

    /// Draws the selected trash entry read-only in the note pane (M145):
    /// toolbar with "Delete permanently" and "Restore", title, info
    /// line and the content.
    pub(in crate::main_window) fn draw_trash_preview(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
    ) {
        let now = Timestamp::now();
        let selected = self.trash.selected;
        // The selected entry may sit inside a trashed group (M218): it shows
        // the deadline of the entry holding it and names its group.
        let found = selected.and_then(|node| {
            let shown = self.document.find_in_trash(node)?;
            let holding = self.document.trash_entry_holding(node)?;
            let group = self
                .document
                .trashed_parent(node)
                .map(|group| group.title.as_str());
            Some((
                node,
                matches!(shown, TreeNode::Note(_)),
                labels::info_line(holding, group, now, language),
            ))
        });
        let Some((entry_id, is_note, info)) = found else {
            draw_text_wrapped(
                ui,
                theme,
                TextStyle::Secondary,
                TrashText::SelectTrashEntry.get(language),
            );
            return;
        };
        let mut toolbar_action = None;
        let mut view_actions = Vec::new();
        let mut found_hits = None;
        let mut search_hits = None;
        Stack::vertical(theme).build(ui, |stack| {
            toolbar_action = stack.add(|| draw_preview_toolbar(ui, language, theme, entry_id));
            stack.add_separator(theme);
            // Find in the previewed note, below the toolbar like in a note
            // (M198, mock v9).
            if is_note {
                self.add_find_bar(stack, ui, language, theme);
            }
            // Looked up again: the find bar above may change the window.
            let Some(shown) = self.document.find_in_trash(entry_id) else {
                return;
            };
            let note = match shown {
                TreeNode::Note(note) => note,
                TreeNode::Group(group) => {
                    stack.add(|| draw_group_preview(ui, language, theme, group, &info));
                    return;
                }
            };
            let finding = self.find_bar_shown() && self.find.query.is_some();
            if finding && std::mem::take(&mut self.find.reveal_pending) {
                self.note_view.reveal_current_hit();
            }
            let find_current = self.find.current;
            let search_current = self.trash.search.current_hit();
            let input = NoteViewInput {
                note: Some(note),
                search: self.trash.search.current().map(|current| MarkedHits {
                    query: &current.query,
                    current: search_current,
                }),
                find: self
                    .find
                    .query
                    .as_ref()
                    .filter(|_| finding)
                    .map(|query| MarkedHits {
                        query,
                        current: find_current,
                    }),
                chips: chip_context(language),
                title_trailing: None,
                read_only: Some(ReadOnly { subtitle: &info }),
            };
            let output = stack.add(|| {
                self.note_view
                    .draw(ui, language, theme, input, &mut self.trash.scroll)
            });
            view_actions = output.actions;
            let hits = output.hit_count.unwrap_or(0);
            if finding {
                found_hits = Some((note.id, hits));
            } else {
                search_hits = Some((note.id, hits));
            }
        });
        if let Some((note, hits)) = found_hits {
            self.record_find_hits(Some(note), hits);
        }
        if let Some((note, hits)) = search_hits {
            self.trash.search.record_drawn_hits(Some(note), hits);
        }
        if let Some(action) = toolbar_action {
            self.actions.push(WindowAction::Trash(action));
        }
        // A trashed note cannot change: links open, and copying (a chip, the
        // menu's Copy) is confirmed like anywhere else (M217).
        for action in view_actions {
            match action {
                NoteViewAction::Markdown(MarkdownAction::OpenLink(url)) => open_link(&url),
                NoteViewAction::Markdown(MarkdownAction::Copied(text)) => {
                    self.actions
                        .push(WindowAction::Markdown(MarkdownAction::Copied(text)));
                }
                NoteViewAction::Markdown(MarkdownAction::ToggleTask(_))
                | NoteViewAction::StartEditing => {}
            }
        }
    }

    /// Draws the toast at the bottom of the window; after a delete it offers
    /// "Undo" and "Open trash" (M142, M148, M158).
    pub(in crate::main_window) fn draw_toast_message(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
    ) {
        let Some(toast) = &mut self.toast else {
            return;
        };
        let trash_actions = [
            ToastAction {
                label: TrashText::Undo.get(language),
                tooltip: Some(TrashText::UndoTooltip.get(language)),
                state: ButtonState::Ready,
            },
            ToastAction {
                label: TrashText::OpenTrash.get(language),
                tooltip: None,
                state: ButtonState::Ready,
            },
        ];
        let actions: &[ToastAction<'_>] = match toast.kind {
            WindowToastKind::Message => &[],
            WindowToastKind::MovedToTrash => &trash_actions,
        };
        let outcome = draw_toast(ui, theme, &mut toast.toast, actions);
        match outcome.clicked_action {
            Some(0) => self.actions.push(WindowAction::Undo),
            Some(_) => self.actions.push(WindowAction::OpenTrash),
            None if outcome.expired => self.actions.push(WindowAction::DismissToast),
            None => {}
        }
    }
}
