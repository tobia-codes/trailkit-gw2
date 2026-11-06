//! The rows of the note list: groups and notes with their titles, counts,
//! badges and search hits, row interaction and the inline rename input.

use std::borrow::Cow;
use trailkit_ui::tooltip::draw_tooltip;

use nexus::imgui::{
    Condition, Key, MouseButton, StyleColor, StyleVar, TreeNode, TreeNodeFlags, Ui,
};
use trailkit_memo_shared::{
    note_tree::{Group, NodeId, Note, TreeNode as Node},
    row_badge::RowBadge,
};
use trailkit_ui::{
    font_registry::FontScope,
    fonts::FontRole,
    glyph_metrics::{draw_glyph_at, glyph_ink_box, glyph_visible_ink},
    selection_bar::{draw_row_highlight, draw_selection_bar, hide_square_row_highlight},
    text_layout::{format_count, truncate_to_width},
    text_search::SearchQuery,
    theme::Theme,
    tree_arrow::draw_tree_arrow,
};

use super::{DRAGGED_ROW_ALPHA, NoteListAction, RenameStep, RowContext, context_menu};
use crate::{
    badge_geometry::{center_matched_size, centered_pen},
    tree_drag::{RowKind, RowRect},
};

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
    let mut flags = TreeNodeFlags::OPEN_ON_ARROW
        | TreeNodeFlags::SPAN_FULL_WIDTH
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
        Some(search) => (
            search.filter.group_count(group.id),
            !search.collapsed.contains(&group.id),
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
            let _square_highlight = hide_square_row_highlight(ui);
            TreeNode::new("group")
                .label::<&str, _>("")
                .flags(flags)
                .opened(expanded, Condition::Always)
                .push(ui)
        };
        draw_row_highlight(ui, context.theme, context.selected == Some(node));
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
        if let Some(search) = context.search {
            draw_title_hits(ui, context.theme, search.query, &label, title_position);
        }
        ui.get_window_draw_list().add_text(
            title_position,
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
    record_row(ui, node, kind, row_start[0], context);
    ui.get_window_draw_list().add_text(
        [count_x, row_start[1] + context.frame_padding[1]],
        context.theme.text.secondary.to_array(),
        count,
    );
    let is_open = token.is_some();
    if ui.is_item_hovered() {
        handle_hovered_row(ui, node, &group.title, &label, context);
    }
    context_menu::draw_context_menu(ui, node, None, context);
    if is_open != expanded {
        let action = if context.search.is_some() {
            NoteListAction::SetCollapsedInSearch(group.id, !is_open)
        } else {
            NoteListAction::SetExpanded(group.id, is_open)
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
    draw_row_highlight(ui, context.theme, context.selected == Some(node));
    if context.selected == Some(node) {
        draw_selection_bar(ui, context.theme);
    }
    let row_start = [row_start[0], ui.item_rect_min()[1]];
    record_row(ui, node, RowKind::Note, row_start[0], context);
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
    let mut title_end = content_end;
    if let Some(badge) = (context.extras.badge)(note) {
        let (badge_start, is_hovered) =
            draw_row_badge(ui, badge, [content_end, row_start[1]], context.theme);
        // The rectangle alone also matches below an open menu.
        if is_hovered && is_row_hovered {
            (context.extras.badge_tooltip)(ui, note);
        }
        title_end = badge_start - context.theme.spacing.small;
    }
    // A note matching through its content shows its hit count (M122).
    if let Some(search) = context.search
        && let Some(hits) = search.filter.content_hits(note.id)
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
            draw_tooltip(ui, search.texts.content_hits.replace("{count}", count));
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
    if let Some(search) = context.search {
        draw_title_hits(
            ui,
            context.theme,
            search.query,
            &label,
            [title_x, row_start[1] + context.frame_padding[1]],
        );
    }
    ui.text(&label);
    // Interaction belongs to the whole row (the tree node item).
    if is_row_hovered {
        handle_hovered_row(ui, node, &note.title, &label, context);
    }
    context_menu::draw_context_menu(ui, node, Some(note), context);
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
        draw_tooltip(ui, title);
    }
    if ui.is_mouse_double_clicked(MouseButton::Left) {
        context.rename_steps.push(RenameStep::Start(node));
    } else if ui.is_mouse_clicked(MouseButton::Left) && !ui.is_item_toggled_open() {
        // Selects on release unless it becomes a drag (M106).
        context.pressed = Some(node);
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
    context.rows.push(RowRect {
        node,
        kind,
        left,
        right: max[0],
        top: min[1],
        bottom: max[1],
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
    context: &mut RowContext<'_, '_>,
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
    // Enter or leaving the input commits, Escape cancels.
    let commits = if entered {
        true
    } else if ui.is_item_deactivated() {
        !ui.is_key_pressed(Key::Escape)
    } else {
        return;
    };
    if commits {
        context
            .actions
            .push(NoteListAction::Rename(rename.node, rename.buffer.clone()));
    }
    context.rename_steps.push(RenameStep::End);
}

/// Draws a row badge (M45, M49, M105): an outlined rounded box with a
/// letter. The badge ends at `anchor[0]` and is vertically centered in the
/// row starting at `anchor[1]` (one frame tall). Returns the badge's left
/// edge and whether the mouse is over it.
fn draw_row_badge(ui: &Ui, badge: RowBadge, anchor: [f32; 2], theme: &Theme) -> (f32, bool) {
    let RowBadge { letter, color } = badge;
    let line_height = ui.text_line_height();
    let padding = theme.spacing.extra_small;
    let room = ui.calc_text_size("W")[0] + 2.0 * padding;
    // A bold letter, centered by its own ink box, not by its advance width
    // or the line box (the text font sits low in it, M85, M105).
    let _bold = FontScope::push(ui, FontRole::BodyBold);
    let letter_size = ui.calc_text_size(letter);
    let character = letter.chars().next();
    let ink = character
        .and_then(glyph_visible_ink)
        .or_else(|| character.and_then(glyph_ink_box))
        .unwrap_or([0.0, 0.0, letter_size[0], letter_size[1]]);
    // Whole pixels, so the outline stays sharp. Every badge has the same
    // width, so badges line up; the letter is placed to the sub-pixel below
    // instead (M132). Capitals share their height, so the height takes the
    // parity of the ink centre and the letter sits on whole pixels.
    let width = room.round();
    let height = center_matched_size(line_height, f32::midpoint(ink[1], ink[3]));
    let start = [
        (anchor[0] - width).round(),
        (anchor[1] + ((ui.frame_height() - height) / 2.0).max(0.0)).round(),
    ];
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
    let pen = [
        start[0] + width / 2.0 - f32::midpoint(ink[0], ink[2]),
        centered_pen(start[1], height, ink[1], ink[3]),
    ];
    if !character.is_some_and(|character| draw_glyph_at(character, pen, color.to_array())) {
        draw_list.add_text([pen[0].round(), pen[1]], color.to_array(), letter);
    }
    (start[0], ui.is_mouse_hovering_rect(start, max))
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
