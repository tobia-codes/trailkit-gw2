//! The memo main window: note tree, note view, editor, empty state and
//! storage hints (Windows only; memo plan M21, M31, M34, M37–M55).
//!
//! The window owns the note tree (M30). Drawing collects user actions into a
//! reused list; they are applied after the tree has been drawn, then changed
//! documents are handed to the storage worker.

mod dialogs;
mod layout;
mod note_list;
mod note_pane;

use std::{collections::HashSet, path::PathBuf, sync::Arc};

use jiff::Timestamp;
use nexus::imgui::{
    Condition, MouseButton, StyleColor, StyleVar, Ui, Window, WindowFlags, WindowFocusedFlags,
    WindowHoveredFlags,
};
use trailkit_ui::{
    editor::view::TextEditor,
    font_registry::FontScope,
    fonts::FontRole,
    markdown::{self, MarkdownDocument},
    markdown_view::{MarkdownAction, open_web_link},
    pane::draw_pane,
    text_search::SearchQuery,
    theme::Theme,
    title_bar::TitleBarTexts,
};

use crate::{
    note_editing::{ActiveFormats, EditSession, FormatCommand, active_formats, format_edit},
    tree_drag::{DropTarget, RowRect},
};
use trailkit_memo_reset_schedule::{
    due_resets::apply_due_resets, schedule::next_reset_of_any_period,
};
use trailkit_memo_search::{
    note_search::{NoteSearchIndex, SearchResults},
    search_field::SearchField,
};
use trailkit_memo_shared::{
    drafts::DraftsDocument,
    i18n::{Language, Text, label_with_percent},
    note_tree::{
        Group, GroupId, InsertionPoint, MoveCheck, NodeId, Note, NoteId, NoteKind, NotesDocument,
        ResetPeriod, TreeNode as Node,
    },
    settings::{Settings, TextScale, UnsavedChanges},
    storage_worker::StorageSaver,
    ui_state::UiState,
};

use self::{dialogs::draw_unsaved_changes_card, layout::draw_empty_state};

/// Opacity of the dragged row and its children while dragging (M106).
const DRAGGED_ROW_ALPHA: f32 = 0.4;

/// Auto-scroll speed while dragging near the list's edge, in rows per
/// second at the very edge (M111).
const DRAG_SCROLL_ROWS_PER_SECOND: f32 = 12.0;

/// Minimum content width of each pane in average characters (M40).
const MINIMUM_PANE_CHARACTERS: f32 = 22.0;
/// Minimum content width of the note list in average characters (M73): the
/// user may drag it narrower than the other panes.
const MINIMUM_TREE_CHARACTERS: f32 = 10.0;
/// Width in characters (of `n`) that editor and preview each keep, so the
/// splitter between them stays usable (M72).
const MINIMUM_PREVIEW_SPLIT_CHARACTERS: f32 = 3.0;

/// How long the text size waits after the last Ctrl+mouse wheel step before
/// the fonts are rebuilt (M80): rebuilding on every step flickers.
const TEXT_SCALE_APPLY_DELAY_SECONDS: f64 = 0.4;

/// How long the text size percentage stays visible after a change (M80).
const TEXT_SCALE_INDICATOR_SECONDS: f64 = 1.5;

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

/// Windows virtual-key code of `B` (Nexus feeds ImGui with virtual-key codes).
const VIRTUAL_KEY_B: i32 = 0x42;

/// Windows virtual-key code of `0` (Ctrl+0 resets the text size, M79).
const VIRTUAL_KEY_0: i32 = 0x30;

/// Windows virtual-key code of `F` (Ctrl+F focuses the search, M124).
const VIRTUAL_KEY_F: i32 = 0x46;

/// Windows virtual-key code of `I`.
const VIRTUAL_KEY_I: i32 = 0x49;

/// Windows virtual-key code of `S` (Ctrl+S finishes editing, M93).
const VIRTUAL_KEY_S: i32 = 0x53;

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

