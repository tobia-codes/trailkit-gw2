//! The search field (memo plan M118, M123, M128; shared search Q581): a
//! `bg.input` field a little taller than a button with a Lucide magnifier, a
//! hint, an accent frame while focused and an `x` at its end. The `x` clears
//! a field with text; on an empty field only a closable search shows it, and
//! it closes the search then (M176, mock v9).

use nexus::imgui::{Key, StyleColor, StyleVar, Ui, WindowHoveredFlags};

use crate::controls::hover::is_item_hovered;
use crate::controls::tooltip::draw_tooltip;
use crate::drawing::draw_list::window_draw_list;
use crate::{
    drawing::style_scope::StyleScope,
    text::font_registry::FontScope,
    text::fonts::FontRole,
    text::glyph_metrics::{capital_middle, icon_visible_box},
    text::icons::Icon,
    theme::Theme,
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
    /// Tooltip of the `x` while the field is empty; `None` for a search that
    /// cannot be closed (the trash, the help), whose empty field has no `x`.
    pub close: Option<&'text str>,
}

/// What the user did with the field in this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchFieldResponse {
    /// The text changed by typing or the `x`.
    pub text_changed: bool,
    /// The `x` of the empty field was clicked: the app closes it (M176).
    pub close_clicked: bool,
    /// Enter was pressed in the field (Shift+Enter: `backward`): the app
    /// goes to the next or previous hit. The field keeps the focus.
    pub submitted: Option<Submit>,
}

/// Which way Enter in the field steps through the hits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Submit {
    /// Enter: the next hit.
    Forward,
    /// Shift+Enter: the previous hit.
    Backward,
}

impl SearchField {
    /// Empties the field; `keep_focus` focuses it again on the next frame.
    pub fn clear(&mut self, keep_focus: bool) {
        self.text.clear();
        self.generation = self.generation.wrapping_add(1);
        self.focus_requested = keep_focus;
    }

    /// Focuses the input on the next frame (M124). ImGui selects its whole
    /// text then, so typing replaces it (find bar, M198).
    pub fn request_focus(&mut self) {
        self.focus_requested = true;
    }

    /// Replaces the text, e.g. to fill the find field (M198).
    pub fn set_text(&mut self, text: &str) {
        text.clone_into(&mut self.text);
        self.generation = self.generation.wrapping_add(1);
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
        let width = ui.content_region_avail()[0];
        self.draw_sized(ui, theme, texts, width)
    }

    /// Draws the field `width` wide and reports what the user did.
    pub fn draw_sized(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        texts: SearchFieldTexts<'_>,
        width: f32,
    ) -> SearchFieldResponse {
        let was_active = self.is_active;
        let start = ui.cursor_screen_pos();
        let height = field_height(ui, theme);
        let end = [start[0] + width, start[1] + height];
        let padding = ui.clone_style().frame_padding;
        // The input draws its text at its line top inside the frame padding;
        // it moves so the text's capitals sit on the field's middle, like a
        // button label (M85), and the icons share that line (R147).
        let capitals = capital_middle().unwrap_or(ui.text_line_height() / 2.0);
        let text_top = (start[1] + height / 2.0 - capitals).round();
        let input_y = text_top - padding[1];
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
        // The icons sit on the capitals of the text the input draws at its
        // line top inside the frame padding, not on the field's middle,
        // so icons and text share one line (R147).
        let middle = text_top + capitals;
        let clear_position = [
            (end[0] - padding[0] - clear[2]).round(),
            (middle - (clear[1] + clear[3]) / 2.0).round(),
        ];
        draw_magnifier(ui, theme, [start[0] + padding[0], middle], magnifier);
        let input_x = start[0] + padding[0] + (magnifier[2] - magnifier[0]) + theme.spacing.small;
        let has_text = !self.text.is_empty();
        let end_tooltip = if has_text {
            Some(texts.clear)
        } else {
            texts.close
        };
        let clear_width = if end_tooltip.is_some() {
            padding[0] + (clear[2] - clear[0]) + theme.spacing.small
        } else {
            padding[0]
        };

        let changed = self.draw_input(
            ui,
            theme,
            texts.hint,
            [input_x, input_y],
            (end[0] - clear_width - input_x).max(1.0),
        );

        // The `x` empties a field with text and closes an empty closable
        // one (M176). It reads the text from before this frame's typing, so
        // the first typed letter does not move it under the mouse.
        let clicked = end_tooltip
            .is_some_and(|tooltip| self.draw_end_button(ui, theme, clear_position, clear, tooltip));
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
        // ImGui ends the input's activation on Enter; the field takes the
        // focus back so the user can keep stepping (M198).
        let submitted = (was_active && ui.is_key_pressed(Key::Enter)).then(|| {
            self.focus_requested = true;
            if ui.io().key_shift {
                Submit::Backward
            } else {
                Submit::Forward
            }
        });
        SearchFieldResponse {
            text_changed: changed || (clicked && has_text),
            close_clicked: clicked && !has_text,
            submitted,
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
        let hovered = is_item_hovered(ui);
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

/// The height of the field: a little taller than a button (M128); the input
/// inside keeps the regular frame height and sits centered.
#[must_use]
pub fn field_height(ui: &Ui, theme: &Theme) -> f32 {
    ui.frame_height() + theme.spacing.small
}

/// The visible boxes of the field's magnifier and `x` (M128, M170). Both icons
/// are placed by their ink, not their line box, in which Lucide glyphs sit off center,
/// so they mirror each other: same size, color and distance to the frame.
fn field_icon_inks(ui: &Ui) -> ([f32; 4], [f32; 4]) {
    let _icons = FontScope::push(ui, FontRole::Icons);
    let line = ui.text_line_height();
    let ink = |icon: Icon| icon_visible_box(icon.glyph(), line);
    (ink(Icon::SEARCH), ink(Icon::X))
}

/// Draws the magnifier with its `ink` box starting at `left`, centered on `middle`.
fn draw_magnifier(ui: &Ui, theme: &Theme, [left, middle]: [f32; 2], ink: [f32; 4]) {
    let _icons = FontScope::push(ui, FontRole::Icons);
    let mut buffer = [0; 4];
    window_draw_list(ui).add_text(
        [
            (left - ink[0]).round(),
            (middle - (ink[1] + ink[3]) / 2.0).round(),
        ],
        theme.text.secondary.to_array(),
        Icon::SEARCH.as_str(&mut buffer),
    );
}
