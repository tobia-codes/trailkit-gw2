//! Two panes side by side with a draggable splitter in the gap (memo M21,
//! M74; ROADMAP R101): a list on the left and its content on the right, as
//! in memo's main window and the help window.
//!
//! The left pane has a fixed width in average body characters, so it
//! follows the font size and a wider window gives the extra room to the
//! right pane; a narrower window squeezes the left pane only while the
//! right one lacks space. The caller keeps the width and saves it.

use nexus::imgui::Ui;

use crate::{pane::draw_pane, splitter::draw_vertical_splitter, theme::Theme};

/// The ImGui id of the splitter between the panes.
const SPLITTER_ID: &str = "splitter";

/// Which pane the content callback of [`draw_two_panes`] fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneSide {
    /// The list on the left.
    Left,
    /// The content on the right.
    Right,
}

/// The ids and widths of the two panes, in average characters (an "n" of
/// the current font) of content width, without the pane padding.
#[derive(Debug, Clone, Copy)]
pub struct TwoPanes<'id> {
    /// The ImGui id of the left pane.
    pub left_id: &'id str,
    /// The ImGui id of the right pane.
    pub right_id: &'id str,
    /// The width of the left pane.
    pub left_characters: f32,
    /// The narrowest the left pane may be dragged.
    pub left_minimum_characters: f32,
    /// The room the right pane keeps beside the left one.
    pub right_minimum_characters: f32,
}

/// What the user did with the splitter in this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TwoPanesResponse {
    /// The new width of the left pane in characters while it is dragged.
    pub left_characters: Option<f32>,
    /// Whether the drag ended in this frame: the moment to save the width.
    pub released: bool,
}

/// Draws the two panes over the remaining height with the splitter between
/// them; `draw` fills the pane it is called with, left first.
pub fn draw_two_panes(
    ui: &Ui,
    theme: &Theme,
    panes: &TwoPanes<'_>,
    mut draw: impl FnMut(PaneSide),
) -> TwoPanesResponse {
    let available = ui.content_region_avail();
    let character_width = ui.calc_text_size("n")[0];
    let padding = 2.0 * theme.pane.padding;
    let width_of = |characters: f32| characters * character_width + padding;
    let left_minimum = width_of(panes.left_minimum_characters);
    let usable = available[0] - theme.pane.gap;
    let left_maximum = (usable - width_of(panes.right_minimum_characters)).max(left_minimum);
    let left_width = width_of(valid_characters(
        panes.left_characters,
        panes.left_minimum_characters,
    ))
    .clamp(left_minimum, left_maximum);

    draw_pane(ui, theme, panes.left_id, [left_width, 0.0], || {
        draw(PaneSide::Left);
    });
    ui.same_line_with_spacing(0.0, 0.0);
    let splitter = draw_vertical_splitter(
        ui,
        theme,
        SPLITTER_ID,
        left_width,
        (left_minimum, left_maximum),
        available[1],
    );
    ui.same_line_with_spacing(0.0, 0.0);
    draw_pane(ui, theme, panes.right_id, [0.0, 0.0], || {
        draw(PaneSide::Right);
    });
    TwoPanesResponse {
        left_characters: splitter
            .left_width
            .map(|width| (width - padding) / character_width),
        released: splitter.released,
    }
}

/// `characters` if it is a usable width, otherwise `fallback` (a broken
/// saved value must not break the layout).
fn valid_characters(characters: f32, fallback: f32) -> f32 {
    if characters.is_finite() && characters > 0.0 {
        characters
    } else {
        fallback
    }
}