/// Something the user did while the window was drawn.
#[derive(Debug, Clone, PartialEq)]
enum WindowAction {
    /// Select a note or group.
    Select(NodeId),
    /// A group was expanded or collapsed.
    SetExpanded(GroupId, bool),
    /// Start renaming a note or group.
    StartRename(NodeId),
    /// Commit the rename with the edited title.
    CommitRename,
    /// Leave rename mode without changes.
    CancelRename,
    /// Create a note at a point, or by the selection for `None` (M43, M96).
    NewNote(Option<InsertionPoint>),
    /// Create a group at a point, or by the selection for `None`.
    NewGroup(Option<InsertionPoint>),
    /// The menu button: hide/show the note list column, or open/close the
    /// drawer in a narrow window (M48).
    ToggleNoteList,
    /// Close the drawer (click on the dimmed note).
    CloseDrawer,
    /// Toggle a task checkbox in the shown note.
    Markdown(MarkdownAction),
    /// Open the addon folder in the file explorer.
    OpenFolder,
    /// Start editing the selected note (M51).
    StartEditing,
    /// Commit the edit ("Save", Ctrl+Enter).
    FinishEditing,
    /// Leave the editor without saving; asks first if something changed.
    CancelEditing,
    /// Move a note or group with its children (drag & drop, "Move to",
    /// M106–M112).
    Move(NodeId, InsertionPoint),
    /// Change a note's reset period (`None` = normal note, M86).
    SetResetPeriod(NoteId, Option<ResetPeriod>),
    /// Apply a formatting button to the editor (M53).
    Format(FormatCommand),
    /// Show or hide the editor preview (M60).
    TogglePreview,
    /// Continue editing a note from its draft (M54).
    RestoreDraft(NoteId),
    /// Delete a note's draft (M54).
    DiscardDraft(NoteId),
    /// Unsaved changes dialog: save, then continue.
    LeaveSaving,
    /// Unsaved changes dialog: discard, then continue.
    LeaveDiscarding,
    /// Unsaved changes dialog: stay in the editor.
    LeaveCancelled,
    /// Collapse (`true`) or expand a group only for the current search
    /// (not saved, M124).
    SetCollapsedInSearch(GroupId, bool),
    /// Empty the search field (empty search result, M126).
    ClearSearch,
}

impl WindowAction {
    /// Whether this action leaves the editor of `note` (M54): choosing
    /// another entry or creating one. These ask first if something changed.
    fn leaves_editor_of(&self, note: NoteId) -> bool {
        match self {
            Self::Select(node) => *node != NodeId::Note(note),
            Self::NewNote(_) | Self::NewGroup(_) => true,
            _ => false,
        }
    }
}

/// What happens once the unsaved changes dialog is answered (M54).
#[derive(Debug, Clone, PartialEq)]
enum LeaveReason {
    /// Continue with this action.
    Action(WindowAction),
    /// Close the window.
    CloseWindow,
    /// Leave the editor ("Cancel").
    Cancel,
}

/// The editor of one note (M51–M53, M58).
#[derive(Debug)]
struct Editor {
    /// Uncommitted title and body.
    session: EditSession,
    /// The word-wrapping body editor; its text is mirrored into
    /// `session.body` on every change.
    body: TextEditor,
    /// The parsed preview of the body (M60); `None` when it must be parsed
    /// again because the text changed.
    preview: Option<MarkdownDocument>,
    /// Whether the title input was active in the last frame.
    title_was_active: bool,
    /// Formatting active at the cursor, cached per text revision and
    /// selection (M69).
    active: Option<(u64, std::ops::Range<usize>, ActiveFormats)>,
}

impl Editor {
    /// Opens an editor for `session`, focused on the body.
    fn new(session: EditSession) -> Self {
        let body = TextEditor::new(session.body.clone());
        Self {
            session,
            body,
            preview: None,
            title_was_active: false,
            active: None,
        }
    }

    /// Applies a formatting command to the body at its selection (M53).
    fn format(&mut self, command: FormatCommand) {
        let edit = format_edit(self.body.text(), self.body.selection(), command);
        self.body
            .apply_edit(edit.range, &edit.replacement, edit.selection);
        self.body.focus();
        self.sync_body();
    }

