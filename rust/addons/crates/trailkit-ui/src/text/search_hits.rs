//! Marks search hits in drawn text (memo plan M122, M197, M198): a filled hit
//! looks like a text marker, an opaque `accent.highlight` fill with the hit
//! drawn again in `text.on_accent`; a framed hit (the other hits of find in
//! a note) gets only an `accent.text` frame. Render thread only.

use std::ops::Range;

use nexus::imgui::Ui;

use crate::{drawing::draw_list::window_draw_list, theme::Theme};

/// How one hit is marked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitMark {
    /// Like a text marker: every hit of the list search, the current hit of
    /// find in a note (M197, M198).
    Filled,
    /// Only a frame: the other hits of find in a note (M198).
    Framed,
}

/// Marks the byte ranges of `text`, which was drawn at `origin` (screen,
/// top left) in the current font, each as its [`HitMark`]. Call it **after**
/// drawing the text: a fill covers the text below it and the hit is drawn
/// again on top, so no edge of the first color shows. Ranges must lie on
/// character boundaries; empty or out-of-range ones are skipped.
pub fn mark_search_hits(
    ui: &Ui,
    theme: &Theme,
    text: &str,
    hits: impl IntoIterator<Item = (Range<usize>, HitMark)>,
    origin: [f32; 2],
) {
    let draw_list = window_draw_list(ui);
    let height = ui.text_line_height();
    for (range, mark) in hits {
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
        let [min, max] = [[left, origin[1]], [right, origin[1] + height]];
        match mark {
            HitMark::Filled => {
                draw_list
                    .add_rect(min, max, theme.accent.highlight.to_array())
                    .filled(true)
                    .rounding(theme.radius.small)
                    .build();
                draw_list.add_text(min, theme.text.on_accent.to_array(), hit);
            }
            HitMark::Framed => {
                draw_list
                    .add_rect(min, max, theme.accent.text.to_array())
                    .rounding(theme.radius.small)
                    .thickness(theme.border.width)
                    .build();
            }
        }
    }
}
