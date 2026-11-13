//! The search field above the note list (memo plan M118, M123, M128): a
//! `bg.input` field a little taller than a button with a Lucide magnifier, a hint, an `x` that
//! clears it and a brass frame while focused.
//!
//! Lives in memo until a second addon needs a search field (AGENTS.md
//! rule 7); the settings window will be the likely second user.

use nexus::imgui::{StyleColor, StyleVar, Ui, WindowHoveredFlags};
use trailkit_ui::draw_list::window_draw_list;
use trailkit_ui::tooltip::draw_tooltip;
use trailkit_ui::{
    font_registry::FontScope, fonts::FontRole, glyph_metrics::glyph_ink_box, icons::Icon,
    style_scope::StyleScope, theme::Theme,
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
    /// Tooltip of the `x` while the field holds text.
    pub clear: &'text str,
    /// Tooltip of the `x` while the field is empty.
    pub close: &'text str,
}

/// What the user did with the field in this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchFieldResponse {
    /// The text changed by typing or the `x`.
    pub text_changed: bool,
    /// The `x` of the empty field was clicked: the app closes it (M176).
    pub close_clicked: bool,
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

    /// Draws the field over the full available width and reports what the
    /// user did.
    pub fn draw(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        texts: SearchFieldTexts<'_>,
    ) -> SearchFieldResponse {
        let start = ui.cursor_screen_pos();
        let width = ui.content_region_avail()[0];
        // A little taller than a button (M128); the input inside keeps the
        // regular frame height and sits centered.
        let height = ui.frame_height() + theme.spacing.small;
        let input_y = start[1] + theme.spacing.small / 2.0;
        let end = [start[0] + width, start[1] + height];
        let padding = ui.clone_style().frame_padding;
        // Like ImGui's own inputs (`FrameBgHovered`), but not while typing.
        let is_hovered = ui.is_mouse_hovering_rect(start, end)
            && ui.is_window_hovered_with_flags(
                WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM,
            );
        let fill = if is_hovered && !self.is_active {
            theme.surface.hover
        } else {
            theme.surface.input
        };
        window_draw_list(ui)
            .add_rect(start, end, fill.to_array())
            .filled(true)
            .rounding(theme.radius.medium)
            .build();

        let (magnifier, clear) = field_icon_inks(ui);
        let middle = start[1] + height / 2.0;
        let clear_position = [
            end[0] - padding[0] - clear[2],
            middle - (clear[1] + clear[3]) / 2.0,
        ];
        draw_magnifier(ui, theme, [start[0] + padding[0], middle], magnifier);
        let input_x = start[0] + padding[0] + (magnifier[2] - magnifier[0]) + theme.spacing.small;
        let clear_width = padding[0] + (clear[2] - clear[0]) + theme.spacing.small;

        let changed = self.draw_input(
            ui,
            theme,
            texts.hint,
            [input_x, input_y],
            (end[0] - clear_width - input_x).max(1.0),
        );

        // The `x` empties a field with text and closes an empty one (M176).
        let has_text = !self.text.is_empty();
        let tooltip = if has_text { texts.clear } else { texts.close };
        let clicked = self.draw_end_button(ui, theme, clear_position, clear, tooltip);
        if clicked && has_text {
            self.clear(true);
        }

        let frame = if self.is_active {
            theme.border.focus
        } else {
            theme.border.color
        };
        window_draw_list(ui)
            .add_rect(start, end, frame.to_array())
            .rounding(theme.radius.medium)
            .thickness(theme.border.width)
            .build();
        // Leave the cursor below the field, as a regular item would.
        ui.set_cursor_screen_pos([start[0], end[1]]);
        ui.dummy([0.0, 0.0]);
        SearchFieldResponse {
            text_changed: changed || (clicked && has_text),
            close_clicked: clicked && !has_text,
        }
    }

    /// Draws the frameless text input at `position` with `width`; returns
    /// whether its text changed.
    fn draw_input(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        hint: &str,
        position: [f32; 2],
        width: f32,
    ) -> bool {
        let _id = ui.push_id(self.generation);
        let _frame = StyleScope::transparent_frame(ui);
        // The hint is a placeholder, `Secondary` (`tokens.md` §7.1).
        let _hint = ui.push_style_color(StyleColor::TextDisabled, theme.text.secondary.to_array());
        let _border = ui.push_style_var(StyleVar::FrameBorderSize(0.0));
        ui.set_cursor_screen_pos(position);
        ui.set_next_item_width(width);
        if std::mem::take(&mut self.focus_requested) {
            ui.set_keyboard_focus_here();
        }
        let changed = ui.input_text("##search", &mut self.text).hint(hint).build();
        self.is_active = ui.is_item_active();
        changed
    }

    /// Draws the `x` at the field's end at pen `position` (`ink` box relative to it):
    /// `text.secondary`, on hover `text.primary` over a small `bg.hover`
    /// square centered on the glyph. Returns whether it was clicked.
    fn draw_end_button(
        &self,
        ui: &Ui,
        theme: &Theme,
        position: [f32; 2],
        ink: [f32; 4],
        tooltip: &str,
    ) -> bool {
        let side = (ink[2] - ink[0]).max(ink[3] - ink[1]) + 2.0 * theme.spacing.extra_small;
        let center = [
            position[0] + (ink[0] + ink[2]) / 2.0,
            position[1] + (ink[1] + ink[3]) / 2.0,
        ];
        let box_start = [center[0] - side / 2.0, center[1] - side / 2.0];
        let box_size = [side, side];
        ui.same_line_with_spacing(0.0, 0.0);
        ui.set_cursor_screen_pos(box_start);
        let clicked = {
            let _id = ui.push_id(self.generation);
            ui.invisible_button("##search_end", box_size)
        };
        let hovered = ui.is_item_hovered();
        if hovered {
            window_draw_list(ui)
                .add_rect(
                    box_start,
                    [box_start[0] + box_size[0], box_start[1] + box_size[1]],
                    theme.surface.hover.to_array(),
                )
                .filled(true)
                .rounding(theme.radius.small)
                .build();
            draw_tooltip(ui, theme, tooltip);
        }
        let color = if hovered {
            theme.text.primary
        } else {
            theme.text.secondary
        };
        let _icons = FontScope::push(ui, FontRole::Icons);
        let mut buffer = [0; 4];
        window_draw_list(ui).add_text(position, color.to_array(), Icon::X.as_str(&mut buffer));
        clicked
    }
}

/// The ink boxes of the field's magnifier and `x` (M128). Both icons are
/// placed by their ink, not their line box, in which Lucide glyphs sit off center,
/// so they mirror each other: same size, color and distance to the frame.
fn field_icon_inks(ui: &Ui) -> ([f32; 4], [f32; 4]) {
    let _icons = FontScope::push(ui, FontRole::Icons);
    let line = ui.text_line_height();
    let ink = |icon: Icon| glyph_ink_box(icon.glyph()).unwrap_or([0.0, 0.0, line, line]);
    (ink(Icon::SEARCH), ink(Icon::X))
}

/// Draws the magnifier with its `ink` box starting at `left`, centered on `middle`.
fn draw_magnifier(ui: &Ui, theme: &Theme, [left, middle]: [f32; 2], ink: [f32; 4]) {
    let _icons = FontScope::push(ui, FontRole::Icons);
    let mut buffer = [0; 4];
    window_draw_list(ui).add_text(
        [left - ink[0], middle - (ink[1] + ink[3]) / 2.0],
        theme.text.secondary.to_array(),
        Icon::SEARCH.as_str(&mut buffer),
    );
}
