//! A rotating Lucide `loader-circle` for running actions (memo M180): an
//! open arc of the icon's circle, one turn per second. ImGui cannot rotate a
//! glyph, so the arc is drawn as short lines in the icon's stroke width.

use std::f32::consts::TAU;

use nexus::imgui::{ImColor32, Ui};

use crate::{color::Color, dropdown::icon_width};

/// The share of the circle the arc covers (the Lucide glyph leaves a gap).
const ARC_TURNS: f32 = 0.8;

/// Lines per arc; enough for a smooth curve at icon sizes.
const ARC_SEGMENTS: u16 = 16;

/// Turns per second.
const TURNS_PER_SECOND: f32 = 1.0;

/// The spinner's width: the width of an icon in the current size.
#[must_use]
pub fn spinner_width(ui: &Ui) -> f32 {
    icon_width(ui)
}

/// Draws the spinner centered at `center` in `color`, sized like an icon of
/// the current font size, into the current window's draw list. Works while
/// a caller holds the window's `DrawListMut`.
pub fn draw_spinner(ui: &Ui, center: [f32; 2], color: Color) {
    let size = spinner_width(ui);
    // Lucide draws on a 24-unit grid: the circle has radius 9, strokes are 2 wide.
    let radius = size * 9.0 / 24.0;
    let thickness = (size * 2.0 / 24.0).max(1.0);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "ImGui's time in seconds only needs f32 precision for an angle"
    )]
    let start = (ui.time() as f32 * TURNS_PER_SECOND).fract() * TAU;
    let packed = ImColor32::from(color.to_array()).to_bits();
    let point = |step: u16| {
        let angle = start + ARC_TURNS * TAU * f32::from(step) / f32::from(ARC_SEGMENTS);
        nexus::imgui::sys::ImVec2 {
            x: center[0] + radius * angle.cos(),
            y: center[1] + radius * angle.sin(),
        }
    };
    for step in 0..ARC_SEGMENTS {
        // SAFETY: called on the render thread inside a frame, so the current
        // window's draw list is valid; the call only adds a line.
        unsafe {
            nexus::imgui::sys::ImDrawList_AddLine(
                nexus::imgui::sys::igGetWindowDrawList(),
                point(step),
                point(step + 1),
                packed,
                thickness,
            );
        }
    }
}
