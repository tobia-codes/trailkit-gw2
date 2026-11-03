//! The note list of the memo main window: list header, tree rows, rename,
//! context menu, "Move to…", search hits and drag & drop (M106–M117).

use std::borrow::Cow;

use jiff::Timestamp;
use nexus::imgui::{
    ChildWindow, Condition, Key, MenuItem, MouseButton, StyleColor, StyleVar, TreeNode,
    TreeNodeFlags, Ui, WindowHoveredFlags,
};
use trailkit_ui::{
    button::{button_width, draw_button, draw_icon_button},
    font_registry::FontScope,
    fonts::FontRole,
    glyph_metrics::glyph_ink_box,
    icons::Icon,
    menu::{begin_submenu, draw_menu_item},
    selection_bar::draw_selection_bar,
    separator::draw_separator,
    text_search::SearchQuery,
    theme::Theme,
    tree_arrow::draw_tree_arrow,
};

use crate::tree_drag::{
    DRAG_THRESHOLD, DropIndicator, EXPAND_DELAY_SECONDS, RowKind, RowRect, resolve_drop,
};
use trailkit_memo_reset_schedule::{
    i18n::ResetText,
    note_kind::{KIND_OPTIONS, reset_badge, reset_tooltip_lines},
};
use trailkit_memo_search::{i18n::SearchText, search_field::SearchFieldTexts};
use trailkit_memo_shared::i18n::LocalizedText;
use trailkit_memo_shared::{
    i18n::{Language, Text},
    note_tree::{
        Group, GroupId, InsertionPoint, MoveCheck, NodeId, Note, NoteKind, TreeNode as Node,
    },
    row_badge::RowBadge,
};

use super::{
    DRAG_SCROLL_ROWS_PER_SECOND, DRAGGED_ROW_ALPHA, MemoWindow, TreeContext, TreeDrag,
    WindowAction, find_title,
};
use super::{
    layout::{centered_text, format_count, indent_to_center, truncate_to_width},
    note_pane::{icon_button_width, note_list_icon},
};

impl MemoWindow {
    /// Draws the fixed header (search field, toolbar) and below it the
    /// scrolling tree (M129). `menu` is the tooltip of a leading menu button
    /// (drawer only, so it closes where it opened).
    pub(super) fn draw_tree_pane(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        menu: Option<Text>,
    ) {
        let search_texts = SearchFieldTexts {
            hint: SearchText::SearchHint.get(language),
            clear: SearchText::ClearSearch.get(language),
        };
        if self.search_field.draw(ui, theme, search_texts) {
            // Apply the new text in this frame already.
            self.update_search();
        }
        draw_toolbar(ui, language, theme, menu, &mut self.actions);
        // Search field and buttons stay put; only the tree below the
        // separator scrolls, with its own scrollbar (M129).
        ChildWindow::new("note_list")
            .size([0.0, 0.0])
            .draw_background(false)
            .build(ui, || self.draw_note_list(ui, language, theme));
    }

