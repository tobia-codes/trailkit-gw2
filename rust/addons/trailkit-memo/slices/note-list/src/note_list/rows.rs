//! The rows of the note list: groups and notes with their titles, counts,
//! markers and search hits, drawn by the shared list row (ROADMAP R155), row
//! interaction and the inline rename input.

use trailkit_ui::controls::tooltip::draw_tooltip;

use nexus::imgui::{MouseButton, StyleVar, Ui};
use trailkit_memo_shared::note_tree::{Group, NodeId, Note, TreeNode as Node};
use trailkit_ui::{
    lists::list_row::{ListRow, ListRowResponse, RowChevron, RowMarker, draw_list_row},
    text::TextStyle,
    text::font_registry::FontScope,
    text::fonts::FontRole,
    text::icons::Icon,
    text::text_layout::format_count,
};

use super::{
    NoteListAction, RowContext, context_menu,
    drag_and_drop::DRAGGED_ROW_ALPHA,
    rename::{RenameStep, draw_rename_input},
};
use crate::tree_drag::{RowKind, RowRect};

/// Draws `nodes` at tree depth `level` and, recursively, the children of
/// expanded groups; during a search only the rows it shows (M118).
pub(super) fn draw_nodes(ui: &Ui, nodes: &[Node], level: usize, context: &mut RowContext<'_, '_>) {
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
            Node::Group(group) => draw_group(ui, group, level, context),
            Node::Note(note) => draw_note_row(ui, note, level, context),
        }
    }
}

/// Draws one group row and, if expanded, its children.
fn draw_group(ui: &Ui, group: &Group, level: usize, context: &mut RowContext<'_, '_>) {
    let _id = ui.push_id(group.id.imgui_id());
    let node = NodeId::Group(group.id);
    let is_renaming = is_renaming(node, context);
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
    // A collapsed group shows that a note inside it has unsaved changes
    // (M185); an expanded one leaves that to the note rows.
    let has_unsaved_changes = context.extras.unsaved_changes;
    let unsaved_mark = [RowMarker::Icon {
        icon: Icon::TRIANGLE_ALERT,
        color: context.theme.status.warning,
    }];
    let markers: &[RowMarker<'_>] =
        if !expanded && group.any_note(&|note| has_unsaved_changes.contains(note)) {
            &unsaved_mark
        } else {
            &[]
        };
    // M50: groups stand out by a bold title and their note count, shown
    // right-aligned in the secondary text color like other row info (M49).
    let response = draw_list_row(
        ui,
        context.theme,
        &ListRow {
            selected: context.selected == Some(node),
            title_style: TextStyle::BodyStrong,
            title_hits: context.search.map(|search| search.query),
            renaming: is_renaming,
            level,
            chevron: if expanded {
                RowChevron::Open
            } else {
                RowChevron::Closed
            },
            markers,
            trailing: Some(count),
            ..ListRow::new("##group", &group.title)
        },
    );
    if response.hovered_marker.is_some() {
        draw_tooltip(ui, context.theme, has_unsaved_changes.tooltip);
    }
    let kind = if expanded {
        RowKind::ExpandedGroup(group.id)
    } else {
        RowKind::CollapsedGroup(group.id)
    };
    record_row(ui, node, kind, &response, context);
    let is_hovered = response.hovered;
    if is_hovered {
        handle_hovered_row(ui, node, response.chevron_clicked, context);
    }
    context_menu::draw_context_menu(ui, node, None, context);
    if response.chevron_clicked {
        let action = if context.search.is_some() {
            NoteListAction::SetCollapsedInSearch(group.id, expanded)
        } else {
            NoteListAction::SetExpanded(group.id, !expanded)
        };
        context.actions.push(action);
    }
    if is_renaming {
        let _bold = FontScope::push(ui, FontRole::BodyBold);
        draw_rename_over(ui, &response, context);
    }
    if expanded {
        draw_nodes(ui, &group.children, level + 1, context);
    }
}

/// Whether `node` is being renamed.
fn is_renaming(node: NodeId, context: &RowContext<'_, '_>) -> bool {
    context
        .rename
        .as_ref()
        .is_some_and(|rename| rename.node == node)
}

/// Draws the rename input over the title of the row just drawn, on the
/// row's line; its focus border starts where the row's content does.
fn draw_rename_over(ui: &Ui, response: &ListRowResponse, context: &mut RowContext<'_, '_>) {
    ui.same_line_with_spacing(0.0, 0.0);
    let row_start = [response.content_left, response.row_rect[0][1]];
    let title_span = [response.title_rect[0][0], response.title_rect[1][0]];
    draw_rename_input(ui, row_start, title_span, context);
}

/// What a marker of a note row stands for, to draw the right tooltip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoteMarker {
    /// The number of search hits in the content (M122).
    ContentHits,
    /// Unsaved changes kept as a draft (M185).
    UnsavedChanges,
    /// The badge of another slice, e.g. the reset badge (M193).
    Badge,
}

