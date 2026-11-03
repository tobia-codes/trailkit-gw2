//! The memo main window: note tree, note view, editor, empty state and
//! storage hints (Windows only; memo plan M21, M31, M34, M37–M55).
//!
//! The window owns the note tree (M30). Drawing collects user actions into a
//! reused list; they are applied after the tree has been drawn, then changed
//! documents are handed to the storage worker.

use std::{borrow::Cow, collections::HashSet, path::PathBuf, sync::Arc};

use jiff::Timestamp;
use nexus::imgui::{
    ChildWindow, Condition, Key, MenuItem, MouseButton, StyleColor, StyleVar, TreeNode,
    TreeNodeFlags, Ui, Window, WindowFlags, WindowFocusedFlags, WindowHoveredFlags,
};
use trailkit_ui::{
    button::{
        button_width, draw_button, draw_button_with_size, draw_icon_button, draw_primary_button,
        draw_primary_button_with_icon, draw_primary_button_with_size, icon_label_button_width,
    },
    chat_link_chip::ChipContext,
    color::Color,
    dropdown::{draw_dropdown, dropdown_width},
    editor::view::TextEditor,
    font_registry::FontScope,
    fonts::FontRole,
    glyph_metrics::{capital_middle, glyph_ink_box},
    icons::Icon,
    markdown::{self, MarkdownDocument},
    markdown_view::{MarkdownAction, SearchHighlight, draw_markdown, open_web_link},
    menu::{begin_submenu, draw_menu_item},
    pane::draw_pane,
    selection_bar::draw_selection_bar,
    separator::draw_separator,
    splitter::draw_vertical_splitter,
    text_search::SearchQuery,
    theme::Theme,
    title_bar::TitleBarTexts,
    tree_arrow::draw_tree_arrow,
};

use crate::{
    drafts::{Draft, DraftsDocument},
    i18n::{Language, Text, chip_text, format_local_time, label_with_percent, reset_countdown},
    note_editing::{ActiveFormats, EditSession, FormatCommand, active_formats, format_edit},
    note_search::{NoteSearchIndex, SearchResults},
    note_tree::{
        Group, GroupId, InsertionPoint, MoveCheck, NodeId, Note, NoteId, NoteKind, NotesDocument,
        TreeNode as Node,
    },
    reset_schedule::{ResetPeriod, next_reset_of_any_period},
    search_field::{SearchField, SearchFieldTexts},
    settings::{Settings, TextScale, UnsavedChanges},
    storage_worker::StorageSaver,
    tree_drag::{
        DRAG_THRESHOLD, DropIndicator, DropTarget, EXPAND_DELAY_SECONDS, RowKind, RowRect,
        resolve_drop,
    },
    ui_state::UiState,
};

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

