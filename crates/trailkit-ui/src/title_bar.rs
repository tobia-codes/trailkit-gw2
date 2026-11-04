//! The trailkit window title bar (trailkit-ui skill §3, ROADMAP R71).
//!
//! ImGui's own title bar has a filled triangle, a drawn cross and a band in
//! its own color. Trailkit windows hide it (`NO_TITLE_BAR`) and draw this
//! one instead, on the window background (ROADMAP R78): the addon's own
//! toggles and the title in bold on the left, a Lucide chevron that
//! collapses the window and a Lucide `x` that closes it on the right. The
//! row uses the window padding, so the first and the last button line up
//! with the panes below. ImGui still moves the window when its empty areas
//! are dragged; windows without a title bar are exempt from
//! `ConfigWindowsMoveFromTitleBarOnly`.

use nexus::imgui::{Condition, StyleColor, Ui, Window, WindowFlags};

use crate::{
    button::{draw_icon_button, draw_icon_toggle, draw_label, icon_button_width},
    dropdown::packed,
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    theme::Theme,
};

/// The title and the tooltips of the title bar buttons, in the addon's
/// language (trailkit-ui has no strings).
#[derive(Debug, Clone, Copy)]
pub struct TitleBarTexts<'texts> {
    /// The window title, shown in bold.
    pub title: &'texts str,
    /// Tooltip of the chevron while the window is expanded.
    pub collapse: &'texts str,
    /// Tooltip of the chevron while the window is collapsed.
    pub expand: &'texts str,
    /// Tooltip of the close button.
    pub close: &'texts str,
}

/// A toggle of the addon on the left of the title (e.g. memo's tool rail
/// and note list, memo M136/M137), drawn with [`draw_icon_toggle`].
#[derive(Debug, Clone, Copy)]
pub struct TitleBarToggle<'texts> {
    /// The Lucide icon.
    pub icon: Icon,
    /// Tooltip naming what the toggle shows or hides.
    pub tooltip: &'texts str,
    /// Whether what it toggles is shown.
    pub active: bool,
}

/// What the title bar remembers between frames: whether the window is
/// collapsed and the size to restore when it is expanded again.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TitleBarState {
    /// Whether only the title bar is shown.
    collapsed: bool,
    /// Window size before collapsing, set again on the next expanded frame.
    restore_size: Option<[f32; 2]>,
    /// The latest window size while expanded.
    expanded_size: [f32; 2],
}

impl TitleBarState {
    /// An expanded window with nothing to restore (usable in a `static`).
    pub const NEW: Self = Self {
        collapsed: false,
        restore_size: None,
        expanded_size: [0.0, 0.0],
    };

    /// Whether the window is collapsed to its title bar.
    #[must_use]
    pub const fn is_collapsed(&self) -> bool {
        self.collapsed
    }
}

/// Builds `window` with the trailkit title bar showing `toggles` and
/// `texts.title` and runs `content` below it unless the window is
/// collapsed. `open` is set to `false` when the close button is clicked.
/// Returns the index of the toggle clicked in this frame; the caller applies
/// it (the content closure may hold the state it changes).
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn draw_window<Label: AsRef<str>>(
    ui: &Ui,
    theme: &Theme,
    window: Window<'_, Label>,
    texts: TitleBarTexts<'_>,
    toggles: &[TitleBarToggle<'_>],
    state: &mut TitleBarState,
    open: &mut bool,
    content: impl FnOnce(),
) -> Option<usize> {
    let mut flags = WindowFlags::NO_TITLE_BAR | WindowFlags::NO_COLLAPSE;
    let mut window = window;
    if state.collapsed {
        flags |= WindowFlags::NO_RESIZE | WindowFlags::NO_SCROLLBAR;
        let height = ui.frame_height() + 2.0 * ui.clone_style().window_padding[1];
        window = window.size([state.expanded_size[0], height], Condition::Always);
    } else if let Some(size) = state.restore_size.take() {
        window = window.size(size, Condition::Always);
    }
    let mut clicked_toggle = None;
    window.flags(flags).build(ui, || {
        if !state.collapsed {
            state.expanded_size = ui.window_size();
        }
        let row = draw_title_row(ui, theme, texts, toggles, state, open);
        clicked_toggle = row.toggle;
        if row.collapse_clicked {
            if state.collapsed {
                state.restore_size = Some(state.expanded_size);
            }
            state.collapsed = !state.collapsed;
        }
        if !state.collapsed {
            content();
        }
    });
    clicked_toggle
}

/// What was clicked in the title row.
#[derive(Debug, Clone, Copy)]
struct TitleRowClicks {
    /// Whether the chevron was clicked.
    collapse_clicked: bool,
    /// The index of the clicked addon toggle.
    toggle: Option<usize>,
}

/// Draws the addon toggles (hidden while collapsed), the bold title, the
/// chevron and the close button in one row.
fn draw_title_row(
    ui: &Ui,
    theme: &Theme,
    texts: TitleBarTexts<'_>,
    toggles: &[TitleBarToggle<'_>],
    state: &TitleBarState,
    open: &mut bool,
) -> TitleRowClicks {
    let _muted = ui.push_style_color(StyleColor::Text, theme.text.secondary.to_array());
    let mut clicks = TitleRowClicks {
        collapse_clicked: false,
        toggle: None,
    };
    let row_top = ui.cursor_screen_pos()[1];
    let mut title_x = ui.cursor_screen_pos()[0];
    if !state.collapsed {
        for (index, toggle) in toggles.iter().enumerate() {
            if index > 0 {
                ui.same_line_with_spacing(0.0, theme.spacing.extra_small);
            }
            if draw_icon_toggle(ui, theme, toggle.icon, toggle.tooltip, toggle.active) {
                clicks.toggle = Some(index);
            }
            title_x = ui.item_rect_max()[0] + theme.spacing.small;
        }
    }
    let row_bottom = row_top + ui.frame_height();
    {
        let _bold = FontScope::push(ui, FontRole::BodyBold);
        draw_label(
            texts.title,
            title_x,
            [row_top, row_bottom],
            packed(theme.text.primary),
        );
    }
    let (icon, tooltip) = if state.collapsed {
        (Icon::CHEVRON_RIGHT, texts.expand)
    } else {
        (Icon::CHEVRON_DOWN, texts.collapse)
    };
    let spacing = theme.spacing.extra_small;
    let right = ui.window_content_region_max()[0];
    let buttons_width = icon_button_width(ui, icon) + spacing + icon_button_width(ui, Icon::X);
    if toggles.is_empty() || state.collapsed {
        // Nothing was drawn on the left yet: keep the row's height.
        ui.dummy([0.0, ui.frame_height()]);
    }
    ui.same_line_with_pos(right - buttons_width);
    clicks.collapse_clicked = draw_icon_button(ui, theme, icon, tooltip);
    ui.same_line_with_spacing(0.0, spacing);
    if draw_icon_button(ui, theme, Icon::X, texts.close) {
        *open = false;
    }
    clicks
}
