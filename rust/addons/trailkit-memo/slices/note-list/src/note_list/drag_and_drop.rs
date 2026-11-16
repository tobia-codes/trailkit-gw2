//! Drag & drop in the note list (memo plan M106–M117, M125): following a
//! press until it becomes a drag, the drop line or group highlight, expanding
//! a collapsed group after a rest, scrolling near the edges and the floating
//! title. The geometry of drop targets lives in [`crate::tree_drag`].

use nexus::imgui::{MouseButton, Ui, WindowHoveredFlags};
use trailkit_memo_shared::note_tree::{GroupId, MoveCheck, NodeId, NotesDocument};
use trailkit_ui::{
    draw_list::window_draw_list,
    icons::Icon,
    text::{IconText, TextFlow, TextStyle, draw_icon_text, draw_text},
    theme::Theme,
    tooltip::{draw_tooltip, draw_tooltip_with_border},
};

use super::{ListSearch, NoteList, NoteListAction, NoteListOutput, node_title};
use crate::tree_drag::{
    DRAG_THRESHOLD, DropIndicator, DropTarget, EXPAND_DELAY_SECONDS, resolve_drop,
};

/// Opacity of the dragged row and its children while dragging (M106).
pub(super) const DRAGGED_ROW_ALPHA: f32 = 0.4;

/// Auto-scroll speed while dragging near the list's edge, in rows per
/// second at the very edge (M111).
const DRAG_SCROLL_ROWS_PER_SECOND: f32 = 12.0;

/// A press on a row that may become a drag (M106).
#[derive(Debug)]
pub(super) struct TreeDrag {
    /// The pressed note or group.
    pub(super) node: NodeId,
    /// Mouse position of the press.
    pub(super) pressed_at: [f32; 2],
    /// Whether the mouse moved far enough to drag; until then a release
    /// selects the row.
    pub(super) is_dragging: bool,
    /// Where a drop would go in this frame, if the mouse is over the list.
    pub(super) target: Option<DropTarget>,
    /// Whether the move to `target` is possible, changes nothing or changes
    /// the tree.
    pub(super) check: MoveCheck,
    /// The collapsed group hovered in its middle and since when (ImGui
    /// time); it expands after [`EXPAND_DELAY_SECONDS`] (M108).
    pub(super) collapsed_hover: Option<(GroupId, f64)>,
}

impl NoteList {
    /// Cancels a running drag (Escape, trailkit-ui skill §4). Returns
    /// whether one was running.
    pub fn cancel_drag(&mut self) -> bool {
        let is_dragging = self.drag.as_ref().is_some_and(|drag| drag.is_dragging);
        if is_dragging {
            self.drag = None;
        }
        is_dragging
    }

    /// Starts following a press on the row `pressed`, which may become a
    /// drag (M106). While searching, positions in the filtered list are
    /// ambiguous: no drag starts, and dragging shows why (M125).
    pub(super) fn start_press(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        pressed: Option<NodeId>,
        search: Option<&ListSearch<'_>>,
    ) {
        if let Some(search) = search {
            if pressed.is_some() {
                self.drag_blocked_by_search = true;
            }
            if !ui.is_mouse_down(MouseButton::Left) {
                self.drag_blocked_by_search = false;
            } else if self.drag_blocked_by_search
                && ui.is_mouse_dragging_with_threshold(MouseButton::Left, DRAG_THRESHOLD)
            {
                draw_tooltip(ui, theme, search.texts.clear_search_to_move);
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
    }

    /// Follows a drag over the list (M106–M111): starts it once the mouse
    /// moved [`DRAG_THRESHOLD`], finds the drop target, draws the drop line
    /// or group highlight, expands a collapsed group after a rest and
    /// scrolls near the list's edges. Runs inside the list's window after
    /// its rows; `indent` is the width of one tree level.
    pub(super) fn track_drag(
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
    /// the list may not be drawn (list hidden).
    pub fn finish_drag(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        document: &NotesDocument,
    ) -> Option<NoteListAction> {
        let drag = self.drag.as_mut()?;
        if !ui.is_mouse_down(MouseButton::Left) {
            let action = if !drag.is_dragging {
                Some(match drag.node {
                    NodeId::Group(group) => NoteListAction::ClickGroup(group),
                    NodeId::Note(_) => NoteListAction::Select(drag.node),
                })
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
        // The list may not be drawn next frame (list hidden); its target
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
        draw_tooltip_with_border(ui, theme, border, || {
            if is_impossible {
                // A status text in the danger color (Q277, Q325).
                draw_icon_text(
                    ui,
                    theme,
                    &IconText {
                        icon: Icon::BAN,
                        icon_color: theme.status.danger,
                        text: title,
                        style: TextStyle::Body,
                        text_color: Some(theme.status.danger),
                        flow: TextFlow::SingleLine,
                    },
                );
            } else {
                draw_text(ui, theme, TextStyle::Body, title);
            }
        });
        None
    }
}

/// Draws the drop line between rows or the highlight of a collapsed group
/// (trailkit-ui skill §4) into the list's window.
fn draw_drop_indicator(ui: &Ui, theme: &Theme, indicator: DropIndicator) {
    let draw_list = window_draw_list(ui);
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