    /// The formatting active at the cursor (recomputed only when the text or
    /// the selection changed).
    fn active_formats(&mut self) -> ActiveFormats {
        let revision = self.body.revision();
        let selection = self.body.selection();
        match &self.active {
            Some((cached_revision, cached_selection, formats))
                if *cached_revision == revision && *cached_selection == selection =>
            {
                *formats
            }
            _ => {
                let formats = active_formats(self.body.text(), selection.clone());
                self.active = Some((revision, selection, formats));
                formats
            }
        }
    }

    /// Copies the editor text into the session and marks the preview stale.
    fn sync_body(&mut self) {
        self.session.body.clear();
        self.session.body.push_str(self.body.text());
        self.preview = None;
    }
}

/// An inline rename in progress (M44).
#[derive(Debug)]
struct RenameState {
    /// The note or group being renamed.
    node: NodeId,
    /// The edited title.
    buffer: String,
    /// Whether keyboard focus was already given to the input.
    focused: bool,
}

/// A press on a note list row that may become a drag (M106).
#[derive(Debug)]
struct TreeDrag {
    /// The pressed note or group.
    node: NodeId,
    /// Mouse position of the press.
    pressed_at: [f32; 2],
    /// Whether the mouse moved far enough to drag; until then a release
    /// selects the row.
    is_dragging: bool,
    /// Where a drop would go in this frame, if the mouse is over the list.
    target: Option<DropTarget>,
    /// Whether the move to `target` is possible, changes nothing or changes
    /// the tree.
    check: MoveCheck,
    /// The collapsed group hovered in its middle and since when (ImGui
    /// time); it expands after [`EXPAND_DELAY_SECONDS`] (M108).
    collapsed_hover: Option<(GroupId, f64)>,
}

/// The parsed markdown of the shown note, re-parsed only when it changes.
#[derive(Debug)]
struct ParsedNote {
    /// The note the cache belongs to.
    note: NoteId,
    /// The body the document was parsed from.
    source: String,
    /// The parsed body.
    document: MarkdownDocument,
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
    /// Inline rename in progress.
    rename: Option<RenameState>,
    /// Markdown cache of the shown note.
    parsed: Option<ParsedNote>,
    /// Narrow window: whether the note list drawer is open (M48).
    drawer_open: bool,
    /// The chat code copied last, while its chip says "Copied" (M75).
    copied_chat_link: Option<CopiedChatLink>,
    /// Whether the window was too narrow for two panes in the last frame;
    /// decides what the menu button does.
    is_narrow: bool,
    /// Whether the saved scroll position still has to be applied.
    restore_scroll: bool,
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
    draft_hint: Option<(NoteId, Language, String)>,
    /// When reset notes must be checked next (M90); `None` checks at once,
    /// e.g. after an action that may have unblocked a waiting reset (M89).
    next_reset_check: Option<Timestamp>,
    /// A press on a list row, possibly a drag (M106).
    drag: Option<TreeDrag>,
    /// Rectangles of the list rows drawn in this frame; reused every frame.
    tree_rows: Vec<RowRect>,
    /// The search field above the note list (M118); its text survives
    /// closing the window, not a game restart (M124).
    search_field: SearchField,
    /// Search index and results, rebuilt only when needed (M119).
    search: SearchCache,
    /// Groups collapsed during the current search; the saved expand state
    /// stays untouched and returns when the search is cleared (M124).
    collapsed_in_search: HashSet<GroupId>,
    /// Scroll the note view to its first search hit on the next draw (M122).
    scroll_to_first_hit: bool,
    /// A row was pressed while searching: dragging cannot move it (M125).
    drag_blocked_by_search: bool,
    /// Counts saves of the notes; a changed value invalidates the search
    /// index.
    document_revision: u64,
}

/// The search index and the results of the current query, each cached until
/// what it depends on changes (M119: never rebuilt per frame).
#[derive(Debug, Default)]
struct SearchCache {
    /// The index and the document revision it was built from.
    index: Option<(u64, NoteSearchIndex)>,
    /// The results of the current query, if the field holds one.
    current: Option<CurrentSearch>,
}

