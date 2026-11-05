//! The memo main window: note tree, note view, editor, empty state and
//! storage hints (Windows only; memo plan M21, M31, M34, M37–M55).
//!
//! The window owns the note tree (M30). Drawing collects user actions into a
//! reused list; they are applied after the tree has been drawn, then changed
//! documents are handed to the storage worker.

mod actions;
mod dialogs;
mod layout;
mod note_list;
mod note_pane;
mod search;
mod text_scale;
mod trash;
mod trash_pane;

use std::{collections::HashSet, path::PathBuf};

use jiff::Timestamp;
use nexus::imgui::{Ui, WindowFocusedFlags};
use trailkit_ui::{
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    pane::draw_pane,
    theme::Theme,
    title_bar::{TitleBarTexts, TitleBarToggle},
    toast::Toast,
    tool_rail::{ToolRailClick, ToolRailItem, draw_tool_rail},
};

use trailkit_memo_drafts::draft_hint::DraftHint;
use trailkit_memo_editor::editor::Editor;
use trailkit_memo_note_list::note_list::NoteList;
use trailkit_memo_note_view::note_view::NoteView;
use trailkit_memo_reset_schedule::{
    due_resets::apply_due_resets, schedule::next_reset_of_any_period,
};
use trailkit_memo_search::search_field::SearchField;
use trailkit_memo_shared::i18n::LocalizedText;
use trailkit_memo_shared::{
    drafts::DraftsDocument,
    i18n::{Language, Text},
    note_tree::{GroupId, NodeId, Note, NotesDocument},
    settings::{Settings, TextScale, UnsavedChanges},
    storage_worker::StorageSaver,
    ui_state::UiState,
    undo::UndoHistory,
};
use trailkit_memo_trash::i18n::TrashText;

use self::{
    actions::{LeaveReason, WindowAction},
    layout::draw_empty_state,
    search::SearchCache,
    trash::{Confirmation, TrashState},
};

/// Minimum content width of each pane in average characters (M40).
const MINIMUM_PANE_CHARACTERS: f32 = 22.0;
/// Minimum content width of the note list in average characters (M73): the
/// user may drag it narrower than the other panes.
const MINIMUM_TREE_CHARACTERS: f32 = 10.0;

/// How long a chat-link chip says "Copied" after a click (M75).
const COPIED_FEEDBACK_SECONDS: f64 = 1.5;

/// The chat code copied by the last chip click (M75).
#[derive(Debug)]
struct CopiedChatLink {
    /// The copied code.
    code: String,
    /// When "Copied" started to show (ImGui time); set on the next frame.
    shown_since: Option<f64>,
}

/// Minimum widths of the two-pane layout in pixels (M40, M73).
#[derive(Debug, Clone, Copy)]
struct PaneWidths {
    /// Narrowest the note list may be dragged.
    tree_minimum: f32,
    /// Room the note pane needs: two panes while editor and preview are
    /// side by side, one otherwise.
    note_minimum: f32,
}

/// Largest share of the window the note list drawer may cover, so a strip
/// of the dimmed note stays visible to click away (M48, `lay-drawer`).
const DRAWER_MAXIMUM_FRACTION: f32 = 0.85;

/// A problem with `notes.json` shown above the notes (M31).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotesProblem {
    /// The file was unreadable and moved away; memo started empty. Holds the
    /// new location relative to the addon folder, ready for display.
    Recovered(String),
    /// The file is from a newer addon version; nothing is saved.
    NewerVersion,
    /// The file cannot be accessed; nothing is saved.
    Inaccessible,
}

