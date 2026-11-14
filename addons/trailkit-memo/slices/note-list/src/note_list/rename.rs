//! Inline rename of a note or group in the list (memo plan M43, M44): the
//! rename state, the steps requested while drawing and the input drawn over
//! the row's title.

use nexus::imgui::{Key, StyleVar, Ui};
use trailkit_memo_shared::note_tree::{NodeId, NotesDocument};
use trailkit_ui::{draw_list::window_draw_list, style_scope::StyleScope};

use super::{NoteList, NoteListAction, RowContext, node_title};

/// An inline rename in progress (M44).
#[derive(Debug)]
pub(super) struct RenameState {
    /// The note or group being renamed.
    pub(super) node: NodeId,
    /// The edited title.
    buffer: String,
    /// Whether keyboard focus was already given to the input.
    focused: bool,
}

/// A change of the rename state requested while drawing; applied after
/// all rows, so every row of a frame sees the same state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenameStep {
    /// Start renaming this node.
    Start(NodeId),
    /// Leave rename mode (committed or cancelled).
    End,
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

    /// Applies the rename changes requested while drawing, in order.
    pub(super) fn apply_rename_steps(&mut self, document: &NotesDocument) {
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
    context: &mut RowContext<'_, '_>,
) {
    let [title_x, title_end] = title_span;
    let Some(rename) = context.rename.as_mut() else {
        return;
    };
    let theme = context.theme;
    let row_end = ui.window_content_region_max()[0] + ui.window_pos()[0];
    window_draw_list(ui)
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
    let _background = StyleScope::transparent_frame(ui);
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
