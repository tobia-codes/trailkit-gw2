//! Drawing the trash (memo M136, M145–M150, M218): the list column with its
//! header, two-line rows, trashed groups as a tree to expand, the context
//! menu, the empty state, and the toolbar and group preview of the note
//! pane.

use std::collections::BTreeSet;

use jiff::Timestamp;
use nexus::imgui::{MouseButton, StyleVar, Ui};
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText},
    list_filter::ListFilter,
    note_tree::{Group, GroupId, NodeId, NotesDocument, TrashEntry, TreeNode, visit_shown},
};
use trailkit_ui::controls::hover::is_item_hovered;
use trailkit_ui::{
    controls::button::{
        draw_danger_button, draw_danger_icon_button, draw_icon_button,
        draw_primary_button_with_icon, icon_button_width, icon_label_button_width,
    },
    controls::menu::{MenuItem, MenuItemTone, draw_menu_item, draw_popup},
    layout::empty_state::{EmptyState, draw_empty_state},
    layout::separator::draw_separator,
    layout::stack::Stack,
    layout::toolbar::move_to_toolbar_end,
    lists::list_row::{ListRow, ListRowDetail, RowChevron, draw_list_row},
    text::icons::Icon,
    text::text_search::SearchQuery,
    text::{TextStyle, draw_text, draw_text_wrapped, draw_truncated_text},
    theme::Theme,
};

use crate::{i18n::TrashText, labels};

/// What the user did in the trash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrashAction {
    /// Show an entry in the note pane.
    Select(NodeId),
    /// Show a trashed group and open or close it (M218, Q563).
    ToggleGroup(GroupId),
    /// Put an entry back (M140).
    Restore(NodeId),
    /// Delete an entry for good; the app asks first (M146).
    DeletePermanently(NodeId),
    /// Empty the trash; the app asks first (M146).
    EmptyTrash,
    /// Go back to the note list (M154).
    LeaveTrash,
}

/// Draws the header of the trash column: "Trash" in bold and, with
/// entries, the `danger` "Empty" button at its right end; the caller's stack
/// puts the divider below (R148). A back arrow left of the title returns to
/// the notes (M154). In a narrow column the button moves `space.sm` below
/// the title, which is shortened if needed (`lay-responsive`) and then shows
/// the full title as a tooltip (Q282).
pub fn draw_trash_header(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    has_entries: bool,
) -> Option<TrashAction> {
    let available = ui.content_region_avail()[0];
    let label = TrashText::EmptyTrash.get(language);
    let mut action = None;
    let mut button_below = false;
    Stack::vertical(theme).build(ui, |stack| {
        stack.add(|| {
            let [left, top] = ui.cursor_screen_pos();
            let row_height = ui.frame_height();
            if draw_icon_button(
                ui,
                theme,
                Icon::ARROW_LEFT,
                TrashText::BackToNotes.get(language),
            ) {
                action = Some(TrashAction::LeaveTrash);
            }
            let title_left = ui.item_rect_max()[0] + theme.spacing.small;
            let title_room = left + available - title_left;
            let title = TrashText::Trash.get(language);
            let (title_width, text_offset) = {
                let _bold = TextStyle::BodyStrong.push(ui, theme);
                (
                    ui.calc_text_size(title)[0],
                    ((row_height - ui.text_line_height()) / 2.0).round(),
                )
            };
            ui.set_cursor_screen_pos([title_left, top + text_offset]);
            draw_truncated_text(ui, theme, TextStyle::BodyStrong, title, title_room);
            if !has_entries {
                return;
            }
            let button_width = trailkit_ui::controls::button::button_width(ui, theme, label);
            if title_width + theme.spacing.small + button_width <= title_room {
                ui.set_cursor_screen_pos([left + available - button_width, top]);
                if draw_danger_button(ui, theme, label) {
                    action = Some(TrashAction::EmptyTrash);
                }
            } else {
                button_below = true;
            }
        });
        if button_below {
            stack.add(|| {
                if draw_danger_button(ui, theme, label) {
                    action = Some(TrashAction::EmptyTrash);
                }
            });
        }
    });
    action
}

