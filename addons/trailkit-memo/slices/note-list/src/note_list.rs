//! The note list: the "New note / New group" header, the tree rows, inline
//! rename, context menus, "Move to…", search hits and drag & drop
//! (memo plan M43–M50, M96–M99, M106–M117, M121–M125, M128).
//!
//! The app draws the menu button and the search field above the header and
//! hands search results and reset-schedule extras in as neutral types
//! ([`ListSearch`], [`NoteRowExtras`], ROADMAP R76 questions 48–53).

mod context_menu;
pub use context_menu::{EntryMenu, draw_note_menu};
mod delete;
mod rows;

use std::collections::HashSet;

use nexus::imgui::{MouseButton, StyleColor, Ui, WindowHoveredFlags};
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText, Text},
    list_filter::ListFilter,
    note_tree::{GroupId, InsertionPoint, MoveCheck, NodeId, Note, NoteId, NotesDocument},
    row_badge::RowBadge,
};
use trailkit_ui::{
    button::icon_button_width, font_registry::FontScope, fonts::FontRole, icons::Icon,
    separator::draw_separator_above_rows, text_search::SearchQuery, theme::Theme,
};

use crate::tree_drag::{
    DRAG_THRESHOLD, DropIndicator, DropTarget, EXPAND_DELAY_SECONDS, RowRect, resolve_drop,
};

/// Opacity of the dragged row and its children while dragging (M106).
const DRAGGED_ROW_ALPHA: f32 = 0.4;

/// Auto-scroll speed while dragging near the list's edge, in rows per
/// second at the very edge (M111).
const DRAG_SCROLL_ROWS_PER_SECOND: f32 = 12.0;

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
}

/// What the list shows this frame.
pub struct NoteListInput<'a> {
    /// The note tree.
    pub document: &'a NotesDocument,
    /// The selected note or group.
    pub selected: Option<NodeId>,
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
    /// A right click selects the row its menu opens for; unlike [`Self::Select`]
    /// this keeps a narrow window's drawer open, the menu lives in it (M163).
    SelectForContextMenu(NodeId),
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

/// A change of the rename state requested while drawing; applied after
/// all rows, so every row of a frame sees the same state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenameStep {
    /// Start renaming this node.
    Start(NodeId),
    /// Leave rename mode (committed or cancelled).
    End,
}

/// A press on a row that may become a drag (M106).
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
    /// A row was pressed while searching: dragging cannot move it (M125).
    drag_blocked_by_search: bool,
}

/// Everything the row drawing functions need besides the nodes.
struct RowContext<'frame, 'input> {
    /// The whole tree ("Move to" lists its groups, M112).
    document: &'input NotesDocument,
    /// The selected note or group.
    selected: Option<NodeId>,
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
}

impl NoteList {
    /// Puts `node` into rename mode with `title` in the input (a just
    /// created entry, M43).
    pub fn start_rename(&mut self, node: NodeId, title: String) {
        self.rename = Some(RenameState {
            node,
            buffer: title,
            focused: false,
        });
    }

    /// Cancels a running drag (Escape, trailkit-ui skill §4). Returns
    /// whether one was running.
    pub fn cancel_drag(&mut self) -> bool {
        let is_dragging = self.drag.as_ref().is_some_and(|drag| drag.is_dragging);
        if is_dragging {
            self.drag = None;
        }
        is_dragging
    }

