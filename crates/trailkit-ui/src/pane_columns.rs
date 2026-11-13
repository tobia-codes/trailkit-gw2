//! Panes side by side with a draggable splitter in each gap (memo M21, M74;
//! ROADMAP R101 Q320), as in memo's main window (note list | note) and the
//! help window (topic list | topic).
//!
//! Every column but the last has a fixed width in average body characters
//! (an "n"), so it follows the font size; the last column fills the rest. A
//! wider window gives the extra room to the last column; a narrower one
//! squeezes the fixed columns only while the later ones lack space. The
//! caller keeps the widths and saves them.

use nexus::imgui::Ui;

use crate::{pane::draw_pane, splitter::draw_vertical_splitter, theme::Theme};

/// The ImGui id of a splitter, unique per gap through `push_id`.
const SPLITTER_ID: &str = "splitter";

/// A column with a width the user drags.
#[derive(Debug, Clone, Copy)]
pub struct FixedColumn<'id> {
    /// The ImGui id of its pane.
    pub id: &'id str,
    /// Its content width in characters, without the pane padding.
    pub characters: f32,
    /// The narrowest it may be dragged, in characters.
    pub minimum_characters: f32,
}

/// The last column, which takes the remaining width.
#[derive(Debug, Clone, Copy)]
pub struct FillColumn<'id> {
    /// The ImGui id of its pane.
    pub id: &'id str,
    /// The room it keeps beside the other columns, in characters.
    pub minimum_characters: f32,
}

/// The columns left to right: the fixed ones, then the one that fills.
#[derive(Debug, Clone, Copy)]
pub struct PaneColumns<'columns> {
    /// The columns with a draggable width.
    pub fixed: &'columns [FixedColumn<'columns>],
    /// The last column.
    pub fill: FillColumn<'columns>,
}

/// What the user did with the splitters in this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PaneColumnsResponse {
    /// While a splitter is dragged: the index of the fixed column left of it
    /// and its new width in characters.
    pub resized: Option<(usize, f32)>,
    /// Whether a drag ended in this frame: the moment to save the widths.
    pub released: bool,
}

/// Draws `columns` over the remaining height with a splitter in each gap;
/// `draw` fills the pane of the column with the given index (the fixed
/// columns in order, then the filling one), left to right.
pub fn draw_resizable_pane_columns(
    ui: &Ui,
    theme: &Theme,
    columns: &PaneColumns<'_>,
    mut draw: impl FnMut(usize),
) -> PaneColumnsResponse {
    let available = ui.content_region_avail();
    let character_width = ui.calc_text_size("n")[0];
    let padding = 2.0 * theme.pane.padding;
    let width_of = |characters: f32| characters * character_width + padding;
    let gap = theme.pane.gap;
    let mut response = PaneColumnsResponse::default();
    let mut remaining = available[0];
    for (index, column) in columns.fixed.iter().enumerate() {
        // The columns to the right keep at least their minimum widths.
        let later_minimum: f32 = columns.fixed[index + 1..]
            .iter()
            .map(|later| width_of(later.minimum_characters) + gap)
            .sum::<f32>()
            + width_of(columns.fill.minimum_characters);
        let minimum = width_of(column.minimum_characters);
        let maximum = (remaining - gap - later_minimum).max(minimum);
        let width = width_of(valid_characters(
            column.characters,
            column.minimum_characters,
        ))
        .clamp(minimum, maximum);
        draw_pane(ui, theme, column.id, [width, 0.0], || draw(index));
        ui.same_line_with_spacing(0.0, 0.0);
        let _splitter_id = ui.push_id(i32::try_from(index).unwrap_or(i32::MAX));
        let splitter = draw_vertical_splitter(
            ui,
            theme,
            SPLITTER_ID,
            width,
            (minimum, maximum),
            available[1],
        );
        if let Some(dragged) = splitter.left_width {
            response.resized = Some((index, (dragged - padding) / character_width));
        }
        response.released |= splitter.released;
        ui.same_line_with_spacing(0.0, 0.0);
        remaining -= width + gap;
    }
    draw_pane(ui, theme, columns.fill.id, [0.0, 0.0], || {
        draw(columns.fixed.len());
    });
    response
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
