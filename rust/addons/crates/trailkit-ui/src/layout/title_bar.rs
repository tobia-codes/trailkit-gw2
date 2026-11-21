//! The trailkit window title bar (trailkit-ui skill §3, ROADMAP R71).
//!
//! ImGui's own title bar has a filled triangle, a drawn cross and a band in
//! its own color. Trailkit windows hide it (`NO_TITLE_BAR`) and draw this
//! one instead, on the window background (ROADMAP R78): the addon's own
//! leading toggles and the title in bold on the left, its trailing toggles
//! (e.g. the help `?`), a Lucide chevron that collapses the window and a
//! Lucide `x` that closes it on the right. The
//! row uses the window padding, so the first and the last button line up
//! with the panes below. Only the title bar moves the window (memo M217
//! follow-up, user wish 2026-11-21): ImGui would move a window by any empty
//! spot, and `ConfigWindowsMoveFromTitleBarOnly` skips windows without its
//! own title bar and would change every addon's windows, so trailkit windows
//! are `NO_MOVE` and the empty parts of this row drag them.

use std::sync::atomic::{AtomicU64, Ordering};

use nexus::imgui::{
    Condition, MouseButton, StyleColor, Ui, Window, WindowFlags, WindowFocusedFlags,
    WindowHoveredFlags, sys,
};

use crate::{
    controls::button::{draw_icon_button, draw_icon_toggle, draw_label, icon_button_width},
    controls::dropdown::packed,
    layout::stack::Stack,
    text::font_registry::FontScope,
    text::fonts::FontRole,
    text::icons::Icon,
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

/// Where a [`TitleBarToggle`] sits in the title row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TogglePlacement {
    /// On the left of the title, e.g. memo's window menu (memo M166).
    #[default]
    Leading,
    /// At the right end, before the chevron and the close button, e.g. the
    /// help `?` (memo Q171, ROADMAP R100).
    Trailing,
}

/// A toggle of the addon in the title row (e.g. memo's window menu, memo
/// M166, or the help `?`), drawn with [`draw_icon_toggle`]. Toggles are
/// hidden while the window is collapsed.
#[derive(Debug, Clone, Copy)]
pub struct TitleBarToggle<'texts> {
    /// The Lucide icon.
    pub icon: Icon,
    /// Tooltip naming what the toggle shows or hides.
    pub tooltip: &'texts str,
    /// Whether what it toggles is shown.
    pub active: bool,
    /// Left of the title or at the right end.
    pub placement: TogglePlacement,
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
    /// The count of [`record_mouse_button_press`] this window has handled.
    seen_mouse_button_presses: u64,
    /// The window position when a drag on the title bar started, while the
    /// left button is held.
    drag_origin: Option<[f32; 2]>,
}