    /// Draws the "new note" / "new group" buttons and the separator below
    /// them. Both share the row in equal widths, never narrower than an icon
    /// button; without room for both they stack (M128). Returns the clicked
    /// button's action.
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
        let mut clicked_action = None;
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
                clicked_action = Some(action);
            }
        }
        draw_separator_above_rows(ui, theme);
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
            edited,
            search,
            mut extras,
        } = input;
        self.rows.clear();
        let style = ui.clone_style();
        let mut context = RowContext {
            document,
            selected,
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
        };
        rows::draw_nodes(ui, &document.root, &mut context);
        let pressed = context.pressed;
        if let Some(search) = &search {
            // Positions in a filtered list are ambiguous: no drag & drop
            // while searching (M125).
            if pressed.is_some() {
                self.drag_blocked_by_search = true;
            }
            if !ui.is_mouse_down(MouseButton::Left) {
                self.drag_blocked_by_search = false;
            } else if self.drag_blocked_by_search
                && ui.is_mouse_dragging_with_threshold(MouseButton::Left, DRAG_THRESHOLD)
            {
                ui.tooltip_text(search.texts.clear_search_to_move);
            }
        } else if let Some(node) = pressed {
            self.drag = Some(TreeDrag {
                node,
                pressed_at: ui.io().mouse_pos,
                is_dragging: false,
                target: None,
                check: MoveCheck::Unchanged,
                collapsed_hover: None,
            });
        }
        self.track_drag(ui, theme, style.indent_spacing, document, &mut output);
        // Right-click on an empty spot of the list (M96): create at the end
        // of the root level.
        if ui.is_window_hovered()
            && !ui.is_any_item_hovered()
            && ui.is_mouse_clicked(MouseButton::Right)
        {
            ui.open_popup("list_context");
        }
        trailkit_ui::menu::draw_popup(ui, "list_context", || {
            context_menu::draw_create_menu_items(
                ui,
                language,
                theme,
                InsertionPoint::End(None),
                &mut output.actions,
            );
        });
        self.apply_rename_steps(document);
        if self.rename.is_none()
            && let Some(action) = delete::delete_key_action(ui, document, selected, edited)
        {
            output.actions.push(action);
        }
        output
    }

    /// Applies the rename changes requested while drawing, in order.
    fn apply_rename_steps(&mut self, document: &NotesDocument) {
        for step in self.rename_steps.drain(..) {
            match step {
                RenameStep::Start(node) => {
                    if let Some(title) = node_title(document, node) {
                        self.rename = Some(RenameState {
                            node,
                            buffer: title.to_owned(),
                            focused: false,
                        });
                    }
                }
                RenameStep::End => self.rename = None,
            }
        }
    }

    /// Follows a drag over the list (M106–M111): starts it once the mouse
    /// moved [`DRAG_THRESHOLD`], finds the drop target, draws the drop line
    /// or group highlight, expands a collapsed group after a rest and
    /// scrolls near the list's edges. Runs inside the list's window after
    /// its rows; `indent` is the width of one tree level.
    fn track_drag(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        indent: f32,
        document: &NotesDocument,
        output: &mut NoteListOutput,
    ) {
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
            .then(|| resolve_drop(&self.rows, mouse[1], indent))
            .flatten();
        drag.check = drag.target.map_or(MoveCheck::Unchanged, |target| {
            document.check_move(drag.node, target.point)
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
                    output
                        .actions
                        .push(NoteListAction::SetExpanded(group, true));
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

    /// Ends a press or drag on the list when the left button is released: a
    /// press without drag selects the row, a drag moves the node to its drop
    /// target (M106–M111). While dragging, the title floats next to the
    /// cursor with a gold border, red with a ban icon over an impossible
    /// target (M109, M116). Call once per frame after the whole window, as
    /// the list may not be drawn (drawer closed).
    pub fn finish_drag(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        document: &NotesDocument,
    ) -> Option<NoteListAction> {
        let drag = self.drag.as_mut()?;
        if !ui.is_mouse_down(MouseButton::Left) {
            let action = if !drag.is_dragging {
                Some(NoteListAction::Select(drag.node))
            } else if drag.check == MoveCheck::Changes
                && let Some(target) = drag.target
            {
                Some(NoteListAction::Move(drag.node, target.point))
            } else {
                None
            };
            self.drag = None;
            return action;
        }
        if !drag.is_dragging {
            return None;
        }
        let title = node_title(document, drag.node).unwrap_or_default();
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
        None
    }
}

/// The title of a note or group in `document`.
fn node_title(document: &NotesDocument, node: NodeId) -> Option<&str> {
    match node {
        NodeId::Note(id) => document.note(id).map(|note| note.title.as_str()),
        NodeId::Group(id) => document.group(id).map(|group| group.title.as_str()),
    }
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