/// Results of one query over one document revision.
#[derive(Debug)]
struct CurrentSearch {
    /// The field text the results belong to.
    text: String,
    /// The parsed query.
    query: SearchQuery,
    /// The document revision the results belong to.
    revision: u64,
    /// The node kept visible (the open note, M125).
    keep_visible: Option<NodeId>,
    /// What the note list shows.
    results: SearchResults,
}

/// Everything the tree drawing functions need besides the nodes.
struct TreeContext<'state> {
    /// The whole tree ("Move to" lists its groups, M112).
    document: &'state NotesDocument,
    /// The selected note or group.
    selected: Option<NodeId>,
    /// The note or group being dragged; it is drawn faded (M106).
    dragged: Option<NodeId>,
    /// The row pressed with the left button in this frame.
    pressed: Option<NodeId>,
    /// Collects the rectangle of every drawn row.
    rows: &'state mut Vec<RowRect>,
    /// Inline rename in progress.
    rename: &'state mut Option<RenameState>,
    /// Collected actions.
    actions: &'state mut Vec<WindowAction>,
    /// UI language.
    language: Language,
    /// Active theme.
    theme: &'state Theme,
    /// ImGui frame padding; rows use it so they are as tall as the rename
    /// input and titles line up with the input text.
    frame_padding: [f32; 2],
    /// The active search and its results (M118).
    search: Option<(&'state SearchQuery, &'state SearchResults)>,
    /// Groups collapsed during the search (M124).
    collapsed_in_search: &'state HashSet<GroupId>,
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
        if drafts.prune(Timestamp::now(), |note| document.note(note).is_some()) {
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
            rename: None,
            parsed: None,
            drawer_open: false,
            copied_chat_link: None,
            is_narrow: false,
            restore_scroll: true,
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
            draft_hint: None,
            dialog_height: None,
            next_reset_check: None,
            drag: None,
            tree_rows: Vec::new(),
            search_field: SearchField::default(),
            search: SearchCache::default(),
            collapsed_in_search: HashSet::new(),
            scroll_to_first_hit: false,
            drag_blocked_by_search: false,
            document_revision: 0,
        }
    }

    /// The text size the fonts should have (M79); lags behind Ctrl+mouse
    /// wheel steps until the wheel rests (M80).
    #[must_use]
    pub fn text_scale(&self) -> TextScale {
        self.applied_text_scale
    }

    /// Ctrl+mouse wheel over the window makes the text larger or smaller,
    /// Ctrl+0 resets it (M79); the change is saved and takes effect when the
    /// addon resizes the fonts next frame.
    fn handle_text_scale_keys(&mut self, ui: &Ui) {
        let io = ui.io();
        if !io.key_ctrl {
            return;
        }
        let hovered = ui.is_window_hovered_with_flags(
            WindowHoveredFlags::ROOT_AND_CHILD_WINDOWS
                | WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM,
        );
        let scale = if hovered && io.mouse_wheel != 0.0 {
            self.settings.text_scale.stepped(io.mouse_wheel.signum())
        } else if self.window_focused && ui.is_key_index_pressed(VIRTUAL_KEY_0) {
            TextScale::DEFAULT
        } else {
            return;
        };
        if scale != self.settings.text_scale {
            self.settings.text_scale = scale;
            self.saver.save_settings(self.settings);
        }
        // Show the size even when it is already at a limit.
        self.text_scale_changed_at = Some(ui.time());
    }

    /// Applies a changed text size once the wheel rests and shows its
    /// percentage next to the mouse while it changes (M80).
    fn update_text_scale(&mut self, ui: &Ui, language: Language) {
        let Some(changed_at) = self.text_scale_changed_at else {
            return;
        };
        let elapsed = ui.time() - changed_at;
        if elapsed >= TEXT_SCALE_APPLY_DELAY_SECONDS {
            self.applied_text_scale = self.settings.text_scale;
        }
        if elapsed < TEXT_SCALE_INDICATOR_SECONDS {
            ui.tooltip_text(label_with_percent(
                Text::TextSize.get(language),
                self.settings.text_scale.factor() * 100.0,
                language,
            ));
        } else {
            self.text_scale_changed_at = None;
        }
    }

    /// Whether Escape should clear the search instead of closing the window:
    /// the focused search field holds text (M124).
    #[must_use]
    pub fn search_takes_escape(&self) -> bool {
        self.search_field.is_active() && !self.search_field.text.is_empty()
    }

    /// Clears the search after Escape (see [`Self::search_takes_escape`]);
    /// the field keeps the focus, so a second Escape closes the window.
    pub fn clear_search_by_escape(&mut self) {
        self.search_field.clear(true);
    }

    /// Ctrl+F focuses the search field (M124, M127): it shows the note list
    /// or opens the drawer first and takes the focus from the editor, which
    /// stays in edit mode.
    fn handle_search_keys(&mut self, ui: &Ui) {
        if !(self.window_focused && ui.io().key_ctrl && ui.is_key_index_pressed(VIRTUAL_KEY_F)) {
            return;
        }
        if self.is_narrow {
            self.drawer_open = true;
        } else if self.ui_state.note_list_hidden {
            self.ui_state.note_list_hidden = false;
            self.saver.save_ui_state(self.ui_state);
        }
        if let Some(editor) = &mut self.editor {
            editor.body.unfocus();
        }
        self.search_field.request_focus();
    }

    /// Brings the search index and results up to date with the field text,
    /// the notes and the open note (M119). Cheap when nothing changed.
    fn update_search(&mut self) {
        let Some(query) = SearchQuery::parse(&self.search_field.text) else {
            if self.search.current.take().is_some() {
                self.collapsed_in_search.clear();
            }
            return;
        };
        let revision = self.document_revision;
        let keep_visible = self.ui_state.selected;
        if let Some(current) = &self.search.current
            && current.revision == revision
            && current.keep_visible == keep_visible
            && current.text == self.search_field.text
        {
            return;
        }
        if self
            .search
            .index
            .as_ref()
            .is_none_or(|(built_from, _)| *built_from != revision)
        {
            self.search.index = Some((revision, NoteSearchIndex::build(&self.document)));
        }
        let Some((_, index)) = &self.search.index else {
            return;
        };
        let query_changed = self
            .search
            .current
            .as_ref()
            .is_none_or(|current| current.query != query);
        if query_changed {
            // A new search starts with every matching group expanded and
            // jumps to the first hit of the open note (M122).
            self.collapsed_in_search.clear();
            self.scroll_to_first_hit = true;
        }
        let results = SearchResults::compute(&self.document, index, &query, keep_visible);
        self.search.current = Some(CurrentSearch {
            text: self.search_field.text.clone(),
            query,
            revision,
            keep_visible,
            results,
        });
    }

    /// Whether the editor holds changes that are not saved yet (M54).
    #[must_use]
    pub fn has_unsaved_changes(&self) -> bool {
        self.editor
            .as_ref()
            .is_some_and(|editor| editor.session.has_changes())
    }

    /// Asks to close the window (close button, keybind, quick access icon,
    /// Escape). Returns `true` if it may close now; otherwise the unsaved
    /// changes dialog opens and [`Self::take_close_approval`] reports the
    /// answer later (M54).
    pub fn request_close(&mut self) -> bool {
        // Escape while dragging cancels the drag (trailkit-ui skill §4).
        if self.drag.as_ref().is_some_and(|drag| drag.is_dragging) {
            self.drag = None;
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
        if !editor.session.has_changes() {
            return;
        }
        let Some(note) = self.document.note(editor.session.note) else {
            return;
        };
        let draft = editor.session.to_draft(Timestamp::now(), note.updated_at);
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
        self.handle_text_scale_keys(ui);
        self.handle_search_keys(ui);
        self.update_search();
        self.update_text_scale(ui, language);
        self.draw_problem(ui, language, theme);

        if self.document.root.is_empty() {
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
                    self.draw_note_pane(ui, language, theme, Text::ShowNoteList);
                });
                // Without a selected note there is nothing to show but the list.
                if self.is_narrow && (self.drawer_open || self.selected_note().is_none()) {
                    self.draw_drawer(ui, language, theme, area_start, available, pane_minimum);
                }
            }
        }

        self.finish_drag(ui, theme);
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

    /// Applies due resets (M88–M90) once a reset time has passed; notes in
    /// the editor or with a draft wait (M89). Cheap on every other frame.
    fn apply_due_resets(&mut self, now: Timestamp) {
        if self.next_reset_check.is_some_and(|check| now < check) {
            return;
        }
        let edited = self.editor.as_ref().map(|editor| editor.session.note);
        let drafts = &self.drafts;
        let changed = apply_due_resets(&mut self.document, now, |note| {
            Some(note) == edited || drafts.get(note).is_some()
        });
        if changed {
            self.save_notes();
        }
        self.next_reset_check = Some(next_reset_of_any_period(now));
    }

    /// Draws the unsaved changes dialog while an answer is pending (M54,
    /// M56, `overlay-dialog-scope`) and applies the answer. Call it after the
    /// main window is finished: the dialog is its own borderless window
    /// exactly over the main window, so it dims only that window (title bar
    /// included) while the game and other windows stay usable. Clicking the
    /// main window brings the dialog back to the front; a click on the dimmed
    /// area keeps editing, like Escape. "Discard" sits apart
    /// from "Keep editing" and the primary "Save" (`lay-one-primary`); Enter
    /// saves, Escape keeps editing.
    pub fn draw_dialog(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        if self.leave_prompt.is_none() {
            self.dialog_height = None;
            return;
        }
        let _body_font = FontScope::push(ui, FontRole::Body);
        let Some(window_rect) = self.window_rect else {
            return;
        };
        let _background =
            ui.push_style_color(StyleColor::WindowBg, theme.surface.modal_overlay.to_array());
        let _border = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
        let mut answer = None;
        Window::new("###trailkit-memo-dialog")
            .position(window_rect[0], Condition::Always)
            .size(window_rect[1], Condition::Always)
            .flags(
                WindowFlags::NO_DECORATION
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SAVED_SETTINGS
                    | WindowFlags::NO_NAV,
            )
            .focused(self.window_focused)
            .build(ui, || {
                let card = draw_unsaved_changes_card(ui, language, theme, self.dialog_height);
                self.dialog_height = Some(card.height);
                answer = card.answer;
                // A click on the dimmed area picks the harmless answer
                // (`overlay-dialog-layout`).
                let mouse = ui.io().mouse_pos;
                let on_card = (card.min[0]..=card.max[0]).contains(&mouse[0])
                    && (card.min[1]..=card.max[1]).contains(&mouse[1]);
                if answer.is_none()
                    && !on_card
                    && ui.is_window_hovered()
                    && ui.is_mouse_clicked(MouseButton::Left)
                {
                    answer = Some(WindowAction::LeaveCancelled);
                }
            });
        if let Some(answer) = answer {
            self.apply(&answer, language);
        }
    }

    /// The selected note, if a note (not a group) is selected.
    fn selected_note(&self) -> Option<&Note> {
        selected_note_in(&self.document, self.ui_state.selected)
    }

    /// Applies one collected action.
    fn apply(&mut self, action: &WindowAction, language: Language) {
        let now = Timestamp::now();
        if let Some(editor) = &self.editor
            && action.leaves_editor_of(editor.session.note)
        {
            if self.leave_prompt.is_some() {
                return;
            }
            match (editor.session.has_changes(), self.settings.unsaved_changes) {
                (true, UnsavedChanges::Ask) => {
                    self.leave_prompt = Some(LeaveReason::Action(action.clone()));
                    return;
                }
                (true, UnsavedChanges::SaveAutomatically) => self.commit_editor(now),
                (false, _) => self.editor = None,
            }
        }
        match action {
            WindowAction::Select(node) => self.select(*node),
            WindowAction::SetExpanded(group, expanded) => {
                if let Some(group) = self.document.group_mut(*group)
                    && group.expanded != *expanded
                {
                    group.expanded = *expanded;
                    self.save_notes();
                }
            }
            WindowAction::Move(node, point) => {
                if self.document.move_node(*node, *point) {
                    self.save_notes();
                }
            }
            WindowAction::StartRename(node) => {
                let title = match node {
                    NodeId::Note(id) => self.document.note(*id).map(|note| note.title.clone()),
                    NodeId::Group(_) => find_title(&self.document.root, *node).map(str::to_owned),
                };
                if let Some(title) = title {
                    self.rename = Some(RenameState {
                        node: *node,
                        buffer: title,
                        focused: false,
                    });
                }
            }
            WindowAction::CommitRename => {
                if let Some(rename) = self.rename.take()
                    && self.document.rename(rename.node, &rename.buffer, now)
                {
                    self.save_notes();
                }
            }
            WindowAction::CancelRename => self.rename = None,
            WindowAction::NewNote(point) => {
                // The new entry must be visible: creating clears the search (M126).
                self.search_field.clear(false);
                let point = self.insertion_point(*point);
                let note = Note::new(
                    Text::NewNote.get(language).to_owned(),
                    NoteKind::Normal,
                    now,
                );
                let id = NodeId::Note(note.id);
                self.document.insert(point, Node::Note(note));
                self.select(id);
                self.start_rename_of_new(id, Text::NewNote.get(language));
                self.save_notes();
            }
            WindowAction::NewGroup(point) => {
                self.search_field.clear(false);
                let point = self.insertion_point(*point);
                let group = Group::new(Text::NewGroup.get(language).to_owned());
                let id = NodeId::Group(group.id);
                self.document.insert(point, Node::Group(group));
                self.select(id);
                self.start_rename_of_new(id, Text::NewGroup.get(language));
                self.save_notes();
            }
            WindowAction::SetCollapsedInSearch(group, collapsed) => {
                if *collapsed {
                    self.collapsed_in_search.insert(*group);
                } else {
                    self.collapsed_in_search.remove(group);
                }
            }
            WindowAction::ClearSearch => self.search_field.clear(false),
            WindowAction::ToggleNoteList => {
                if self.is_narrow {
                    self.drawer_open = !self.drawer_open;
                } else {
                    self.ui_state.note_list_hidden = !self.ui_state.note_list_hidden;
                    self.saver.save_ui_state(self.ui_state);
                }
            }
            WindowAction::CloseDrawer => self.drawer_open = false,
            WindowAction::Markdown(MarkdownAction::ToggleTask(marker)) => {
                let Some(NodeId::Note(id)) = self.ui_state.selected else {
                    return;
                };
                let Some(note) = self.document.note_mut(id) else {
                    return;
                };
                if let Some(body) = markdown::toggle_task(&note.body, *marker) {
                    note.body = body;
                    note.updated_at = now;
                    self.save_notes();
                }
            }
            WindowAction::Markdown(MarkdownAction::OpenLink(url)) => open_web_link(url),
            WindowAction::Markdown(MarkdownAction::ChatLinkCopied(code)) => {
                self.copied_chat_link = Some(CopiedChatLink {
                    code: code.clone(),
                    shown_since: None,
                });
            }
            WindowAction::OpenFolder => {
                if let Err(error) = std::process::Command::new("explorer")
                    .arg(&self.addon_directory)
                    .spawn()
                {
                    log::warn!("cannot open the addon folder: {error}");
                }
            }
            WindowAction::StartEditing => {
                if self.editor.is_none()
                    && let Some(note) = self.selected_note()
                {
                    self.editor = Some(Editor::new(EditSession::start(note)));
                }
            }
            WindowAction::FinishEditing => self.commit_editor(now),
            WindowAction::SetResetPeriod(note, period) => {
                if self.document.set_reset_period(*note, *period, now) {
                    self.save_notes();
                }
            }
            WindowAction::CancelEditing => {
                if self.has_unsaved_changes()
                    && self.settings.unsaved_changes == UnsavedChanges::Ask
                {
                    self.leave_prompt = Some(LeaveReason::Cancel);
                } else {
                    self.editor = None;
                }
            }
            WindowAction::Format(command) => {
                if let Some(editor) = self.editor.as_mut() {
                    editor.format(*command);
                }
            }
            WindowAction::TogglePreview => {
                self.ui_state.preview_visible = !self.ui_state.preview_visible;
                self.saver.save_ui_state(self.ui_state);
            }
            WindowAction::RestoreDraft(id) => {
                if self.editor.is_none()
                    && let Some(note) = self.document.note(*id)
                    && let Some(draft) = self.drafts.take(*id)
                {
                    self.editor = Some(Editor::new(EditSession::from_draft(note, draft)));
                    self.saver.save_drafts(self.drafts.clone());
                }
            }
            WindowAction::DiscardDraft(id) => {
                if self.drafts.take(*id).is_some() {
                    self.saver.save_drafts(self.drafts.clone());
                }
            }
            WindowAction::LeaveSaving | WindowAction::LeaveDiscarding => {
                let Some(reason) = self.leave_prompt.take() else {
                    return;
                };
                if *action == WindowAction::LeaveSaving {
                    self.commit_editor(now);
                } else {
                    self.editor = None;
                }
                match reason {
                    LeaveReason::Action(next) => self.apply(&next, language),
                    LeaveReason::CloseWindow => self.close_approved = true,
                    LeaveReason::Cancel => {}
                }
            }
            WindowAction::LeaveCancelled => self.leave_prompt = None,
        }
    }

    /// Writes the editor's title and body back into the note, saves, and
    /// deletes the note's draft (M52, M55). An empty title keeps the old one.
    fn commit_editor(&mut self, now: Timestamp) {
        let Some(editor) = self.editor.take() else {
            return;
        };
        let session = editor.session;
        let mut changed = self
            .document
            .rename(NodeId::Note(session.note), &session.title, now);
        if let Some(note) = self.document.note_mut(session.note)
            && note.body != session.body
        {
            note.body = session.body;
            note.updated_at = now;
            changed = true;
        }
        if changed {
            self.save_notes();
        }
        if self.drafts.take(session.note).is_some() {
            self.saver.save_drafts(self.drafts.clone());
        }
    }

    /// Selects `node`, shows it in the single-pane layout and remembers it.
    fn select(&mut self, node: NodeId) {
        if self.ui_state.selected != Some(node) {
            self.ui_state.selected = Some(node);
            self.ui_state.note_scroll = 0.0;
            self.restore_scroll = true;
            self.scroll_to_first_hit = self.search.current.is_some();
            self.saver.save_ui_state(self.ui_state);
        }
        // Choosing a note closes the drawer; groups keep it open (M48).
        if matches!(node, NodeId::Note(_)) {
            self.drawer_open = false;
        }
    }

    /// Resolves where a new entry goes: `point` from a context menu (M96),
    /// or for the toolbar buttons the end of the selected group (M43).
    fn insertion_point(&self, point: Option<InsertionPoint>) -> InsertionPoint {
        point.unwrap_or_else(|| {
            InsertionPoint::End(self.document.insertion_parent(self.ui_state.selected))
        })
    }

    /// Puts a just-created note or group into rename mode (M43).
    fn start_rename_of_new(&mut self, node: NodeId, title: &str) {
        // The rename happens in the list, so a narrow window shows it.
        self.drawer_open = true;
        self.rename = Some(RenameState {
            node,
            buffer: title.to_owned(),
            focused: false,
        });
    }

    /// Hands a copy of the note tree to the storage worker and marks the
    /// search index stale.
    fn save_notes(&mut self) {
        self.document_revision = self.document_revision.wrapping_add(1);
        self.saver.save_notes(Arc::new(self.document.clone()));
    }
}

/// The note `selected` refers to, if it is a note that exists.
fn selected_note_in(document: &NotesDocument, selected: Option<NodeId>) -> Option<&Note> {
    match selected? {
        NodeId::Note(id) => document.note(id),
        NodeId::Group(_) => None,
    }
}

/// Finds the title of any node in the tree.
fn find_title(nodes: &[Node], id: NodeId) -> Option<&str> {
    nodes.iter().find_map(|node| {
        if node.id() == id {
            return Some(node.title());
        }
        match node {
            Node::Group(group) => find_title(&group.children, id),
            Node::Note(_) => None,
        }
    })
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
