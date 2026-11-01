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

use nexus::imgui::{
    ChildWindow, InputTextCallbackHandler, InputTextMultilineCallback, Key, MouseButton,
    MouseCursor, StyleColor, StyleVar, TextCallbackData, Ui, WindowHoveredFlags,
};

use super::{
    buffer::{Direction, Step, TextBuffer},
    highlight::SpanStyle,
    layout::Layout,
};
use crate::{font_registry::FontScope, fonts::FontRole, theme::Theme};

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

/// The width of `text` in the current font, exactly as ImGui draws it.
/// `Ui::calc_text_size` rounds every result up to a whole pixel, which adds
/// up when characters are measured one by one and moves the cursor away
/// from the text.
fn unrounded_width(text: &str) -> f32 {
    let mut size = nexus::imgui::sys::ImVec2 { x: 0.0, y: 0.0 };
    let begin = text.as_ptr().cast::<std::ffi::c_char>();
    // SAFETY: called on the render thread inside a frame, so the current
    // font is valid; `begin..begin + len` is the UTF-8 text, which ImGui
    // only reads; the remaining-text output pointer may be null.
    unsafe {
        nexus::imgui::sys::ImFont_CalcTextSizeA(
            &raw mut size,
            nexus::imgui::sys::igGetFont(),
            nexus::imgui::sys::igGetFontSize(),
            f32::MAX,
            0.0,
            begin,
            begin.add(text.len()),
            std::ptr::null_mut(),
        );
    }
    size.x
}

/// The layout of a specific text revision, width and font size.
#[derive(Debug)]
struct CachedLayout {
    /// Text revision the layout was built for.
    revision: u64,
    /// Wrap width.
    width: f32,
    /// Font size.
    font_size: f32,
    /// The layout.
    layout: Layout,
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
}

/// Empties the keyboard sink every frame, so its text never grows.
struct ClearSink;

impl InputTextCallbackHandler for ClearSink {
    /// Clears the sink's text while it is active.
    fn on_always(&mut self, mut data: TextCallbackData) {
        data.clear();
    }
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

    /// Whether the editor has the keyboard focus.
    #[must_use]
    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    /// Gives the editor the keyboard focus (e.g. after a toolbar button).
    pub fn focus(&mut self) {
        self.focused = true;
    }

    /// Takes the keyboard focus away (e.g. while a dialog is shown).
    pub fn unfocus(&mut self) {
        self.focused = false;
        self.dragging = false;
        self.sink_was_active = false;
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
    /// input. `id` must be unique in the current window.
    pub fn draw(&mut self, ui: &Ui, theme: &Theme, id: &str, size: [f32; 2]) -> EditorResponse {
        let mut response = EditorResponse::default();
        let padding = theme.spacing.small;
        let _colors = ui.push_style_color(StyleColor::ChildBg, theme.surface.input.to_array());
        let _padding = ui.push_style_var(StyleVar::WindowPadding([padding, padding]));
        let Some(_child) = ChildWindow::new(id)
            .size(size)
            .always_use_window_padding(true)
            .begin(ui)
        else {
            return response;
        };

        if self.focused {
            // Read the typed characters before the sink consumes them.
            let typed = ui.io().peek_input_characters();
            self.draw_keyboard_sink(ui);
            self.handle_keyboard(ui, &typed, &mut response);
        } else {
            self.sink_was_active = false;
        }

        let width = ui.content_region_avail()[0].max(1.0);
        let font_size = ui.current_font_size();
        self.refresh_layout(ui, width, font_size);
        let Some(cached) = self.layout.as_ref() else {
            return response;
        };
        let layout = &cached.layout;
        let line_height = ui.text_line_height_with_spacing();
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
        );
        response
    }

    /// Keeps an invisible 1×1 ImGui text input active while the editor is
    /// focused, so Nexus routes key presses to ImGui, not to the game.
    fn draw_keyboard_sink(&mut self, ui: &Ui) {
        let start = ui.cursor_pos();
        // Not 0.0: ImGui 1.80 skips all items of a window begun with alpha 0,
        // and a multiline input is a child window. The smallest positive
        // alpha still rounds to fully transparent colors.
        let _invisible = ui.push_style_var(StyleVar::Alpha(f32::MIN_POSITIVE));
        if !self.sink_was_active {
            ui.set_keyboard_focus_here();
        }
        ui.input_text_multiline("##keyboard_sink", &mut self.sink, [1.0, 1.0])
            .callback(InputTextMultilineCallback::ALWAYS, ClearSink)
            .build();
        let sink_active = ui.is_item_active();
        self.sink_was_active = sink_active;
        self.sink.clear();
        ui.set_cursor_pos(start);
    }

