//! The current window's draw list without imgui-rs' one-at-a-time lock
//! (ROADMAP R104, Q355; crash trap in the nexus-addon-dev skill §2.9).
//!
//! imgui-rs hands out one `DrawListMut` per draw list at a time and panics on
//! a second, e.g. when a tooltip or button draws while its caller still holds
//! one; that panic once crashed the game. [`window_draw_list`] returns a thin
//! handle on the raw `ImDrawList` instead, which any number of helpers may
//! hold at once. Its builders mirror imgui-rs' (`add_line`, `add_rect`,
//! `add_circle`, `add_text`) with the same defaults and the same `ImDrawList_*`
//! calls, so switching draws exactly the same pixels.
//! `Ui::get_window_draw_list` is banned in `clippy.toml`.

use std::{marker::PhantomData, os::raw::c_char};

use nexus::imgui::{ImColor32, Ui, sys};

/// All four corners rounded (`ImDrawCornerFlags_All`), imgui-rs' default.
const ALL_CORNERS: i32 = 0xF;

/// The draw list of the window being drawn; valid for the frame of `'ui`.
/// Copying it is cheap and allowed while other handles exist.
#[derive(Debug, Clone, Copy)]
pub struct WindowDrawList<'ui> {
    /// ImGui's draw list of the current window.
    raw: *mut sys::ImDrawList,
    /// Ties the handle to the frame's `Ui`.
    frame: PhantomData<&'ui Ui<'ui>>,
}

/// The draw list of the window that is being drawn, like
/// `ui.get_window_draw_list()` but safe to take while a caller already
/// holds one.
#[must_use]
pub fn window_draw_list<'ui>(_ui: &'ui Ui<'ui>) -> WindowDrawList<'ui> {
    // SAFETY: a `Ui` exists only inside a frame on the render thread, where
    // ImGui always has a current window with a valid draw list.
    let raw = unsafe { sys::igGetWindowDrawList() };
    WindowDrawList {
        raw,
        frame: PhantomData,
    }
}

impl WindowDrawList<'_> {
    /// A line from `start` to `end` in `color`, 1 px thick until
    /// [`Line::thickness`].
    pub fn add_line(self, start: [f32; 2], end: [f32; 2], color: impl Into<ImColor32>) -> Line {
        Line {
            raw: self.raw,
            start,
            end,
            color: color.into(),
            thickness: 1.0,
        }
    }

    /// A rectangle from the upper-left corner `min` to the lower-right
    /// corner `max` in `color`: an outline 1 px thick with sharp corners
    /// until changed.
    pub fn add_rect(self, min: [f32; 2], max: [f32; 2], color: impl Into<ImColor32>) -> Rect {
        Rect {
            raw: self.raw,
            min,
            max,
            color: color.into(),
            rounding: 0.0,
            thickness: 1.0,
            filled: false,
        }
    }

    /// A circle around `center`: an outline 1 px thick with ImGui's
    /// automatic segment count until changed.
    pub fn add_circle(self, center: [f32; 2], radius: f32, color: impl Into<ImColor32>) -> Circle {
        Circle {
            raw: self.raw,
            center,
            radius,
            color: color.into(),
            segments: 0,
            thickness: 1.0,
            filled: false,
        }
    }

    /// Draws `text` with the current font, its upper-left corner at
    /// `position`.
    pub fn add_text(self, position: [f32; 2], color: impl Into<ImColor32>, text: impl AsRef<str>) {
        let text = text.as_ref();
        let start = text.as_ptr().cast::<c_char>();
        // SAFETY: `raw` is the frame's draw list (see `window_draw_list`);
        // `start..end` is the UTF-8 text, which ImGui reads without a
        // terminating zero and does not keep.
        unsafe {
            let end = start.add(text.len());
            sys::ImDrawList_AddText_Vec2(
                self.raw,
                position.into(),
                color.into().into(),
                start,
                end,
            );
        }
    }
}

