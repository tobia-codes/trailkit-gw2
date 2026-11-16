//! A neutral badge at the end of a note-list row (M45, M49, M193): a slice
//! describes it, the note list draws it, so neither depends on the other
//! (ROADMAP R76, question 38).

use trailkit_ui::icons::Icon;

/// What a row badge shows: one Lucide icon in `text.secondary` (M193).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowBadge {
    /// The icon, e.g. `calendar-1` for a daily reset note.
    pub icon: Icon,
}
