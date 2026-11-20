//! Vertical and horizontal stacks with one gap between their elements
//! (ROADMAP R148), like a CSS flex container with `gap`.
//!
//! Every gap is measured between visible edges, as the form spacing of R144:
//! an element is wrapped in an ImGui group, whose rectangle ends at its last
//! item without the item spacing ImGui adds after it. No element brings space
//! above or below itself; the stack alone places it. The divider is the one
//! exception: [`StackLayout::add_separator`] brings `space.sm` on both sides
//! in place of the stack's gap. A stack is a group too, so stacks nest: a
//! stack inside another counts as one element. Every element starts on
//! whole pixels, so its edges stay sharp.

use nexus::imgui::Ui;

use crate::{layout::separator::draw_rule, theme::Theme};

/// The direction a [`Stack`] lays its elements out in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Top to bottom, left edges aligned.
    Vertical,
    /// Left to right, top edges aligned.
    Horizontal,
}

/// A stack's direction and gap, configured before [`Stack::build`] draws it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stack {
    /// The direction of the elements.
    axis: Axis,
    /// Pixels between the visible edges of two neighboring elements.
    gap: f32,
}

impl Stack {
    /// A top-to-bottom stack with the default gap `space.sm`.
    #[must_use]
    pub fn vertical(theme: &Theme) -> Self {
        Self {
            axis: Axis::Vertical,
            gap: theme.spacing.small,
        }
    }

    /// A left-to-right stack with the default gap `space.sm`.
    #[must_use]
    pub fn horizontal(theme: &Theme) -> Self {
        Self {
            axis: Axis::Horizontal,
            gap: theme.spacing.small,
        }
    }

    /// Replaces the gap; pass a `theme.spacing` step, e.g. `space.xs` for
    /// list and menu rows.
    #[must_use]
    pub const fn gap(self, gap: f32) -> Self {
        Self { gap, ..self }
    }

    /// Starts the stack at the cursor and lets `content` add its elements
    /// through [`StackLayout::add`]. The whole stack is one ImGui group, so
    /// the caller sees it as one item.
    pub fn build<R>(self, ui: &Ui, content: impl FnOnce(&mut StackLayout<'_>) -> R) -> R {
        ui.group(|| {
            let mut layout = StackLayout {
                ui,
                stack: self,
                origin: round_position(ui.cursor_screen_pos()),
                previous_end: None,
                next_gap: None,
            };
            content(&mut layout)
        })
    }
}

/// The elements of a stack being drawn; handed out by [`Stack::build`].
#[derive(Debug)]
pub struct StackLayout<'ui> {
    /// The frame's UI.
    ui: &'ui Ui<'ui>,
    /// Direction and gap.
    stack: Stack,
    /// Screen position of the stack's top-left corner.
    origin: [f32; 2],
    /// Bottom-right screen corner of the last element that drew something.
    previous_end: Option<[f32; 2]>,
    /// The gap before the next element when a divider sets its own instead
    /// of the stack's.
    next_gap: Option<f32>,
}

impl StackLayout<'_> {
    /// Draws one element with `draw`, `gap` after the previous one. An
    /// element that draws nothing takes no room and no gap.
    pub fn add<R>(&mut self, draw: impl FnOnce() -> R) -> R {
        let gap = self.next_gap.take().unwrap_or(self.stack.gap);
        let start = self.previous_end.map_or(self.origin, |end| {
            next_position(self.stack.axis, self.origin, end, gap)
        });
        self.ui.set_cursor_screen_pos(start);
        let result = self.ui.group(draw);
        if self.ui.item_rect_size() == [0.0, 0.0] {
            self.ui.set_cursor_screen_pos(start);
        } else {
            self.previous_end = Some(self.ui.item_rect_max());
        }
        result
    }

    /// Adds the divider: `space.sm` from the previous element to the line and
    /// from the line to the next element's edge (a button, a field, the fill
    /// of a list row), whatever the stack's gap is.
    ///
    /// # Panics
    ///
    /// In debug builds, if the stack is horizontal: the divider is a
    /// horizontal line.
    pub fn add_separator(&mut self, theme: &Theme) {
        self.add_rule(theme, theme.spacing.small);
    }

    /// Adds the line `space.sm` after the previous element and makes `below`
    /// the gap before the next one.
    fn add_rule(&mut self, theme: &Theme, below: f32) {
        debug_assert_eq!(
            self.stack.axis,
            Axis::Vertical,
            "the divider is a horizontal line"
        );
        self.next_gap = Some(theme.spacing.small);
        let ui = self.ui;
        self.add(|| draw_rule(ui, theme));
        self.next_gap = Some(below);
    }
}

/// Where the next element starts: `gap` below the previous one at the
/// stack's left edge, or `gap` right of it at the stack's top edge, on whole
/// pixels.
fn next_position(axis: Axis, origin: [f32; 2], previous_end: [f32; 2], gap: f32) -> [f32; 2] {
    round_position(match axis {
        Axis::Vertical => [origin[0], previous_end[1] + gap],
        Axis::Horizontal => [previous_end[0] + gap, origin[1]],
    })
}

/// `position` rounded to whole pixels: an element starting between two pixel
/// rows draws its edges blurred across both, and its gaps come out half a
/// pixel off (in-game test 2026-11-20, a text line of 1.5 × 19 px).
fn round_position([x, y]: [f32; 2]) -> [f32; 2] {
    [x.round(), y.round()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_next_element_starts_one_gap_after_the_previous_end() {
        let origin = [10.0, 20.0];
        let end = [110.0, 48.0];
        assert_eq!(
            next_position(Axis::Vertical, origin, end, 8.0),
            [10.0, 56.0]
        );
        assert_eq!(
            next_position(Axis::Horizontal, origin, end, 4.0),
            [114.0, 20.0]
        );
    }

    #[test]
    fn elements_start_on_whole_pixels() {
        let origin = [10.0, 20.0];
        let below = |end_y| next_position(Axis::Vertical, origin, [50.0, end_y], 8.0);
        assert_eq!(below(48.5), [10.0, 57.0]);
        assert_eq!(below(48.4), [10.0, 56.0]);
        assert_eq!(round_position([3.6, 7.2]), [4.0, 7.0]);
    }
}
