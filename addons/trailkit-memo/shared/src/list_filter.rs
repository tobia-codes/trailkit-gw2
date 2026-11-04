//! What the note list needs from an active filter such as the search
//! (ROADMAP R76, question 48): the note-list slice reads it through this
//! trait, so it never depends on the search slice.

use crate::note_tree::{GroupId, NodeId, NoteId};

/// Decides which rows the note list shows and what it shows next to them
/// while a filter is active.
pub trait ListFilter {
    /// Whether the row of `node` is shown.
    fn is_visible(&self, node: NodeId) -> bool;

    /// The note count shown next to `group` (matching notes inside it,
    /// nested groups included, M121).
    fn group_count(&self, group: GroupId) -> usize;

    /// The hit count shown at the end of the row of `note`, if it has one
    /// (M122).
    fn content_hits(&self, note: NoteId) -> Option<usize>;
}