/// The search over the trash while its field holds a query (BL31).
#[derive(Clone, Copy)]
pub struct TrashSearch<'a> {
    /// Which rows show and the counts at their ends.
    pub filter: &'a dyn ListFilter,
    /// Whose hits are marked in the titles (M122).
    pub query: &'a SearchQuery,
    /// Groups closed during the search; every other shown group is open,
    /// so the matches inside show (M121).
    pub collapsed: &'a BTreeSet<GroupId>,
}

/// What the trash list shows.
#[derive(Clone, Copy)]
pub struct TrashListInput<'a> {
    /// The notes with their trash.
    pub document: &'a NotesDocument,
    /// The entry shown in the note pane.
    pub selected: Option<NodeId>,
    /// Trashed groups that show what they hold below them, indented like
    /// the note list (M218); ignored while searching.
    pub open_groups: &'a BTreeSet<GroupId>,
    /// The active search, if any.
    pub search: Option<TrashSearch<'a>>,
}

/// Draws the trash entries, newest first, into the current (scrolling)
/// window, or the empty state. While searching only matches show, with
/// their hits marked and counted (BL31).
pub fn draw_trash_entries(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    input: TrashListInput<'_>,
    now: Timestamp,
) -> Option<TrashAction> {
    let TrashListInput {
        document,
        selected,
        open_groups,
        search,
    } = input;
    if document.trash.is_empty() {
        // Icon, "The trash is empty." and how long entries stay (Q283 b).
        let state = EmptyState {
            icon: Some(Icon::TRASH_2),
            title: TrashText::TrashIsEmpty.get(language),
            hint: Some(TrashText::TrashKeepsEntries.get(language)),
            buttons: None,
        };
        draw_empty_state(ui, theme, &state);
        return None;
    }
    let is_open = |group: &Group| match search {
        Some(search) => !search.collapsed.contains(&group.id),
        None => open_groups.contains(&group.id),
    };
    let is_shown =
        |node: &TreeNode| search.is_none_or(|search| search.filter.is_visible(node.id()));
    let marks = search.map(|search| RowMarks {
        filter: search.filter,
        query: search.query,
    });
    // Rows are `space.xs` apart (R148). The gap is ImGui's item spacing, as
    // in the note list, so the indent guides run through it unbroken.
    let _rows = ui.push_style_var(StyleVar::ItemSpacing([0.0, theme.spacing.extra_small]));
    let mut action = None;
    for entry in document.trash.iter().rev() {
        if !is_shown(&entry.node) {
            continue;
        }
        let row = EntryRow {
            entry,
            from_group: entry
                .former_parent
                .and_then(|group| document.group_title_anywhere(group)),
            open: matches!(&entry.node, TreeNode::Group(group) if is_open(group)),
        };
        if let Some(clicked) = draw_entry_row(ui, language, theme, &row, (selected, marks), now) {
            action = Some(clicked);
        }
        let TreeNode::Group(group) = &entry.node else {
            continue;
        };
        if !row.open {
            continue;
        }
        // What a trashed group holds, one level deeper (Q565).
        visit_shown(&group.children, &is_shown, &is_open, &mut |shown| {
            let child = ChildRow {
                node: shown.node,
                level: shown.level + 1,
                open: shown.open,
            };
            if let Some(clicked) = draw_child_row(ui, language, theme, &child, (selected, marks)) {
                action = Some(clicked);
            }
        });
    }
    action
}

/// The search's marks on a row: the hits in its title and the count at its
/// end (M121, M122).
#[derive(Clone, Copy)]
struct RowMarks<'a> {
    /// The counts.
    filter: &'a dyn ListFilter,
    /// The query whose hits are marked.
    query: &'a SearchQuery,
}

impl RowMarks<'_> {
    /// The count at the end of `node`'s row in the search: matching notes
    /// of a group, the hits of a note that matches by its text.
    fn count(&self, node: &TreeNode) -> Option<String> {
        match node {
            TreeNode::Group(group) => Some(self.filter.group_count(group.id).to_string()),
            TreeNode::Note(note) => self
                .filter
                .content_hits(note.id)
                .map(|hits| hits.to_string()),
        }
    }
}

