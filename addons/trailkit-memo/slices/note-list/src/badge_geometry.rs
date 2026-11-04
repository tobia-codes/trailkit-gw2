//! Whole-pixel geometry of a row badge, so its letter sits exactly in the
//! middle at every text size (memo plan M105, R76 question 56).
//!
//! A glyph can only be centered on whole pixels when the space left beside
//! it is even: a 10 px wide "D" in a 25 px box leaves 15 px, split 7/8, and
//! the letter looks half a pixel off. The box therefore grows by one pixel
//! when the parity of box and ink differs. ImGui-free, so it is tested on
//! any host (ROADMAP R23).

/// The box size in whole pixels for content of `ink` pixels: at least
/// `minimum` (rounded), one pixel more if needed so that the space left
/// beside the ink is even.
#[must_use]
pub fn parity_matched_size(minimum: f32, ink: f32) -> f32 {
    let size = minimum.round();
    if (size - ink.round()).rem_euclid(2.0) == 0.0 {
        size
    } else {
        size + 1.0
    }
}

/// The pen position along one axis that centers ink spanning
/// `ink_start..ink_end` (relative to the pen) in a box starting at
/// `box_start` of `box_size` whole pixels; rounded, as ImGui snaps text to
/// whole pixels.
#[must_use]
pub fn centered_pen(box_start: f32, box_size: f32, ink_start: f32, ink_end: f32) -> f32 {
    let ink = (ink_end - ink_start).round();
    (box_start + (box_size - ink) / 2.0 - ink_start).round()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grows_by_one_pixel_when_the_parity_differs() {
        // The in-game case: 25 px box, 10 px "D" → 26 px, 8 px on each side.
        assert!((parity_matched_size(25.0, 10.0) - 26.0).abs() < f32::EPSILON);
        assert!((parity_matched_size(24.6, 10.0) - 26.0).abs() < f32::EPSILON);
    }

    #[test]
    fn keeps_the_size_when_the_parity_matches() {
        assert!((parity_matched_size(24.0, 10.0) - 24.0).abs() < f32::EPSILON);
        assert!((parity_matched_size(25.0, 11.0) - 25.0).abs() < f32::EPSILON);
        assert!((parity_matched_size(18.2, 12.4) - 18.0).abs() < f32::EPSILON);
    }

    #[test]
    fn centers_the_ink_with_equal_space_on_both_sides() {
        // Box 265..291 (26 px), ink 1..11 relative to the pen (10 px).
        let pen = centered_pen(265.0, 26.0, 1.0, 11.0);
        let ink_left = pen + 1.0;
        let ink_right = pen + 11.0;
        assert!((ink_left - 265.0 - (291.0 - ink_right)).abs() < f32::EPSILON);
    }
}
