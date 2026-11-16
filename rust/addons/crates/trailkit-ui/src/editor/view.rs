//! The ImGui side of the editor (memo plan M58–M60): draws the wrapped,
//! highlighted text with selection and cursor, and turns mouse and keyboard
//! input into buffer operations (Windows only).
//!
//! While the editor has the focus it keeps an invisible ImGui text input
//! active (the "keyboard sink"): Nexus passes key presses such as arrows,
//! Backspace or Enter to ImGui instead of the game only while a real ImGui
//! text input is active (verified in game 2026-11-01). The editor reads the
//! typed characters before the sink consumes them, and writes the clipboard
//! after it. The layout is rebuilt only when the text, the width or the font
//! size changes; only visible lines are drawn.

mod keyboard;
mod lines;

use nexus::imgui::{
    ChildWindow, MouseButton, MouseCursor, StyleColor, StyleVar, Ui, WindowHoveredFlags,
};

use self::{keyboard::typed_characters, lines::CachedLayout};
use super::buffer::TextBuffer;
use crate::{chat_link_chip::ChipContext, theme::Theme};

/// Half a cursor blink period in seconds (on, then off).
const BLINK_HALF_PERIOD: f64 = 0.6;

/// What happened in the editor during one frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EditorResponse {
    /// The text changed.
    pub changed: bool,
    /// Ctrl+Enter was pressed (commit, M52).
    pub submitted: bool,
}

/// Whether the mouse is over the editor's child window. The keyboard sink
/// is an active item, and ImGui's plain `IsWindowHovered` reports `false`
/// while any item is active, so the check must allow that (and the sink's
/// own tiny child window).
fn is_editor_hovered(ui: &Ui) -> bool {
    ui.is_window_hovered_with_flags(
        WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM | WindowHoveredFlags::CHILD_WINDOWS,
    )
}

/// A word-wrapping markdown editor.
#[derive(Debug)]
pub struct TextEditor {
    /// Text, cursor, selection and undo.
    buffer: TextBuffer,
    /// Bumped on every text change; invalidates the layout.
    revision: u64,
    /// The layout of the current text, if built.
    layout: Option<CachedLayout>,
    /// Whether the editor has the keyboard focus.
    focused: bool,
    /// Column to keep while moving up and down.
    preferred_x: Option<f32>,
    /// Whether a mouse selection is being dragged.
    dragging: bool,
    /// When the cursor last moved (restarts the blink).
    blink_start: f64,
    /// Whether the view should scroll to the cursor next frame.
    scroll_to_cursor: bool,
    /// Buffer of the invisible keyboard sink input; emptied every frame.
    sink: String,
    /// Whether the keyboard sink was active in the last frame.
    sink_was_active: bool,
    /// Whether the editor just got the focus, so the sink must take it.
    sink_activation_pending: bool,
    /// Line height of the last drawn frame (for Page Up/Down).
    line_height: f32,
}

