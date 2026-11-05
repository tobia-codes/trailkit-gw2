//! Small text placement helpers shared by addon windows and their slices:
//! centering, truncation with "…" (`lay-truncation`) and allocation-free
//! number formatting for per-frame labels.

use std::borrow::Cow;

use nexus::imgui::Ui;

/// Draws `text` horizontally centered in a region of `width`. Too long text
/// wraps at spaces with each line centered; a single word wider than `width`
/// falls back to ImGui's left-aligned wrapping.
pub fn centered_text(ui: &Ui, text: &str, width: f32) {
    let text_width = ui.calc_text_size(text)[0];
    if text_width <= width {
        indent_to_center(ui, text_width, width);
        ui.text(text);
        return;
    }
    if text
        .split_whitespace()
        .any(|word| ui.calc_text_size(word)[0] > width)
    {
        ui.text_wrapped(text);
        return;
    }
    let mut line = String::new();
    for word in text.split_whitespace() {
        let previous_length = line.len();
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
        if ui.calc_text_size(&line)[0] > width {
            line.truncate(previous_length);
            draw_centered_line(ui, &line, width);
            line.clear();
            line.push_str(word);
        }
    }
    draw_centered_line(ui, &line, width);
}

/// Draws one `line` that fits into `width`, centered in it.
fn draw_centered_line(ui: &Ui, line: &str, width: f32) {
    indent_to_center(ui, ui.calc_text_size(line)[0], width);
    ui.text(line);
}

/// Moves the cursor so an item of `item_width` is centered in `width`.
pub fn indent_to_center(ui: &Ui, item_width: f32, width: f32) {
    let position = ui.cursor_pos();
    ui.set_cursor_pos([
        position[0] + ((width - item_width) / 2.0).max(0.0),
        position[1],
    ]);
}

/// Formats `count` into `buffer` without allocating (drawn every frame).
pub fn format_count(count: usize, buffer: &mut [u8; 20]) -> &str {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(&mut buffer[..]);
    // 20 bytes hold every `usize`, so the write cannot fail.
    let written = match write!(cursor, "{count}") {
        Ok(()) => usize::try_from(cursor.position()).unwrap_or(0),
        Err(_) => 0,
    };
    std::str::from_utf8(&buffer[..written]).unwrap_or_default()
}

/// Shortens `text` with "…" so it fits into `width` (`lay-truncation`).
/// Allocates only when shortening is needed.
pub fn truncate_to_width<'text>(ui: &Ui, text: &'text str, width: f32) -> Cow<'text, str> {
    if ui.calc_text_size(text)[0] <= width {
        return Cow::Borrowed(text);
    }
    let ellipsis = "…";
    let available = width - ui.calc_text_size(ellipsis)[0];
    let mut end = 0;
    for (index, character) in text.char_indices() {
        let next = index + character.len_utf8();
        if ui.calc_text_size(&text[..next])[0] > available {
            break;
        }
        end = next;
    }
    Cow::Owned(format!("{}{ellipsis}", text[..end].trim_end()))
}