    /// Draws the scrolling part of the note list: the tree (or the empty
    /// search result), drag & drop and the context menu of empty spots.
    pub(super) fn draw_note_list(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        self.tree_rows.clear();
        if let Some(current) = &self.search.current
            && !current.results.has_hits()
        {
            draw_no_search_results(ui, language, theme, &current.text, &mut self.actions);
            return;
        }
        let style = ui.clone_style();
        let mut context = TreeContext {
            document: &self.document,
            selected: self.ui_state.selected,
            dragged: self
                .drag
                .as_ref()
                .filter(|drag| drag.is_dragging)
                .map(|drag| drag.node),
            pressed: None,
            rows: &mut self.tree_rows,
            rename: &mut self.rename,
            actions: &mut self.actions,
            language,
            theme,
            frame_padding: style.frame_padding,
            search: self
                .search
                .current
                .as_ref()
                .map(|current| (&current.query, &current.results)),
            collapsed_in_search: &self.collapsed_in_search,
        };
        draw_nodes(ui, &self.document.root, &mut context);
        let is_searching = context.search.is_some();
        if is_searching {
            // Positions in a filtered list are ambiguous: no drag & drop
            // while searching (M125).
            if context.pressed.is_some() {
                self.drag_blocked_by_search = true;
            }
            if !ui.is_mouse_down(MouseButton::Left) {
                self.drag_blocked_by_search = false;
            } else if self.drag_blocked_by_search
                && ui.is_mouse_dragging_with_threshold(MouseButton::Left, DRAG_THRESHOLD)
            {
                ui.tooltip_text(SearchText::ClearSearchToMove.get(language));
            }
        } else if let Some(node) = context.pressed {
            self.drag = Some(TreeDrag {
                node,
                pressed_at: ui.io().mouse_pos,
                is_dragging: false,
                target: None,
                check: MoveCheck::Unchanged,
                collapsed_hover: None,
            });
        }
        self.track_drag(ui, theme, style.indent_spacing);
        // Right-click on an empty spot of the list (M96): create at the end
        // of the root level.
        if ui.is_window_hovered()
            && !ui.is_any_item_hovered()
            && ui.is_mouse_clicked(MouseButton::Right)
        {
            ui.open_popup("list_context");
        }
        let actions = &mut self.actions;
        ui.popup("list_context", || {
            draw_create_menu_items(ui, language, InsertionPoint::End(None), actions);
        });
    }