impl TextEditor {
    /// An editor for `text`, focused, with the cursor at the end.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            buffer: TextBuffer::new(text),
            revision: 0,
            layout: None,
            focused: true,
            preferred_x: None,
            dragging: false,
            blink_start: 0.0,
            scroll_to_cursor: true,
            sink: String::new(),
            sink_was_active: false,
            sink_activation_pending: true,
            line_height: 0.0,
        }
    }

    /// The text.
    #[must_use]
    pub fn text(&self) -> &str {
        self.buffer.text()
    }

    /// The selected range (start ≤ end).
    #[must_use]
    pub fn selection(&self) -> std::ops::Range<usize> {
        self.buffer.selection()
    }

    /// Selects `range` (byte offsets, clamped); an empty range places the
    /// cursor.
    pub fn select(&mut self, range: std::ops::Range<usize>) {
        self.buffer.select(range.start, range.end);
        self.scroll_to_cursor = true;
    }

    /// A number that changes whenever the text changes (for caches).
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Whether the editor has the keyboard focus.
    #[must_use]
    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    /// Gives the editor the keyboard focus (e.g. after a toolbar button).
    pub fn focus(&mut self) {
        self.focused = true;
        self.sink_activation_pending = true;
    }

    /// Takes the keyboard focus away (e.g. while a dialog is shown).
    pub fn unfocus(&mut self) {
        self.focused = false;
        self.dragging = false;
        self.sink_was_active = false;
        self.sink_activation_pending = false;
    }

    /// Replaces `range` with `replacement` as one undo step and selects
    /// `selection` afterwards (formatting commands, M53).
    pub fn apply_edit(
        &mut self,
        range: std::ops::Range<usize>,
        replacement: &str,
        selection: std::ops::Range<usize>,
    ) {
        self.buffer.replace_range(range, replacement);
        self.buffer.select_after_edit(selection);
        self.after_edit();
    }

    /// Draws the editor filling `size` (ImGui child sizing) and handles
    /// input. `id` must be unique in the current window; `chips` supplies the
    /// texts of chat-link chips (memo plan M75).
    pub fn draw(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        id: &str,
        size: [f32; 2],
        chips: ChipContext<'_>,
    ) -> EditorResponse {
        let mut response = EditorResponse::default();
        let padding = theme.spacing.small;
        let _colors = ui.push_style_color(StyleColor::ChildBg, theme.surface.input.to_array());
        let _padding = ui.push_style_var(StyleVar::WindowPadding([padding, padding]));
        // `movable(false)`: dragging to select text must not move the whole
        // window (ImGui moves windows when dragging where no item is).
        let Some(_child) = ChildWindow::new(id)
            .size(size)
            .always_use_window_padding(true)
            .movable(false)
            .begin(ui)
        else {
            return response;
        };

        if self.focused {
            // Read the typed characters before the sink consumes them.
            let typed = typed_characters();
            self.draw_keyboard_sink(ui);
            if self.focused {
                self.handle_keyboard(ui, &typed, &mut response);
            }
        } else {
            self.sink_was_active = false;
        }

        let width = ui.content_region_avail()[0].max(1.0);
        let font_size = ui.current_font_size();
        self.refresh_layout(ui, theme, width, font_size, chips);
        let Some(cached) = self.layout.as_ref() else {
            return response;
        };
        let layout = &cached.layout;
        // Same line height as the markdown view (`typography.line_height`).
        let line_height = theme.typography.line_height_for(font_size);
        self.line_height = line_height;
        let origin = ui.cursor_screen_pos();
        let line_count = layout.lines().len();
        #[expect(clippy::cast_precision_loss, reason = "line counts are small")]
        let content_height = line_count as f32 * line_height;
        let visible_height = ui.window_size()[1];
        // Reserve the content height so the child scrolls.
        ui.dummy([width, content_height.max(visible_height - 2.0 * padding)]);

        self.handle_mouse(ui, origin, line_height);
        let Some(cached) = self.layout.as_ref() else {
            return response;
        };
        let layout = &cached.layout;

        let scroll = ui.scroll_y();
        let cursor_line = layout.line_index_of(self.buffer.cursor());
        if std::mem::take(&mut self.scroll_to_cursor) {
            #[expect(clippy::cast_precision_loss, reason = "line counts are small")]
            let cursor_top = cursor_line as f32 * line_height;
            if cursor_top < scroll {
                ui.set_scroll_y(cursor_top);
            } else if cursor_top + line_height > scroll + visible_height - 2.0 * padding {
                ui.set_scroll_y(cursor_top + line_height - visible_height + 2.0 * padding);
            }
        }
        if is_editor_hovered(ui) {
            ui.set_mouse_cursor(Some(MouseCursor::TextInput));
        }

        self.draw_lines(
            ui,
            theme,
            layout,
            origin,
            line_height,
            scroll,
            visible_height,
            chips,
        );
        response
    }

    /// Handles clicks, double-clicks and drags inside the editor; a click
    /// elsewhere takes the focus away.
    fn handle_mouse(&mut self, ui: &Ui, origin: [f32; 2], line_height: f32) {
        let hovered = is_editor_hovered(ui);
        if ui.is_mouse_clicked(MouseButton::Left) && !hovered {
            self.focused = false;
            self.dragging = false;
            return;
        }
        let Some(cached) = self.layout.as_ref() else {
            return;
        };
        let mouse = ui.io().mouse_pos;
        let hit = {
            let layout = &cached.layout;
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "line indices are small and clamped"
            )]
            let index = (((mouse[1] - origin[1]) / line_height).floor().max(0.0) as usize)
                .min(layout.lines().len() - 1);
            layout.lines()[index].position_at(mouse[0] - origin[0])
        };
        if hovered && ui.is_mouse_double_clicked(MouseButton::Left) {
            self.buffer.select_word_at(hit);
            self.after_move(ui);
            self.focus();
            self.dragging = false;
        } else if hovered && ui.is_mouse_clicked(MouseButton::Left) {
            self.buffer.set_cursor(hit, ui.io().key_shift);
            self.after_move(ui);
            self.focus();
            self.dragging = true;
        } else if self.dragging && ui.is_mouse_down(MouseButton::Left) {
            if hit != self.buffer.cursor() {
                self.buffer.set_cursor(hit, true);
                self.after_move(ui);
            }
        } else {
            self.dragging = false;
        }
    }

    /// After a cursor move: restart the blink, forget the column, scroll.
    fn after_move(&mut self, ui: &Ui) {
        self.blink_start = ui.time();
        self.preferred_x = None;
        self.scroll_to_cursor = true;
    }

    /// After a text change: invalidate the layout and scroll to the cursor.
    fn after_edit(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.preferred_x = None;
        self.scroll_to_cursor = true;
    }
}
