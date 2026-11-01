//! The memo main window: note tree, note view, editor, empty state and
//! storage hints (Windows only; memo plan M21, M31, M34, M37–M55).
//!
//! The window owns the note tree (M30). Drawing collects user actions into a
//! reused list; they are applied after the tree has been drawn, then changed
//! documents are handed to the storage worker.

use std::{borrow::Cow, path::PathBuf, sync::Arc};

use jiff::{Timestamp, tz::TimeZone};
use nexus::imgui::{
    ChildWindow, Condition, Key, MenuItem, MouseButton, StyleColor, StyleVar, TreeNode,
    TreeNodeFlags, Ui, Window, WindowFlags, WindowFocusedFlags,
};
use trailkit_ui::{
    button::{button_width, draw_primary_button, draw_primary_button_with_size},
    editor::view::TextEditor,
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    markdown::{self, MarkdownDocument},
    markdown_view::{MarkdownAction, draw_markdown, open_web_link},
    pane::draw_pane,
    separator::draw_separator,
    splitter::draw_vertical_splitter,
    theme::Theme,
};

use crate::{
    drafts::{Draft, DraftsDocument},
    i18n::{Language, Text},
    note_editing::{ActiveFormats, EditSession, FormatCommand, active_formats, format_edit},
    note_tree::{Group, GroupId, NodeId, Note, NoteId, NoteKind, NotesDocument, TreeNode as Node},
    settings::UnsavedChanges,
    storage_worker::StorageSaver,
    ui_state::UiState,
};

/// Minimum content width of each pane in average characters (M40).
const MINIMUM_PANE_CHARACTERS: f32 = 22.0;

/// Largest share of the window the note list drawer may cover, so a strip
/// of the dimmed note stays visible to click away (M48, `lay-drawer`).
const DRAWER_MAXIMUM_FRACTION: f32 = 0.85;

/// Windows virtual-key code of `B` (Nexus feeds ImGui with virtual-key codes).
const VIRTUAL_KEY_B: i32 = 0x42;

/// Windows virtual-key code of `I`.
const VIRTUAL_KEY_I: i32 = 0x49;

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
    /// Create a note.
    NewNote,
    /// Create a group.
    NewGroup,
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
    /// Commit the edit ("Done", Ctrl+Enter).
    FinishEditing,
    /// Leave the editor without saving; asks first if something changed.
    CancelEditing,
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
}

