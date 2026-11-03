//! Layout of the memo main window: two panes or drawer (M48, M72–M74),
//! the empty state (M42) and small text placement helpers.

use std::borrow::Cow;

use nexus::imgui::{ChildWindow, MouseButton, Ui};
use trailkit_ui::{
    button::{button_width, draw_button, draw_primary_button},
    pane::draw_pane,
    splitter::draw_vertical_splitter,
    theme::Theme,
};

use trailkit_memo_shared::i18n::{Language, Text};

use super::{DRAWER_MAXIMUM_FRACTION, MemoWindow, PaneWidths, WindowAction};

impl MemoWindow {
    /// Draws tree and note as two panes with a draggable splitter in the gap
    /// between them (M21). The splitter shows an accent line only while it
    /// is hovered or dragged.
    pub(super) fn draw_two_panes(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        widths: PaneWidths,
    ) {
        let available = ui.content_region_avail();
        let gap = theme.pane.gap;
        let usable = available[0] - gap;
        let tree_maximum = (usable - widths.note_minimum).max(widths.tree_minimum);
        // A fixed width (M74): a wider window gives the extra room to the
        // note; a narrower one squeezes the list only while it lacks space.
        let character_width = ui.calc_text_size("n")[0];
        let padding = 2.0 * theme.pane.padding;
        let tree_width = (self.ui_state.valid_note_list_characters() * character_width + padding)
            .clamp(widths.tree_minimum, tree_maximum);

        draw_pane(ui, theme, "tree", [tree_width, 0.0], || {
            self.draw_tree_pane(ui, language, theme, None);
        });

        ui.same_line_with_spacing(0.0, 0.0);
        let splitter = draw_vertical_splitter(
            ui,
            theme,
            "splitter",
            tree_width,
            (widths.tree_minimum, tree_maximum),
            available[1],
        );
        if let Some(width) = splitter.left_width {
            self.ui_state.note_list_characters = (width - padding) / character_width;
        }
        if splitter.released {
            self.saver.save_ui_state(self.ui_state);
        }

        ui.same_line_with_spacing(0.0, 0.0);
        draw_pane(ui, theme, "note", [0.0, 0.0], || {
            self.draw_note_pane(ui, language, theme, Text::HideNoteList);
        });
    }

    /// Draws the note list as a drawer over the dimmed note (narrow window,
    /// M48, `lay-drawer`): it appears instantly (`overlay-motion`) and closes
    /// on a click on the dimmed area. `start` and `size` cover the note pane.
    pub(super) fn draw_drawer(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        start: [f32; 2],
        size: [f32; 2],
        pane_minimum: f32,
    ) {
        ui.set_cursor_screen_pos(start);
        // The scrim takes the clicks on the note. ImGui renders a child's
        // background into the parent's draw list, below the note pane drawn
        // before, so the dimming is a rectangle in the scrim's own list.
        let Some(_scrim) = ChildWindow::new("drawer_scrim")
            .size(size)
            .scrollable(false)
            .scroll_bar(false)
            .draw_background(false)
            .begin(ui)
        else {
            return;
        };
        let scrim_start = ui.window_pos();
        let scrim_end = [scrim_start[0] + size[0], scrim_start[1] + size[1]];
        ui.get_window_draw_list()
            .add_rect(
                scrim_start,
                scrim_end,
                theme.surface.modal_overlay.to_array(),
            )
            .filled(true)
            .build();
        let width = pane_minimum.min(size[0] * DRAWER_MAXIMUM_FRACTION);
        draw_pane(ui, theme, "drawer", [width, 0.0], || {
            self.draw_tree_pane(ui, language, theme, Some(Text::HideNoteList));
        });
        if ui.is_window_hovered() && ui.is_mouse_clicked(MouseButton::Left) {
            self.actions.push(WindowAction::CloseDrawer);
        }
    }
}

/// Continues the toolbar line at its right end for `width` of items,
/// wrapping onto a new line when they do not fit (`lay-responsive`).
pub(super) fn move_to_toolbar_end(ui: &Ui, width: f32) {
    let right_edge = ui.window_content_region_max()[0] + ui.window_pos()[0];
    ui.same_line();
    if ui.cursor_screen_pos()[0] + width > right_edge {
        ui.new_line();
    }
    let y = ui.cursor_screen_pos()[1];
    ui.set_cursor_screen_pos([right_edge - width, y]);
}

/// Draws the empty state (M42): headline, primary and secondary button.
pub(super) fn draw_empty_state(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    actions: &mut Vec<WindowAction>,
) {
    let available = ui.content_region_avail();
    let block_height = ui.text_line_height_with_spacing() + 2.0 * ui.frame_height_with_spacing();
    let start = ui.cursor_pos();
    ui.set_cursor_pos([
        start[0],
        start[1] + ((available[1] - block_height) / 2.0).max(0.0),
    ]);

    centered_text(ui, Text::NoNotesYet.get(language), available[0]);
    ui.dummy([0.0, theme.spacing.small]);

    let primary = Text::CreateFirstNote.get(language);
    let secondary = Text::NewGroup.get(language);
    let primary_width = button_width(ui, theme, primary);
    let secondary_width = button_width(ui, theme, secondary);
    let spacing = theme.spacing.small;
    let fits_one_row = primary_width + spacing + secondary_width <= available[0];

    // Primary action uses the accent (`lay-one-primary`).
    let row_width = if fits_one_row {
        primary_width + spacing + secondary_width
    } else {
        primary_width
    };
    indent_to_center(ui, row_width, available[0]);
    if draw_primary_button(ui, theme, primary) {
        actions.push(WindowAction::NewNote(None));
    }
    if fits_one_row {
        ui.same_line_with_spacing(0.0, spacing);
    } else {
        indent_to_center(ui, secondary_width, available[0]);
    }
    if draw_button(ui, theme, secondary) {
        actions.push(WindowAction::NewGroup(None));
    }
}

/// Draws `text` horizontally centered in a region of `width`, wrapping if needed.
pub(super) fn centered_text(ui: &Ui, text: &str, width: f32) {
    let text_width = ui.calc_text_size(text)[0];
    if text_width <= width {
        indent_to_center(ui, text_width, width);
        ui.text(text);
    } else {
        ui.text_wrapped(text);
    }
}

/// Moves the cursor so an item of `item_width` is centered in `width`.
pub(super) fn indent_to_center(ui: &Ui, item_width: f32, width: f32) {
    let position = ui.cursor_pos();
    ui.set_cursor_pos([
        position[0] + ((width - item_width) / 2.0).max(0.0),
        position[1],
    ]);
}

/// Formats `count` into `buffer` without allocating (drawn every frame).
pub(super) fn format_count(count: usize, buffer: &mut [u8; 20]) -> &str {
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
pub(super) fn truncate_to_width<'text>(ui: &Ui, text: &'text str, width: f32) -> Cow<'text, str> {
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