    /// Rebuilds the layout if the text, width or font size changed.
    fn refresh_layout(&mut self, ui: &Ui, width: f32, font_size: f32) {
        let is_current = self.layout.as_ref().is_some_and(|cached| {
            cached.revision == self.revision
                && (cached.width - width).abs() < 0.5
                && (cached.font_size - font_size).abs() < f32::EPSILON
        });
        if is_current {
            return;
        }
        let layout = Layout::build(self.buffer.text(), width, |character, style| {
            if style == SpanStyle::Strong {
                let _bold = FontScope::push(ui, FontRole::BodyBold);
                unrounded_width(character)
            } else {
                unrounded_width(character)
            }
        });
        self.layout = Some(CachedLayout {
            revision: self.revision,
            width,
            font_size,
            layout,
        });
    }

    /// Draws the visible lines: selection, highlighted text and cursor.
    #[expect(clippy::too_many_arguments, reason = "drawing state of one frame")]
    fn draw_lines(
        &self,
        ui: &Ui,
        theme: &Theme,
        layout: &Layout,
        origin: [f32; 2],
        line_height: f32,
        scroll: f32,
        visible_height: f32,
    ) {
        let draw_list = ui.get_window_draw_list();
        let selection = self.buffer.selection();
        let cursor = self.buffer.cursor();
        let cursor_line = layout.line_index_of(cursor);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "scroll offsets are small and non-negative"
        )]
        let first_visible = (scroll / line_height).floor().max(0.0) as usize;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "line counts are small"
        )]
        let visible_lines = (visible_height / line_height).ceil() as usize + 1;
        let text = self.buffer.text();
        for (index, line) in layout
            .lines()
            .iter()
            .enumerate()
            .skip(first_visible)
            .take(visible_lines)
        {
            #[expect(clippy::cast_precision_loss, reason = "line counts are small")]
            let top = origin[1] + index as f32 * line_height;
            if !selection.is_empty()
                && selection.start <= line.range.end
                && selection.end >= line.range.start
            {
                let start_x = line.x_of(selection.start.max(line.range.start));
                let mut end_x = line.x_of(selection.end.min(line.range.end));
                // A selected line break shows as a small extra width.
                if selection.end > line.range.end && line.ends_logical_line {
                    end_x += ui.calc_text_size(" ")[0];
                }
                draw_list
                    .add_rect(
                        [origin[0] + start_x, top],
                        [origin[0] + end_x, top + line_height],
                        theme.accent.base.with_alpha(0.35).to_array(),
                    )
                    .filled(true)
                    .build();
            }
            for span in layout.spans_in(line.range.clone()) {
                let position = [origin[0] + line.x_of(span.range.start), top];
                let piece = &text[span.range.clone()];
                match span.style {
                    SpanStyle::Strong => {
                        let _bold = FontScope::push(ui, FontRole::BodyBold);
                        draw_list.add_text(position, theme.text.primary.to_array(), piece);
                    }
                    SpanStyle::Marker => {
                        draw_list.add_text(position, theme.text.secondary.to_array(), piece);
                    }
                    SpanStyle::Text => {
                        draw_list.add_text(position, theme.text.primary.to_array(), piece);
                    }
                }
            }
            #[expect(clippy::cast_possible_truncation, reason = "only the parity matters")]
            let blink_on = ((ui.time() - self.blink_start) / BLINK_HALF_PERIOD) as i64 % 2 == 0;
            if self.focused && index == cursor_line && blink_on {
                let x = origin[0] + line.x_of(cursor);
                draw_list
                    .add_line(
                        [x, top],
                        [x, top + ui.text_line_height()],
                        theme.text.primary.to_array(),
                    )
                    .thickness(1.0)
                    .build();
            }
        }
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
            self.focused = true;
            self.dragging = false;
        } else if hovered && ui.is_mouse_clicked(MouseButton::Left) {
            self.buffer.set_cursor(hit, ui.io().key_shift);
            self.after_move(ui);
            self.focused = true;
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

    /// Handles typing, navigation, editing shortcuts and the clipboard.
    fn handle_keyboard(&mut self, ui: &Ui, typed: &str, response: &mut EditorResponse) {
        let io = ui.io();
        let control = io.key_ctrl;
        let shift = io.key_shift;
        let step = if control { Step::Word } else { Step::Character };
        let pressed = |key: Key| ui.is_key_pressed(key);

        if control && (pressed(Key::Enter) || pressed(Key::KeyPadEnter)) {
            response.submitted = true;
            return;
        }
        if control && pressed(Key::A) {
            self.buffer.select_all();
            self.after_move(ui);
        }
        if control && (pressed(Key::C) || pressed(Key::X)) && self.buffer.has_selection() {
            ui.set_clipboard_text(self.buffer.selected_text());
            if pressed(Key::X) {
                self.buffer.delete(Direction::Backward, Step::Character);
                self.after_edit();
                response.changed = true;
            }
        }
        if control
            && pressed(Key::V)
            && let Some(clipboard) = ui.clipboard_text()
        {
            let normalized = clipboard.replace("\r\n", "\n");
            self.buffer.insert(&normalized);
            self.after_edit();
            response.changed = true;
        }
        if control && pressed(Key::Z) {
            if self.buffer.undo() {
                self.after_edit();
                response.changed = true;
            }
        } else if control && pressed(Key::Y) && self.buffer.redo() {
            self.after_edit();
            response.changed = true;
        }

        let horizontal = [
            (Key::LeftArrow, Direction::Backward),
            (Key::RightArrow, Direction::Forward),
        ];
        for (key, direction) in horizontal {
            if pressed(key) {
                self.buffer.move_horizontally(direction, step, shift);
                self.after_move(ui);
            }
        }
        if pressed(Key::Backspace) {
            self.buffer.delete(Direction::Backward, step);
            self.after_edit();
            response.changed = true;
        }
        if pressed(Key::Delete) {
            self.buffer.delete(Direction::Forward, step);
            self.after_edit();
            response.changed = true;
        }
        if !control && (pressed(Key::Enter) || pressed(Key::KeyPadEnter)) {
            self.buffer.insert_line_break();
            self.after_edit();
            response.changed = true;
        }
        self.handle_vertical_keys(ui, control, shift);

        if !control {
            let printable: String = typed
                .chars()
                .filter(|character| !character.is_control())
                .collect();
            if !printable.is_empty() {
                let mut encoded = [0; 4];
                for character in printable.chars() {
                    self.buffer.insert(character.encode_utf8(&mut encoded));
                }
                self.after_edit();
                response.changed = true;
            }
        }
    }

    /// Up/Down, PageUp/PageDown and Home/End (with Ctrl: text start/end).
    fn handle_vertical_keys(&mut self, ui: &Ui, control: bool, shift: bool) {
        let Some(cached) = self.layout.as_ref() else {
            return;
        };
        let layout = &cached.layout;
        let cursor = self.buffer.cursor();
        let x = self
            .preferred_x
            .unwrap_or_else(|| layout.line_of(cursor).x_of(cursor));
        #[expect(clippy::cast_possible_truncation, reason = "page sizes are small")]
        let page = ((ui.window_size()[1] / ui.text_line_height_with_spacing()) as isize).max(1);
        let target = if ui.is_key_pressed(Key::UpArrow) {
            Some((layout.position_vertically(cursor, -1, x), true))
        } else if ui.is_key_pressed(Key::DownArrow) {
            Some((layout.position_vertically(cursor, 1, x), true))
        } else if ui.is_key_pressed(Key::PageUp) {
            Some((layout.position_vertically(cursor, -page, x), true))
        } else if ui.is_key_pressed(Key::PageDown) {
            Some((layout.position_vertically(cursor, page, x), true))
        } else if ui.is_key_pressed(Key::Home) {
            let start = if control {
                0
            } else {
                layout.visual_line_start(cursor)
            };
            Some((start, false))
        } else if ui.is_key_pressed(Key::End) {
            let end = if control {
                self.buffer.text().len()
            } else {
                layout.visual_line_end(cursor)
            };
            Some((end, false))
        } else {
            None
        };
        if let Some((target, keeps_column)) = target {
            self.buffer.set_cursor(target, shift);
            self.after_move(ui);
            if keeps_column {
                self.preferred_x = Some(x);
            }
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
