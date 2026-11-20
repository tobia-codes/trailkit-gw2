//! The note list: the "New note / New group" header, the tree rows, inline
//! rename, context menus, "Move to…", search hits and drag & drop
//! (memo plan M43–M50, M96–M99, M106–M117, M121–M125, M128).
//!
//! The app draws the menu button and the search field above the header and
//! hands search results and reset-schedule extras in as neutral types
//! ([`ListSearch`], [`NoteRowExtras`], ROADMAP R76 questions 48–53).

mod context_menu;
pub use context_menu::{EntryMenu, LeadingItem, NoteMenuChoice, draw_note_menu};
mod delete;
mod drag_and_drop;
mod rename;
mod rows;

use std::collections::{BTreeSet, HashSet};
use trailkit_ui::controls::hover::is_item_hovered;

use nexus::imgui::{MouseButton, StyleVar, Ui};
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText, Text},
    list_filter::ListFilter,
    note_tree::{GroupId, InsertionPoint, NodeId, Note, NoteId, NotesDocument},
    row_badge::RowBadge,
};
use trailkit_ui::{
    controls::button::{Button, icon_button_width},
    controls::tooltip::draw_tooltip,
    layout::stack::Stack,
    lists::list_row::tree_indent,
    text::icons::Icon,
    text::text_search::SearchQuery,
    theme::Theme,
};

use self::{
    drag_and_drop::TreeDrag,
    rename::{RenameState, RenameStep},
};
use crate::tree_drag::RowRect;

/// The search texts the list shows, filled by the app from the search
/// slice's texts (question 50).
#[derive(Debug, Clone, Copy)]
pub struct ListSearchTexts<'a> {
    /// Tooltip of a row's content hit count; `{count}` is replaced (M122).
    pub content_hits: &'a str,
    /// Tooltip while dragging a row during a search (M125).
    pub clear_search_to_move: &'a str,
}

/// An active search as the list sees it (question 48).
pub struct ListSearch<'a> {
    /// The query whose hits are marked in the row titles (M122).
    pub query: &'a SearchQuery,
    /// Which rows are shown and the counts next to them.
    pub filter: &'a dyn ListFilter,
    /// Groups collapsed during this search only (M124).
    pub collapsed: &'a HashSet<GroupId>,
    /// The texts the list shows during a search.
    pub texts: ListSearchTexts<'a>,
}

/// What other slices add to note rows, handed in by the app (question 51).
pub struct NoteRowExtras<'a> {
    /// The badge at the end of a note row, if the note has one.
    pub badge: &'a dyn Fn(&Note) -> Option<RowBadge>,
    /// Draws the tooltip of a hovered badge.
    pub badge_tooltip: &'a mut dyn FnMut(&Ui, &Note),
    /// Draws extra items of a note's context menu, after "Move to" (M160).
    pub context_menu: &'a mut dyn FnMut(&Ui, &Note),
    /// Which notes have unsaved changes kept as a draft (M185).
    pub unsaved_changes: UnsavedChanges<'a>,
}

/// The notes whose rows carry the unsaved-changes mark (M185).
#[derive(Clone, Copy)]
pub struct UnsavedChanges<'a> {
    /// The notes with a draft.
    pub notes: &'a [NoteId],
    /// The mark's tooltip.
    pub tooltip: &'a str,
}

impl UnsavedChanges<'_> {
    /// Whether `note` carries the mark.
    #[must_use]
    pub fn contains(&self, note: NoteId) -> bool {
        self.notes.contains(&note)
    }
}

/// What the list shows this frame.
pub struct NoteListInput<'a> {
    /// The note tree.
    pub document: &'a NotesDocument,
    /// The selected note or group.
    pub selected: Option<NodeId>,
    /// The groups the user collapsed (M201); all others are expanded.
    pub collapsed_groups: &'a BTreeSet<GroupId>,
    /// The note open in the editor: it and the groups holding it cannot be
    /// deleted (M141).
    pub edited: Option<NoteId>,
    /// The active search, if any.
    pub search: Option<ListSearch<'a>>,
    /// Badges and menu items of other slices.
    pub extras: NoteRowExtras<'a>,
}

