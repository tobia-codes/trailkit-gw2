//! The memo main window: note tree, note view, empty state and storage hints
//! (Windows only; memo plan M21, M31, M34, M37–M45).
//!
//! The window owns the note tree (M30). Drawing collects user actions into a
//! reused list; they are applied after the tree has been drawn, then changed
//! documents are handed to the storage worker.

use std::{borrow::Cow, path::PathBuf, sync::Arc};

use jiff::Timestamp;
use nexus::imgui::{
    Condition, Key, MenuItem, MouseButton, MouseCursor, StyleColor, StyleVar, TreeNode,
    TreeNodeFlags, Ui,
};
use trailkit_ui::{
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    markdown::{self, MarkdownDocument},
    markdown_view::{MarkdownAction, draw_markdown, open_web_link},
    pane::draw_pane,
    separator::draw_separator,
    theme::Theme,
};

use crate::{
    i18n::{Language, Text},
    note_tree::{Group, GroupId, NodeId, Note, NoteId, NoteKind, NotesDocument, TreeNode as Node},
    storage_worker::StorageSaver,
    ui_state::UiState,
};

/// Minimum content width of each pane in average characters (M40).
const MINIMUM_PANE_CHARACTERS: f32 = 22.0;

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
    /// Show the tree again (single-pane layout).
    Back,
    /// Toggle a task checkbox in the shown note.
    Markdown(MarkdownAction),
    /// Open the addon folder in the file explorer.
    OpenFolder,
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
    /// Single-pane layout: whether the note (not the tree) is shown.
    single_pane_shows_note: bool,
    /// Whether the saved scroll position still has to be applied.
    restore_scroll: bool,
    /// Actions collected while drawing; reused every frame.
    actions: Vec<WindowAction>,
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
    ) -> Self {
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
            single_pane_shows_note: false,
            restore_scroll: true,
            actions: Vec::new(),
        }
    }

    /// Draws the window content and applies what the user did.
    pub fn draw(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        self.actions.clear();
        self.draw_problem(ui, language, theme);

        if self.document.root.is_empty() {
            draw_empty_state(ui, language, theme, &mut self.actions);
        } else {
            let pane_minimum =
                ui.calc_text_size("n")[0] * MINIMUM_PANE_CHARACTERS + 2.0 * theme.pane.padding;
            let available = ui.content_region_avail();
            if available[0] >= 2.0 * pane_minimum + theme.pane.gap {
                self.draw_two_panes(ui, language, theme, pane_minimum);
            } else if self.single_pane_shows_note && self.selected_note().is_some() {
                self.draw_back_button(ui, language, theme);
                draw_pane(ui, theme, "note", [0.0, 0.0], || {
                    self.draw_note_pane(ui, language, theme);
                });
            } else {
                draw_pane(ui, theme, "tree", [0.0, 0.0], || {
                    self.draw_tree_pane(ui, language, theme);
                });
            }
        }

        let actions = std::mem::take(&mut self.actions);
        for action in &actions {
            self.apply(action, language);
        }
        self.actions = actions;
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
            self.draw_tree_pane(ui, language, theme);
        });

        ui.same_line_with_spacing(0.0, 0.0);
        let splitter_start = ui.cursor_screen_pos();
        ui.invisible_button("splitter", [gap, available[1].max(1.0)]);
        let is_dragged = ui.is_item_active();
        let is_hovered = ui.is_item_hovered();
        if is_hovered || is_dragged {
            ui.set_mouse_cursor(Some(MouseCursor::ResizeEW));
            let color = if is_dragged {
                theme.accent.active
            } else {
                theme.accent.hover
            };
            let line_x = splitter_start[0] + gap / 2.0;
            ui.get_window_draw_list()
                .add_line(
                    [line_x, splitter_start[1]],
                    [line_x, splitter_start[1] + available[1]],
                    color.to_array(),
                )
                .thickness(theme.border.width.max(1.0))
                .build();
        }
        if is_dragged {
            let new_width =
                (tree_width + ui.io().mouse_delta[0]).clamp(pane_minimum, usable - pane_minimum);
            self.ui_state.splitter_fraction = new_width / usable;
        }
        if ui.is_item_deactivated() {
            self.saver.save_ui_state(self.ui_state);
        }

        ui.same_line_with_spacing(0.0, 0.0);
        draw_pane(ui, theme, "note", [0.0, 0.0], || {
            self.draw_note_pane(ui, language, theme);
        });
    }

    /// Draws the toolbar and the tree.
    fn draw_tree_pane(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        draw_toolbar(ui, language, theme, &mut self.actions);
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

    /// Draws the "Back" button of the single-pane layout.
    fn draw_back_button(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        {
            let _icons = FontScope::push(ui, FontRole::Icons);
            let mut buffer = [0; 4];
            ui.text(Icon::CHEVRON_LEFT.as_str(&mut buffer));
        }
        ui.same_line();
        if ui.small_button(Text::Back.get(language)) {
            self.actions.push(WindowAction::Back);
        }
        draw_separator(ui, theme);
    }

    /// Draws the selected note: its title and rendered content.
    fn draw_note_pane(&mut self, ui: &Ui, language: Language, theme: &Theme) {
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

        {
            let _heading = FontScope::push(ui, FontRole::Heading1);
            ui.text_wrapped(&note.title);
        }
        draw_separator(ui, theme);

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
            WindowAction::Back => self.single_pane_shows_note = false,
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
        self.single_pane_shows_note = matches!(node, NodeId::Note(_));
    }

    /// Puts a just-created note or group into rename mode (M43).
    fn start_rename_of_new(&mut self, node: NodeId, title: &str) {
        self.single_pane_shows_note = false;
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

/// Draws the "new note" / "new group" icon buttons.
fn draw_toolbar(ui: &Ui, language: Language, theme: &Theme, actions: &mut Vec<WindowAction>) {
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

    let row_start = ui.cursor_screen_pos();
    let title_offset = title_offset(ui, context);
    let row_width = ui.content_region_avail()[0] - title_offset;
    let label = if is_renaming {
        Cow::Borrowed("")
    } else {
        truncate_to_width(ui, &group.title, row_width)
    };
    let token = TreeNode::new("group")
        .label::<&str, _>(&label)
        .flags(flags)
        .opened(group.expanded, Condition::Always)
        .push(ui);
    let is_open = token.is_some();
    handle_row_interaction(ui, node, &group.title, &label, context);
    if is_open != group.expanded {
        context
            .actions
            .push(WindowAction::SetExpanded(group.id, is_open));
    }
    if is_renaming {
        ui.same_line_with_spacing(0.0, 0.0);
        draw_rename_input(ui, row_start, row_start[0] + title_offset, context);
    }
    if let Some(_token) = token {
        draw_nodes(ui, &group.children, context);
    }
}

/// Horizontal offset of a row title from the row start. ImGui puts the title
/// of a tree node after the arrow (font size plus twice the horizontal frame
/// padding); note titles use the same offset, so titles on one level align.
fn title_offset(ui: &Ui, context: &TreeContext<'_>) -> f32 {
    ui.current_font_size() + 2.0 * context.frame_padding[0]
}

/// Draws one note row: the badge in the arrow slot (M45) and the title.
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
    let is_renaming = context
        .rename
        .as_ref()
        .is_some_and(|rename| rename.node == node);

    // Notes never have an arrow, so the badge column sits in its slot.
    let arrow_center_x = row_start[0] + context.frame_padding[0] + ui.current_font_size() / 2.0;
    draw_badge(ui, note.kind, [arrow_center_x, row_start[1]], context);
    let title_x = row_start[0] + title_offset(ui, context);
    ui.same_line_with_spacing(0.0, 0.0);
    ui.set_cursor_screen_pos([title_x, row_start[1]]);
    if is_renaming {
        draw_rename_input(ui, row_start, title_x, context);
        return;
    }
    let row_end = ui.window_content_region_max()[0] + ui.window_pos()[0];
    let label = truncate_to_width(ui, &note.title, row_end - ui.cursor_screen_pos()[0]);
    ui.text(&label);
    // Interaction belongs to the whole row (the tree node item).
    let row_min = row_start;
    let row_max = [row_end, row_start[1] + ui.frame_height()];
    if ui.is_mouse_hovering_rect(row_min, row_max) && ui.is_window_hovered() {
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
/// whole row starting at `row_start`. The input text sits at `title_x`, where
/// the title was drawn before renaming.
fn draw_rename_input(ui: &Ui, row_start: [f32; 2], title_x: f32, context: &mut TreeContext<'_>) {
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
    ui.set_next_item_width(-1.0);
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

/// Draws the reset badge of a note row (M45): an outlined rounded box with a
/// letter for reset notes, nothing for normal notes. The badge is centered
/// horizontally on `anchor[0]` and vertically in the row starting at
/// `anchor[1]` (one frame tall).
fn draw_badge(ui: &Ui, kind: NoteKind, anchor: [f32; 2], context: &TreeContext<'_>) {
    let (language, theme) = (context.language, context.theme);
    let line_height = ui.text_line_height();
    let padding = theme.spacing.extra_small;
    let width = ui.calc_text_size("W")[0] + 2.0 * padding;
    let start = [
        anchor[0] - width / 2.0,
        anchor[1] + ((ui.frame_height() - line_height) / 2.0).max(0.0),
    ];

    let (letter, tooltip) = match kind {
        NoteKind::Normal => return,
        NoteKind::DailyReset { .. } => (Text::DailyBadge, Text::DailyTooltip),
        NoteKind::WeeklyReset { .. } => (Text::WeeklyBadge, Text::WeeklyTooltip),
    };
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
    {
        let _colors = [
            ui.push_style_color(StyleColor::Button, theme.accent.base.to_array()),
            ui.push_style_color(StyleColor::ButtonHovered, theme.accent.hover.to_array()),
            ui.push_style_color(StyleColor::ButtonActive, theme.accent.active.to_array()),
            ui.push_style_color(StyleColor::Text, theme.text.on_accent.to_array()),
        ];
        let row_width = if fits_one_row {
            primary_width + spacing + secondary_width
        } else {
            primary_width
        };
        indent_to_center(ui, row_width, available[0]);
        if ui.button(primary) {
            actions.push(WindowAction::NewNote);
        }
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