/// State of the main window, owned by the render thread.
pub struct MemoWindow {
    /// The note tree (M30).
    document: NotesDocument,
    /// A loading problem the user must be told about.
    problem: Option<NotesProblem>,
    /// Restorable window state (M39).
    ui_state: UiState,
    /// Queues saves on the storage worker.
    saver: StorageSaver,
    /// The addon folder, for "Open folder".
    addon_directory: PathBuf,
    /// The note list: rows, rename, drag & drop (R76 slice).
    note_list: NoteList,
    /// The reading view of the selected note.
    note_view: NoteView,
    /// Narrow window: whether the note list drawer is open (M48).
    drawer_open: bool,
    /// The chat code copied last, while its chip says "Copied" (M75).
    copied_chat_link: Option<CopiedChatLink>,
    /// Whether the window was too narrow for two panes in the last frame;
    /// decides what the menu button does.
    is_narrow: bool,
    /// Actions collected while drawing; reused every frame.
    actions: Vec<WindowAction>,
    /// Drafts kept from earlier unloads (M54, M55).
    drafts: DraftsDocument,
    /// The open editor, if a note is being edited (M51).
    editor: Option<Editor>,
    /// A pending answer of the unsaved changes dialog (M54).
    leave_prompt: Option<LeaveReason>,
    /// Position and size of the main window in the last frame (for the
    /// dialog window that covers it).
    window_rect: Option<[[f32; 2]; 2]>,
    /// Whether the main window had the focus in the last frame.
    window_focused: bool,
    /// The user settings; the window changes the text scale (M57, M79).
    settings: Settings,
    /// The text scale the fonts are built for; follows
    /// `settings.text_scale` once the wheel rests (M80).
    applied_text_scale: TextScale,
    /// When the text scale last changed (ImGui time), while its percentage
    /// is shown or the change is not applied yet (M80).
    text_scale_changed_at: Option<f64>,
    /// Height of the dialog card measured in the last frame (its content
    /// wraps, so the height is only known after drawing it once).
    dialog_height: Option<f32>,
    /// Set once closing the window is allowed; read by the addon glue.
    close_approved: bool,
    /// Cached draft hint label (note, language, text), so it is not
    /// formatted every frame.
    draft_hint: DraftHint,
    /// When reset notes must be checked next (M90); `None` checks at once,
    /// e.g. after an action that may have unblocked a waiting reset (M89).
    next_reset_check: Option<Timestamp>,
    /// The search field above the note list (M118); its text survives
    /// closing the window, not a game restart (M124).
    search_field: SearchField,
    /// Search index and results, rebuilt only when needed (M119).
    search: SearchCache,
    /// Groups collapsed during the current search; the saved expand state
    /// stays untouched and returns when the search is cleared (M124).
    collapsed_in_search: HashSet<GroupId>,
    /// Counts saves of the notes; a changed value invalidates the search
    /// index.
    document_revision: u64,
    /// Deletes and moves that can be undone (M135); memory only.
    undo: UndoHistory,
    /// The toast offering to undo the last delete (M148).
    toast: Option<Toast>,
    /// The trash view (M136, M145).
    trash: TrashState,
    /// An open delete question (M134, M146).
    confirmation: Option<Confirmation>,
}

impl MemoWindow {
    /// Creates the window state from what the storage worker loaded.
    #[must_use]
    pub fn new(
        document: NotesDocument,
        problem: Option<NotesProblem>,
        ui_state: UiState,
        saver: StorageSaver,
        addon_directory: PathBuf,
        drafts: DraftsDocument,
        settings: Settings,
    ) -> Self {
        let mut drafts = drafts;
        // A trashed note keeps its draft until it is deleted for good (M151).
        let note_exists =
            |note| document.note(note).is_some() || document.note_in_trash(note).is_some();
        if drafts.prune(Timestamp::now(), note_exists) {
            saver.save_drafts(drafts.clone());
        }
        let mut ui_state = ui_state;
        if ui_state
            .selected
            .is_some_and(|selected| !document.contains(selected))
        {
            ui_state.selected = None;
        }
        Self {
            document,
            problem,
            ui_state,
            saver,
            addon_directory,
            note_list: NoteList::default(),
            note_view: NoteView::default(),
            drawer_open: false,
            copied_chat_link: None,
            is_narrow: false,
            actions: Vec::new(),
            drafts,
            editor: None,
            leave_prompt: None,
            window_rect: None,
            window_focused: false,
            applied_text_scale: settings.text_scale,
            text_scale_changed_at: None,
            settings,
            close_approved: false,
            draft_hint: DraftHint::default(),
            dialog_height: None,
            next_reset_check: None,
            search_field: SearchField::default(),
            search: SearchCache::default(),
            collapsed_in_search: HashSet::new(),
            document_revision: 0,
            undo: UndoHistory::default(),
            toast: None,
            trash: TrashState::default(),
            confirmation: None,
        }
    }

    /// The text size the fonts should have (M79); lags behind Ctrl+mouse
    /// wheel steps until the wheel rests (M80).
    #[must_use]
    pub fn text_scale(&self) -> TextScale {
        self.applied_text_scale
    }

    /// Whether the editor holds changes that are not saved yet (M54).
    #[must_use]
    pub fn has_unsaved_changes(&self) -> bool {
        self.editor
            .as_ref()
            .is_some_and(|editor| editor.session().has_changes())
    }

