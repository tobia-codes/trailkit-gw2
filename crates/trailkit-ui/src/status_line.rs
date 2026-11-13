//! The status line: an icon and a wrapping text in a status color (ROADMAP
//! R96, Q285; `tokens.md` §7.1). State is shown by this line, never by
//! greying out. Dialogs use [`TextStyle::Body`], the log export uses
//! [`TextStyle::Small`].

use nexus::imgui::Ui;

use crate::{
    icons::Icon,
    text::{IconText, TextFlow, TextStyle, draw_icon_text},
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
/// in `style`'s font, both in the status color, wrapped at the current wrap
/// position ([`draw_icon_text`] with [`TextFlow::Wrapped`]). Returns the
/// screen x where the text starts.
pub fn draw_status_line(
    ui: &Ui,
    theme: &Theme,
    status: Status,
    text: &str,
    style: TextStyle,
) -> f32 {
    let color = match status {
        Status::Success => theme.status.success,
        Status::Warning => theme.status.warning,
        Status::Danger => theme.status.danger,
        Status::Info => theme.status.info,
    };
    draw_icon_text(
        ui,
        theme,
        &IconText {
            icon: status.icon(),
            icon_color: color,
            text,
            style,
            text_color: Some(color),
            flow: TextFlow::Wrapped,
        },
    )
}
