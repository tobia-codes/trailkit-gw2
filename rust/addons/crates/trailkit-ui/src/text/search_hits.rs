//! Marks search hits in drawn text like a text marker (memo plan M122, M197):
//! an opaque `accent.highlight` fill over each hit with the hit drawn again
//! in `text.on_accent`. Render thread only.

use std::ops::Range;

use nexus::imgui::Ui;

use crate::{drawing::draw_list::window_draw_list, theme::Theme};

/// Marks the byte `ranges` of `text`, which was drawn at `origin` (screen,
/// top left) in the current font. Call it **after** drawing the text: the
/// fill covers the text below it and the hit is drawn again on top, so no
/// edge of the first color shows. Ranges must lie on character boundaries;
/// empty or out-of-range ones are skipped.
pub fn mark_search_hits(
    ui: &Ui,
    theme: &Theme,
    text: &str,
    ranges: impl IntoIterator<Item = Range<usize>>,
    origin: [f32; 2],
) {
    let draw_list = window_draw_list(ui);
    let height = ui.text_line_height();
    for range in ranges {
        let (Some(before), Some(through), Some(hit)) = (
            text.get(..range.start),
            text.get(..range.end),
            text.get(range.clone()),
        ) else {
            continue;
        };
        if hit.is_empty() {
            continue;
        }
        let left = origin[0] + ui.calc_text_size(before)[0];
        let right = origin[0] + ui.calc_text_size(through)[0];
        draw_list
            .add_rect(
                [left, origin[1]],
                [right, origin[1] + height],
                theme.accent.highlight.to_array(),
            )
            .filled(true)
            .rounding(theme.radius.small)
            .build();
        draw_list.add_text([left, origin[1]], theme.text.on_accent.to_array(), hit);
    }
}