    /// Asks to close the window (close button, keybind, quick access icon,
    /// Escape). Returns `true` if it may close now; otherwise the unsaved
    /// changes dialog opens and [`Self::take_close_approval`] reports the
    /// answer later (M54).
    pub fn request_close(&mut self) -> bool {
        // Escape while dragging cancels the drag (trailkit-ui skill §4).
        if self.note_list.cancel_drag() {
            return false;
        }
        // Escape (sent by Nexus as a close request) while the dialog is shown
        // picks the harmless answer (`overlay-dialog-layout`).
        if self.leave_prompt.is_some() {
            self.leave_prompt = None;
            return false;
        }
        if self.has_unsaved_changes()
            && self.settings.unsaved_changes == UnsavedChanges::SaveAutomatically
        {
            self.commit_editor(Timestamp::now());
        }
        if self.has_unsaved_changes() {
            if self.leave_prompt.is_none() {
                self.leave_prompt = Some(LeaveReason::CloseWindow);
            }
            return false;
        }
        self.editor = None;
        true
    }

    /// Whether the unsaved changes dialog allowed closing the window since the
    /// last call.
    pub fn take_close_approval(&mut self) -> bool {
        std::mem::take(&mut self.close_approved)
    }

    /// Keeps unsaved changes as a draft before the addon unloads (M54); the
    /// storage worker writes it during its shutdown.
    pub fn keep_draft_for_unload(&mut self) {
        let Some(editor) = &self.editor else {
            return;
        };
        if !editor.session().has_changes() {
            return;
        }
        let Some(note) = self.document.note(editor.session().note) else {
            return;
        };
        let draft = editor.session().to_draft(Timestamp::now(), note.updated_at);
        self.drafts.upsert(draft);
        self.saver.save_drafts(self.drafts.clone());
    }

    /// Draws the window content and applies what the user did.
    pub fn draw(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        // Everything in the window uses the shared body font at Nexus'
        // `FontUI` size (M35, M62); the window's default font differs.
        let _body_font = FontScope::push(ui, FontRole::Body);
        self.actions.clear();
        if let Some(copied) = &mut self.copied_chat_link {
            let since = *copied.shown_since.get_or_insert(ui.time());
            if ui.time() - since > COPIED_FEEDBACK_SECONDS {
                self.copied_chat_link = None;
            }
        }
        self.window_rect = Some([ui.window_pos(), ui.window_size()]);
        self.window_focused =
            ui.is_window_focused_with_flags(WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS);
        self.apply_due_resets(Timestamp::now());
        self.purge_trash(Timestamp::now());
        self.handle_undo_keys(ui);
        self.handle_text_scale_keys(ui);
        self.handle_search_keys(ui);
        self.update_search();
        self.update_text_scale(ui, language);
        self.draw_problem(ui, language, theme);

        if self.ui_state.tool_rail_visible {
            self.draw_tool_rail(ui, language, theme);
            ui.same_line_with_spacing(0.0, theme.pane.gap);
        }
        // A group keeps every line of the content right of the rail.
        ui.group(|| self.draw_content(ui, language, theme));
        // Last, so it lies above the content (M148).
        self.draw_undo_toast(ui, language, theme);

        if let Some(action) = self.note_list.finish_drag(ui, theme, &self.document) {
            self.actions.push(action.into());
        }
        let actions = std::mem::take(&mut self.actions);
        for action in &actions {
            self.apply(action, language);
        }
        // Finishing an edit or discarding a draft may unblock a waiting
        // reset (M89); a kind change may make one due. Check next frame.
        if !actions.is_empty() {
            self.next_reset_check = None;
        }
        self.actions = actions;
    }

    /// Draws the notes area: the empty state, two panes, or the note pane
    /// with the drawer in a narrow window.
    fn draw_content(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        if self.document.root.is_empty() && !self.trash.visible {
            draw_empty_state(ui, language, theme, &mut self.actions);
        } else {
            let character_width = ui.calc_text_size("n")[0];
            let pane_minimum = character_width * MINIMUM_PANE_CHARACTERS + 2.0 * theme.pane.padding;
            let widths = PaneWidths {
                tree_minimum: character_width * MINIMUM_TREE_CHARACTERS + 2.0 * theme.pane.padding,
                // Editor and preview side by side need room for two panes
                // (M73), so the note list turns into a drawer earlier.
                note_minimum: if self.editor.is_some() && self.ui_state.preview_visible {
                    2.0 * pane_minimum + theme.pane.gap
                } else {
                    pane_minimum
                },
            };
            let available = ui.content_region_avail();
            // The drawer threshold keeps the regular pane minimum for the
            // list; only dragging the splitter may make it narrower (M73).
            self.is_narrow = available[0] < pane_minimum + theme.pane.gap + widths.note_minimum;
            if !self.is_narrow && !self.ui_state.note_list_hidden {
                self.draw_two_panes(ui, language, theme, widths);
            } else {
                let area_start = ui.cursor_screen_pos();
                draw_pane(ui, theme, "note", [0.0, 0.0], || {
                    self.draw_note_pane(ui, language, theme);
                });
                // Without a selected note there is nothing to show but the list.
                if self.is_narrow && (self.drawer_open || self.selected_note().is_none()) {
                    self.draw_drawer(ui, language, theme, area_start, available, pane_minimum);
                }
            }
        }
    }

