//! The trailkit window title bar (trailkit-ui skill §3, ROADMAP R71).
//!
//! ImGui's own title bar has a filled triangle, a drawn cross and a band in
//! its own color. Trailkit windows hide it (`NO_TITLE_BAR`) and draw this
//! one instead, on the window background: a Lucide chevron that collapses
//! the window, the title in bold and a Lucide `x` that closes it. ImGui
//! still moves the window when its empty areas are dragged; windows without
//! a title bar are exempt from `ConfigWindowsMoveFromTitleBarOnly`.

use nexus::imgui::{Condition, StyleColor, Ui, Window, WindowFlags};

use crate::{
    button::{draw_icon_button, draw_label},
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

/// Builds `window` with the trailkit title bar showing `texts.title` and runs
/// `content` below it unless the window is collapsed. `open` is set to
/// `false` when the close button is clicked.
pub fn draw_window<Label: AsRef<str>>(
    ui: &Ui,
    theme: &Theme,
    window: Window<'_, Label>,
    texts: TitleBarTexts<'_>,
    state: &mut TitleBarState,
    open: &mut bool,
    content: impl FnOnce(),
) {
    let mut flags = WindowFlags::NO_TITLE_BAR | WindowFlags::NO_COLLAPSE;
    let mut window = window;
    if state.collapsed {
        flags |= WindowFlags::NO_RESIZE | WindowFlags::NO_SCROLLBAR;
        let height = ui.frame_height() + 2.0 * ui.clone_style().window_padding[1];
        window = window.size([state.expanded_size[0], height], Condition::Always);
    } else if let Some(size) = state.restore_size.take() {
        window = window.size(size, Condition::Always);
    }
    window.flags(flags).build(ui, || {
        if !state.collapsed {
            state.expanded_size = ui.window_size();
        }
        if draw_title_row(ui, theme, texts, state, open) {
            if state.collapsed {
                state.restore_size = Some(state.expanded_size);
            }
            state.collapsed = !state.collapsed;
        }
        if !state.collapsed {
            content();
        }
    });
}

/// Draws chevron, bold title and close button in one row; returns whether
/// the chevron was clicked.
fn draw_title_row(
    ui: &Ui,
    theme: &Theme,
    texts: TitleBarTexts<'_>,
    state: &TitleBarState,
    open: &mut bool,
) -> bool {
    let _muted = ui.push_style_color(StyleColor::Text, theme.text.secondary.to_array());
    let (icon, tooltip) = if state.collapsed {
        (Icon::CHEVRON_RIGHT, texts.expand)
    } else {
        (Icon::CHEVRON_DOWN, texts.collapse)
    };
    let toggled = draw_icon_button(ui, theme, icon, tooltip);
    let [row_min, row_max] = [ui.item_rect_min(), ui.item_rect_max()];
    {
        let _bold = FontScope::push(ui, FontRole::BodyBold);
        draw_label(
            texts.title,
            row_max[0] + theme.spacing.extra_small,
            [row_min[1], row_max[1]],
            packed(theme.text.primary),
        );
    }
    let close_width = {
        let _icons = FontScope::push(ui, FontRole::Icons);
        let mut buffer = [0; 4];
        ui.calc_text_size(Icon::X.as_str(&mut buffer))[0] + 2.0 * ui.clone_style().frame_padding[0]
    };
    let right = ui.window_content_region_max()[0];
    ui.same_line_with_pos(right - close_width);
    if draw_icon_button(ui, theme, Icon::X, texts.close) {
        *open = false;
    }
    toggled
}