    /// Follows a drag over the note list (M106–M111): starts it once the
    /// mouse moved [`DRAG_THRESHOLD`], finds the drop target, draws the drop
    /// line or group highlight, expands a collapsed group after a rest and
    /// scrolls near the list's edges. Runs inside the list's window after
    /// its rows; `indent` is the width of one tree level.
    pub(super) fn track_drag(&mut self, ui: &Ui, theme: &Theme, indent: f32) {
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
            .then(|| resolve_drop(&self.tree_rows, mouse[1], indent))
            .flatten();
        drag.check = drag.target.map_or(MoveCheck::Unchanged, |target| {
            self.document.check_move(drag.node, target.point)
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
                    self.actions.push(WindowAction::SetExpanded(group, true));
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

    /// Ends a press or drag on the note list when the left button is
    /// released: a press without drag selects the row, a drag moves the
    /// node to its drop target (M106–M111). While dragging, the title floats
    /// next to the cursor with a gold border, red with a ban icon over an
    /// impossible target (M109, M116).
    pub(super) fn finish_drag(&mut self, ui: &Ui, theme: &Theme) {
        let Some(drag) = &mut self.drag else {
            return;
        };
        if !ui.is_mouse_down(MouseButton::Left) {
            if !drag.is_dragging {
                self.actions.push(WindowAction::Select(drag.node));
            } else if drag.check == MoveCheck::Changes
                && let Some(target) = drag.target
            {
                self.actions
                    .push(WindowAction::Move(drag.node, target.point));
            }
            self.drag = None;
            return;
        }
        if !drag.is_dragging {
            return;
        }
        let title = find_title(&self.document.root, drag.node).unwrap_or_default();
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
    }
}

/// Draws the "new note" / "new group" icon buttons, led by the menu button
/// when `menu` holds its tooltip.
pub(super) fn draw_toolbar(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    menu: Option<Text>,
    actions: &mut Vec<WindowAction>,
) {
    if let Some(tooltip) = menu {
        if draw_icon_button(ui, theme, note_list_icon(tooltip), tooltip.get(language)) {
            actions.push(WindowAction::ToggleNoteList);
        }
        ui.same_line();
    }
    let buttons = [
        (Icon::FILE_PLUS, Text::NewNote, WindowAction::NewNote(None)),
        (
            Icon::FOLDER_PLUS,
            Text::NewGroup,
            WindowAction::NewGroup(None),
        ),
    ];
    // Both buttons share the row in equal widths, never narrower than an
    // icon button; without room for both they stack (M128).
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
            actions.push(action);
        }
    }
    draw_separator(ui, theme);
}

/// Draws `nodes` and, recursively, the children of expanded groups; during a
/// search only the rows it shows (M118).
pub(super) fn draw_nodes(ui: &Ui, nodes: &[Node], context: &mut TreeContext<'_>) {
    for node in nodes {
        if let Some((_, results)) = context.search
            && !results.is_visible(node.id())
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
pub(super) fn draw_group(ui: &Ui, group: &Group, context: &mut TreeContext<'_>) {
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
    // During a search the count is the matching notes (M121) and groups
    // are expanded unless collapsed for this search only (M124).
    let (note_count, expanded) = match context.search {
        Some((_, results)) => (
            results.group_count(group.id),
            !context.collapsed_in_search.contains(&group.id),
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
            TreeNode::new("group")
                .label::<&str, _>("")
                .flags(flags)
                .opened(expanded, Condition::Always)
                .push(ui)
        };
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
        if let Some((query, _)) = context.search {
            draw_title_hits(ui, context.theme, query, &label, title_position);
        }
        ui.get_window_draw_list().add_text(
            [
                row_start[0] + title_offset,
                row_start[1] + context.frame_padding[1],
            ],
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
    record_row(ui, node, kind, context);
    ui.get_window_draw_list().add_text(
        [count_x, row_start[1] + context.frame_padding[1]],
        context.theme.text.secondary.to_array(),
        count,
    );
    let is_open = token.is_some();
    handle_row_interaction(ui, node, &group.title, &label, context);
    if is_open != expanded {
        let action = if context.search.is_some() {
            WindowAction::SetCollapsedInSearch(group.id, !is_open)
        } else {
            WindowAction::SetExpanded(group.id, is_open)
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
pub(super) fn group_title_offset(ui: &Ui, context: &TreeContext<'_>) -> f32 {
    ui.current_font_size() + 2.0 * context.frame_padding[0]
}

/// Screen x where row content (title, badges) ends: the row's right edge
/// minus the horizontal frame padding, mirroring the inset on the left.
pub(super) fn row_content_end(ui: &Ui, context: &TreeContext<'_>) -> f32 {
    ui.window_content_region_max()[0] + ui.window_pos()[0] - context.frame_padding[0]
}

/// Draws one note row (M49): the title flush left, a reset badge at the
/// right edge.
pub(super) fn draw_note_row(ui: &Ui, note: &Note, context: &mut TreeContext<'_>) {
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
    if context.selected == Some(node) {
        draw_selection_bar(ui, context.theme);
    }
    let row_start = ui.item_rect_min();
    record_row(ui, node, RowKind::Note, context);
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
    if let Some(badge) = reset_badge(note.kind, context.language, context.theme) {
        let (badge_start, is_hovered) =
            draw_row_badge(ui, badge, [content_end, row_start[1]], context.theme);
        if is_hovered && let Some(period) = note.kind.period() {
            // The same kind and countdown as the note toolbar (M95),
            // formatted only while hovered.
            let lines = reset_tooltip_lines(period, context.language, Timestamp::now());
            ui.tooltip(|| {
                for line in &lines {
                    ui.text(line);
                }
            });
        }
        title_end = badge_start - context.theme.spacing.small;
    }
    // A note matching through its content shows its hit count (M122).
    if let Some(hits) = context
        .search
        .and_then(|(_, results)| results.content_hits(note.id))
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
            ui.tooltip_text(
                SearchText::ContentHits
                    .get(context.language)
                    .replace("{count}", count),
            );
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
    if let Some((query, _)) = context.search {
        draw_title_hits(
            ui,
            context.theme,
            query,
            &label,
            [title_x, row_start[1] + context.frame_padding[1]],
        );
    }
    ui.text(&label);
    // Interaction belongs to the whole row (the tree node item).
    if is_row_hovered {
        handle_hovered_row(ui, node, &note.title, &label, context);
    }
    draw_context_menu(ui, node, Some(note.kind), context);
}

/// Click, double-click, context menu and tooltip of the last drawn tree row.
pub(super) fn handle_row_interaction(
    ui: &Ui,
    node: NodeId,
    title: &str,
    label: &str,
    context: &mut TreeContext<'_>,
) {
    if ui.is_item_hovered() {
        handle_hovered_row(ui, node, title, label, context);
    }
    draw_context_menu(ui, node, None, context);
}

/// Handles a hovered row: selection, rename on double-click, context menu,
/// and the full title as tooltip if it was truncated.
pub(super) fn handle_hovered_row(
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
        // Selects on release unless it becomes a drag (M106), so dragging a
        // note keeps the drawer open (M113).
        context.pressed = Some(node);
    }
    if ui.is_mouse_clicked(MouseButton::Right) {
        context.actions.push(WindowAction::Select(node));
        ui.open_popup("context");
    }
}

/// Draws the context menu of a row if it is open (must run every frame).
pub(super) fn draw_context_menu(
    ui: &Ui,
    node: NodeId,
    kind: Option<NoteKind>,
    context: &mut TreeContext<'_>,
) {
    let (language, theme) = (context.language, context.theme);
    ui.popup("context", || {
        if MenuItem::new(Text::Rename.get(language)).build(ui) {
            context.actions.push(WindowAction::StartRename(node));
        }
        // Notes get the reset type submenu with the current kind checked
        // (M86); groups have no kind.
        if let (NodeId::Note(note), Some(kind)) = (node, kind)
            && let Some(_submenu) = begin_submenu(ui, theme, ResetText::ResetType.get(language))
        {
            for (period, text) in KIND_OPTIONS {
                let is_current = kind.period() == period;
                if draw_menu_item(ui, theme, text.get(language), is_current) && !is_current {
                    context
                        .actions
                        .push(WindowAction::SetResetPeriod(note, period));
                }
            }
        }
        if let Some(_submenu) = begin_submenu(ui, theme, Text::MoveTo.get(language)) {
            draw_move_targets(ui, node, context);
        }
        // Only groups offer creating, into themselves (M96, M97).
        if let NodeId::Group(group) = node {
            ui.separator();
            draw_create_menu_items(
                ui,
                language,
                InsertionPoint::End(Some(group)),
                context.actions,
            );
        }
    });
}

/// Draws the "Move to" submenu items (M112): "Top level" and every group as
/// an indented flat list; the current place is checked, the moved group
/// and its descendants are left out. A choice moves to the end.
pub(super) fn draw_move_targets(ui: &Ui, node: NodeId, context: &mut TreeContext<'_>) {
    let (theme, language) = (context.theme, context.language);
    let current = context.document.parent(node).flatten();
    if draw_menu_item(ui, theme, Text::TopLevel.get(language), current.is_none())
        && current.is_some()
    {
        context
            .actions
            .push(WindowAction::Move(node, InsertionPoint::End(None)));
    }
    draw_move_target_groups(ui, &context.document.root, node, current, context);
}

/// Draws one "Move to" item per group in `nodes`, nested groups indented
/// below their parent (M112); skips the moved group `node` with its
/// descendants. `current` is the moved node's parent.
pub(super) fn draw_move_target_groups(
    ui: &Ui,
    nodes: &[Node],
    node: NodeId,
    current: Option<GroupId>,
    context: &mut TreeContext<'_>,
) {
    let indent = context.theme.list.indent;
    for child in nodes {
        let Node::Group(group) = child else {
            continue;
        };
        if NodeId::Group(group.id) == node {
            continue;
        }
        let _id = ui.push_id(group.id.imgui_id());
        ui.indent_by(indent);
        let is_current = current == Some(group.id);
        if draw_menu_item(ui, context.theme, &group.title, is_current) && !is_current {
            context.actions.push(WindowAction::Move(
                node,
                InsertionPoint::End(Some(group.id)),
            ));
        }
        draw_move_target_groups(ui, &group.children, node, current, context);
        ui.unindent_by(indent);
    }
}

/// Records the rectangle of the row just drawn (the tree node item) for the
/// drop zones (M107).
pub(super) fn record_row(ui: &Ui, node: NodeId, kind: RowKind, context: &mut TreeContext<'_>) {
    let [min, max] = [ui.item_rect_min(), ui.item_rect_max()];
    context.rows.push(RowRect {
        node,
        kind,
        left: min[0],
        right: max[0],
        top: min[1],
        bottom: max[1],
    });
}

/// Draws the drop line between rows or the highlight of a collapsed group
/// (trailkit-ui skill §4) into the list's window.
pub(super) fn draw_drop_indicator(ui: &Ui, theme: &Theme, indicator: DropIndicator) {
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
pub(super) fn scroll_near_edges(ui: &Ui, theme: &Theme) {
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

/// Draws "New note" and "New group" menu items that create at `point`
/// (M96); the new entry is renamed right away (M43).
pub(super) fn draw_create_menu_items(
    ui: &Ui,
    language: Language,
    point: InsertionPoint,
    actions: &mut Vec<WindowAction>,
) {
    if MenuItem::new(Text::NewNote.get(language)).build(ui) {
        actions.push(WindowAction::NewNote(Some(point)));
    }
    if MenuItem::new(Text::NewGroup.get(language)).build(ui) {
        actions.push(WindowAction::NewGroup(Some(point)));
    }
}

/// Draws the inline rename input and records commit or cancel (M44).
///
/// Inline editing in lists (`lay-inline-edit`): the input has no background
/// of its own, so the row keeps one uniform color; a focus border frames the
/// whole row starting at `row_start`. The input text spans `title_span`
/// (start and end x), where the title was drawn before renaming, so badges
/// next to it stay visible.
pub(super) fn draw_rename_input(
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

/// Draws a row badge (M45, M49, M105): an outlined rounded box with a
/// letter. The badge ends at `anchor[0]` and is vertically centered in the
/// row starting at `anchor[1]` (one frame tall). Returns the badge's left
/// edge and whether the mouse is over it.
pub(super) fn draw_row_badge(
    ui: &Ui,
    badge: RowBadge,
    anchor: [f32; 2],
    theme: &Theme,
) -> (f32, bool) {
    let RowBadge { letter, color } = badge;
    let line_height = ui.text_line_height();
    let padding = theme.spacing.extra_small;
    // Whole pixels, so the outline stays sharp and the letter can be
    // centered exactly.
    let width = (ui.calc_text_size("W")[0] + 2.0 * padding).round();
    let start = [
        (anchor[0] - width).round(),
        (anchor[1] + ((ui.frame_height() - line_height) / 2.0).max(0.0)).round(),
    ];
    let height = line_height.round();
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
    // A bold letter, centered by its own ink box, not by its advance width
    // or the line box (the text font sits low in it, M85, M105).
    {
        let _bold = FontScope::push(ui, FontRole::BodyBold);
        let letter_size = ui.calc_text_size(letter);
        let ink = letter.chars().next().and_then(glyph_ink_box).unwrap_or([
            0.0,
            0.0,
            letter_size[0],
            letter_size[1],
        ]);
        let center = [start[0] + width / 2.0, start[1] + height / 2.0];
        draw_list.add_text(
            [
                (center[0] - (ink[0] + ink[2]) / 2.0).round(),
                (center[1] - (ink[1] + ink[3]) / 2.0).round(),
            ],
            color.to_array(),
            letter,
        );
    }
    (start[0], ui.is_mouse_hovering_rect(start, max))
}

/// Marks the search hits in a row title drawn at `position` with the
/// `accent.highlight` background (M122); call before drawing the text.
pub(super) fn draw_title_hits(
    ui: &Ui,
    theme: &Theme,
    query: &SearchQuery,
    title: &str,
    position: [f32; 2],
) {
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

/// The empty search result (M126): what was searched for and a button that
/// clears the search.
pub(super) fn draw_no_search_results(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    text: &str,
    actions: &mut Vec<WindowAction>,
) {
    let width = ui.content_region_avail()[0];
    ui.dummy([0.0, theme.spacing.medium]);
    let message = SearchText::NoSearchResults
        .get(language)
        .replace("{query}", text.trim());
    {
        let _secondary = ui.push_style_color(StyleColor::Text, theme.text.secondary.to_array());
        centered_text(ui, &message, width);
    }
    ui.dummy([0.0, theme.spacing.small]);
    let label = SearchText::ClearSearch.get(language);
    indent_to_center(ui, button_width(ui, theme, label), width);
    if draw_button(ui, theme, label) {
        actions.push(WindowAction::ClearSearch);
    }
}