/// What the user did in the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteListAction {
    /// Select a note or group.
    Select(NodeId),
    /// A click on an empty spot of the list: nothing is selected any more;
    /// the shown note stays (M216).
    ClearSelection,
    /// A click on a group's row, once the double-click time passed without
    /// a second click: select it and toggle it (M172).
    ClickGroup(GroupId),
    /// A group was expanded or collapsed.
    SetExpanded(GroupId, bool),
    /// An inline rename was committed with this title (M44).
    Rename(NodeId, String),
    /// Create a note at a point, or by the selection for `None` (M43, M96).
    NewNote(Option<InsertionPoint>),
    /// Create a group at a point, or by the selection for `None`.
    NewGroup(Option<InsertionPoint>),
    /// Move a note or group with its children (M106–M112).
    Move(NodeId, InsertionPoint),
    /// Collapse (`true`) or expand a group only for the current search
    /// (M124).
    SetCollapsedInSearch(GroupId, bool),
    /// Move a note or group to the trash ("Delete" in the context menu or
    /// the Delete key, M133).
    Delete(NodeId),
}

/// The result of drawing the list for one frame.
#[derive(Debug, Default)]
pub struct NoteListOutput {
    /// The user's actions, in order.
    pub actions: Vec<NoteListAction>,
    /// Whether the Delete key belongs to the list: it has the focus, an entry is
    /// selected and no text is being typed (M133, M175).
    pub takes_delete_key: bool,
}

/// State of the list that outlives a frame.
#[derive(Debug, Default)]
pub struct NoteList {
    /// Inline rename in progress.
    rename: Option<RenameState>,
    /// Rename changes requested in this frame.
    rename_steps: Vec<RenameStep>,
    /// A press on a row, possibly a drag (M106).
    drag: Option<TreeDrag>,
    /// Rectangles of the rows drawn in this frame; reused every frame.
    rows: Vec<RowRect>,
    /// The row pressed while searching and the mouse position of the press:
    /// dragging cannot move it (M125), a release in place selects it.
    search_press: Option<(NodeId, [f32; 2])>,
    /// A row to scroll into the middle of the list when it is drawn next
    /// (M185).
    reveal: Option<NodeId>,
    /// A group clicked once and when: it toggles once the double-click time
    /// has passed, unless a double-click renames it instead (M172).
    pending_group_click: Option<(GroupId, f64)>,
}

/// Everything the row drawing functions need besides the nodes.
struct RowContext<'frame, 'input> {
    /// The whole tree ("Move to" lists its groups, M112).
    document: &'input NotesDocument,
    /// The selected note or group.
    selected: Option<NodeId>,
    /// The groups the user collapsed (M201).
    collapsed_groups: &'input BTreeSet<GroupId>,
    /// The note open in the editor (M141).
    edited: Option<NoteId>,
    /// The note or group being dragged; it is drawn faded (M106).
    dragged: Option<NodeId>,
    /// The row pressed with the left button in this frame.
    pressed: Option<NodeId>,
    /// Collects the rectangle of every drawn row.
    rows: &'frame mut Vec<RowRect>,
    /// Inline rename in progress.
    rename: &'frame mut Option<RenameState>,
    /// Collects rename changes.
    rename_steps: &'frame mut Vec<RenameStep>,
    /// Collected actions.
    actions: &'frame mut Vec<NoteListAction>,
    /// UI language.
    language: Language,
    /// Active theme.
    theme: &'frame Theme,
    /// ImGui frame padding; rows use it so they are as tall as the rename
    /// input and titles line up with the input text.
    frame_padding: [f32; 2],
    /// The active search (M118).
    search: Option<&'frame ListSearch<'input>>,
    /// Badges and menu items of other slices.
    extras: &'frame mut NoteRowExtras<'input>,
    /// The row to scroll to, taken once it is drawn.
    reveal: &'frame mut Option<NodeId>,
}

impl NoteList {
    /// Scrolls the list so the row of `node` sits in its middle the next time
    /// it is drawn (M185). The caller makes the row visible: groups
    /// expanded, no search hiding it.
    pub fn reveal(&mut self, node: NodeId) {
        self.reveal = Some(node);
    }

