//! Whole-pixel geometry of a row badge, so its letter sits exactly in the
//! middle at every text size (memo plan M105, M132).
//!
//! ImGui draws text at whole-pixel pen positions, so a letter's ink centre
//! keeps the fraction it has relative to the pen (e.g. 0.5 px). The box's
//! centre lies on a whole pixel for an even size and on a half pixel for an
//! odd one; the box therefore takes the parity that matches the ink centre.
//! The ink *width* is no guide: oversampling makes the measured width
//! fractional, and rounding it flips the parity. ImGui-free, so it is tested
//! on any host (ROADMAP R23).

/// The box size in whole pixels: at least `minimum` (rounded), one pixel
/// more if needed so that the box centre has the same fraction as
/// `ink_center` (the ink's centre relative to the pen).
#[must_use]
pub fn center_matched_size(minimum: f32, ink_center: f32) -> f32 {
    let size = minimum.round();
    let fraction = ink_center.rem_euclid(1.0);
    let wants_odd = (0.25..0.75).contains(&fraction);
    let is_odd = size.rem_euclid(2.0) >= 1.0;
    if wants_odd == is_odd {
        size
    } else {
        size + 1.0
    }
}

/// The whole-pixel pen position along one axis that puts the centre of ink
/// spanning `ink_start..ink_end` (relative to the pen) on the centre of a
/// box starting at `box_start` with `box_size` pixels.
#[must_use]
pub fn centered_pen(box_start: f32, box_size: f32, ink_start: f32, ink_end: f32) -> f32 {
    (box_start + box_size / 2.0 - (ink_start + ink_end) / 2.0).round()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Whether ink spanning `ink` relative to the pen ends up exactly in the
    /// middle of a box of `minimum` starting at `box_start`.
    fn is_centered(box_start: f32, minimum: f32, ink: [f32; 2]) -> bool {
        let size = center_matched_size(minimum, f32::midpoint(ink[0], ink[1]));
        let pen = centered_pen(box_start, size, ink[0], ink[1]);
        let left = pen + ink[0] - box_start;
        let right = box_start + size - (pen + ink[1]);
        (left - right).abs() < 0.5 + f32::EPSILON
    }

    #[test]
    fn grows_to_an_even_box_for_ink_centred_on_a_whole_pixel() {
        // The in-game case: 25 px box, "D" centred on a whole pixel → 26 px.
        assert!((center_matched_size(25.0, 5.0) - 26.0).abs() < f32::EPSILON);
        assert!((center_matched_size(24.0, 5.1) - 24.0).abs() < f32::EPSILON);
    }

    #[test]
    fn grows_to_an_odd_box_for_ink_centred_on_a_half_pixel() {
        assert!((center_matched_size(24.0, 4.5) - 25.0).abs() < f32::EPSILON);
        assert!((center_matched_size(25.0, 4.6) - 25.0).abs() < f32::EPSILON);
    }

    #[test]
    fn centres_ink_of_any_width_and_phase() {
        for minimum in [18.0, 18.6, 19.0, 25.0, 26.4] {
            for ink in [[1.0, 11.0], [0.7, 10.1], [1.3, 9.9], [0.4, 9.4]] {
                assert!(is_centered(265.0, minimum, ink), "{minimum} {ink:?}");
            }
        }
    }
}
