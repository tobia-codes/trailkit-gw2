//! Keyboard handling of the editor (memo plan M58–M60, M178): the invisible
//! keyboard sink that makes Nexus route keys to ImGui, typing, navigation,
//! editing keys and shortcuts.

use nexus::imgui::{
    InputTextCallbackHandler, InputTextMultilineCallback, Key, StyleVar, TextCallbackData, Ui,
};

use super::{EditorResponse, TextEditor};
use crate::editor::buffer::{Direction, Step};

/// The characters typed in this frame (ImGui's input queue), without
/// consuming them.
///
/// `Io::peek_input_characters` is not used: it builds a slice from the
/// queue's data pointer, which is null until the first character arrives in
/// a context, and `slice::from_raw_parts(null, 0)` aborts debug builds (found
/// by the UI tests, M65).
pub(super) fn typed_characters() -> String {
    // SAFETY: called on the render thread inside a frame, so the IO is valid;
    // the queue holds `Size` UTF-16 code units at `Data` (only read when
    // `Size > 0`, so `Data` is non-null).
    let units: &[u16] = unsafe {
        let queue = &(*nexus::imgui::sys::igGetIO()).InputQueueCharacters;
        match usize::try_from(queue.Size) {
            Ok(size) if size > 0 && !queue.Data.is_null() => {
                std::slice::from_raw_parts(queue.Data, size)
            }
            _ => &[],
        }
    };
    String::from_utf16_lossy(units)
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
    /// Keeps an invisible 1×1 ImGui text input active while the editor is
    /// focused, so Nexus routes key presses to ImGui, not to the game. It only
    /// takes the keyboard when the editor gets the focus: an input that loses it
    /// from outside, such as a click into the game world, which ImGui never
    /// reports as a click, ends the focus, so the game gets its keys back (M178).
    pub(super) fn draw_keyboard_sink(&mut self, ui: &Ui) {
        let start = ui.cursor_pos();
        // At the visible top, not the content top: ImGui does not activate
        // an input scrolled out of view, and then Nexus sends the keys to
        // the game (long notes, M83).
        ui.set_cursor_pos([start[0], start[1] + ui.scroll_y()]);
        // Not 0.0: ImGui 1.80 skips all items of a window begun with alpha 0,
        // and a multiline input is a child window. The smallest positive
        // alpha still rounds to fully transparent colors.
        let _invisible = ui.push_style_var(StyleVar::Alpha(f32::MIN_POSITIVE));
        if self.sink_activation_pending && !self.sink_was_active {
            ui.set_keyboard_focus_here();
        }
        ui.input_text_multiline("##keyboard_sink", &mut self.sink, [1.0, 1.0])
            .callback(InputTextMultilineCallback::ALWAYS, ClearSink)
            .build();
        let sink_active = ui.is_item_active();
        if sink_active {
            self.sink_activation_pending = false;
        } else if self.sink_was_active && !self.sink_activation_pending {
            self.focused = false;
            self.dragging = false;
        }
        self.sink_was_active = sink_active;
        self.sink.clear();
        ui.set_cursor_pos(start);
    }

    /// Handles typing, navigation and editing keys, and the shortcuts.
    pub(super) fn handle_keyboard(&mut self, ui: &Ui, typed: &str, response: &mut EditorResponse) {
        let io = ui.io();
        let control = io.key_ctrl;
        let shift = io.key_shift;
        let step = if control { Step::Word } else { Step::Character };
        let pressed = |key: Key| ui.is_key_pressed(key);

        self.handle_shortcuts(ui, response);
        if response.submitted {
            return;
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
        let page = ((ui.window_size()[1] / self.line_height.max(1.0)) as isize).max(1);
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

    /// Handles the Ctrl shortcuts: submit (Ctrl+Enter, M52), select all,
    /// copy, cut, paste, undo and redo. Sets `response.submitted` on submit
    /// and does nothing else then.
    fn handle_shortcuts(&mut self, ui: &Ui, response: &mut EditorResponse) {
        if !ui.io().key_ctrl {
            return;
        }
        let pressed = |key: Key| ui.is_key_pressed(key);
        if pressed(Key::Enter) || pressed(Key::KeyPadEnter) {
            response.submitted = true;
            return;
        }
        if pressed(Key::A) {
            self.buffer.select_all();
            self.after_move(ui);
        }
        if (pressed(Key::C) || pressed(Key::X)) && self.buffer.has_selection() {
            ui.set_clipboard_text(self.buffer.selected_text());
            if pressed(Key::X) {
                self.buffer.delete(Direction::Backward, Step::Character);
                self.after_edit();
                response.changed = true;
            }
        }
        if pressed(Key::V)
            && let Some(clipboard) = ui.clipboard_text()
        {
            let normalized = clipboard.replace("\r\n", "\n");
            self.buffer.insert(&normalized);
            self.after_edit();
            response.changed = true;
        }
        if pressed(Key::Z) {
            if self.buffer.undo() {
                self.after_edit();
                response.changed = true;
            }
        } else if pressed(Key::Y) && self.buffer.redo() {
            self.after_edit();
            response.changed = true;
        }
    }
}