    /// Draws the "new note" / "new group" buttons. Both share the row in
    /// equal widths, `space.sm` apart, never narrower than an icon button;
    /// without room for both they stack (M128, R148). Returns the clicked
    /// button's action; the caller places the divider below.
    pub fn draw_header(ui: &Ui, language: Language, theme: &Theme) -> Option<NoteListAction> {
        let buttons = [
            (
                Icon::FILE_PLUS,
                Text::NewNote,
                NoteListAction::NewNote(None),
            ),
            (
                Icon::FOLDER_PLUS,
                Text::NewGroup,
                NoteListAction::NewGroup(None),
            ),
        ];
        let spacing = theme.spacing.small;
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
        let stack = if side_by_side {
            Stack::horizontal(theme)
        } else {
            Stack::vertical(theme)
        };
        let mut clicked_action = None;
        stack.build(ui, |stack| {
            for (index, (icon, tooltip, action)) in buttons.into_iter().enumerate() {
                stack.add(|| {
                    let _id = ui.push_id(i32::try_from(index).unwrap_or_default());
                    // Icon only; the tooltip names it.
                    let clicked = Button::new("").icon(icon).width(width).draw(ui, theme);
                    if is_item_hovered(ui) {
                        draw_tooltip(ui, theme, tooltip.get(language));
                    }
                    if clicked {
                        clicked_action = Some(action);
                    }
                });
            }
        });
        clicked_action
    }

    /// Draws the rows into the current (scrolling) window: the tree, drag &
    /// drop and the context menu of empty spots.
    pub fn draw(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        input: NoteListInput<'_>,
    ) -> NoteListOutput {
        let mut output = NoteListOutput::default();
        let NoteListInput {
            document,
            selected,
            collapsed_groups,
            edited,
            search,
            mut extras,
        } = input;
        self.rows.clear();
        let style = ui.clone_style();
        let mut context = RowContext {
            document,
            selected,
            collapsed_groups,
            edited,
            dragged: self
                .drag
                .as_ref()
                .filter(|drag| drag.is_dragging)
                .map(|drag| drag.node),
            pressed: None,
            rows: &mut self.rows,
            rename: &mut self.rename,
            rename_steps: &mut self.rename_steps,
            actions: &mut output.actions,
            language,
            theme,
            frame_padding: style.frame_padding,
            search: search.as_ref(),
            extras: &mut extras,
            reveal: &mut self.reveal,
        };
        {
            // Rows are `space.xs` apart (R148). The rows of every depth are
            // drawn one after another, so the gap is set for the list here,
            // like a menu sets it for its rows; the drag bands count with it.
            let _rows = ui.push_style_var(StyleVar::ItemSpacing([0.0, theme.spacing.extra_small]));
            rows::draw_nodes(ui, &document.root, 0, &mut context);
        }
        let pressed = context.pressed;
        // A double-click renames: the toggle of its first click is dropped.
        if ui.is_mouse_double_clicked(MouseButton::Left) {
            self.pending_group_click = None;
        }
        self.start_press(ui, theme, pressed, search.as_ref());
        // The drop levels use the list's own tree indent (R156).
        self.track_drag(ui, theme, tree_indent(ui, theme), document, &mut output);
        // A click on an empty spot of the list: a left click clears the
        // selection (M216), a right click offers creating at the end of the
        // root level (M96).
        if ui.is_window_hovered() && !ui.is_any_item_hovered() {
            if ui.is_mouse_clicked(MouseButton::Left) && selected.is_some() {
                output.actions.push(NoteListAction::ClearSelection);
            }
            if ui.is_mouse_clicked(MouseButton::Right) {
                ui.open_popup("list_context");
            }
        }
        trailkit_ui::controls::menu::draw_popup(ui, theme, "list_context", || {
            context_menu::draw_create_menu_items(
                ui,
                language,
                theme,
                InsertionPoint::End(None),
                &mut output.actions,
            );
        });
        self.apply_rename_steps(document);
        output.takes_delete_key = self.rename.is_none()
            && selected.is_some()
            && ui.is_window_focused()
            && !ui.io().want_text_input;
        output
    }
}

/// The title of a note or group in `document`.
fn node_title(document: &NotesDocument, node: NodeId) -> Option<&str> {
    match node {
        NodeId::Note(id) => document.note(id).map(|note| note.title.as_str()),
        NodeId::Group(id) => document.group(id).map(|group| group.title.as_str()),
    }
}
