//! A one-line text input with a visible label (`form-labels`, ROADMAP R139):
//! the label above the field, an "(optional)" mark switched on per field, a
//! placeholder and an error directly below it (`form-errors`).

use nexus::imgui::{StyleColor, StyleVar, Ui};

use crate::{
    drawing::draw_list::window_draw_list,
    text::icons::Icon,
    text::{IconText, TextFlow, TextStyle, draw_icon_text, draw_text},
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
    pub fn draw(self, ui: &Ui, theme: &Theme, texts: TextFieldTexts<'_>) -> TextFieldResponse {
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
        let response = {
            let _fill = ui.push_style_color(StyleColor::FrameBg, theme.surface.input.to_array());
            let _hover =
                ui.push_style_color(StyleColor::FrameBgHovered, theme.surface.hover.to_array());
            let _active =
                ui.push_style_color(StyleColor::FrameBgActive, theme.surface.input.to_array());
            let _hint =
                ui.push_style_color(StyleColor::TextDisabled, theme.text.secondary.to_array());
            let _rounding = ui.push_style_var(StyleVar::FrameRounding(theme.radius.medium));
            let _border = ui.push_style_var(StyleVar::FrameBorderSize(0.0));
            ui.set_next_item_width(self.width);
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
        };

        let frame = if self.error.is_some() {
            theme.status.danger
        } else if response.is_active {
            theme.border.focus
        } else {
            theme.border.color
        };
        let end = [start[0] + self.width, start[1] + ui.frame_height()];
        window_draw_list(ui)
            .add_rect(start, end, frame.to_array())
            .rounding(theme.radius.medium)
            .thickness(theme.border.width)
            .build();

        if let Some(error) = self.error {
            let _wrap = ui.push_text_wrap_pos_with_pos(ui.cursor_pos()[0] + self.width);
            draw_icon_text(
                ui,
                theme,
                &IconText {
                    icon: Icon::CIRCLE_ALERT,
                    icon_color: theme.status.danger,
                    text: error,
                    style: TextStyle::Small,
                    text_color: Some(theme.status.danger),
                    flow: TextFlow::Wrapped,
                },
            );
        }
        response
    }
}
