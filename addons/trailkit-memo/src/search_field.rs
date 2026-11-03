//! The search field above the note list (memo plan M118, M123): a
//! button-tall `bg.input` field with a Lucide magnifier, a hint, an `x` that
//! clears it and a brass frame while focused.
//!
//! Lives in memo until a second addon needs a search field (AGENTS.md
//! rule 7); the settings window will be the likely second user.

use nexus::imgui::{StyleColor, StyleVar, Ui};
use trailkit_ui::{
    button::draw_icon_button, font_registry::FontScope, fonts::FontRole, icons::Icon, theme::Theme,
};

/// State of the search field that outlives a frame.
#[derive(Debug, Default)]
pub struct SearchField {
    /// What the user typed.
    pub text: String,
    /// Part of the input's ImGui id. Clearing the field changes it: ImGui
    /// keeps its own copy of the text while the input is active and would
    /// write the old text back otherwise.
    generation: i32,
    /// Focus the input on the next frame (Ctrl+F, after clearing).
    focus_requested: bool,
    /// Whether the input had the keyboard focus last frame.
    is_active: bool,
}

/// The texts the field shows.
#[derive(Debug, Clone, Copy)]
pub struct SearchFieldTexts<'text> {
    /// Placeholder while empty ("Search …").
    pub hint: &'text str,
    /// Tooltip of the clear button.
    pub clear: &'text str,
}

impl SearchField {
    /// Empties the field; `keep_focus` focuses it again on the next frame.
    pub fn clear(&mut self, keep_focus: bool) {
        self.text.clear();
        self.generation = self.generation.wrapping_add(1);
        self.focus_requested = keep_focus;
    }

    /// Focuses the input on the next frame (Ctrl+F, M124).
    pub fn request_focus(&mut self) {
        self.focus_requested = true;
    }

    /// Whether the input has the keyboard focus.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.is_active
    }

    /// Draws the field over the full available width. Returns whether the
    /// text changed (typing or the clear button).
    pub fn draw(&mut self, ui: &Ui, theme: &Theme, texts: SearchFieldTexts<'_>) -> bool {
        let start = ui.cursor_screen_pos();
        let width = ui.content_region_avail()[0];
        let height = ui.frame_height();
        let end = [start[0] + width, start[1] + height];
        let padding = ui.clone_style().frame_padding;
        // `DrawListMut` is a singleton: take it only for each drawing step.
        ui.get_window_draw_list()
            .add_rect(start, end, theme.surface.input.to_array())
            .filled(true)
            .rounding(theme.radius.medium)
            .build();

        // Magnifier, centered vertically like the icon buttons' glyphs.
        let icon_width = {
            let _icons = FontScope::push(ui, FontRole::Icons);
            let mut buffer = [0; 4];
            let icon = Icon::SEARCH.as_str(&mut buffer);
            let size = ui.calc_text_size(icon);
            ui.get_window_draw_list().add_text(
                [start[0] + padding[0], start[1] + (height - size[1]) / 2.0],
                theme.text.secondary.to_array(),
                icon,
            );
            size[0]
        };
        let input_x = start[0] + padding[0] + icon_width + theme.spacing.extra_small;
        let clear_width = if self.text.is_empty() { 0.0 } else { height };

        let changed = {
            let _id = ui.push_id(self.generation);
            let _colors = [
                ui.push_style_color(StyleColor::FrameBg, [0.0; 4]),
                ui.push_style_color(StyleColor::FrameBgHovered, [0.0; 4]),
                ui.push_style_color(StyleColor::FrameBgActive, [0.0; 4]),
                ui.push_style_color(StyleColor::TextDisabled, theme.text.secondary.to_array()),
            ];
            let _border = ui.push_style_var(StyleVar::FrameBorderSize(0.0));
            ui.set_cursor_screen_pos([input_x, start[1]]);
            ui.set_next_item_width((end[0] - clear_width - input_x).max(1.0));
            if std::mem::take(&mut self.focus_requested) {
                ui.set_keyboard_focus_here();
            }
            let changed = ui
                .input_text("##search", &mut self.text)
                .hint(texts.hint)
                .build();
            self.is_active = ui.is_item_active();
            changed
        };

        let mut cleared = false;
        if !self.text.is_empty() {
            ui.same_line_with_spacing(0.0, 0.0);
            ui.set_cursor_screen_pos([end[0] - clear_width, start[1]]);
            if draw_icon_button(ui, theme, Icon::X, texts.clear) {
                self.clear(true);
                cleared = true;
            }
        }

        let frame = if self.is_active {
            theme.border.focus
        } else {
            theme.border.color
        };
        ui.get_window_draw_list()
            .add_rect(start, end, frame.to_array())
            .rounding(theme.radius.medium)
            .thickness(theme.border.width)
            .build();
        // Leave the cursor below the field, as a regular item would.
        ui.set_cursor_screen_pos([start[0], end[1]]);
        ui.dummy([0.0, 0.0]);
        changed || cleared
    }
}
