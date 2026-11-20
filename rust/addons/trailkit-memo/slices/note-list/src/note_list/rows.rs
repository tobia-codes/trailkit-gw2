//! The rows of the note list: groups and notes with their titles, counts,
//! badges and search hits, row interaction and the inline rename input.

use std::borrow::Cow;
use trailkit_ui::controls::hover::is_item_hovered;
use trailkit_ui::controls::tooltip::draw_tooltip;
use trailkit_ui::drawing::draw_list::window_draw_list;

use nexus::imgui::{
    Condition, MouseButton, StyleColor, StyleVar, TreeNode, TreeNodeFlags, TreeNodeToken, Ui,
};
use trailkit_memo_shared::note_tree::{Group, NodeId, Note, TreeNode as Node};
use trailkit_ui::{
    color::Color,
    lists::selection_bar::{draw_row_state, hide_square_row_highlight},
    lists::tree_arrow::draw_tree_arrow,
    text::font_registry::FontScope,
    text::fonts::FontRole,
    text::glyph_metrics::{capital_middle, icon_visible_box},
    text::icons::Icon,
    text::search_hits::{HitMark, mark_search_hits},
    text::text_layout::{format_count, truncate_to_width},
};

use super::{
    NoteListAction, RowContext, context_menu,
    drag_and_drop::DRAGGED_ROW_ALPHA,
    rename::{RenameStep, draw_rename_input},
};
use crate::tree_drag::{RowKind, RowRect};