    /// Draws the tool rail (M136) and applies its clicks: the search toggle
    /// (M138) and the trash; settings follow in their step.
    fn draw_tool_rail(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let search_visible = self.ui_state.search_visible && !self.trash.visible;
        let top = [
            ToolRailItem {
                icon: Icon::SEARCH,
                tooltip: if search_visible {
                    Text::HideSearch.get(language)
                } else {
                    Text::ShowSearch.get(language)
                },
                active: search_visible,
            },
            ToolRailItem {
                icon: Icon::TRASH_2,
                tooltip: if self.trash.visible {
                    TrashText::HideTrash.get(language)
                } else {
                    TrashText::ShowTrash.get(language)
                },
                active: self.trash.visible,
            },
        ];
        match draw_tool_rail(ui, theme, &top, &[]) {
            Some(ToolRailClick::Top(0)) => self.actions.push(WindowAction::ToggleSearch),
            Some(ToolRailClick::Top(1)) => self.actions.push(WindowAction::ToggleTrash),
            _ => {}
        }
    }

    /// The memo toggles of the title bar (R78): the tool rail (M136) and the
    /// note list (M137). Empty while the notes are not loaded.
    #[must_use]
    pub fn title_bar_toggles(&self, language: Language) -> [TitleBarToggle<'static>; 2] {
        let list_visible = if self.is_narrow {
            self.drawer_open
        } else {
            !self.ui_state.note_list_hidden
        };
        [
            TitleBarToggle {
                icon: Icon::ELLIPSIS_VERTICAL,
                tooltip: if self.ui_state.tool_rail_visible {
                    Text::HideTools.get(language)
                } else {
                    Text::ShowTools.get(language)
                },
                active: self.ui_state.tool_rail_visible,
            },
            TitleBarToggle {
                icon: Icon::PANEL_LEFT,
                tooltip: if list_visible {
                    Text::HideSidebar.get(language)
                } else {
                    Text::ShowSidebar.get(language)
                },
                active: list_visible,
            },
        ]
    }

    /// Applies a click on the title bar toggle at `index` (see
    /// [`Self::title_bar_toggles`]).
    pub fn apply_title_bar_toggle(&mut self, index: usize, language: Language) {
        let action = match index {
            0 => WindowAction::ToggleToolRail,
            1 => WindowAction::ToggleNoteList,
            _ => return,
        };
        self.apply(&action, language);
    }

    /// Applies due resets (M88–M90) once a reset time has passed; notes in
    /// the editor or with a draft wait (M89). Cheap on every other frame.
    fn apply_due_resets(&mut self, now: Timestamp) {
        if self.next_reset_check.is_some_and(|check| now < check) {
            return;
        }
        let edited = self.editor.as_ref().map(|editor| editor.session().note);
        let drafts = &self.drafts;
        let changed = apply_due_resets(&mut self.document, now, |note| {
            Some(note) == edited || drafts.get(note).is_some()
        });
        if changed {
            self.save_notes();
        }
        self.next_reset_check = Some(next_reset_of_any_period(now));
    }

    /// The selected note, if a note (not a group) is selected.
    fn selected_note(&self) -> Option<&Note> {
        selected_note_in(&self.document, self.ui_state.selected)
    }
}

/// The note `selected` refers to, if it is a note that exists.
fn selected_note_in(document: &NotesDocument, selected: Option<NodeId>) -> Option<&Note> {
    match selected? {
        NodeId::Note(id) => document.note(id),
        NodeId::Group(_) => None,
    }
}

/// The localized title and tooltips of the main window's title bar (R71).
pub(crate) fn title_bar_texts(language: Language) -> TitleBarTexts<'static> {
    TitleBarTexts {
        title: Text::MainWindowTitle.get(language),
        collapse: Text::CollapseWindow.get(language),
        expand: Text::ExpandWindow.get(language),
        close: Text::CloseWindow.get(language),
    }
}

#[cfg(test)]
mod tests;
