//! A one-line text input with a visible label (`form-labels`, ROADMAP R139):
//! the label above the field, an "(optional)" mark switched on per field, a
//! placeholder and an error directly below it (`form-errors`).

use nexus::imgui::{StyleColor, StyleVar, Ui, WindowHoveredFlags};

use crate::{
    drawing::draw_list::window_draw_list,
    drawing::style_scope::StyleScope,
    feedback::status_line::{Status, draw_status_line},
    text::{TextStyle, draw_text},
    theme::Theme,
};

/// What the user did with a [`TextField`] in this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TextFieldResponse {
    /// The text changed.
    pub changed: bool,
    /// Enter was pressed in the field.
    pub submitted: bool,
    /// The field has the keyboard focus.
    pub is_active: bool,
}

/// Draws `error` at the cursor as the problem of the form field above it:
/// the `danger` icon and small text, wrapped at `width` (`form-errors`). Every
/// form shows its field errors through here, so they look the same.
pub fn draw_field_error(ui: &Ui, theme: &Theme, error: &str, width: f32) {
    let _wrap = ui.push_text_wrap_pos_with_pos(ui.cursor_pos()[0] + width);
    draw_status_line(ui, theme, Status::Danger, error, TextStyle::Small);
}

/// The texts every field shares. The addon translates them once (this crate
/// has no strings of its own) and passes them to each field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextFieldTexts<'a> {
    /// The mark behind the label of an optional field, e.g. "(optional)".
    pub optional: &'a str,
}

/// A labelled text field, configured and drawn in one expression.
#[derive(Debug)]
pub struct TextField<'a> {
    /// ImGui id, unique among the fields of the window.
    id: &'a str,
    /// The visible label.
    label: &'a str,
    /// The edited text.
    value: &'a mut String,
    /// Width of the field in pixels.
    width: f32,
    /// Hint shown while the field is empty.
    placeholder: &'a str,
    /// Whether the field may stay empty, which its label shows.
    is_optional: bool,
    /// The problem to show below the field.
    error: Option<&'a str>,
    /// Whether the field asks for the keyboard focus in this frame.
    takes_focus: bool,
}

impl<'a> TextField<'a> {
    /// A field `width` pixels wide with `label` above it, editing `value`.
    #[must_use]
    pub const fn new(id: &'a str, label: &'a str, value: &'a mut String, width: f32) -> Self {
        Self {
            id,
            label,
            value,
            width,
            placeholder: "",
            is_optional: false,
            error: None,
            takes_focus: false,
        }
    }

    /// Sets the hint shown while the field is empty.
    #[must_use]
    pub const fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    /// Sets whether the field may stay empty; its label then says so.
    #[must_use]
    pub const fn optional(mut self, is_optional: bool) -> Self {
        self.is_optional = is_optional;
        self
    }

    /// Shows `error` below the field and frames the field in `danger`.
    #[must_use]
    pub const fn error(mut self, error: Option<&'a str>) -> Self {
        self.error = error;
        self
    }

    /// Asks for the keyboard focus in this frame if `takes_focus` is set.
    #[must_use]
    pub const fn focus(mut self, takes_focus: bool) -> Self {
        self.takes_focus = takes_focus;
        self
    }

    /// Draws the label, the field and its error at the cursor.
    pub fn draw(mut self, ui: &Ui, theme: &Theme, texts: TextFieldTexts<'_>) -> TextFieldResponse {
        let _id = ui.push_id(self.id);
        let _spacing = ui.push_style_var(StyleVar::ItemSpacing([
            theme.spacing.small,
            theme.spacing.extra_small,
        ]));
        draw_text(ui, theme, TextStyle::Label, self.label);
        if self.is_optional {
            ui.same_line_with_spacing(0.0, theme.spacing.extra_small);
            draw_text(ui, theme, TextStyle::Secondary, texts.optional);
        }

        let start = ui.cursor_screen_pos();
        let height = ui.frame_height() + theme.spacing.small;
        let end = [start[0] + self.width, start[1] + height];
        let is_hovered = ui.is_mouse_hovering_rect(start, end)
            && ui.is_window_hovered_with_flags(
                WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM,
            );
        // The fill depends on the input's state but lies below its text.
        let draw_list = window_draw_list(ui);
        let response = draw_list.with_layer_below(
            || self.draw_input(ui, theme, start, height),
            |response| {
                let fill = if is_hovered && !response.is_active {
                    theme.surface.hover
                } else {
                    theme.surface.input
                };
                draw_list
                    .add_rect(start, end, fill.to_array())
                    .filled(true)
                    .rounding(theme.radius.medium)
                    .build();
            },
        );
        let frame = if self.error.is_some() {
            theme.status.danger
        } else if response.is_active {
            theme.border.focus
        } else {
            theme.border.color
        };
        draw_list
            .add_rect(start, end, frame.to_array())
            .rounding(theme.radius.medium)
            .thickness(theme.border.width)
            .build();
        // Leave the cursor below the field, as a regular item would.
        ui.set_cursor_screen_pos([start[0], end[1]]);
        ui.dummy([0.0, 0.0]);

        if let Some(error) = self.error {
            draw_field_error(ui, theme, error, self.width);
        }
        response
    }

    /// Draws the frameless input inside the field at `start`. It stays clear
    /// of the field's sides, so long text ends before the frame.
    fn draw_input(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        start: [f32; 2],
        height: f32,
    ) -> TextFieldResponse {
        let inset = ui.clone_style().frame_padding[0];
        let _frame = StyleScope::transparent_frame(ui);
        let _hint = ui.push_style_color(StyleColor::TextDisabled, theme.text.secondary.to_array());
        let _border = ui.push_style_var(StyleVar::FrameBorderSize(0.0));
        let _padding = ui.push_style_var(StyleVar::FramePadding([
            0.0,
            (height - ui.text_line_height()) / 2.0,
        ]));
        ui.set_cursor_screen_pos([start[0] + inset, start[1]]);
        ui.set_next_item_width((self.width - 2.0 * inset).max(1.0));
        if self.takes_focus {
            ui.set_keyboard_focus_here();
        }
        let submitted = ui
            .input_text("##input", self.value)
            .hint(self.placeholder)
            .enter_returns_true(true)
            .build();
        TextFieldResponse {
            changed: ui.is_item_edited(),
            submitted,
            is_active: ui.is_item_active(),
        }
    }
}