/// A trash entry as its row shows it.
#[derive(Debug, Clone, Copy)]
struct EntryRow<'entry> {
    /// The entry.
    entry: &'entry TrashEntry,
    /// The group it was deleted out of, by title (Q566).
    from_group: Option<&'entry str>,
    /// Whether a group shows what it holds.
    open: bool,
}

/// Draws one two-line row: the title (groups in bold with their note
/// count and the chevron) and below it the days left, `danger` in the last
/// week (M150), after the group it came from (Q566).
fn draw_entry_row(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    row: &EntryRow<'_>,
    (selected, marks): (Option<NodeId>, Option<RowMarks<'_>>),
    now: Timestamp,
) -> Option<TrashAction> {
    let entry = row.entry;
    let id = entry.id();
    let _id = ui.push_id(id.imgui_id());
    let shape = RowShape::of(&entry.node, row.open, marks);
    let detail = labels::row_detail(entry, row.from_group, now, language);
    let response = draw_list_row(
        ui,
        theme,
        &ListRow {
            selected: selected == Some(id),
            title_style: shape.title_style,
            trailing: shape.count.as_deref(),
            title_hits: marks.map(|marks| marks.query),
            chevron: shape.chevron,
            detail: Some(ListRowDetail {
                text: &detail,
                color: if labels::is_urgent(entry, now) {
                    theme.status.danger
                } else {
                    theme.text.secondary
                },
            }),
            ..ListRow::new("##entry", entry.node.title())
        },
    );
    row_action(ui, language, theme, &entry.node, response.clicked)
}

/// A node inside a trashed group as its row shows it.
#[derive(Debug, Clone, Copy)]
struct ChildRow<'node> {
    /// The note or group.
    node: &'node TreeNode,
    /// Its depth in the trash list.
    level: usize,
    /// Whether a group shows what it holds.
    open: bool,
}