impl WindowAction {
    /// Whether this action leaves the editor of `note` (M54): choosing
    /// another entry or creating one. These ask first if something changed.
    fn leaves_editor_of(&self, note: NoteId) -> bool {
        match self {
            Self::Select(node) => *node != NodeId::Note(note),
            Self::NewNote | Self::NewGroup => true,
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
    /// Whether leaving the editor with changes asks or saves (M57).
    unsaved_changes: UnsavedChanges,
    /// Height of the dialog card measured in the last frame (its content
    /// wraps, so the height is only known after drawing it once).
    dialog_height: Option<f32>,
    /// Set once closing the window is allowed; read by the addon glue.
    close_approved: bool,
    /// Cached draft hint label (note, language, text), so it is not
    /// formatted every frame.
    draft_hint: Option<(NoteId, Language, String)>,
}

/// Everything the tree drawing functions need besides the nodes.
struct TreeContext<'state> {
    /// The selected note or group.
    selected: Option<NodeId>,
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
        unsaved_changes: UnsavedChanges,
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
            is_narrow: false,
            restore_scroll: true,
            actions: Vec::new(),
            drafts,
            editor: None,
            leave_prompt: None,
            window_rect: None,
            window_focused: false,
            unsaved_changes,
            close_approved: false,
            draft_hint: None,
            dialog_height: None,
        }
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
        // Escape (sent by Nexus as a close request) while the dialog is shown
        // picks the harmless answer (`overlay-dialog-layout`).
        if self.leave_prompt.is_some() {
            self.leave_prompt = None;
            return false;
        }
        if self.has_unsaved_changes() && self.unsaved_changes == UnsavedChanges::SaveAutomatically {
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
        self.actions.clear();
        self.window_rect = Some([ui.window_pos(), ui.window_size()]);
        self.window_focused =
            ui.is_window_focused_with_flags(WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS);
        self.draw_problem(ui, language, theme);

        if self.document.root.is_empty() {
            draw_empty_state(ui, language, theme, &mut self.actions);
        } else {
            let pane_minimum =
                ui.calc_text_size("n")[0] * MINIMUM_PANE_CHARACTERS + 2.0 * theme.pane.padding;
            let available = ui.content_region_avail();
            self.is_narrow = available[0] < 2.0 * pane_minimum + theme.pane.gap;
            if !self.is_narrow && !self.ui_state.note_list_hidden {
                self.draw_two_panes(ui, language, theme, pane_minimum);
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

        let actions = std::mem::take(&mut self.actions);
        for action in &actions {
            self.apply(action, language);
        }
        self.actions = actions;
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
        if offers_folder && ui.button(Text::OpenFolder.get(language)) {
            self.actions.push(WindowAction::OpenFolder);
        }
        draw_separator(ui, theme);
    }

    /// Draws tree and note as two panes with a draggable splitter in the gap
    /// between them (M21). The splitter shows an accent line only while it
    /// is hovered or dragged.
    fn draw_two_panes(&mut self, ui: &Ui, language: Language, theme: &Theme, pane_minimum: f32) {
        let available = ui.content_region_avail();
        let gap = theme.pane.gap;
        let usable = available[0] - gap;
        let tree_width = (usable * self.ui_state.clamped_splitter_fraction())
            .clamp(pane_minimum, usable - pane_minimum);

        draw_pane(ui, theme, "tree", [tree_width, 0.0], || {
            self.draw_tree_pane(ui, language, theme, None);
        });

        ui.same_line_with_spacing(0.0, 0.0);
        let splitter = draw_vertical_splitter(
            ui,
            theme,
            "splitter",
            tree_width,
            (pane_minimum, usable - pane_minimum),
            available[1],
        );
        if let Some(width) = splitter.left_width {
            self.ui_state.splitter_fraction = width / usable;
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
        // The scrim dims the note and takes its clicks; ImGui reads the
        // background color in `BeginChild`, so it is popped right after.
        let scrim_color =
            ui.push_style_color(StyleColor::ChildBg, theme.surface.modal_overlay.to_array());
        let scrim = ChildWindow::new("drawer_scrim")
            .size(size)
            .scrollable(false)
            .scroll_bar(false)
            .begin(ui);
        scrim_color.pop();
        let Some(_scrim) = scrim else {
            return;
        };
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
        draw_toolbar(ui, language, theme, menu, &mut self.actions);
        let mut context = TreeContext {
            selected: self.ui_state.selected,
            rename: &mut self.rename,
            actions: &mut self.actions,
            language,
            theme,
            frame_padding: ui.clone_style().frame_padding,
        };
        draw_nodes(ui, &self.document.root, &mut context);
    }

    /// Draws the note pane: the note toolbar with the menu button (M48),
    /// then the note in its own scroll region so the toolbar stays in place.
    /// `menu_tooltip` names what the menu button does.
    fn draw_note_pane(&mut self, ui: &Ui, language: Language, theme: &Theme, menu_tooltip: Text) {
        // The menu button is "on" while the note list is visible (M71).
        let list_visible = menu_tooltip == Text::HideNoteList;
        if draw_icon_toggle(
            ui,
            theme,
            Icon::MENU,
            menu_tooltip.get(language),
            list_visible,
        ) {
            self.actions.push(WindowAction::ToggleNoteList);
        }
        self.draw_note_toolbar_end(ui, language, theme);
        draw_separator(ui, theme);
        if self.editor.is_some() {
            self.draw_editor(ui, language, theme);
        } else {
            ChildWindow::new("note_body")
                .size([0.0, 0.0])
                .draw_background(false)
                .build(ui, || self.draw_note_body(ui, language, theme));
        }
    }

    /// Draws the right end of the note toolbar (M51): the edit button while
    /// viewing, "Cancel" and the primary "Done" while editing.
    fn draw_note_toolbar_end(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let right_edge = ui.window_content_region_max()[0];
        let spacing = ui.clone_style().item_spacing[0];
        if self.editor.is_some() {
            let cancel = Text::Cancel.get(language);
            let done = Text::Done.get(language);
            let width = button_width(ui, cancel) + spacing + button_width(ui, done);
            ui.same_line_with_pos(right_edge - width);
            if ui.button(cancel) {
                self.actions.push(WindowAction::CancelEditing);
            }
            ui.same_line();
            if draw_primary_button(ui, theme, done) {
                self.actions.push(WindowAction::FinishEditing);
            }
            if ui.is_item_hovered() {
                ui.tooltip_text(Text::DoneTooltip.get(language));
            }
        } else if self.selected_note().is_some() {
            let width = {
                let _icons = FontScope::push(ui, FontRole::Icons);
                let mut buffer = [0; 4];
                button_width(ui, Icon::PENCIL.as_str(&mut buffer))
            };
            ui.same_line_with_pos(right_edge - width);
            if draw_icon_button(ui, theme, Icon::PENCIL, Text::EditNote.get(language)) {
                self.actions.push(WindowAction::StartEditing);
            }
        }
    }

    /// Draws the editor (M51–M53): formatting toolbar, title input and the
    /// markdown body input filling the rest of the pane.
    fn draw_editor(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let active = self
            .editor
            .as_mut()
            .map(Editor::active_formats)
            .unwrap_or_default();
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
        draw_separator(ui, theme);
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
        if editor.body.is_focused() && ui.io().key_ctrl {
            if ui.is_key_index_pressed(VIRTUAL_KEY_B) {
                editor.format(FormatCommand::Bold);
            } else if ui.is_key_index_pressed(VIRTUAL_KEY_I) {
                editor.format(FormatCommand::Italic);
            }
        }
        // Preview (M60): side by side when two panes fit, otherwise the
        // button switches between editor and preview.
        let available = ui.content_region_avail()[0];
        let pane_minimum =
            ui.calc_text_size("n")[0] * MINIMUM_PANE_CHARACTERS + 2.0 * theme.pane.padding;
        let gap = theme.pane.gap;
        let side_by_side = available >= 2.0 * pane_minimum + gap;
        let shows_editor = !self.ui_state.preview_visible || side_by_side;
        let usable = available - gap;
        let editor_width = (usable * self.ui_state.clamped_preview_split_fraction())
            .clamp(pane_minimum, (usable - pane_minimum).max(pane_minimum));
        if shows_editor {
            let width = if self.ui_state.preview_visible {
                editor_width
            } else {
                0.0
            };
            let response = editor.body.draw(ui, theme, "##body", [width, 0.0]);
            if response.changed {
                editor.sync_body();
            }
            if response.submitted {
                self.actions.push(WindowAction::FinishEditing);
            }
        } else {
            editor.body.unfocus();
        }
        if self.ui_state.preview_visible {
            if shows_editor {
                ui.same_line_with_spacing(0.0, 0.0);
                let splitter = draw_vertical_splitter(
                    ui,
                    theme,
                    "preview_splitter",
                    editor_width,
                    (pane_minimum, usable - pane_minimum),
                    ui.content_region_avail()[1],
                );
                if let Some(width) = splitter.left_width {
                    self.ui_state.preview_split_fraction = width / usable;
                }
                if splitter.released {
                    self.saver.save_ui_state(self.ui_state);
                }
                ui.same_line_with_spacing(0.0, 0.0);
            }
            let preview = editor
                .preview
                .get_or_insert_with(|| markdown::parse(&editor.session.body));
            let actions = &mut self.actions;
            draw_pane(ui, theme, "preview", [0.0, 0.0], || {
                // The preview is read-only: links open, checkboxes do not toggle.
                if let Some(MarkdownAction::OpenLink(url)) = draw_markdown(ui, preview, theme) {
                    actions.push(WindowAction::Markdown(MarkdownAction::OpenLink(url)));
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

        {
            let _heading = FontScope::push(ui, FontRole::Heading1);
            ui.text_wrapped(&note.title);
        }
        draw_separator(ui, theme);
        if ui.is_window_hovered() && ui.is_mouse_double_clicked(MouseButton::Left) {
            self.actions.push(WindowAction::StartEditing);
        }

        if self.restore_scroll {
            ui.set_scroll_y(self.ui_state.note_scroll);
            self.restore_scroll = false;
        }
        match &self.parsed {
            Some(parsed) if !parsed.document.blocks.is_empty() => {
                if let Some(action) = draw_markdown(ui, &parsed.document, theme) {
                    self.actions.push(WindowAction::Markdown(action));
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
            match (editor.session.has_changes(), self.unsaved_changes) {
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
            WindowAction::NewNote => {
                let parent = self.document.insertion_parent(self.ui_state.selected);
                let note = Note::new(
                    Text::NewNote.get(language).to_owned(),
                    NoteKind::Normal,
                    now,
                );
                let id = NodeId::Note(note.id);
                self.document.append(parent, Node::Note(note));
                self.select(id);
                self.start_rename_of_new(id, Text::NewNote.get(language));
                self.save_notes();
            }
            WindowAction::NewGroup => {
                let parent = self.document.insertion_parent(self.ui_state.selected);
                let group = Group::new(Text::NewGroup.get(language).to_owned());
                let id = NodeId::Group(group.id);
                self.document.append(parent, Node::Group(group));
                self.select(id);
                self.start_rename_of_new(id, Text::NewGroup.get(language));
                self.save_notes();
            }
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
            WindowAction::CancelEditing => {
                if self.has_unsaved_changes() && self.unsaved_changes == UnsavedChanges::Ask {
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
            self.saver.save_ui_state(self.ui_state);
        }
        // Choosing a note closes the drawer; groups keep it open (M48).
        if matches!(node, NodeId::Note(_)) {
            self.drawer_open = false;
        }
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

    /// Hands a copy of the note tree to the storage worker.
    fn save_notes(&self) {
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
        .build();
    draw_list
        .add_rect(min, max, theme.border.color.to_array())
        .thickness(theme.border.width.max(1.0))
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
        if ui.button_with_size(keep_editing, full) {
            answer = Some(WindowAction::LeaveCancelled);
        }
        ui.dummy([0.0, theme.spacing.small]);
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        if ui.button_with_size(discard, full) {
            answer = Some(WindowAction::LeaveDiscarding);
        }
    } else {
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        if ui.button(discard) {
            answer = Some(WindowAction::LeaveDiscarding);
        }
        let right_group = button_width(ui, keep_editing) + spacing[0] + button_width(ui, save);
        ui.same_line_with_pos(left + content_width - right_group - window_position[0]);
        if ui.button(keep_editing) {
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

/// Draws a flat icon button: no background, highlighted only on hover, with
/// a tooltip (`a11y-icon-tooltip`; menu button M48, note toolbar M51, M53).
/// Returns whether it was clicked.
fn draw_icon_button(ui: &Ui, theme: &Theme, icon: Icon, tooltip: &str) -> bool {
    draw_icon_toggle(ui, theme, icon, tooltip, false)
}

/// Like [`draw_icon_button`], but the icon is drawn in the accent color when
/// `active` (toggles whose state is on, M69, M71).
fn draw_icon_toggle(ui: &Ui, theme: &Theme, icon: Icon, tooltip: &str, active: bool) -> bool {
    let _active_color =
        active.then(|| ui.push_style_color(StyleColor::Text, theme.accent.base.to_array()));
    let clicked = {
        let _colors = [
            ui.push_style_color(StyleColor::Button, [0.0; 4]),
            ui.push_style_color(StyleColor::ButtonHovered, theme.surface.hover.to_array()),
            ui.push_style_color(StyleColor::ButtonActive, theme.surface.active.to_array()),
        ];
        let _icons = FontScope::push(ui, FontRole::Icons);
        let mut buffer = [0; 4];
        ui.button(icon.as_str(&mut buffer))
    };
    if ui.is_item_hovered() {
        ui.tooltip_text(tooltip);
    }
    clicked
}

/// What the formatting toolbar shows as active.
#[derive(Debug, Clone, Copy)]
struct ToolbarState {
    /// Formatting at the cursor (M69).
    active: ActiveFormats,
    /// Whether the preview is shown (M60).
    preview_visible: bool,
}

/// Draws the formatting buttons of the editor (M53), wrapping onto a new
/// line when the pane is too narrow (`lay-responsive`). The event button is
/// disabled without trailkit-core (`form-disabled-why`); buttons whose
/// formatting applies at the cursor are drawn in the accent color (M69).
fn draw_format_toolbar(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    state: ToolbarState,
    actions: &mut Vec<WindowAction>,
) {
    let preview_visible = state.preview_visible;
    let buttons = [
        (
            Icon::HEADING,
            Text::FormatHeading,
            Some(FormatCommand::Heading),
        ),
        (Icon::BOLD, Text::FormatBold, Some(FormatCommand::Bold)),
        (
            Icon::ITALIC,
            Text::FormatItalic,
            Some(FormatCommand::Italic),
        ),
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
        (Icon::LINK, Text::FormatLink, Some(FormatCommand::Link)),
        (Icon::CALENDAR_CLOCK, Text::InsertEventRequiresCore, None),
    ];
    let right_edge = ui.window_content_region_max()[0] + ui.window_pos()[0];
    for (index, (icon, tooltip, command)) in buttons.into_iter().enumerate() {
        let width = {
            let _icons = FontScope::push(ui, FontRole::Icons);
            let mut buffer = [0; 4];
            button_width(ui, icon.as_str(&mut buffer))
        };
        if index > 0 {
            ui.same_line();
            if ui.cursor_screen_pos()[0] + width > right_edge {
                ui.new_line();
            }
        }
        let _id = ui.push_id(i32::try_from(index).unwrap_or_default());
        // ImGui 1.80 has no disabled items: a disabled button is drawn in
        // `text.disabled` and its clicks are ignored.
        // Disabled (no command): `text.disabled`; active at the cursor
        // (M69): the accent color, like the preview toggle.
        let icon_color = match command {
            None => Some(theme.text.disabled),
            Some(command) if state.active.is_active(command) => Some(theme.accent.base),
            Some(_) => None,
        };
        let _icon_color =
            icon_color.map(|color| ui.push_style_color(StyleColor::Text, color.to_array()));
        if draw_icon_button(ui, theme, icon, tooltip.get(language))
            && let Some(command) = command
        {
            actions.push(WindowAction::Format(command));
        }
    }

    // The preview toggle sits apart from the formatting buttons; its icon is
    // drawn in the accent color while the preview is visible.
    let width = {
        let _icons = FontScope::push(ui, FontRole::Icons);
        let mut buffer = [0; 4];
        button_width(ui, Icon::EYE.as_str(&mut buffer))
    };
    ui.same_line_with_spacing(0.0, theme.spacing.extra_large);
    if ui.cursor_screen_pos()[0] + width > right_edge {
        ui.new_line();
    }
    let (tooltip, _selected) = if preview_visible {
        (
            Text::HidePreview,
            Some(ui.push_style_color(StyleColor::Text, theme.accent.base.to_array())),
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
        let pattern = match language {
            Language::German => "%d.%m.%Y, %H:%M",
            Language::English => "%Y-%m-%d %H:%M",
        };
        let time = draft
            .saved_at
            .to_zoned(TimeZone::system())
            .strftime(pattern);
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
    if ui.button(Text::DiscardChanges.get(language)) {
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
        if draw_icon_toggle(
            ui,
            theme,
            Icon::MENU,
            tooltip.get(language),
            tooltip == Text::HideNoteList,
        ) {
            actions.push(WindowAction::ToggleNoteList);
        }
        ui.same_line();
    }
    let buttons = [
        (Icon::FILE_PLUS, Text::NewNote, WindowAction::NewNote),
        (Icon::FOLDER_PLUS, Text::NewGroup, WindowAction::NewGroup),
    ];
    for (index, (icon, tooltip, action)) in buttons.into_iter().enumerate() {
        if index > 0 {
            ui.same_line();
        }
        let _id = ui.push_id(i32::try_from(index).unwrap_or_default());
        let clicked = {
            let _icons = FontScope::push(ui, FontRole::Icons);
            let mut buffer = [0; 4];
            ui.button(icon.as_str(&mut buffer))
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

/// Draws `nodes` and, recursively, the children of expanded groups.
fn draw_nodes(ui: &Ui, nodes: &[Node], context: &mut TreeContext<'_>) {
    for node in nodes {
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
    let mut count_buffer = [0; 20];
    let count = format_count(group.note_count(), &mut count_buffer);
    let count_x = row_content_end(ui, context) - ui.calc_text_size(count)[0];
    let title_end = count_x - context.theme.spacing.small;
    let (label, token) = {
        let _bold = FontScope::push(ui, FontRole::BodyBold);
        let label = if is_renaming {
            Cow::Borrowed("")
        } else {
            truncate_to_width(ui, &group.title, title_end - (row_start[0] + title_offset))
        };
        let token = TreeNode::new("group")
            .label::<&str, _>(&label)
            .flags(flags)
            .opened(group.expanded, Condition::Always)
            .push(ui);
        (label, token)
    };
    ui.get_window_draw_list().add_text(
        [count_x, row_start[1] + context.frame_padding[1]],
        context.theme.text.secondary.to_array(),
        count,
    );
    let is_open = token.is_some();
    handle_row_interaction(ui, node, &group.title, &label, context);
    if is_open != group.expanded {
        context
            .actions
            .push(WindowAction::SetExpanded(group.id, is_open));
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
    let row_start = ui.item_rect_min();
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
    let title_end = match draw_badge(ui, note.kind, [content_end, row_start[1]], context) {
        Some(badge_start) => badge_start - context.theme.spacing.small,
        None => content_end,
    };
    ui.same_line_with_spacing(0.0, 0.0);
    ui.set_cursor_screen_pos([title_x, row_start[1]]);
    if is_renaming {
        draw_rename_input(ui, row_start, [title_x, title_end], context);
        return;
    }
    let label = truncate_to_width(ui, &note.title, title_end - title_x);
    ui.text(&label);
    // Interaction belongs to the whole row (the tree node item).
    if is_row_hovered {
        handle_hovered_row(ui, node, &note.title, &label, context);
    }
    draw_context_menu(ui, node, context);
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
    draw_context_menu(ui, node, context);
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
        context.actions.push(WindowAction::Select(node));
    }
    if ui.is_mouse_clicked(MouseButton::Right) {
        context.actions.push(WindowAction::Select(node));
        ui.open_popup("context");
    }
}

/// Draws the context menu of a row if it is open (must run every frame).
fn draw_context_menu(ui: &Ui, node: NodeId, context: &mut TreeContext<'_>) {
    ui.popup("context", || {
        if MenuItem::new(Text::Rename.get(context.language)).build(ui) {
            context.actions.push(WindowAction::StartRename(node));
        }
    });
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
        NoteKind::DailyReset { .. } => (Text::DailyBadge, Text::DailyTooltip),
        NoteKind::WeeklyReset { .. } => (Text::WeeklyBadge, Text::WeeklyTooltip),
    };
    let line_height = ui.text_line_height();
    let padding = theme.spacing.extra_small;
    let width = ui.calc_text_size("W")[0] + 2.0 * padding;
    let start = [
        anchor[0] - width,
        anchor[1] + ((ui.frame_height() - line_height) / 2.0).max(0.0),
    ];
    let letter = letter.get(language);
    let color = theme.accent.base.to_array();
    // Lucide style (`lay-icons`): outline only, round corners, stroke ≈ 2/24 of the size.
    let stroke = (line_height / 12.0).max(1.0);
    let max = [start[0] + width, start[1] + line_height];
    let draw_list = ui.get_window_draw_list();
    draw_list
        .add_rect(start, max, color)
        .rounding(line_height * 0.25)
        .thickness(stroke)
        .build();
    let letter_size = ui.calc_text_size(letter);
    draw_list.add_text(
        [
            start[0] + (width - letter_size[0]) / 2.0,
            start[1] + (line_height - letter_size[1]) / 2.0,
        ],
        color,
        letter,
    );
    if ui.is_mouse_hovering_rect(start, max) {
        ui.tooltip_text(tooltip.get(language));
    }
    Some(start[0])
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
        actions.push(WindowAction::NewNote);
    }
    if fits_one_row {
        ui.same_line_with_spacing(0.0, spacing);
    } else {
        indent_to_center(ui, secondary_width, available[0]);
    }
    if ui.button(secondary) {
        actions.push(WindowAction::NewGroup);
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