/// A line to draw; see [`WindowDrawList::add_line`].
#[must_use = "call `build` to draw the line"]
#[derive(Debug, Clone, Copy)]
pub struct Line {
    /// The draw list.
    raw: *mut sys::ImDrawList,
    /// Start point.
    start: [f32; 2],
    /// End point.
    end: [f32; 2],
    /// Color.
    color: ImColor32,
    /// Thickness in pixels.
    thickness: f32,
}

impl Line {
    /// Sets the thickness in pixels.
    pub const fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    /// Draws the line.
    pub fn build(self) {
        // SAFETY: `raw` is the frame's draw list (see `window_draw_list`).
        unsafe {
            sys::ImDrawList_AddLine(
                self.raw,
                self.start.into(),
                self.end.into(),
                self.color.into(),
                self.thickness,
            );
        }
    }
}

/// A rectangle to draw; see [`WindowDrawList::add_rect`].
#[must_use = "call `build` to draw the rectangle"]
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    /// The draw list.
    raw: *mut sys::ImDrawList,
    /// Upper-left corner.
    min: [f32; 2],
    /// Lower-right corner.
    max: [f32; 2],
    /// Color.
    color: ImColor32,
    /// Corner radius of all four corners.
    rounding: f32,
    /// Outline thickness; unused when filled.
    thickness: f32,
    /// Filled instead of an outline.
    filled: bool,
}

impl Rect {
    /// Rounds all four corners with `rounding` pixels.
    pub const fn rounding(mut self, rounding: f32) -> Self {
        self.rounding = rounding;
        self
    }

    /// Sets the outline thickness in pixels.
    pub const fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    /// Fills the rectangle instead of drawing its outline.
    pub const fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    /// Draws the rectangle.
    pub fn build(self) {
        // SAFETY: `raw` is the frame's draw list (see `window_draw_list`).
        unsafe {
            if self.filled {
                sys::ImDrawList_AddRectFilled(
                    self.raw,
                    self.min.into(),
                    self.max.into(),
                    self.color.into(),
                    self.rounding,
                    ALL_CORNERS,
                );
            } else {
                sys::ImDrawList_AddRect(
                    self.raw,
                    self.min.into(),
                    self.max.into(),
                    self.color.into(),
                    self.rounding,
                    ALL_CORNERS,
                    self.thickness,
                );
            }
        }
    }
}

/// A circle to draw; see [`WindowDrawList::add_circle`].
#[must_use = "call `build` to draw the circle"]
#[derive(Debug, Clone, Copy)]
pub struct Circle {
    /// The draw list.
    raw: *mut sys::ImDrawList,
    /// Center.
    center: [f32; 2],
    /// Radius.
    radius: f32,
    /// Color.
    color: ImColor32,
    /// Segment count; 0 lets ImGui choose.
    segments: u32,
    /// Outline thickness; unused when filled.
    thickness: f32,
    /// Filled instead of an outline.
    filled: bool,
}

impl Circle {
    /// Sets the segment count (0 lets ImGui choose).
    pub const fn num_segments(mut self, segments: u32) -> Self {
        self.segments = segments;
        self
    }

    /// Sets the outline thickness in pixels.
    pub const fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    /// Fills the circle instead of drawing its outline.
    pub const fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    /// Draws the circle.
    pub fn build(self) {
        // ImGui takes the count as `int`; real counts are tiny.
        let segments = i32::try_from(self.segments).unwrap_or(i32::MAX);
        // SAFETY: `raw` is the frame's draw list (see `window_draw_list`).
        unsafe {
            if self.filled {
                sys::ImDrawList_AddCircleFilled(
                    self.raw,
                    self.center.into(),
                    self.radius,
                    self.color.into(),
                    segments,
                );
            } else {
                sys::ImDrawList_AddCircle(
                    self.raw,
                    self.center.into(),
                    self.radius,
                    self.color.into(),
                    segments,
                    self.thickness,
                );
            }
        }
    }
}