/// Draws `nodes` and, recursively, the children of expanded groups; during a
/// search only the rows it shows (M118).
pub(super) fn draw_nodes(ui: &Ui, nodes: &[Node], context: &mut RowContext<'_, '_>) {
    for node in nodes {
        if let Some(search) = context.search
            && !search.filter.is_visible(node.id())
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
fn draw_group(ui: &Ui, group: &Group, context: &mut RowContext<'_, '_>) {
    let _id = ui.push_id(group.id.imgui_id());
    let node = NodeId::Group(group.id);
    let is_renaming = context
        .rename
        .as_ref()
        .is_some_and(|rename| rename.node == node);
    // M50: groups stand out by a bold title and their note count, shown
    // right-aligned in the secondary text color like other row info (M49).
    let row_start = ui.cursor_screen_pos();
    let title_offset = group_title_offset(ui, context);
    // During a search the count is the matching notes (M121) and groups
    // are expanded unless collapsed for this search only (M124).
    let (note_count, expanded) = match context.search {
        Some(search) => (
            search.filter.group_count(group.id),
            !search.collapsed.contains(&group.id),
        ),
        None => (
            group.note_count(),
            !context.collapsed_groups.contains(&group.id),
        ),
    };
    let mut count_buffer = [0; 20];
    let count = format_count(note_count, &mut count_buffer);
    let count_x = row_content_end(ui, context) - ui.calc_text_size(count)[0];
    let mut title_end = count_x - context.theme.spacing.small;
    // A collapsed group shows that a note inside it has unsaved changes
    // (M185); an expanded one leaves that to the note rows.
    let has_unsaved_changes = context.extras.unsaved_changes;
    if !expanded && group.any_note(&|note| has_unsaved_changes.contains(note)) {
        let (mark_start, is_hovered) = draw_row_icon(
            ui,
            Icon::TRIANGLE_ALERT,
            context.theme.status.warning,
            title_end,
            row_start[1] + context.frame_padding[1],
        );
        if is_hovered {
            draw_tooltip(ui, context.theme, has_unsaved_changes.tooltip);
        }
        title_end = mark_start - context.theme.spacing.small;
    }
    let title_span = [row_start[0] + title_offset, title_end];
    let (label, token) = draw_group_title(ui, group, expanded, row_start, title_span, context);
    let kind = if expanded {
        RowKind::ExpandedGroup(group.id)
    } else {
        RowKind::CollapsedGroup(group.id)
    };
    record_row(ui, node, kind, row_start[0], context);
    window_draw_list(ui).add_text(
        [count_x, row_start[1] + context.frame_padding[1]],
        context.theme.text.secondary.to_array(),
        count,
    );
    let is_open = token.is_some();
    let is_hovered = is_item_hovered(ui);
    // The first click of a double-click toggled the group: the second, which
    // starts the rename, turns it back. The arrow's second click already did.
    let turn_back =
        is_hovered && ui.is_mouse_double_clicked(MouseButton::Left) && !ui.is_item_toggled_open();
    if is_hovered {
        handle_hovered_row(ui, node, &group.title, &label, context);
    }
    context_menu::draw_context_menu(ui, node, None, context);
    if is_open != expanded || turn_back {
        let action = if context.search.is_some() {
            NoteListAction::SetCollapsedInSearch(group.id, expanded)
        } else {
            NoteListAction::SetExpanded(group.id, !expanded)
        };
        context.actions.push(action);
    }
    if is_renaming {
        ui.same_line_with_spacing(0.0, 0.0);
        let _bold = FontScope::push(ui, FontRole::BodyBold);
        draw_rename_input(ui, row_start, title_span, context);
    }
    if let Some(_token) = token {
        draw_nodes(ui, &group.children, context);
    }
}

/// Draws the group's tree node with its selection state, chevron and bold
/// title between `title_span` (start and end x), the title truncated and
/// its search hits marked; empty while it is being renamed. Returns the
/// drawn title and the node's token, `Some` while the group is open.
fn draw_group_title<'ui, 'group>(
    ui: &'ui Ui,
    group: &'group Group,
    expanded: bool,
    row_start: [f32; 2],
    title_span: [f32; 2],
    context: &RowContext<'_, '_>,
) -> (Cow<'group, str>, Option<TreeNodeToken<'ui>>) {
    let node = NodeId::Group(group.id);
    let is_renaming = context
        .rename
        .as_ref()
        .is_some_and(|rename| rename.node == node);
    let mut flags = TreeNodeFlags::OPEN_ON_ARROW
        | TreeNodeFlags::SPAN_FULL_WIDTH
        | TreeNodeFlags::FRAME_PADDING;
    if context.selected == Some(node) {
        flags |= TreeNodeFlags::SELECTED;
    }
    let [title_x, title_end] = title_span;
    let _bold = FontScope::push(ui, FontRole::BodyBold);
    let label = if is_renaming {
        Cow::Borrowed("")
    } else {
        truncate_to_width(ui, &group.title, title_end - title_x)
    };
    // The node draws hover, selection and its triangle in the text
    // color; a transparent text color hides the triangle, and the
    // Lucide chevron and the title are drawn over it (R71).
    let token = {
        let _hidden_triangle = ui.push_style_color(StyleColor::Text, [0.0; 4]);
        let _square_highlight = hide_square_row_highlight(ui);
        TreeNode::new("group")
            .label::<&str, _>("")
            .flags(flags)
            .opened(expanded, Condition::Always)
            .push(ui)
    };
    draw_row_state(ui, context.theme, context.selected == Some(node));
    draw_tree_arrow(
        ui,
        context.theme,
        token.is_some(),
        row_start,
        context.frame_padding,
    );
    let title_position = [title_x, row_start[1] + context.frame_padding[1]];
    window_draw_list(ui).add_text(
        title_position,
        context.theme.text.primary.to_array(),
        &*label,
    );
    if let Some(search) = context.search {
        mark_search_hits(
            ui,
            context.theme,
            &label,
            search
                .query
                .hit_ranges(&label)
                .into_iter()
                .map(|range| (range, HitMark::Filled)),
            title_position,
        );
    }
    (label, token)
}

/// Horizontal offset of a group title from the row start: ImGui puts it
/// after the arrow (font size plus twice the horizontal frame padding).
fn group_title_offset(ui: &Ui, context: &RowContext<'_, '_>) -> f32 {
    ui.current_font_size() + 2.0 * context.frame_padding[0]
}

/// Screen x where row content (title, badges) ends: the row's right edge
/// minus the horizontal frame padding, mirroring the inset on the left.
fn row_content_end(ui: &Ui, context: &RowContext<'_, '_>) -> f32 {
    ui.window_content_region_max()[0] + ui.window_pos()[0] - context.frame_padding[0]
}

/// Draws one note row (M49): the title flush left, a badge at the right
/// edge.
fn draw_note_row(ui: &Ui, note: &Note, context: &mut RowContext<'_, '_>) {
    let _id = ui.push_id(note.id.imgui_id());
    let node = NodeId::Note(note.id);
    let mut flags = TreeNodeFlags::LEAF
        | TreeNodeFlags::NO_TREE_PUSH_ON_OPEN
        | TreeNodeFlags::SPAN_FULL_WIDTH
        | TreeNodeFlags::FRAME_PADDING;
    if context.selected == Some(node) {
        flags |= TreeNodeFlags::SELECTED;
    }
    // The node spans the full width (M144) and takes hover and clicks; its
    // rounded highlight, badge and title are drawn after it, the title at
    // the row's indentation.
    let row_start = ui.cursor_screen_pos();
    let _token = {
        let _square_highlight = hide_square_row_highlight(ui);
        TreeNode::new("note")
            .label::<&str, _>("")
            .flags(flags)
            .push(ui)
    };
    draw_row_state(ui, context.theme, context.selected == Some(node));
    let row_start = [row_start[0], ui.item_rect_min()[1]];
    record_row(ui, node, RowKind::Note, row_start[0], context);
    // Ask the tree node itself: it spans the whole row, and once a click
    // makes it the active item, `is_window_hovered` would report false.
    let is_row_hovered = is_item_hovered(ui);
    let is_renaming = context
        .rename
        .as_ref()
        .is_some_and(|rename| rename.node == node);

    // Notes have no arrow: the title starts where a group's arrow does.
    let title_x = row_start[0] + context.frame_padding[0];
    let title_end = draw_note_markers(ui, note, row_start, is_row_hovered, context);
    ui.same_line_with_spacing(0.0, 0.0);
    ui.set_cursor_screen_pos([title_x, row_start[1]]);
    if is_renaming {
        draw_rename_input(ui, row_start, [title_x, title_end], context);
        return;
    }
    let label = truncate_to_width(ui, &note.title, title_end - title_x);
    ui.text(&label);
    if let Some(search) = context.search {
        mark_search_hits(
            ui,
            context.theme,
            &label,
            search
                .query
                .hit_ranges(&label)
                .into_iter()
                .map(|range| (range, HitMark::Filled)),
            [title_x, row_start[1] + context.frame_padding[1]],
        );
    }
    // Interaction belongs to the whole row (the tree node item).
    if is_row_hovered {
        handle_hovered_row(ui, node, &note.title, &label, context);
    }
    context_menu::draw_context_menu(ui, node, Some(note), context);
}

/// Draws the markers at the right edge of a note row, right to left: its
/// badge, the mark for unsaved changes kept as a draft (M185) and the number
/// of search hits in its content (M122), each with its tooltip while the row
/// is hovered. Returns the screen x where the title must end.
fn draw_note_markers(
    ui: &Ui,
    note: &Note,
    row_start: [f32; 2],
    is_row_hovered: bool,
    context: &mut RowContext<'_, '_>,
) -> f32 {
    let content_end = row_content_end(ui, context);
    let mut title_end = content_end;
    if let Some(badge) = (context.extras.badge)(note) {
        let (badge_start, is_hovered) = draw_row_icon(
            ui,
            badge.icon,
            context.theme.text.secondary,
            content_end,
            row_start[1] + context.frame_padding[1],
        );
        // The rectangle alone also matches below an open menu.
        if is_hovered && is_row_hovered {
            (context.extras.badge_tooltip)(ui, note);
        }
        title_end = badge_start - context.theme.spacing.small;
    }
    // Unsaved changes kept as a draft (M185).
    if context.extras.unsaved_changes.contains(note.id) {
        let (mark_start, is_hovered) = draw_row_icon(
            ui,
            Icon::TRIANGLE_ALERT,
            context.theme.status.warning,
            title_end,
            row_start[1] + context.frame_padding[1],
        );
        if is_hovered && is_row_hovered {
            draw_tooltip(ui, context.theme, context.extras.unsaved_changes.tooltip);
        }
        title_end = mark_start - context.theme.spacing.small;
    }
    // A note matching through its content shows its hit count (M122).
    if let Some(search) = context.search
        && let Some(hits) = search.filter.content_hits(note.id)
    {
        let mut count_buffer = [0; 20];
        let count = format_count(hits, &mut count_buffer);
        let count_x = title_end - ui.calc_text_size(count)[0];
        window_draw_list(ui).add_text(
            [count_x, row_start[1] + context.frame_padding[1]],
            context.theme.text.secondary.to_array(),
            count,
        );
        if is_row_hovered && ui.io().mouse_pos[0] >= count_x {
            draw_tooltip(
                ui,
                context.theme,
                search.texts.content_hits.replace("{count}", count),
            );
        }
        title_end = count_x - context.theme.spacing.small;
    }
    title_end
}

/// Handles a hovered row: selection, rename on double-click, context menu,
/// and the full title as tooltip if it was truncated.
fn handle_hovered_row(
    ui: &Ui,
    node: NodeId,
    title: &str,
    label: &str,
    context: &mut RowContext<'_, '_>,
) {
    if label != title && !label.is_empty() {
        draw_tooltip(ui, context.theme, title);
    }
    if ui.is_mouse_double_clicked(MouseButton::Left) {
        context.rename_steps.push(RenameStep::Start(node));
    } else if ui.is_mouse_clicked(MouseButton::Left) {
        if ui.is_item_toggled_open() {
            // The arrow toggled the group: it selects like the rest of the row.
            context.actions.push(NoteListAction::Select(node));
        } else {
            // Acts on release unless it becomes a drag (M106).
            context.pressed = Some(node);
        }
    }
    if ui.is_mouse_clicked(MouseButton::Right) {
        context.actions.push(NoteListAction::Select(node));
        ui.open_popup("context");
    }
}

/// Records the rectangle of the row just drawn (the tree node item) for the
/// drop zones (M107). `left` is where the row's indented content starts:
/// the node spans the full width (M144), the drop line keeps the level.
fn record_row(ui: &Ui, node: NodeId, kind: RowKind, left: f32, context: &mut RowContext<'_, '_>) {
    let [min, max] = [ui.item_rect_min(), ui.item_rect_max()];
    if *context.reveal == Some(node) {
        // Scroll the list so the revealed row sits in its middle (M185).
        ui.set_scroll_from_pos_y_with_ratio(
            f32::midpoint(min[1], max[1]) - ui.window_pos()[1],
            0.5,
        );
        *context.reveal = None;
    }
    context.rows.push(RowRect {
        node,
        kind,
        left,
        right: max[0],
        top: min[1],
        bottom: max[1],
    });
}

/// Draws a row marker icon in `color` ending at `end`, its middle on the
/// middle of the title's capitals for a title drawn at `text_top`: the reset
/// badge (M193) and the unsaved-changes mark (M185). Returns its left edge
/// and whether the mouse is over it.
fn draw_row_icon(ui: &Ui, icon: Icon, color: Color, end: f32, text_top: f32) -> (f32, bool) {
    let line_height = ui.text_line_height();
    let text_middle = capital_middle().unwrap_or(line_height / 2.0);
    let _icons = FontScope::push(ui, FontRole::Icons);
    let mut buffer = [0; 4];
    let glyph = icon.as_str(&mut buffer);
    let width = ui.calc_text_size(glyph)[0];
    let ink = icon_visible_box(icon.glyph(), ui.text_line_height());
    let left = (end - width).round();
    let top = (text_top + text_middle - f32::midpoint(ink[1], ink[3])).round();
    window_draw_list(ui).add_text([left, top], color.to_array(), glyph);
    let is_hovered = ui.is_mouse_hovering_rect([left, top], [end, top + ui.text_line_height()]);
    (left, is_hovered)
}