impl TitleBarState {
    /// An expanded window with nothing to restore (usable in a `static`).
    pub const NEW: Self = Self {
        collapsed: false,
        restore_size: None,
        expanded_size: [0.0, 0.0],
        seen_mouse_button_presses: 0,
        drag_origin: None,
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
    let mut flags = WindowFlags::NO_TITLE_BAR | WindowFlags::NO_COLLAPSE | WindowFlags::NO_MOVE;
    let mut window = window;
    if let Some([x, y]) = state.drag_origin {
        if ui.is_mouse_down(MouseButton::Left) {
            let [moved_x, moved_y] = ui.mouse_drag_delta_with_threshold(MouseButton::Left, 0.0);
            window = window.position([x + moved_x, y + moved_y], Condition::Always);
        } else {
            state.drag_origin = None;
        }
    }
    if state.collapsed {
        flags |= WindowFlags::NO_RESIZE | WindowFlags::NO_SCROLLBAR;
        let height = title_row_height(ui) + 2.0 * ui.clone_style().window_padding[1];
        window = window.size([state.expanded_size[0], height], Condition::Always);
    } else if let Some(size) = state.restore_size.take() {
        window = window.size(size, Condition::Always);
    }
    let mut clicked_toggle = None;
    window.flags(flags).build(ui, || {
        release_focus_on_outside_click(ui, state);
        if !state.collapsed {
            state.expanded_size = ui.window_size();
        }
        // The content starts `space.sm` below the title row (R148).
        Stack::vertical(theme).build(ui, |stack| {
            let row = stack.add(|| draw_title_row(ui, theme, texts, toggles, state, open));
            start_drag_on_title_bar(ui, state);
            clicked_toggle = row.toggle;
            if row.collapse_clicked {
                if state.collapsed {
                    state.restore_size = Some(state.expanded_size);
                }
                state.collapsed = !state.collapsed;
            }
            if !state.collapsed {
                stack.add(content);
            }
        });
    });
    clicked_toggle
}

/// Starts moving the window when the left button is pressed on the title
/// row (or the window padding above it) where no button is. Call right after
/// drawing the row, so its buttons are known as hovered.
fn start_drag_on_title_bar(ui: &Ui, state: &mut TitleBarState) {
    if !ui.is_mouse_clicked(MouseButton::Left)
        || !ui.is_window_hovered()
        || ui.is_any_item_hovered()
    {
        return;
    }
    let [_, mouse_y] = ui.io().mouse_pos;
    let window_top = ui.window_pos()[1];
    let row_bottom = ui.item_rect_max()[1];
    if (window_top..=row_bottom).contains(&mouse_y) {
        state.drag_origin = Some(ui.window_pos());
    }
}

/// Right and middle mouse button presses seen by the addon's `WndProc`.
static MOUSE_BUTTON_PRESSES: AtomicU64 = AtomicU64::new(0);

/// Counts a right or middle mouse button press. An addon calls it from its
/// `WndProc` callback (input thread; it only touches an atomic): Nexus does
/// not report clicks into the game world to ImGui (nexus-addon-dev skill
/// §2), so ImGui alone never sees them (memo M186).
pub fn record_mouse_button_press() {
    MOUSE_BUTTON_PRESSES.fetch_add(1, Ordering::Relaxed);
}

/// Takes the focus from the current window when the right or middle mouse
/// button clicks outside every ImGui window, e.g. into the game. ImGui does
/// that only for the left button; keys must not keep going to a window the
/// player clicked away from (memo M186). Must run before the window's
/// content, so its widgets see the click. The press comes from ImGui or from
/// [`record_mouse_button_press`]; each window keeps its own count, so every
/// trailkit window sees every press.
fn release_focus_on_outside_click(ui: &Ui, state: &mut TitleBarState) {
    let presses = MOUSE_BUTTON_PRESSES.load(Ordering::Relaxed);
    let recorded = presses != state.seen_mouse_button_presses;
    state.seen_mouse_button_presses = presses;
    let clicked = recorded
        || ui.is_mouse_clicked(MouseButton::Right)
        || ui.is_mouse_clicked(MouseButton::Middle);
    if clicked
        && ui.is_window_focused_with_flags(WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS)
        && !ui.is_window_hovered_with_flags(WindowHoveredFlags::ANY_WINDOW)
    {
        // Treated as a left click into the void, which ImGui answers itself:
        // a text input that is not hovered lets go when it is drawn, and at
        // the end of the frame ImGui focuses no window. Focusing no window
        // right here instead crashed the game: ImGui had already planned
        // keyboard navigation for this frame and read the then missing
        // navigation window in the next button (`ItemAdd`).
        // SAFETY: called on the render thread inside a frame, so `igGetIO`
        // returns the current context's IO, which lives as long as it.
        unsafe { (*sys::igGetIO()).MouseClicked[0] = true };
    }
}

/// The height of the title row: that of its icon buttons, which are as tall
/// as a button in the body font (Q524). The window's own font can be
/// smaller in game, though not in the tests.
fn title_row_height(ui: &Ui) -> f32 {
    let _body = FontScope::push(ui, FontRole::Body);
    ui.frame_height()
}

/// What was clicked in the title row.
#[derive(Debug, Clone, Copy)]
struct TitleRowClicks {
    /// Whether the chevron was clicked.
    collapse_clicked: bool,
    /// The index of the clicked addon toggle.
    toggle: Option<usize>,
}

/// Draws the leading toggles, the bold title, the trailing toggles (all
/// toggles hidden while collapsed), the chevron and the close button in one
/// row.
fn draw_title_row(
    ui: &Ui,
    theme: &Theme,
    texts: TitleBarTexts<'_>,
    toggles: &[TitleBarToggle<'_>],
    state: &TitleBarState,
    open: &mut bool,
) -> TitleRowClicks {
    let _muted = ui.push_style_color(StyleColor::Text, theme.text.secondary.to_array());
    // The buttons are sized by the body font, like every icon button.
    let _body = FontScope::push(ui, FontRole::Body);
    let mut clicks = TitleRowClicks {
        collapse_clicked: false,
        toggle: None,
    };
    let row_top = ui.cursor_screen_pos()[1];
    let mut title_x = ui.cursor_screen_pos()[0];
    let shown = |placement| {
        toggles
            .iter()
            .enumerate()
            .filter(move |(_, toggle)| !state.collapsed && toggle.placement == placement)
    };
    let mut any_leading = false;
    for (index, toggle) in shown(TogglePlacement::Leading) {
        if any_leading {
            ui.same_line_with_spacing(0.0, theme.spacing.small);
        }
        any_leading = true;
        if draw_icon_toggle(ui, theme, toggle.icon, toggle.tooltip, toggle.active) {
            clicks.toggle = Some(index);
        }
        title_x = ui.item_rect_max()[0] + theme.spacing.small;
    }
    let row_height = title_row_height(ui);
    let row_bottom = row_top + row_height;
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
    let trailing_width: f32 = shown(TogglePlacement::Trailing)
        .map(|(_, toggle)| icon_button_width(ui, toggle.icon) + spacing)
        .sum();
    let buttons_width =
        trailing_width + icon_button_width(ui, icon) + spacing + icon_button_width(ui, Icon::X);
    if !any_leading {
        // Nothing was drawn on the left yet: keep the row's height.
        ui.dummy([0.0, row_height]);
    }
    // Absolute, not `same_line_with_pos`: inside a stack's group that one
    // counts from the group's left edge, not the window's.
    ui.same_line_with_spacing(0.0, 0.0);
    let y = ui.cursor_screen_pos()[1];
    ui.set_cursor_screen_pos([ui.window_pos()[0] + right - buttons_width, y]);
    for (index, toggle) in shown(TogglePlacement::Trailing) {
        if draw_icon_toggle(ui, theme, toggle.icon, toggle.tooltip, toggle.active) {
            clicks.toggle = Some(index);
        }
        ui.same_line_with_spacing(0.0, spacing);
    }
    clicks.collapse_clicked = draw_icon_button(ui, theme, icon, tooltip);
    ui.same_line_with_spacing(0.0, spacing);
    if draw_icon_button(ui, theme, Icon::X, texts.close) {
        *open = false;
    }
    // The toggles keep `spacing.small` to each other and the regular item
    // spacing (also `spacing.small`) to the content below, so the gaps
    // match in both directions.
    clicks
}