/// Draws a row inside a trashed group: indented with the indent guide,
/// without its own days left, since the group's deadline applies (Q565).
fn draw_child_row(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    row: &ChildRow<'_>,
    (selected, marks): (Option<NodeId>, Option<RowMarks<'_>>),
) -> Option<TrashAction> {
    let id = row.node.id();
    let _id = ui.push_id(id.imgui_id());
    let shape = RowShape::of(row.node, row.open, marks);
    let response = draw_list_row(
        ui,
        theme,
        &ListRow {
            selected: selected == Some(id),
            title_style: shape.title_style,
            trailing: shape.count.as_deref(),
            title_hits: marks.map(|marks| marks.query),
            chevron: shape.chevron,
            level: row.level,
            ..ListRow::new("##child", row.node.title())
        },
    );
    row_action(ui, language, theme, row.node, response.clicked)
}

/// How a row shows a note or a group, as in the note list (M218, Q561 B).
#[derive(Debug)]
struct RowShape {
    /// Body for a note, bold for a group.
    title_style: TextStyle,
    /// A group's note count; while searching the search's count (M121).
    count: Option<String>,
    /// A group's chevron.
    chevron: RowChevron,
}

impl RowShape {
    /// The shape of `node`, a group `open` or not, with the search's
    /// counts while `marks` is given.
    fn of(node: &TreeNode, open: bool, marks: Option<RowMarks<'_>>) -> Self {
        let search_count = marks.and_then(|marks| marks.count(node));
        match node {
            TreeNode::Note(_) => Self {
                title_style: TextStyle::Body,
                count: search_count,
                chevron: RowChevron::None,
            },
            TreeNode::Group(group) => Self {
                title_style: TextStyle::BodyStrong,
                count: search_count.or_else(|| Some(group.note_count().to_string())),
                chevron: if open {
                    RowChevron::Open
                } else {
                    RowChevron::Closed
                },
            },
        }
    }
}

/// What a click on the row just drawn asks for: a note is shown, a group
/// shown and toggled at once (Q563); a right click opens the entry menu.
fn row_action(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    node: &TreeNode,
    clicked: bool,
) -> Option<TrashAction> {
    let id = node.id();
    let select = match node {
        TreeNode::Group(group) => TrashAction::ToggleGroup(group.id),
        TreeNode::Note(_) => TrashAction::Select(id),
    };
    let mut action = clicked.then_some(select);
    if is_item_hovered(ui) && ui.is_mouse_clicked(MouseButton::Right) {
        action = Some(TrashAction::Select(id));
        ui.open_popup("trash_context");
    }
    draw_popup(ui, theme, "trash_context", || {
        if let Some(chosen) = draw_entry_menu(ui, language, theme, id) {
            action = Some(chosen);
        }
    });
    action
}

/// Draws the items of an entry's context menu: "Restore" and, set off,
/// the `danger` "Delete permanently …" (M146).
fn draw_entry_menu(ui: &Ui, language: Language, theme: &Theme, id: NodeId) -> Option<TrashAction> {
    let mut action = None;
    let restore = MenuItem::new(TrashText::Restore.get(language)).icon(Icon::ROTATE_CCW);
    if draw_menu_item(ui, theme, &restore) {
        action = Some(TrashAction::Restore(id));
    }
    draw_separator(ui, theme);
    let label = format!("{} …", TrashText::DeletePermanently.get(language));
    let delete = MenuItem::new(&label)
        .icon(Icon::TRASH_2)
        .tone(MenuItemTone::Danger);
    if draw_menu_item(ui, theme, &delete) {
        action = Some(TrashAction::DeletePermanently(id));
    }
    action
}

/// Draws the note pane toolbar of a trashed entry (M145): the `danger`
/// "Delete permanently" icon and the primary "Restore" at the right end.
pub fn draw_preview_toolbar(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    entry: NodeId,
) -> Option<TrashAction> {
    let restore = TrashText::Restore.get(language);
    let spacing = theme.spacing.small;
    let width = icon_button_width(ui, Icon::TRASH_2)
        + spacing
        + icon_label_button_width(ui, theme, restore);
    move_to_toolbar_end(ui, theme, width);
    let mut action = None;
    if draw_danger_icon_button(
        ui,
        theme,
        Icon::TRASH_2,
        TrashText::DeletePermanently.get(language),
    ) {
        action = Some(TrashAction::DeletePermanently(entry));
    }
    ui.same_line_with_spacing(0.0, spacing);
    if draw_primary_button_with_icon(ui, theme, Icon::ROTATE_CCW, restore) {
        action = Some(TrashAction::Restore(entry));
    }
    action
}

/// Draws a trashed group in the note pane (M147): its title (Q280), the
/// `info` line, and the titles of the notes it holds.
pub fn draw_group_preview(ui: &Ui, language: Language, theme: &Theme, group: &Group, info: &str) {
    Stack::vertical(theme).build(ui, |stack| {
        stack.add(|| draw_text_wrapped(ui, theme, TextStyle::Heading1, &group.title));
        stack.add(|| {
            draw_text(ui, theme, TextStyle::Secondary, info);
        });
        stack.add_separator(theme);
        stack.add(|| {
            draw_text(
                ui,
                theme,
                TextStyle::Secondary,
                &labels::note_count(group.note_count(), language),
            );
            draw_child_titles(ui, theme, &group.children);
        });
    });
}

/// Lists the titles of `nodes`, nested groups indented below theirs.
fn draw_child_titles(ui: &Ui, theme: &Theme, nodes: &[TreeNode]) {
    for node in nodes {
        match node {
            TreeNode::Note(note) => draw_text(ui, theme, TextStyle::Body, &note.title),
            TreeNode::Group(group) => {
                draw_text(ui, theme, TextStyle::BodyStrong, &group.title);
                ui.indent_by(theme.spacing.large);
                draw_child_titles(ui, theme, &group.children);
                ui.unindent_by(theme.spacing.large);
            }
        }
    }
}
