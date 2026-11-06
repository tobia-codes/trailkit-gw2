//! Drawing the trash (memo M136, M145–M150): the list column with its
//! header, two-line rows and context menu, the empty state, and the
//! toolbar and group preview of the note pane.

use std::borrow::Cow;
use trailkit_ui::tooltip::draw_tooltip;

use jiff::Timestamp;
use nexus::imgui::{MouseButton, Selectable, StyleColor, Ui};
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText},
    note_tree::{NodeId, TrashEntry, TreeNode},
};
use trailkit_ui::{
    button::{
        draw_danger_button, draw_danger_icon_button, draw_icon_button,
        draw_primary_button_with_icon, icon_button_width, icon_label_button_width,
    },
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    menu::{MenuItemTone, draw_menu_item_with_icon, draw_popup},
    selection_bar::{draw_row_highlight_in, draw_selection_bar_in, hide_square_row_highlight},
    separator::{draw_compact_separator, draw_separator, draw_separator_above_rows},
    text_layout::centered_text,
    text_layout::truncate_to_width,
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
/// (`lay-responsive`).
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
    let title_width = {
        let _bold = FontScope::push(ui, FontRole::BodyBold);
        let text_offset = (row_height - ui.text_line_height()) / 2.0;
        ui.set_cursor_screen_pos([title_left, top + text_offset]);
        let title = TrashText::Trash.get(language);
        ui.text(truncate_to_width(ui, title, title_room));
        ui.calc_text_size(title)[0]
    };
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
        draw_empty_state(ui, language, theme);
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
    let is_selected = selected == Some(id);
    let padding = ui.clone_style().frame_padding;
    let line = ui.text_line_height();
    let [left, top] = ui.cursor_screen_pos();
    let row_size = [ui.content_region_avail()[0], 2.0 * line + 2.0 * padding[1]];
    let clicked = {
        let _square = hide_square_row_highlight(ui);
        Selectable::new("##entry")
            .selected(is_selected)
            .size(row_size)
            .build(ui)
    };
    // The selectable's item rect reaches half the item spacing beyond the
    // row, where the list clips; draw on the row itself.
    let row = [[left, top], [left + row_size[0], top + row_size[1]]];
    draw_row_highlight_in(ui, theme, is_selected, row);
    if is_selected {
        draw_selection_bar_in(ui, theme, row);
    }
    let title_truncated = draw_entry_texts(ui, language, theme, entry, row, now);
    if title_truncated && ui.is_item_hovered() {
        draw_tooltip(
            ui,
            match &entry.node {
                TreeNode::Note(note) => &note.title,
                TreeNode::Group(group) => &group.title,
            },
        );
    }
    let mut action = clicked.then_some(TrashAction::Select(id));
    if ui.is_item_hovered() && ui.is_mouse_clicked(MouseButton::Right) {
        action = Some(TrashAction::Select(id));
        ui.open_popup("trash_context");
    }
    draw_popup(ui, "trash_context", || {
        if let Some(chosen) = draw_entry_menu(ui, language, theme, id) {
            action = Some(chosen);
        }
    });
    action
}

/// Draws the texts of an entry row `[min, max]`: the title (groups in bold
/// with their note count at the right end) and the days left below it,
/// `danger` in the last week (M150). Texts that do not fit are shortened
/// (`lay-truncation`); returns whether the title was.
fn draw_entry_texts(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    entry: &TrashEntry,
    [min, max]: [[f32; 2]; 2],
    now: Timestamp,
) -> bool {
    let padding = ui.clone_style().frame_padding;
    let [left, top] = [min[0] + padding[0], min[1] + padding[1]];
    let right = max[0] - padding[0];
    let draw_list = ui.get_window_draw_list();
    let title_truncated = match &entry.node {
        TreeNode::Note(note) => {
            let title = truncate_to_width(ui, &note.title, right - left);
            draw_list.add_text([left, top], theme.text.primary.to_array(), &title);
            matches!(title, Cow::Owned(_))
        }
        TreeNode::Group(group) => {
            let count = group.note_count().to_string();
            let count_x = right - ui.calc_text_size(&count)[0];
            draw_list.add_text([count_x, top], theme.text.secondary.to_array(), &count);
            let _bold = FontScope::push(ui, FontRole::BodyBold);
            let room = count_x - theme.spacing.small - left;
            let title = truncate_to_width(ui, &group.title, room);
            draw_list.add_text([left, top], theme.text.primary.to_array(), &title);
            matches!(title, Cow::Owned(_))
        }
    };
    let days_color = if labels::is_urgent(entry, now) {
        theme.status.danger
    } else {
        theme.text.secondary
    };
    draw_list.add_text(
        [left, top + ui.text_line_height()],
        days_color.to_array(),
        truncate_to_width(
            ui,
            labels::days_left(entry, now, language).as_ref(),
            right - left,
        ),
    );
    title_truncated
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

/// Draws the empty trash: icon, "The trash is empty." and how long entries
/// stay, centered in the remaining space.
fn draw_empty_state(ui: &Ui, language: Language, theme: &Theme) {
    let lines = [
        TrashText::TrashIsEmpty.get(language),
        TrashText::TrashKeepsEntries.get(language),
    ];
    let available = ui.content_region_avail();
    let line = ui.text_line_height_with_spacing();
    // The icon and two text lines; the longer text may wrap in a narrow list.
    let height = 4.0 * line;
    let [left, top] = ui.cursor_screen_pos();
    let y = top + ((available[1] - height) / 2.0).max(0.0);
    {
        let _icons = FontScope::push(ui, FontRole::Icons);
        let mut buffer = [0; 4];
        let icon = Icon::TRASH_2.as_str(&mut buffer);
        let width = ui.calc_text_size(icon)[0];
        ui.set_cursor_screen_pos([left + ((available[0] - width) / 2.0).max(0.0), y]);
        ui.text_colored(theme.text.disabled.to_array(), icon);
    }
    ui.set_cursor_screen_pos([left, y + line]);
    let _secondary = ui.push_style_color(StyleColor::Text, theme.text.secondary.to_array());
    for text in lines {
        centered_text(ui, text, available[0]);
    }
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

/// Draws a trashed group in the note pane (M147): its muted title, the
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
    {
        let _heading = FontScope::push(ui, FontRole::Heading1);
        let _muted = ui.push_style_color(StyleColor::Text, theme.text.secondary.to_array());
        ui.text_wrapped(&group.title);
    }
    ui.text_colored(
        theme.text.secondary.to_array(),
        labels::info_line(entry, now, language),
    );
    draw_separator(ui, theme);
    ui.text_colored(
        theme.text.secondary.to_array(),
        labels::note_count(group.note_count(), language),
    );
    draw_child_titles(ui, theme, &group.children);
}

/// Lists the titles of `nodes`, nested groups indented below theirs.
fn draw_child_titles(ui: &Ui, theme: &Theme, nodes: &[TreeNode]) {
    for node in nodes {
        match node {
            TreeNode::Note(note) => ui.text(&note.title),
            TreeNode::Group(group) => {
                {
                    let _bold = FontScope::push(ui, FontRole::BodyBold);
                    ui.text(&group.title);
                }
                ui.indent_by(theme.list.indent);
                draw_child_titles(ui, theme, &group.children);
                ui.unindent_by(theme.list.indent);
            }
        }
    }
}
