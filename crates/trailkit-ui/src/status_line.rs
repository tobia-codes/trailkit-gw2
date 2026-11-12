//! The status line: an icon and a wrapping text in a status color (ROADMAP
//! R96, Q285; `tokens.md` §7.1). State is shown by this line, never by
//! greying out. Dialogs use [`TextStyle::Body`], the log export uses
//! [`TextStyle::Small`].

use nexus::imgui::{StyleColor, Ui};

use crate::{
    font_registry::FontScope,
    icons::Icon,
    text::{TextStyle, draw_icon},
    theme::Theme,
};

/// What a status line reports; it fixes the icon and the color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Something worked (`success`, `circle-check`).
    Success,
    /// Something needs attention (`warning`, `triangle-alert`).
    Warning,
    /// Something failed (`danger`, `circle-alert`).
    Danger,
    /// Something to know, e.g. why a button is disabled (`info`, `info`).
    Info,
}

impl Status {
    /// The icon in front of the text.
    #[must_use]
    pub const fn icon(self) -> Icon {
        match self {
            Self::Success => Icon::CIRCLE_CHECK,
            Self::Warning => Icon::TRIANGLE_ALERT,
            Self::Danger => Icon::CIRCLE_ALERT,
            Self::Info => Icon::INFO,
        }
    }
}

/// Draws the status line at the cursor: the icon of `status`, then `text`
/// in `style`'s font and the status color, wrapped at the current wrap
/// position (the window's right edge unless one is pushed). Continuation
/// lines hang under the text, not under the icon.
pub fn draw_status_line(ui: &Ui, theme: &Theme, status: Status, text: &str, style: TextStyle) {
    let color = match status {
        Status::Success => theme.status.success,
        Status::Warning => theme.status.warning,
        Status::Danger => theme.status.danger,
        Status::Info => theme.status.info,
    };
    let top = ui.cursor_screen_pos()[1];
    draw_icon(ui, status.icon(), color);
    ui.same_line_with_spacing(0.0, theme.spacing.extra_small * 1.5);
    let text_left = ui.cursor_screen_pos()[0];
    // Wrapped lines start at the window's content edge; indent them to the
    // text and put the first line back where the icon's line continues.
    let content_left = ui.window_pos()[0] + ui.window_content_region_min()[0];
    let hang = text_left - content_left;
    ui.indent_by(hang);
    ui.set_cursor_screen_pos([text_left, top]);
    {
        let _font = FontScope::push(ui, style.font());
        let _color = ui.push_style_color(StyleColor::Text, color.to_array());
        ui.text_wrapped(text);
    }
    ui.unindent_by(hang);
}
