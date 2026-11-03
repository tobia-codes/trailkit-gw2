//! A neutral badge at the end of a note-list row (M45, M49): a slice
//! describes it, the note list draws it, so neither depends on the other
//! (ROADMAP R76, question 38).

use trailkit_ui::color::Color;

/// What a row badge shows: an outlined rounded box with one letter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RowBadge {
    /// The letter in the box, already localized.
    pub letter: &'static str,
    /// Outline, faint fill and letter color.
    pub color: Color,
}
