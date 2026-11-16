//! Drawing the trash (memo M136, M145–M150): the list column with its
//! header, two-line rows and context menu, the empty state, and the
//! toolbar and group preview of the note pane.

use jiff::Timestamp;
use nexus::imgui::{MouseButton, Ui};
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText},
    note_tree::{NodeId, TrashEntry, TreeNode},
};
use trailkit_ui::{
    button::{
        draw_danger_button, draw_danger_icon_button, draw_icon_button,
        draw_primary_button_with_icon, icon_button_width, icon_label_button_width,
    },
    empty_state::{EmptyState, draw_empty_state},
    icons::Icon,
    list_row::{ListRow, ListRowDetail, draw_list_row},
    menu::{MenuItemTone, draw_menu_item_with_icon, draw_popup},
    separator::{draw_compact_separator, draw_separator, draw_separator_above_rows},
    text::{TextStyle, draw_text, draw_text_wrapped, draw_truncated_text},
    theme::Theme,
    toolbar::move_to_toolbar_end,
};

use crate::{i18n::TrashText, labels};

/// What the user did in the trash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrashAction {
    /// Show an entry in the note pane.
    Select(NodeId),
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
/// entries, the `danger` "Empty" button at its right end; then a separator.
/// A back arrow left of the title returns to the notes (M154). In a narrow
/// column the button moves below the title, which is shortened if needed
/// (`lay-responsive`) and then shows the full title as a tooltip (Q282).
pub fn draw_trash_header(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    has_entries: bool,
) -> Option<TrashAction> {
    let top = ui.cursor_screen_pos()[1];
    let left = ui.cursor_screen_pos()[0];
    let row_height = ui.frame_height();
    let available = ui.content_region_avail()[0];
    let mut action = None;
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
            (row_height - ui.text_line_height()) / 2.0,
        )
    };
    ui.set_cursor_screen_pos([title_left, top + text_offset]);
    draw_truncated_text(ui, theme, TextStyle::BodyStrong, title, title_room);
    if has_entries {
        let label = TrashText::EmptyTrash.get(language);
        let button_width = trailkit_ui::button::button_width(ui, theme, label);
        if title_width + theme.spacing.small + button_width <= title_room {
            ui.same_line();
            move_to_toolbar_end(ui, button_width);
            ui.set_cursor_screen_pos([ui.cursor_screen_pos()[0], top]);
        } else {
            let row_gap = ui.clone_style().item_spacing[1];
            ui.set_cursor_screen_pos([left, top + row_height + row_gap]);
        }
        if draw_danger_button(ui, theme, label) {
            action = Some(TrashAction::EmptyTrash);
        }
    } else {
        ui.set_cursor_screen_pos([left, top]);
        ui.dummy([0.0, row_height]);
    }
    draw_separator_above_rows(ui, theme);
    action
}

/// Draws the trash entries, newest first, into the current (scrolling)
/// window, or the empty state. `selected` is the entry shown in the note
/// pane.
pub fn draw_trash_entries(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    trash: &[TrashEntry],
    selected: Option<NodeId>,
    now: Timestamp,
) -> Option<TrashAction> {
    if trash.is_empty() {
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
    let mut action = None;
    for entry in trash.iter().rev() {
        if let Some(clicked) = draw_entry_row(ui, language, theme, entry, selected, now) {
            action = Some(clicked);
        }
    }
    action
}

/// Draws one two-line row: the title (groups in bold with their note
/// count) and the days left below it, `danger` in the last week (M150).
fn draw_entry_row(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    entry: &TrashEntry,
    selected: Option<NodeId>,
    now: Timestamp,
) -> Option<TrashAction> {
    let id = entry.id();
    let _id = ui.push_id(id.imgui_id());
    let (title, title_style) = match &entry.node {
        TreeNode::Note(note) => (note.title.as_str(), TextStyle::Body),
        TreeNode::Group(group) => (group.title.as_str(), TextStyle::BodyStrong),
    };
    let count = match &entry.node {
        TreeNode::Note(_) => None,
        TreeNode::Group(group) => Some(group.note_count().to_string()),
    };
    let days_left = labels::days_left(entry, now, language);
    let response = draw_list_row(
        ui,
        theme,
        &ListRow {
            id: "##entry",
            selected: selected == Some(id),
            title,
            title_style,
            trailing: count.as_deref(),
            detail: Some(ListRowDetail {
                text: days_left.as_ref(),
                color: if labels::is_urgent(entry, now) {
                    theme.status.danger
                } else {
                    theme.text.secondary
                },
            }),
        },
    );
    let mut action = response.clicked.then_some(TrashAction::Select(id));
    if ui.is_item_hovered() && ui.is_mouse_clicked(MouseButton::Right) {
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
    if draw_menu_item_with_icon(
        ui,
        theme,
        Icon::ROTATE_CCW,
        TrashText::Restore.get(language),
        MenuItemTone::Regular,
        None,
    ) {
        action = Some(TrashAction::Restore(id));
    }
    draw_compact_separator(ui, theme);
    let label = format!("{} …", TrashText::DeletePermanently.get(language));
    if draw_menu_item_with_icon(ui, theme, Icon::TRASH_2, &label, MenuItemTone::Danger, None) {
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
    let spacing = ui.clone_style().item_spacing[0];
    let width = icon_button_width(ui, Icon::TRASH_2)
        + spacing
        + icon_label_button_width(ui, theme, restore);
    move_to_toolbar_end(ui, width);
    let mut action = None;
    if draw_danger_icon_button(
        ui,
        theme,
        Icon::TRASH_2,
        TrashText::DeletePermanently.get(language),
    ) {
        action = Some(TrashAction::DeletePermanently(entry));
    }
    ui.same_line();
    if draw_primary_button_with_icon(ui, theme, Icon::ROTATE_CCW, restore) {
        action = Some(TrashAction::Restore(entry));
    }
    action
}

/// Draws a trashed group in the note pane (M147): its title (Q280), the
/// info line, and the titles of the notes it holds.
pub fn draw_group_preview(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    entry: &TrashEntry,
    now: Timestamp,
) {
    let TreeNode::Group(group) = &entry.node else {
        return;
    };
    draw_text_wrapped(ui, theme, TextStyle::Heading1, &group.title);
    draw_text(
        ui,
        theme,
        TextStyle::Secondary,
        &labels::info_line(entry, now, language),
    );
    draw_separator(ui, theme);
    draw_text(
        ui,
        theme,
        TextStyle::Secondary,
        &labels::note_count(group.note_count(), language),
    );
    draw_child_titles(ui, theme, &group.children);
}

/// Lists the titles of `nodes`, nested groups indented below theirs.
fn draw_child_titles(ui: &Ui, theme: &Theme, nodes: &[TreeNode]) {
    for node in nodes {
        match node {
            TreeNode::Note(note) => draw_text(ui, theme, TextStyle::Body, &note.title),
            TreeNode::Group(group) => {
                draw_text(ui, theme, TextStyle::BodyStrong, &group.title);
                ui.indent_by(theme.list.indent);
                draw_child_titles(ui, theme, &group.children);
                ui.unindent_by(theme.list.indent);
            }
        }
    }
}