/// Draws one note row (M49): the title at the row's indentation, at the
/// right end its markers: the number of search hits in its content (M122),
/// the mark for unsaved changes (M185) and its badge.
fn draw_note_row(ui: &Ui, note: &Note, level: usize, context: &mut RowContext<'_, '_>) {
    let _id = ui.push_id(note.id.imgui_id());
    let node = NodeId::Note(note.id);
    let is_renaming = is_renaming(node, context);
    let mut count_buffer = [0; 20];
    let content_hits = context
        .search
        .and_then(|search| search.filter.content_hits(note.id))
        .map(|hits| format_count(hits, &mut count_buffer));
    let mut markers = [RowMarker::Text(""); 3];
    let mut kinds = [NoteMarker::Badge; 3];
    let mut marker_count = 0;
    let mut add = |marker, kind| {
        markers[marker_count] = marker;
        kinds[marker_count] = kind;
        marker_count += 1;
    };
    if let Some(count) = content_hits {
        add(RowMarker::Text(count), NoteMarker::ContentHits);
    }
    if context.extras.unsaved_changes.contains(note.id) {
        let warning = context.theme.status.warning;
        let icon = RowMarker::Icon {
            icon: Icon::TRIANGLE_ALERT,
            color: warning,
        };
        add(icon, NoteMarker::UnsavedChanges);
    }
    if let Some(badge) = (context.extras.badge)(note) {
        let secondary = context.theme.text.secondary;
        let icon = RowMarker::Icon {
            icon: badge.icon,
            color: secondary,
        };
        add(icon, NoteMarker::Badge);
    }
    // The row spans the full width (M144) and takes hover and clicks.
    let response = draw_list_row(
        ui,
        context.theme,
        &ListRow {
            selected: context.selected == Some(node),
            title_hits: context.search.map(|search| search.query),
            renaming: is_renaming,
            level,
            markers: &markers[..marker_count],
            ..ListRow::new("##note", &note.title)
        },
    );
    record_row(ui, node, RowKind::Note, &response, context);
    match response.hovered_marker.map(|index| kinds[index]) {
        Some(NoteMarker::Badge) => (context.extras.badge_tooltip)(ui, note),
        Some(NoteMarker::UnsavedChanges) => {
            draw_tooltip(ui, context.theme, context.extras.unsaved_changes.tooltip);
        }
        Some(NoteMarker::ContentHits) => {
            if let (Some(search), Some(count)) = (context.search, content_hits) {
                let tooltip = search.texts.content_hits.replace("{count}", count);
                draw_tooltip(ui, context.theme, tooltip);
            }
        }
        None => {}
    }
    if is_renaming {
        draw_rename_over(ui, &response, context);
        return;
    }
    if response.hovered {
        handle_hovered_row(ui, node, false, context);
    }
    context_menu::draw_context_menu(ui, node, Some(note), context);
}

/// Handles a hovered row: selection, rename on double-click and the
/// context menu. A click on the chevron (`chevron_clicked`) selects at once.
fn handle_hovered_row(
    ui: &Ui,
    node: NodeId,
    chevron_clicked: bool,
    context: &mut RowContext<'_, '_>,
) {
    if ui.is_mouse_double_clicked(MouseButton::Left) {
        context.rename_steps.push(RenameStep::Start(node));
    } else if ui.is_mouse_clicked(MouseButton::Left) {
        if chevron_clicked {
            // The chevron toggled the group: it selects like the rest of the row.
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

/// Records the rectangle of the row just drawn for the drop zones (M107):
/// the row spans the full width (M144), its left edge for the drop line is
/// where its indented content starts.
fn record_row(
    ui: &Ui,
    node: NodeId,
    kind: RowKind,
    response: &ListRowResponse,
    context: &mut RowContext<'_, '_>,
) {
    let [min, max] = response.row_rect;
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
        left: response.content_left,
        right: max[0],
        top: min[1],
        bottom: max[1],
    });
}