/// The note kinds in menu and dropdown order with their labels (M86).
const KIND_OPTIONS: [(Option<ResetPeriod>, Text); 3] = [
    (None, Text::KindNormal),
    (Some(ResetPeriod::Daily), Text::KindDaily),
    (Some(ResetPeriod::Weekly), Text::KindWeekly),
];

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
        let changed = self.document.apply_due_resets(now, |note| {
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

    /// Draws the storage problem hint, if any (M31).
    fn draw_problem(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let Some(problem) = &self.problem else {
            return;
        };
        {
            let _icons = FontScope::push(ui, FontRole::Icons);
            let mut buffer = [0; 4];
            ui.text_colored(
                theme.status.warning.to_array(),
                Icon::TRIANGLE_ALERT.as_str(&mut buffer),
            );
        }
        ui.same_line();
        let offers_folder = match problem {
            NotesProblem::Recovered(path) => {
                ui.text_wrapped(Text::NotesRecovered.get(language));
                ui.text_wrapped(path);
                true
            }
            NotesProblem::NewerVersion => {
                ui.text_wrapped(Text::NotesNewerVersion.get(language));
                false
            }
            NotesProblem::Inaccessible => {
                ui.text_wrapped(Text::NotesInaccessible.get(language));
                true
            }
        };
        if offers_folder && draw_button(ui, Text::OpenFolder.get(language)) {
            self.actions.push(WindowAction::OpenFolder);
        }
        draw_separator(ui, theme);
    }

    /// Draws tree and note as two panes with a draggable splitter in the gap
    /// between them (M21). The splitter shows an accent line only while it
    /// is hovered or dragged.
    fn draw_two_panes(&mut self, ui: &Ui, language: Language, theme: &Theme, widths: PaneWidths) {
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
            self.draw_tree_pane(ui, language, theme, None);
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
            self.draw_note_pane(ui, language, theme, Text::HideNoteList);
        });
    }

    /// Draws the note list as a drawer over the dimmed note (narrow window,
    /// M48, `lay-drawer`): it appears instantly (`overlay-motion`) and closes
    /// on a click on the dimmed area. `start` and `size` cover the note pane.
    fn draw_drawer(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        start: [f32; 2],
        size: [f32; 2],
        pane_minimum: f32,
    ) {
        ui.set_cursor_screen_pos(start);
        // The scrim takes the clicks on the note. ImGui renders a child's
        // background into the parent's draw list, below the note pane drawn
        // before, so the dimming is a rectangle in the scrim's own list.
        let Some(_scrim) = ChildWindow::new("drawer_scrim")
            .size(size)
            .scrollable(false)
            .scroll_bar(false)
            .draw_background(false)
            .begin(ui)
        else {
            return;
        };
        let scrim_start = ui.window_pos();
        let scrim_end = [scrim_start[0] + size[0], scrim_start[1] + size[1]];
        ui.get_window_draw_list()
            .add_rect(
                scrim_start,
                scrim_end,
                theme.surface.modal_overlay.to_array(),
            )
            .filled(true)
            .build();
        let width = pane_minimum.min(size[0] * DRAWER_MAXIMUM_FRACTION);
        draw_pane(ui, theme, "drawer", [width, 0.0], || {
            self.draw_tree_pane(ui, language, theme, Some(Text::HideNoteList));
        });
        if ui.is_window_hovered() && ui.is_mouse_clicked(MouseButton::Left) {
            self.actions.push(WindowAction::CloseDrawer);
        }
    }

    /// Draws the toolbar and the tree. `menu` is the tooltip of a leading
    /// menu button (drawer only, so it closes where it opened).
    fn draw_tree_pane(&mut self, ui: &Ui, language: Language, theme: &Theme, menu: Option<Text>) {
        let search_texts = SearchFieldTexts {
            hint: Text::SearchHint.get(language),
            clear: Text::ClearSearch.get(language),
        };
        if self.search_field.draw(ui, theme, search_texts) {
            // Apply the new text in this frame already.
            self.update_search();
        }
        draw_toolbar(ui, language, theme, menu, &mut self.actions);
        self.tree_rows.clear();
        if let Some(current) = &self.search.current
            && !current.results.has_hits()
        {
            draw_no_search_results(ui, language, theme, &current.text, &mut self.actions);
            return;
        }
        let style = ui.clone_style();
        let mut context = TreeContext {
            document: &self.document,
            selected: self.ui_state.selected,
            dragged: self
                .drag
                .as_ref()
                .filter(|drag| drag.is_dragging)
                .map(|drag| drag.node),
            pressed: None,
            rows: &mut self.tree_rows,
            rename: &mut self.rename,
            actions: &mut self.actions,
            language,
            theme,
            frame_padding: style.frame_padding,
            search: self
                .search
                .current
                .as_ref()
                .map(|current| (&current.query, &current.results)),
            collapsed_in_search: &self.collapsed_in_search,
        };
        draw_nodes(ui, &self.document.root, &mut context);
        let is_searching = context.search.is_some();
        if is_searching {
            // Positions in a filtered list are ambiguous: no drag & drop
            // while searching (M125).
            if context.pressed.is_some() {
                self.drag_blocked_by_search = true;
            }
            if !ui.is_mouse_down(MouseButton::Left) {
                self.drag_blocked_by_search = false;
            } else if self.drag_blocked_by_search
                && ui.is_mouse_dragging_with_threshold(MouseButton::Left, DRAG_THRESHOLD)
            {
                ui.tooltip_text(Text::ClearSearchToMove.get(language));
            }
        } else if let Some(node) = context.pressed {
            self.drag = Some(TreeDrag {
                node,
                pressed_at: ui.io().mouse_pos,
                is_dragging: false,
                target: None,
                check: MoveCheck::Unchanged,
                collapsed_hover: None,
            });
        }
        self.track_drag(ui, theme, style.indent_spacing);
        // Right-click on an empty spot of the list (M96): create at the end
        // of the root level.
        if ui.is_window_hovered()
            && !ui.is_any_item_hovered()
            && ui.is_mouse_clicked(MouseButton::Right)
        {
            ui.open_popup("list_context");
        }
        let actions = &mut self.actions;
        ui.popup("list_context", || {
            draw_create_menu_items(ui, language, InsertionPoint::End(None), actions);
        });
    }

    /// Follows a drag over the note list (M106–M111): starts it once the
    /// mouse moved [`DRAG_THRESHOLD`], finds the drop target, draws the drop
    /// line or group highlight, expands a collapsed group after a rest and
    /// scrolls near the list's edges. Runs inside the list's window after
    /// its rows; `indent` is the width of one tree level.
    fn track_drag(&mut self, ui: &Ui, theme: &Theme, indent: f32) {
        let Some(drag) = &mut self.drag else {
            return;
        };
        let mouse = ui.io().mouse_pos;
        if !drag.is_dragging {
            let moved = [mouse[0] - drag.pressed_at[0], mouse[1] - drag.pressed_at[1]];
            if !ui.is_mouse_down(MouseButton::Left) || moved[0].hypot(moved[1]) < DRAG_THRESHOLD {
                return;
            }
            drag.is_dragging = true;
        }
        // The pressed row stays the active item while dragging.
        let is_over_list =
            ui.is_window_hovered_with_flags(WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM);
        drag.target = is_over_list
            .then(|| resolve_drop(&self.tree_rows, mouse[1], indent))
            .flatten();
        drag.check = drag.target.map_or(MoveCheck::Unchanged, |target| {
            self.document.check_move(drag.node, target.point)
        });
        if !is_over_list {
            drag.collapsed_hover = None;
            return;
        }

        // A collapsed group expands after the drag rested on it (M108).
        let collapsed_group = drag
            .target
            .and_then(|target| target.collapsed_group)
            .filter(|_| drag.check != MoveCheck::Impossible);
        drag.collapsed_hover = match (collapsed_group, drag.collapsed_hover) {
            (Some(group), Some((hovered, since))) if hovered == group => {
                if ui.time() - since >= EXPAND_DELAY_SECONDS {
                    self.actions.push(WindowAction::SetExpanded(group, true));
                }
                Some((hovered, since))
            }
            (Some(group), _) => Some((group, ui.time())),
            (None, _) => None,
        };

        if drag.check == MoveCheck::Changes
            && let Some(target) = drag.target
        {
            draw_drop_indicator(ui, theme, target.indicator);
        }
        scroll_near_edges(ui, theme);
    }

    /// Ends a press or drag on the note list when the left button is
    /// released: a press without drag selects the row, a drag moves the
    /// node to its drop target (M106–M111). While dragging, the title floats
    /// next to the cursor with a gold border, red with a ban icon over an
    /// impossible target (M109, M116).
    fn finish_drag(&mut self, ui: &Ui, theme: &Theme) {
        let Some(drag) = &mut self.drag else {
            return;
        };
        if !ui.is_mouse_down(MouseButton::Left) {
            if !drag.is_dragging {
                self.actions.push(WindowAction::Select(drag.node));
            } else if drag.check == MoveCheck::Changes
                && let Some(target) = drag.target
            {
                self.actions
                    .push(WindowAction::Move(drag.node, target.point));
            }
            self.drag = None;
            return;
        }
        if !drag.is_dragging {
            return;
        }
        let title = find_title(&self.document.root, drag.node).unwrap_or_default();
        let is_impossible = drag.target.is_some() && drag.check == MoveCheck::Impossible;
        // The list may not be drawn next frame (drawer closed); its target
        // is only valid for this one.
        drag.target = None;
        drag.check = MoveCheck::Unchanged;
        // A gold border ties the label to the drop line; red over an
        // impossible target (M116).
        let border = if is_impossible {
            theme.status.danger
        } else {
            theme.accent.text
        };
        let _border = ui.push_style_color(StyleColor::Border, border.to_array());
        ui.tooltip(|| {
            let _danger = is_impossible
                .then(|| ui.push_style_color(StyleColor::Text, theme.status.danger.to_array()));
            if is_impossible {
                let _icons = FontScope::push(ui, FontRole::Icons);
                let mut buffer = [0; 4];
                ui.text(Icon::BAN.as_str(&mut buffer));
                ui.same_line_with_spacing(0.0, theme.spacing.small);
            }
            ui.text(title);
        });
    }

    /// Draws the note pane: the note toolbar with the menu button (M48),
    /// then the note in its own scroll region so the toolbar stays in place.
    /// `menu_tooltip` names what the menu button does.
    fn draw_note_pane(&mut self, ui: &Ui, language: Language, theme: &Theme, menu_tooltip: Text) {
        if draw_icon_button(
            ui,
            theme,
            note_list_icon(menu_tooltip),
            menu_tooltip.get(language),
        ) {
            self.actions.push(WindowAction::ToggleNoteList);
        }
        // One toolbar for the note (M94, M102): the formatting groups while
        // editing and the edit or Cancel/Save buttons at the end; it wraps in
        // narrow panes (`lay-responsive`).
        if let Some(editor) = self.editor.as_mut() {
            let active = editor.active_formats();
            draw_format_toolbar(
                ui,
                language,
                theme,
                ToolbarState {
                    active,
                    preview_visible: self.ui_state.preview_visible,
                },
                &mut self.actions,
            );
        }
        self.draw_note_toolbar_end(ui, language, theme);
        draw_separator(ui, theme);
        if self.editor.is_some() {
            self.draw_editor(ui, language, theme);
        } else {
            self.draw_note_body(ui, language, theme);
        }
    }

    /// Draws the right end of the note toolbar (M51): the edit button while
    /// viewing, "Cancel" and the primary "Save" while editing.
    fn draw_note_toolbar_end(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let spacing = ui.clone_style().item_spacing[0];
        if self.editor.is_some() {
            let cancel = Text::Cancel.get(language);
            let save = Text::Save.get(language);
            let width = button_width(ui, cancel) + spacing + button_width(ui, save);
            move_to_toolbar_end(ui, width);
            if draw_button(ui, cancel) {
                self.actions.push(WindowAction::CancelEditing);
            }
            ui.same_line();
            if draw_primary_button(ui, theme, save) {
                self.actions.push(WindowAction::FinishEditing);
            }
            if ui.is_item_hovered() {
                ui.tooltip_text(Text::SaveTooltip.get(language));
            }
        } else if self.selected_note().is_some() {
            // Editing is the main action of the reading view: the one brass
            // button of the view (R71).
            let edit = Text::EditNote.get(language);
            move_to_toolbar_end(ui, icon_label_button_width(ui, theme, edit));
            if draw_primary_button_with_icon(ui, theme, Icon::PENCIL, edit) {
                self.actions.push(WindowAction::StartEditing);
            }
        }
    }

    /// Draws the editor (M51–M53): formatting toolbar, title input and the
    /// markdown body input filling the rest of the pane.
    fn draw_editor(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        // While the unsaved changes dialog is shown, typing must not change
        // what the dialog asks about.
        let is_dialog_open = self.leave_prompt.is_some();
        let Some(editor) = self.editor.as_mut() else {
            return;
        };
        // ImGui reverts an input to its content at activation when Escape is
        // pressed; keep the typed text instead (only copied on that frame).
        let escape_pressed = ui.is_key_pressed(Key::Escape);

        let title_backup =
            (escape_pressed && editor.title_was_active).then(|| editor.session.title.clone());
        {
            let _heading = FontScope::push(ui, FontRole::Heading1);
            ui.set_next_item_width(-f32::MIN_POSITIVE);
            ui.input_text("##title", &mut editor.session.title)
                .hint(Text::TitleHint.get(language))
                .read_only(is_dialog_open)
                .build();
        }
        editor.title_was_active = ui.is_item_active();
        if ui.is_item_deactivated()
            && let Some(title) = title_backup
        {
            editor.session.title = title;
        }

        if is_dialog_open {
            editor.body.unfocus();
        }
        // Ctrl+S works like "Save" from title and text (M93).
        if !is_dialog_open
            && (editor.body.is_focused() || editor.title_was_active)
            && ui.io().key_ctrl
            && ui.is_key_index_pressed(VIRTUAL_KEY_S)
        {
            self.actions.push(WindowAction::FinishEditing);
        }
        if editor.body.is_focused() && ui.io().key_ctrl {
            if ui.is_key_index_pressed(VIRTUAL_KEY_B) {
                editor.format(FormatCommand::Bold);
            } else if ui.is_key_index_pressed(VIRTUAL_KEY_I) {
                editor.format(FormatCommand::Italic);
            }
        }
        // Preview (M60, M72): editor and preview always stay side by side,
        // however narrow; the minimum only keeps the splitter usable.
        let available = ui.content_region_avail()[0];
        let gap = theme.pane.gap;
        let usable = (available - gap).max(0.0);
        let preview_minimum = (ui.calc_text_size("n")[0] * MINIMUM_PREVIEW_SPLIT_CHARACTERS
            + 2.0 * theme.pane.padding)
            .min(usable / 2.0);
        let editor_width = (usable * self.ui_state.clamped_preview_split_fraction())
            .clamp(preview_minimum, usable - preview_minimum);
        let width = if self.ui_state.preview_visible {
            editor_width
        } else {
            0.0
        };
        let chip_texts = |text| chip_text(text, language);
        let chips = ChipContext {
            text: &chip_texts,
            copied: self
                .copied_chat_link
                .as_ref()
                .map(|copied| copied.code.as_str()),
        };
        let response = editor.body.draw(ui, theme, "##body", [width, 0.0], chips);
        if response.changed {
            editor.sync_body();
        }
        if response.submitted {
            self.actions.push(WindowAction::FinishEditing);
        }
        if self.ui_state.preview_visible {
            ui.same_line_with_spacing(0.0, 0.0);
            let splitter = draw_vertical_splitter(
                ui,
                theme,
                "preview_splitter",
                editor_width,
                (preview_minimum, usable - preview_minimum),
                ui.content_region_avail()[1],
            );
            if let Some(width) = splitter.left_width {
                self.ui_state.preview_split_fraction = width / usable;
            }
            if splitter.released {
                self.saver.save_ui_state(self.ui_state);
            }
            ui.same_line_with_spacing(0.0, 0.0);
            let preview = editor
                .preview
                .get_or_insert_with(|| markdown::parse(&editor.session.body));
            let actions = &mut self.actions;
            let search_query = self.search.current.as_ref().map(|current| &current.query);
            draw_pane(ui, theme, "preview", [0.0, 0.0], || {
                // The preview is read-only: links open and chips copy, but
                // checkboxes do not toggle. It marks search hits (M122).
                let mut highlight = search_query.map(SearchHighlight::new);
                match draw_markdown(ui, preview, theme, chips, highlight.as_mut()) {
                    Some(
                        action @ (MarkdownAction::OpenLink(_) | MarkdownAction::ChatLinkCopied(_)),
                    ) => {
                        actions.push(WindowAction::Markdown(action));
                    }
                    Some(MarkdownAction::ToggleTask(_)) | None => {}
                }
            });
        }
    }

    /// Draws the selected note: its title and rendered content.
    fn draw_note_body(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        // Borrow only the document, so the markdown cache can be updated below.
        let Some(note) = selected_note_in(&self.document, self.ui_state.selected) else {
            ui.text_colored(
                theme.text.secondary.to_array(),
                Text::SelectNote.get(language),
            );
            return;
        };
        let note_id = note.id;
        let is_stale = self
            .parsed
            .as_ref()
            .is_none_or(|parsed| parsed.note != note_id || parsed.source != note.body);
        if is_stale {
            self.parsed = Some(ParsedNote {
                note: note_id,
                source: note.body.clone(),
                document: markdown::parse(&note.body),
            });
        }

        if let Some(draft) = self.drafts.get(note_id) {
            draw_draft_hint(
                ui,
                language,
                theme,
                draft,
                note.updated_at,
                &mut self.draft_hint,
                &mut self.actions,
            );
        }

        draw_title_with_kind(
            ui,
            language,
            theme,
            &note.title,
            (note.id, note.kind),
            &mut self.actions,
        );
        draw_separator(ui, theme);
        if ui.is_window_hovered() && ui.is_mouse_double_clicked(MouseButton::Left) {
            self.actions.push(WindowAction::StartEditing);
        }
        // Title and separator stay at the top; only the content scrolls, so
        // its scrollbar starts below the separator (M81).
        ChildWindow::new("note_body")
            .size([0.0, 0.0])
            .draw_background(false)
            .build(ui, || self.draw_note_content(ui, language, theme));
    }

    /// Draws the rendered content of the selected note in the scrolling
    /// region below its title.
    fn draw_note_content(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        if ui.is_window_hovered() && ui.is_mouse_double_clicked(MouseButton::Left) {
            self.actions.push(WindowAction::StartEditing);
        }
        if self.restore_scroll {
            ui.set_scroll_y(self.ui_state.note_scroll);
            self.restore_scroll = false;
        }
        match &self.parsed {
            Some(parsed) if !parsed.document.blocks.is_empty() => {
                let chip_texts = |text| chip_text(text, language);
                let chips = ChipContext {
                    text: &chip_texts,
                    copied: self
                        .copied_chat_link
                        .as_ref()
                        .map(|copied| copied.code.as_str()),
                };
                // Search hits are marked; a new search or a newly opened
                // note jumps to the first one (M122).
                let mut highlight = self
                    .search
                    .current
                    .as_ref()
                    .map(|current| SearchHighlight::new(&current.query));
                if let Some(action) =
                    draw_markdown(ui, &parsed.document, theme, chips, highlight.as_mut())
                {
                    self.actions.push(WindowAction::Markdown(action));
                }
                if std::mem::take(&mut self.scroll_to_first_hit)
                    && let Some(first_hit) =
                        highlight.as_ref().and_then(SearchHighlight::first_hit_y)
                {
                    ui.set_scroll_y((first_hit - ui.text_line_height()).max(0.0));
                }
            }
            _ => ui.text_colored(
                theme.text.secondary.to_array(),
                Text::EmptyNote.get(language),
            ),
        }
        let scroll = ui.scroll_y();
        if (scroll - self.ui_state.note_scroll).abs() >= 1.0 {
            self.ui_state.note_scroll = scroll;
            self.saver.save_ui_state(self.ui_state);
        }
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

/// Where the dialog card was drawn and what the user chose.
struct DialogCard {
    /// Top-left corner (screen).
    min: [f32; 2],
    /// Bottom-right corner (screen).
    max: [f32; 2],
    /// The measured height, for centering in the next frame.
    height: f32,
    /// The chosen answer, if any.
    answer: Option<WindowAction>,
}

/// Minimum content width of a dialog card in average characters.
const DIALOG_MINIMUM_CHARACTERS: f32 = 28.0;

/// Largest share of the window a dialog card may take.
const DIALOG_MAXIMUM_FRACTION: f32 = 0.9;

/// Draws the card of the unsaved changes dialog centered in the current
/// window (M54, M56, M70, `overlay-dialog-layout`): `bg.panel` with a
/// `border` frame and `space.lg` padding; heading, wrapped question, then
/// "Discard" on the left and "Keep editing" plus the primary "Save" on the
/// right, stacked full width (Save, Keep editing, Discard) when they do not
/// fit. `last_height` is the card height measured in the previous frame.
/// Enter saves, Escape keeps editing.
fn draw_unsaved_changes_card(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    last_height: Option<f32>,
) -> DialogCard {
    let title = Text::UnsavedChangesTitle.get(language);
    let question = Text::UnsavedChangesQuestion.get(language);
    let discard = Text::DiscardChanges.get(language);
    let keep_editing = Text::KeepEditing.get(language);
    let save = Text::KeepChanges.get(language);
    let spacing = ui.clone_style().item_spacing;
    let padding = theme.spacing.large;
    let window_position = ui.window_pos();
    let window_size = ui.window_size();
    let row_width = button_width(ui, discard)
        + theme.spacing.extra_large
        + button_width(ui, keep_editing)
        + spacing[0]
        + button_width(ui, save);
    let maximum_content = (window_size[0] * DIALOG_MAXIMUM_FRACTION - 2.0 * padding).max(1.0);
    let content_width = row_width
        .max(ui.calc_text_size(question)[0])
        .max(ui.calc_text_size("n")[0] * DIALOG_MINIMUM_CHARACTERS)
        .min(maximum_content);
    let stacked = row_width > content_width;
    let width = content_width + 2.0 * padding;
    let height = last_height.unwrap_or(window_size[1] / 3.0);
    let min = [
        window_position[0] + ((window_size[0] - width) / 2.0).max(0.0),
        window_position[1] + ((window_size[1] - height) / 2.0).max(0.0),
    ];
    let max = [min[0] + width, min[1] + height];
    let draw_list = ui.get_window_draw_list();
    draw_list
        .add_rect(min, max, theme.surface.panel.to_array())
        .filled(true)
        .rounding(theme.radius.large)
        .build();
    draw_list
        .add_rect(min, max, theme.border.subtle.to_array())
        .thickness(theme.border.width.max(1.0))
        .rounding(theme.radius.large)
        .build();
    let left = min[0] + padding;
    ui.set_cursor_screen_pos([left, min[1] + padding]);
    {
        let _heading = FontScope::push(ui, FontRole::Heading3);
        ui.text(title);
    }
    {
        // Every new line starts at the window padding; move into the card.
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        let _wrap = ui.push_text_wrap_pos_with_pos(left - window_position[0] + content_width);
        ui.text_wrapped(question);
    }
    ui.dummy([0.0, theme.spacing.small]);
    let mut answer = None;
    if stacked {
        let full = [content_width, 0.0];
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        if draw_primary_button_with_size(ui, theme, save, full) {
            answer = Some(WindowAction::LeaveSaving);
        }
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        if draw_button_with_size(ui, keep_editing, full) {
            answer = Some(WindowAction::LeaveCancelled);
        }
        ui.dummy([0.0, theme.spacing.small]);
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        if draw_button_with_size(ui, discard, full) {
            answer = Some(WindowAction::LeaveDiscarding);
        }
    } else {
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        if draw_button(ui, discard) {
            answer = Some(WindowAction::LeaveDiscarding);
        }
        let right_group = button_width(ui, keep_editing) + spacing[0] + button_width(ui, save);
        ui.same_line_with_pos(left + content_width - right_group - window_position[0]);
        if draw_button(ui, keep_editing) {
            answer = Some(WindowAction::LeaveCancelled);
        }
        ui.same_line();
        if draw_primary_button(ui, theme, save) {
            answer = Some(WindowAction::LeaveSaving);
        }
    }
    if ui.is_key_pressed(Key::Escape) {
        answer = Some(WindowAction::LeaveCancelled);
    } else if ui.is_key_pressed(Key::Enter) {
        answer = Some(WindowAction::LeaveSaving);
    }
    // The last item adds item spacing below itself; the card ends there plus padding.
    let measured = ui.cursor_screen_pos()[1] - spacing[1] - min[1] + padding;
    DialogCard {
        min,
        max,
        height: measured,
        answer,
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

/// What the formatting toolbar shows as active.
#[derive(Debug, Clone, Copy)]
struct ToolbarState {
    /// Formatting at the cursor (M69).
    active: ActiveFormats,
    /// Whether the preview is shown (M60).
    preview_visible: bool,
}

/// The sidebar icon of the note list toggle (M101): the icon itself shows
/// what a click does, so the button needs no accent "on" state (replaces
/// the gold menu button of M71).
fn note_list_icon(tooltip: Text) -> Icon {
    if tooltip == Text::HideNoteList {
        Icon::PANEL_LEFT_CLOSE
    } else {
        Icon::PANEL_LEFT_OPEN
    }
}

/// One formatting button: icon, tooltip and command (`None` = disabled).
type FormatButton = (Icon, Text, Option<FormatCommand>);

/// The formatting buttons in groups, like Word's ribbon (M53, M100, M102):
/// headings, emphasis, lists, inserts. The event insert is disabled until
/// trailkit-core exists.
const FORMAT_GROUPS: [&[FormatButton]; 4] = [
    &[
        (
            Icon::HEADING_1,
            Text::FormatHeading1,
            Some(FormatCommand::Heading1),
        ),
        (
            Icon::HEADING_2,
            Text::FormatHeading2,
            Some(FormatCommand::Heading2),
        ),
    ],
    &[
        (Icon::BOLD, Text::FormatBold, Some(FormatCommand::Bold)),
        (
            Icon::ITALIC,
            Text::FormatItalic,
            Some(FormatCommand::Italic),
        ),
    ],
    &[
        (
            Icon::LIST,
            Text::FormatBulletList,
            Some(FormatCommand::BulletList),
        ),
        (
            Icon::LIST_TODO,
            Text::FormatCheckbox,
            Some(FormatCommand::Checkbox),
        ),
    ],
    &[
        (Icon::LINK, Text::FormatLink, Some(FormatCommand::Link)),
        (Icon::CALENDAR_CLOCK, Text::InsertEventRequiresCore, None),
    ],
];

/// The width of an icon button showing `icon`.
fn icon_button_width(ui: &Ui, icon: Icon) -> f32 {
    let _icons = FontScope::push(ui, FontRole::Icons);
    let mut buffer = [0; 4];
    button_width(ui, icon.as_str(&mut buffer))
}

/// Starts a toolbar group of `width` on the current line, after a thin
/// vertical separator (M102); in a narrow pane the group moves to a new line
/// instead, without a separator (`lay-responsive`).
fn begin_toolbar_group(ui: &Ui, theme: &Theme, width: f32) {
    let gap = theme.spacing.small;
    let height = ui.item_rect_size()[1];
    let right_edge = ui.window_content_region_max()[0] + ui.window_pos()[0];
    ui.same_line_with_spacing(0.0, gap);
    let [x, y] = ui.cursor_screen_pos();
    if x + 1.0 + gap + width > right_edge {
        ui.new_line();
        return;
    }
    let inset = (height * 0.2).round();
    ui.get_window_draw_list()
        .add_line(
            [x + 0.5, y + inset],
            [x + 0.5, y + height - inset],
            theme.border.color.to_array(),
        )
        .thickness(1.0)
        .build();
    ui.dummy([1.0, height]);
    ui.same_line_with_spacing(0.0, gap);
}

/// Draws the formatting buttons of the editor in groups with separators
/// (M53, M102), then the preview toggle. Buttons of the formatting at the
/// cursor are drawn in the accent color (M69); the disabled event button in
/// `text.disabled`.
fn draw_format_toolbar(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    state: ToolbarState,
    actions: &mut Vec<WindowAction>,
) {
    let spacing = ui.clone_style().item_spacing[0];
    let mut id = 0;
    for group in FORMAT_GROUPS {
        let width = group
            .iter()
            .map(|(icon, _, _)| icon_button_width(ui, *icon))
            .sum::<f32>()
            + spacing * (group.len().saturating_sub(1)) as f32;
        begin_toolbar_group(ui, theme, width);
        for (index, (icon, tooltip, command)) in group.iter().enumerate() {
            if index > 0 {
                ui.same_line();
            }
            id += 1;
            let _id = ui.push_id(id);
            // ImGui 1.80 has no disabled items: a disabled button is drawn in
            // `text.disabled` and its clicks are ignored.
            let icon_color = match command {
                None => Some(theme.text.disabled),
                Some(command) if state.active.is_active(*command) => Some(theme.accent.text),
                Some(_) => None,
            };
            let _icon_color =
                icon_color.map(|color| ui.push_style_color(StyleColor::Text, color.to_array()));
            if draw_icon_button(ui, theme, *icon, tooltip.get(language))
                && let Some(command) = command
            {
                actions.push(WindowAction::Format(*command));
            }
        }
    }

    // The preview toggle is its own group; its icon is drawn in the accent
    // color while the preview is visible.
    begin_toolbar_group(ui, theme, icon_button_width(ui, Icon::EYE));
    let (tooltip, _selected) = if state.preview_visible {
        (
            Text::HidePreview,
            Some(ui.push_style_color(StyleColor::Text, theme.accent.text.to_array())),
        )
    } else {
        (Text::ShowPreview, None)
    };
    if draw_icon_button(ui, theme, Icon::EYE, tooltip.get(language)) {
        actions.push(WindowAction::TogglePreview);
    }
}

/// Draws the draft hint above a note (M54): when the draft was saved, whether
/// the note changed since, and "Restore" (primary) / "Discard".
fn draw_draft_hint(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    draft: &Draft,
    note_updated_at: Timestamp,
    cache: &mut Option<(NoteId, Language, String)>,
    actions: &mut Vec<WindowAction>,
) {
    let is_cached = cache.as_ref().is_some_and(|(note, cached_language, _)| {
        *note == draft.note && *cached_language == language
    });
    if !is_cached {
        let time = format_local_time(draft.saved_at, language);
        *cache = Some((
            draft.note,
            language,
            format!("{} {time}", Text::DraftFound.get(language)),
        ));
    }
    if let Some((_, _, label)) = cache.as_ref() {
        ui.text_colored(theme.status.warning.to_array(), label);
    }
    if draft.note_updated_at != note_updated_at {
        ui.text_colored(
            theme.text.secondary.to_array(),
            Text::DraftNoteChangedSince.get(language),
        );
    }
    if draw_primary_button(ui, theme, Text::RestoreDraft.get(language)) {
        actions.push(WindowAction::RestoreDraft(draft.note));
    }
    ui.same_line();
    if draw_button(ui, Text::DiscardChanges.get(language)) {
        actions.push(WindowAction::DiscardDraft(draft.note));
    }
    draw_separator(ui, theme);
}

/// Draws the "new note" / "new group" icon buttons, led by the menu button
/// when `menu` holds its tooltip.
fn draw_toolbar(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    menu: Option<Text>,
    actions: &mut Vec<WindowAction>,
) {
    if let Some(tooltip) = menu {
        if draw_icon_button(ui, theme, note_list_icon(tooltip), tooltip.get(language)) {
            actions.push(WindowAction::ToggleNoteList);
        }
        ui.same_line();
    }
    let buttons = [
        (Icon::FILE_PLUS, Text::NewNote, WindowAction::NewNote(None)),
        (
            Icon::FOLDER_PLUS,
            Text::NewGroup,
            WindowAction::NewGroup(None),
        ),
    ];
    // Both buttons share the row in equal widths, never narrower than an
    // icon button; without room for both they stack (M128).
    let spacing = ui.clone_style().item_spacing[0];
    let row_x = ui.cursor_pos()[0];
    let available = ui.content_region_avail()[0];
    let minimum = buttons
        .iter()
        .map(|(icon, _, _)| icon_button_width(ui, *icon))
        .fold(0.0, f32::max);
    let side_by_side = available >= 2.0 * minimum + spacing;
    let width = if side_by_side {
        (available - spacing) / 2.0
    } else {
        available.max(minimum)
    };
    for (index, (icon, tooltip, action)) in buttons.into_iter().enumerate() {
        if index > 0 {
            if side_by_side {
                ui.same_line();
            } else {
                ui.set_cursor_pos([row_x, ui.cursor_pos()[1]]);
            }
        }
        let _id = ui.push_id(i32::try_from(index).unwrap_or_default());
        let clicked = {
            let _icons = FontScope::push(ui, FontRole::Icons);
            let mut buffer = [0; 4];
            ui.button_with_size(icon.as_str(&mut buffer), [width, 0.0])
        };
        if ui.is_item_hovered() {
            ui.tooltip_text(tooltip.get(language));
        }
        if clicked {
            actions.push(action);
        }
    }
    draw_separator(ui, theme);
}

/// Draws `nodes` and, recursively, the children of expanded groups; during a
/// search only the rows it shows (M118).
fn draw_nodes(ui: &Ui, nodes: &[Node], context: &mut TreeContext<'_>) {
    for node in nodes {
        if let Some((_, results)) = context.search
            && !results.is_visible(node.id())
        {
            continue;
        }
        // The dragged row and its children are faded (M106).
        let _faded = (context.dragged == Some(node.id()))
            .then(|| ui.push_style_var(StyleVar::Alpha(DRAGGED_ROW_ALPHA)));
        match node {
            Node::Group(group) => draw_group(ui, group, context),
            Node::Note(note) => draw_note_row(ui, note, context),
        }
    }
}

/// Draws one group row and, if expanded, its children.
fn draw_group(ui: &Ui, group: &Group, context: &mut TreeContext<'_>) {
    let _id = ui.push_id(group.id.imgui_id());
    let node = NodeId::Group(group.id);
    let is_renaming = context
        .rename
        .as_ref()
        .is_some_and(|rename| rename.node == node);
    let mut flags = TreeNodeFlags::OPEN_ON_ARROW
        | TreeNodeFlags::SPAN_AVAIL_WIDTH
        | TreeNodeFlags::FRAME_PADDING;
    if context.selected == Some(node) {
        flags |= TreeNodeFlags::SELECTED;
    }

    // M50: groups stand out by a bold title and their note count, shown
    // right-aligned in the secondary text color like other row info (M49).
    let row_start = ui.cursor_screen_pos();
    let title_offset = group_title_offset(ui, context);
    // During a search the count is the matching notes (M121) and groups
    // are expanded unless collapsed for this search only (M124).
    let (note_count, expanded) = match context.search {
        Some((_, results)) => (
            results.group_count(group.id),
            !context.collapsed_in_search.contains(&group.id),
        ),
        None => (group.note_count(), group.expanded),
    };
    let mut count_buffer = [0; 20];
    let count = format_count(note_count, &mut count_buffer);
    let count_x = row_content_end(ui, context) - ui.calc_text_size(count)[0];
    let title_end = count_x - context.theme.spacing.small;
    let (label, token) = {
        let _bold = FontScope::push(ui, FontRole::BodyBold);
        let label = if is_renaming {
            Cow::Borrowed("")
        } else {
            truncate_to_width(ui, &group.title, title_end - (row_start[0] + title_offset))
        };
        // The node draws hover, selection and its triangle in the text
        // color; a transparent text color hides the triangle, and the
        // Lucide chevron and the title are drawn over it (R71).
        let token = {
            let _hidden_triangle = ui.push_style_color(StyleColor::Text, [0.0; 4]);
            TreeNode::new("group")
                .label::<&str, _>("")
                .flags(flags)
                .opened(expanded, Condition::Always)
                .push(ui)
        };
        if context.selected == Some(node) {
            draw_selection_bar(ui, context.theme);
        }
        draw_tree_arrow(
            ui,
            context.theme,
            token.is_some(),
            row_start,
            context.frame_padding,
        );
        let title_position = [
            row_start[0] + title_offset,
            row_start[1] + context.frame_padding[1],
        ];
        if let Some((query, _)) = context.search {
            draw_title_hits(ui, context.theme, query, &label, title_position);
        }
        ui.get_window_draw_list().add_text(
            [
                row_start[0] + title_offset,
                row_start[1] + context.frame_padding[1],
            ],
            context.theme.text.primary.to_array(),
            &*label,
        );
        (label, token)
    };
    let kind = if expanded {
        RowKind::ExpandedGroup(group.id)
    } else {
        RowKind::CollapsedGroup(group.id)
    };
    record_row(ui, node, kind, context);
    ui.get_window_draw_list().add_text(
        [count_x, row_start[1] + context.frame_padding[1]],
        context.theme.text.secondary.to_array(),
        count,
    );
    let is_open = token.is_some();
    handle_row_interaction(ui, node, &group.title, &label, context);
    if is_open != expanded {
        let action = if context.search.is_some() {
            WindowAction::SetCollapsedInSearch(group.id, !is_open)
        } else {
            WindowAction::SetExpanded(group.id, is_open)
        };
        context.actions.push(action);
    }
    if is_renaming {
        ui.same_line_with_spacing(0.0, 0.0);
        let _bold = FontScope::push(ui, FontRole::BodyBold);
        draw_rename_input(
            ui,
            row_start,
            [row_start[0] + title_offset, title_end],
            context,
        );
    }
    if let Some(_token) = token {
        draw_nodes(ui, &group.children, context);
    }
}

/// Horizontal offset of a group title from the row start: ImGui puts it
/// after the arrow (font size plus twice the horizontal frame padding).
fn group_title_offset(ui: &Ui, context: &TreeContext<'_>) -> f32 {
    ui.current_font_size() + 2.0 * context.frame_padding[0]
}

/// Screen x where row content (title, badges) ends: the row's right edge
/// minus the horizontal frame padding, mirroring the inset on the left.
fn row_content_end(ui: &Ui, context: &TreeContext<'_>) -> f32 {
    ui.window_content_region_max()[0] + ui.window_pos()[0] - context.frame_padding[0]
}

/// Draws one note row (M49): the title flush left, a reset badge at the
/// right edge.
fn draw_note_row(ui: &Ui, note: &Note, context: &mut TreeContext<'_>) {
    let _id = ui.push_id(note.id.imgui_id());
    let node = NodeId::Note(note.id);
    let mut flags = TreeNodeFlags::LEAF
        | TreeNodeFlags::NO_TREE_PUSH_ON_OPEN
        | TreeNodeFlags::SPAN_AVAIL_WIDTH
        | TreeNodeFlags::FRAME_PADDING;
    if context.selected == Some(node) {
        flags |= TreeNodeFlags::SELECTED;
    }
    // The node draws selection and hover; badge and title are drawn after it.
    let _token = TreeNode::new("note")
        .label::<&str, _>("")
        .flags(flags)
        .push(ui);
    if context.selected == Some(node) {
        draw_selection_bar(ui, context.theme);
    }
    let row_start = ui.item_rect_min();
    record_row(ui, node, RowKind::Note, context);
    // Ask the tree node itself: it spans the whole row, and once a click
    // makes it the active item, `is_window_hovered` would report false.
    let is_row_hovered = ui.is_item_hovered();
    let is_renaming = context
        .rename
        .as_ref()
        .is_some_and(|rename| rename.node == node);

    // Notes have no arrow: the title starts where a group's arrow does.
    let title_x = row_start[0] + context.frame_padding[0];
    let content_end = row_content_end(ui, context);
    let mut title_end = match draw_badge(ui, note.kind, [content_end, row_start[1]], context) {
        Some(badge_start) => badge_start - context.theme.spacing.small,
        None => content_end,
    };
    // A note matching through its content shows its hit count (M122).
    if let Some(hits) = context
        .search
        .and_then(|(_, results)| results.content_hits(note.id))
    {
        let mut count_buffer = [0; 20];
        let count = format_count(hits, &mut count_buffer);
        let count_x = title_end - ui.calc_text_size(count)[0];
        ui.get_window_draw_list().add_text(
            [count_x, row_start[1] + context.frame_padding[1]],
            context.theme.text.secondary.to_array(),
            count,
        );
        if is_row_hovered && ui.io().mouse_pos[0] >= count_x {
            ui.tooltip_text(
                Text::ContentHits
                    .get(context.language)
                    .replace("{count}", count),
            );
        }
        title_end = count_x - context.theme.spacing.small;
    }
    ui.same_line_with_spacing(0.0, 0.0);
    ui.set_cursor_screen_pos([title_x, row_start[1]]);
    if is_renaming {
        draw_rename_input(ui, row_start, [title_x, title_end], context);
        return;
    }
    let label = truncate_to_width(ui, &note.title, title_end - title_x);
    if let Some((query, _)) = context.search {
        draw_title_hits(
            ui,
            context.theme,
            query,
            &label,
            [title_x, row_start[1] + context.frame_padding[1]],
        );
    }
    ui.text(&label);
    // Interaction belongs to the whole row (the tree node item).
    if is_row_hovered {
        handle_hovered_row(ui, node, &note.title, &label, context);
    }
    draw_context_menu(ui, node, Some(note.kind), context);
}

/// Click, double-click, context menu and tooltip of the last drawn tree row.
fn handle_row_interaction(
    ui: &Ui,
    node: NodeId,
    title: &str,
    label: &str,
    context: &mut TreeContext<'_>,
) {
    if ui.is_item_hovered() {
        handle_hovered_row(ui, node, title, label, context);
    }
    draw_context_menu(ui, node, None, context);
}

/// Handles a hovered row: selection, rename on double-click, context menu,
/// and the full title as tooltip if it was truncated.
fn handle_hovered_row(
    ui: &Ui,
    node: NodeId,
    title: &str,
    label: &str,
    context: &mut TreeContext<'_>,
) {
    if label != title && !label.is_empty() {
        ui.tooltip_text(title);
    }
    if ui.is_mouse_double_clicked(MouseButton::Left) {
        context.actions.push(WindowAction::StartRename(node));
    } else if ui.is_mouse_clicked(MouseButton::Left) && !ui.is_item_toggled_open() {
        // Selects on release unless it becomes a drag (M106), so dragging a
        // note keeps the drawer open (M113).
        context.pressed = Some(node);
    }
    if ui.is_mouse_clicked(MouseButton::Right) {
        context.actions.push(WindowAction::Select(node));
        ui.open_popup("context");
    }
}

/// Draws the context menu of a row if it is open (must run every frame).
fn draw_context_menu(ui: &Ui, node: NodeId, kind: Option<NoteKind>, context: &mut TreeContext<'_>) {
    let (language, theme) = (context.language, context.theme);
    ui.popup("context", || {
        if MenuItem::new(Text::Rename.get(language)).build(ui) {
            context.actions.push(WindowAction::StartRename(node));
        }
        // Notes get the reset type submenu with the current kind checked
        // (M86); groups have no kind.
        if let (NodeId::Note(note), Some(kind)) = (node, kind)
            && let Some(_submenu) = begin_submenu(ui, theme, Text::ResetType.get(language))
        {
            for (period, text) in KIND_OPTIONS {
                let is_current = kind.period() == period;
                if draw_menu_item(ui, theme, text.get(language), is_current) && !is_current {
                    context
                        .actions
                        .push(WindowAction::SetResetPeriod(note, period));
                }
            }
        }
        if let Some(_submenu) = begin_submenu(ui, theme, Text::MoveTo.get(language)) {
            draw_move_targets(ui, node, context);
        }
        // Only groups offer creating, into themselves (M96, M97).
        if let NodeId::Group(group) = node {
            ui.separator();
            draw_create_menu_items(
                ui,
                language,
                InsertionPoint::End(Some(group)),
                context.actions,
            );
        }
    });
}

/// Draws the "Move to" submenu items (M112): "Top level" and every group as
/// an indented flat list; the current place is checked, the moved group
/// and its descendants are left out. A choice moves to the end.
fn draw_move_targets(ui: &Ui, node: NodeId, context: &mut TreeContext<'_>) {
    let (theme, language) = (context.theme, context.language);
    let current = context.document.parent(node).flatten();
    if draw_menu_item(ui, theme, Text::TopLevel.get(language), current.is_none())
        && current.is_some()
    {
        context
            .actions
            .push(WindowAction::Move(node, InsertionPoint::End(None)));
    }
    draw_move_target_groups(ui, &context.document.root, node, current, context);
}

/// Draws one "Move to" item per group in `nodes`, nested groups indented
/// below their parent (M112); skips the moved group `node` with its
/// descendants. `current` is the moved node's parent.
fn draw_move_target_groups(
    ui: &Ui,
    nodes: &[Node],
    node: NodeId,
    current: Option<GroupId>,
    context: &mut TreeContext<'_>,
) {
    let indent = context.theme.list.indent;
    for child in nodes {
        let Node::Group(group) = child else {
            continue;
        };
        if NodeId::Group(group.id) == node {
            continue;
        }
        let _id = ui.push_id(group.id.imgui_id());
        ui.indent_by(indent);
        let is_current = current == Some(group.id);
        if draw_menu_item(ui, context.theme, &group.title, is_current) && !is_current {
            context.actions.push(WindowAction::Move(
                node,
                InsertionPoint::End(Some(group.id)),
            ));
        }
        draw_move_target_groups(ui, &group.children, node, current, context);
        ui.unindent_by(indent);
    }
}

/// Records the rectangle of the row just drawn (the tree node item) for the
/// drop zones (M107).
fn record_row(ui: &Ui, node: NodeId, kind: RowKind, context: &mut TreeContext<'_>) {
    let [min, max] = [ui.item_rect_min(), ui.item_rect_max()];
    context.rows.push(RowRect {
        node,
        kind,
        left: min[0],
        right: max[0],
        top: min[1],
        bottom: max[1],
    });
}

/// Draws the drop line between rows or the highlight of a collapsed group
/// (trailkit-ui skill §4) into the list's window.
fn draw_drop_indicator(ui: &Ui, theme: &Theme, indicator: DropIndicator) {
    let draw_list = ui.get_window_draw_list();
    let color = theme.accent.text.to_array();
    let thickness = 2.0 * theme.border.width;
    match indicator {
        DropIndicator::Line { y, left, right } => {
            draw_list
                .add_line([left, y], [right, y], color)
                .thickness(thickness)
                .build();
        }
        DropIndicator::Highlight { min, max } => {
            draw_list
                .add_rect(min, max, color)
                .rounding(theme.radius.medium)
                .thickness(thickness)
                .build();
        }
    }
}

/// Scrolls the current window while the mouse is within one row height of
/// its top or bottom edge, faster closer to the edge (M111).
fn scroll_near_edges(ui: &Ui, theme: &Theme) {
    let band =
        ui.text_line_height() + 2.0 * ui.clone_style().frame_padding[1] + theme.list.item_spacing;
    let top = ui.window_pos()[1];
    let bottom = top + ui.window_size()[1];
    let mouse_y = ui.io().mouse_pos[1];
    let depth = if mouse_y < top + band {
        -(top + band - mouse_y) / band
    } else if mouse_y > bottom - band {
        (mouse_y - (bottom - band)) / band
    } else {
        return;
    };
    let step = depth.clamp(-1.0, 1.0) * DRAG_SCROLL_ROWS_PER_SECOND * band * ui.io().delta_time;
    ui.set_scroll_y((ui.scroll_y() + step).clamp(0.0, ui.scroll_max_y()));
}

/// Draws "New note" and "New group" menu items that create at `point`
/// (M96); the new entry is renamed right away (M43).
fn draw_create_menu_items(
    ui: &Ui,
    language: Language,
    point: InsertionPoint,
    actions: &mut Vec<WindowAction>,
) {
    if MenuItem::new(Text::NewNote.get(language)).build(ui) {
        actions.push(WindowAction::NewNote(Some(point)));
    }
    if MenuItem::new(Text::NewGroup.get(language)).build(ui) {
        actions.push(WindowAction::NewGroup(Some(point)));
    }
}

/// Continues the toolbar line at its right end for `width` of items,
/// wrapping onto a new line when they do not fit (`lay-responsive`).
fn move_to_toolbar_end(ui: &Ui, width: f32) {
    let right_edge = ui.window_content_region_max()[0] + ui.window_pos()[0];
    ui.same_line();
    if ui.cursor_screen_pos()[0] + width > right_edge {
        ui.new_line();
    }
    let y = ui.cursor_screen_pos()[1];
    ui.set_cursor_screen_pos([right_edge - width, y]);
}

/// The color of a reset kind (M104): badge and kind chip share it.
fn reset_color(theme: &Theme, period: ResetPeriod) -> Color {
    match period {
        ResetPeriod::Daily => theme.reset.daily,
        ResetPeriod::Weekly => theme.reset.weekly,
    }
}

/// Draws the note title with the kind dropdown right behind it (M102, the
/// one shared dropdown since R71), centered on the title's capitals; the
/// dropdown moves below a title that leaves no room.
/// Pushes a chosen kind as an action.
fn draw_title_with_kind(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    title: &str,
    (note, kind): (NoteId, NoteKind),
    actions: &mut Vec<WindowAction>,
) {
    let labels = KIND_OPTIONS.map(|(_, text)| text.get(language));
    let tint = kind.period().map(|period| reset_color(theme, period));
    let dropdown_size = [dropdown_width(ui, &labels), ui.frame_height()];
    let gap = theme.spacing.medium;
    let available = ui.content_region_avail()[0];
    let [left, top] = ui.cursor_screen_pos();
    let (title_height, title_middle, is_beside) = {
        let _heading = FontScope::push(ui, FontRole::Heading1);
        let height = ui.text_line_height();
        let middle = capital_middle().unwrap_or(height / 2.0);
        let is_beside = ui.calc_text_size(title)[0] + gap + dropdown_size[0] <= available;
        if is_beside {
            ui.text(title);
        } else {
            ui.text_wrapped(title);
        }
        (height, middle, is_beside)
    };
    if is_beside {
        ui.same_line_with_spacing(0.0, gap);
        let x = ui.cursor_screen_pos()[0];
        ui.set_cursor_screen_pos([x, (top + title_middle - dropdown_size[1] / 2.0).round()]);
    }
    let selected = KIND_OPTIONS
        .iter()
        .position(|(period, _)| *period == kind.period())
        .unwrap_or(0);
    if let Some(index) = draw_dropdown(
        ui,
        theme,
        tint,
        "kind",
        Text::ResetType.get(language),
        &labels,
        selected,
    ) && let Some((period, _)) = KIND_OPTIONS.get(index)
    {
        actions.push(WindowAction::SetResetPeriod(note, *period));
    }
    if is_beside {
        // Continue below the title, not below the centered dropdown.
        let spacing = ui.clone_style().item_spacing[1];
        ui.set_cursor_screen_pos([left, top + title_height + spacing]);
    }
}

/// Whole minutes from `now` to the next `period` reset, rounded up (M91).
fn minutes_until_next_reset(period: ResetPeriod, now: Timestamp) -> i64 {
    let seconds = period.next_reset(now).as_second() - now.as_second();
    (seconds + 59) / 60
}

/// Draws the inline rename input and records commit or cancel (M44).
///
/// Inline editing in lists (`lay-inline-edit`): the input has no background
/// of its own, so the row keeps one uniform color; a focus border frames the
/// whole row starting at `row_start`. The input text spans `title_span`
/// (start and end x), where the title was drawn before renaming, so badges
/// next to it stay visible.
fn draw_rename_input(
    ui: &Ui,
    row_start: [f32; 2],
    title_span: [f32; 2],
    context: &mut TreeContext<'_>,
) {
    let [title_x, title_end] = title_span;
    let Some(rename) = context.rename.as_mut() else {
        return;
    };
    let theme = context.theme;
    let row_end = ui.window_content_region_max()[0] + ui.window_pos()[0];
    ui.get_window_draw_list()
        .add_rect(
            row_start,
            [row_end, row_start[1] + ui.frame_height()],
            theme.border.focus.to_array(),
        )
        .thickness(theme.border.width.max(1.0))
        .rounding(theme.radius.medium)
        .build();

    let padding_x = theme.spacing.extra_small;
    let _padding = ui.push_style_var(StyleVar::FramePadding([
        padding_x,
        context.frame_padding[1],
    ]));
    let transparent = [0.0; 4];
    let _background = [
        ui.push_style_color(StyleColor::FrameBg, transparent),
        ui.push_style_color(StyleColor::FrameBgHovered, transparent),
        ui.push_style_color(StyleColor::FrameBgActive, transparent),
    ];
    ui.set_cursor_screen_pos([title_x - padding_x, row_start[1]]);
    if !rename.focused {
        ui.set_keyboard_focus_here();
        rename.focused = true;
    }
    ui.set_next_item_width((title_end - title_x + 2.0 * padding_x).max(1.0));
    let entered = ui
        .input_text("##rename", &mut rename.buffer)
        .auto_select_all(true)
        .enter_returns_true(true)
        .build();
    if entered {
        context.actions.push(WindowAction::CommitRename);
    } else if ui.is_item_deactivated() {
        let action = if ui.is_key_pressed(Key::Escape) {
            WindowAction::CancelRename
        } else {
            WindowAction::CommitRename
        };
        context.actions.push(action);
    }
}

/// Draws the reset badge of a note row (M45, M49): an outlined rounded box
/// with a letter for reset notes, nothing for normal notes. The badge ends at
/// `anchor[0]` and is vertically centered in the row starting at `anchor[1]`
/// (one frame tall). Returns the badge's left edge, or `None` if no badge
/// was drawn.
fn draw_badge(ui: &Ui, kind: NoteKind, anchor: [f32; 2], context: &TreeContext<'_>) -> Option<f32> {
    let (language, theme) = (context.language, context.theme);
    let (letter, tooltip) = match kind {
        NoteKind::Normal => return None,
        NoteKind::DailyReset { .. } => (Text::DailyBadge, Text::KindDaily),
        NoteKind::WeeklyReset { .. } => (Text::WeeklyBadge, Text::KindWeekly),
    };
    let line_height = ui.text_line_height();
    let padding = theme.spacing.extra_small;
    // Whole pixels, so the outline stays sharp and the letter can be
    // centered exactly.
    let width = (ui.calc_text_size("W")[0] + 2.0 * padding).round();
    let start = [
        (anchor[0] - width).round(),
        (anchor[1] + ((ui.frame_height() - line_height) / 2.0).max(0.0)).round(),
    ];
    let height = line_height.round();
    let letter = letter.get(language);
    let color = kind
        .period()
        .map_or(theme.accent.text, |period| reset_color(theme, period));
    // Lucide style (`lay-icons`): round corners, stroke ≈ 2/24 of the size;
    // a faint fill of the kind color makes it readable on dark panes (M105).
    // Whole pixels, so the outline stays sharp instead of a soft band.
    let stroke = (line_height / 12.0).round().max(1.0);
    let max = [start[0] + width, start[1] + height];
    let draw_list = ui.get_window_draw_list();
    draw_list
        .add_rect(start, max, color.with_alpha(0.16).to_array())
        .rounding(line_height * 0.25)
        .filled(true)
        .build();
    draw_list
        .add_rect(start, max, color.to_array())
        .rounding(line_height * 0.25)
        .thickness(stroke)
        .build();
    // A bold letter, centered by its own ink box, not by its advance width
    // or the line box (the text font sits low in it, M85, M105).
    {
        let _bold = FontScope::push(ui, FontRole::BodyBold);
        let letter_size = ui.calc_text_size(letter);
        let ink = letter.chars().next().and_then(glyph_ink_box).unwrap_or([
            0.0,
            0.0,
            letter_size[0],
            letter_size[1],
        ]);
        let center = [start[0] + width / 2.0, start[1] + height / 2.0];
        draw_list.add_text(
            [
                (center[0] - (ink[0] + ink[2]) / 2.0).round(),
                (center[1] - (ink[1] + ink[3]) / 2.0).round(),
            ],
            color.to_array(),
            letter,
        );
    }
    if ui.is_mouse_hovering_rect(start, max) {
        // The same kind and countdown as the note toolbar (M95), formatted
        // only while hovered.
        let countdown = kind.period().map(|period| {
            reset_countdown(minutes_until_next_reset(period, Timestamp::now()), language)
        });
        ui.tooltip(|| {
            ui.text(tooltip.get(language));
            if let Some(countdown) = &countdown {
                ui.text(countdown);
            }
        });
    }
    Some(start[0])
}

/// Marks the search hits in a row title drawn at `position` with the
/// `accent.highlight` background (M122); call before drawing the text.
fn draw_title_hits(ui: &Ui, theme: &Theme, query: &SearchQuery, title: &str, position: [f32; 2]) {
    for range in query.hit_ranges(title) {
        let left = position[0] + ui.calc_text_size(&title[..range.start])[0];
        let right = position[0] + ui.calc_text_size(&title[..range.end])[0];
        ui.get_window_draw_list()
            .add_rect(
                [left, position[1]],
                [right, position[1] + ui.text_line_height()],
                theme.accent.highlight.to_array(),
            )
            .filled(true)
            .rounding(theme.radius.small)
            .build();
    }
}

/// The empty search result (M126): what was searched for and a button that
/// clears the search.
fn draw_no_search_results(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    text: &str,
    actions: &mut Vec<WindowAction>,
) {
    let width = ui.content_region_avail()[0];
    ui.dummy([0.0, theme.spacing.medium]);
    let message = Text::NoSearchResults
        .get(language)
        .replace("{query}", text.trim());
    {
        let _secondary = ui.push_style_color(StyleColor::Text, theme.text.secondary.to_array());
        centered_text(ui, &message, width);
    }
    ui.dummy([0.0, theme.spacing.small]);
    let label = Text::ClearSearch.get(language);
    indent_to_center(ui, button_width(ui, label), width);
    if draw_button(ui, label) {
        actions.push(WindowAction::ClearSearch);
    }
}

/// Draws the empty state (M42): headline, primary and secondary button.
fn draw_empty_state(ui: &Ui, language: Language, theme: &Theme, actions: &mut Vec<WindowAction>) {
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
    let padding = ui.clone_style().frame_padding[0] * 2.0;
    let primary_width = ui.calc_text_size(primary)[0] + padding;
    let secondary_width = ui.calc_text_size(secondary)[0] + padding;
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
    if draw_button(ui, secondary) {
        actions.push(WindowAction::NewGroup(None));
    }
}

/// Draws `text` horizontally centered in a region of `width`, wrapping if needed.
fn centered_text(ui: &Ui, text: &str, width: f32) {
    let text_width = ui.calc_text_size(text)[0];
    if text_width <= width {
        indent_to_center(ui, text_width, width);
        ui.text(text);
    } else {
        ui.text_wrapped(text);
    }
}

/// Moves the cursor so an item of `item_width` is centered in `width`.
fn indent_to_center(ui: &Ui, item_width: f32, width: f32) {
    let position = ui.cursor_pos();
    ui.set_cursor_pos([
        position[0] + ((width - item_width) / 2.0).max(0.0),
        position[1],
    ]);
}

/// Formats `count` into `buffer` without allocating (drawn every frame).
fn format_count(count: usize, buffer: &mut [u8; 20]) -> &str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buffer[..]);
    // 20 bytes hold every `usize`, so the write cannot fail.
    let written = match write!(cursor, "{count}") {
        Ok(()) => usize::try_from(cursor.position()).unwrap_or(0),
        Err(_) => 0,
    };
    std::str::from_utf8(&buffer[..written]).unwrap_or_default()
}

/// Shortens `text` with "…" so it fits into `width` (`lay-truncation`).
/// Allocates only when shortening is needed.
fn truncate_to_width<'text>(ui: &Ui, text: &'text str, width: f32) -> Cow<'text, str> {
    if ui.calc_text_size(text)[0] <= width {
        return Cow::Borrowed(text);
    }
    let ellipsis = "…";
    let available = width - ui.calc_text_size(ellipsis)[0];
    let mut end = 0;
    for (index, character) in text.char_indices() {
        let next = index + character.len_utf8();
        if ui.calc_text_size(&text[..next])[0] > available {
            break;
        }
        end = next;
    }
    Cow::Owned(format!("{}{ellipsis}", text[..end].trim_end()))
}

#[cfg(test)]
#[path = "main_window_tests.rs"]
mod tests;
