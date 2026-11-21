//! One search over a list as the window runs it (M119, BL30, BL31, mock
//! v9): the cached index and results, and the current hit the arrows step
//! through, inside the shown note and on into the next or previous matching
//! note. The note list and the trash each own one.
//!
//! ImGui-free so it is unit-tested on any platform; nothing is rebuilt per
//! frame, only when the field text or the notes change.

use trailkit_memo_shared::note_tree::{NoteId, NotesDocument};
use trailkit_ui::text::text_search::SearchQuery;

use crate::note_search::{HitStop, NoteSearchIndex, SearchResults};

/// Which part of the notes a search looks through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchScope {
    /// The note list.
    Notes,
    /// The trash, with what trashed groups hold (BL31).
    Trash,
}

/// The index, the results of the current query and the current hit, each
/// kept until what it depends on changes (M119).
#[derive(Debug)]
pub struct ListSearch {
    /// What is searched.
    scope: SearchScope,
    /// The index and the document revision it was built from.
    index: Option<(u64, NoteSearchIndex)>,
    /// The results of the current query, if the field holds one.
    current: Option<CurrentSearch>,
    /// Which hit of the shown note the arrows are at.
    cursor: HitCursor,
}

/// Results of one query over one document revision.
#[derive(Debug)]
pub struct CurrentSearch {
    /// The field text the results belong to.
    pub text: String,
    /// The parsed query.
    pub query: SearchQuery,
    /// The document revision the results belong to.
    revision: u64,
    /// What the list shows.
    pub results: SearchResults,
}

/// The current hit inside the shown note.
#[derive(Debug, Default)]
struct HitCursor {
    /// The note the position belongs to; another shown note starts at its
    /// first hit.
    note: Option<NoteId>,
    /// The current hit in the note, 0-based in reading order.
    current: usize,
    /// How many hits the reading view drew in the note in the last frame;
    /// it replaces the index's count, which cannot see chips.
    drawn: Option<usize>,
}

impl HitCursor {
    /// Starts over at the hit `current` of `note`.
    fn start_at(&mut self, note: Option<NoteId>, current: usize) {
        self.note = note;
        self.current = current;
        self.drawn = None;
    }
}

/// What changed when the search was brought up to date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchUpdate {
    /// Nothing that the list shows.
    Unchanged,
    /// The field was emptied: the search ended.
    Ended,
    /// Another query: the list starts expanded and the shown note at its
    /// first hit (M122).
    NewQuery,
    /// The same query over changed notes.
    Refreshed,
}

/// Where the shown note's current hit lies among all hits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitPosition {
    /// The 1-based place of the current hit, `None` while the shown note is
    /// no stop (e.g. a note shown because its group's title matches).
    pub current: Option<usize>,
    /// All hits of all matching notes.
    pub total: usize,
}

/// Where a step with the arrows leads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitStep {
    /// Another hit of the shown note: scroll it into view.
    InsideNote,
    /// The hit `current` of another `note`: select and show it.
    ToNote {
        /// The note to show.
        note: NoteId,
        /// Its current hit.
        current: usize,
    },
}

impl ListSearch {
    /// An empty search over `scope`.
    #[must_use]
    pub fn new(scope: SearchScope) -> Self {
        Self {
            scope,
            index: None,
            current: None,
            cursor: HitCursor::default(),
        }
    }

    /// The current query and its results, while the field holds one.
    #[must_use]
    pub const fn current(&self) -> Option<&CurrentSearch> {
        self.current.as_ref()
    }

    /// The current hit inside the shown note, 0-based.
    #[must_use]
    pub const fn current_hit(&self) -> usize {
        self.cursor.current
    }

    /// Brings index and results up to date with the field `text` and the
    /// notes at `revision`; `shown` starts at its first hit on a new query.
    /// Cheap when nothing changed: the text is compared before it is
    /// parsed (review item 4).
    pub fn update(
        &mut self,
        text: &str,
        document: &NotesDocument,
        revision: u64,
        shown: Option<NoteId>,
    ) -> SearchUpdate {
        if let Some(current) = &self.current
            && current.revision == revision
            && current.text == text
        {
            return SearchUpdate::Unchanged;
        }
        let Some(query) = SearchQuery::parse(text) else {
            return if self.current.take().is_some() {
                SearchUpdate::Ended
            } else {
                SearchUpdate::Unchanged
            };
        };
        let index = match &mut self.index {
            Some((built_from, index)) if *built_from == revision => index,
            slot => {
                let index = match self.scope {
                    SearchScope::Notes => NoteSearchIndex::build(document),
                    SearchScope::Trash => NoteSearchIndex::build_trash(document),
                };
                &mut slot.insert((revision, index)).1
            }
        };
        let results = match self.scope {
            SearchScope::Notes => SearchResults::compute(document, index, &query),
            SearchScope::Trash => SearchResults::compute_trash(document, index, &query),
        };
        let is_new_query = self
            .current
            .as_ref()
            .is_none_or(|current| current.query != query);
        if is_new_query {
            self.cursor.start_at(shown, 0);
        }
        self.current = Some(CurrentSearch {
            text: text.to_owned(),
            query,
            revision,
            results,
        });
        if is_new_query {
            SearchUpdate::NewQuery
        } else {
            SearchUpdate::Refreshed
        }
    }

    /// The hits of `stop`: the drawn count once the reading view knows it,
    /// else the index's.
    fn stop_hits(&self, stop: &HitStop) -> usize {
        match self.cursor.drawn {
            Some(drawn) if self.cursor.note == Some(stop.note) => drawn.max(1),
            _ => stop.hits,
        }
    }

    /// Where the current hit of `shown` lies among all hits, `None` without
    /// a query or without a matching note.
    #[must_use]
    pub fn position(&self, shown: Option<NoteId>) -> Option<HitPosition> {
        let stops = self.current.as_ref()?.results.hit_stops();
        let mut total = 0;
        let mut current = None;
        for stop in stops {
            let hits = self.stop_hits(stop);
            if Some(stop.note) == shown {
                current = Some(total + self.cursor.current.min(hits - 1) + 1);
            }
            total += hits;
        }
        (total > 0).then_some(HitPosition { current, total })
    }

    /// Steps to the next hit (`forward`) or the previous one from `shown`,
    /// wrapping at either end (Q505): inside the note first, then into the
    /// next or previous matching note in list order. `None` without hits.
    pub fn step(&mut self, shown: Option<NoteId>, forward: bool) -> Option<HitStep> {
        let stops = self.current.as_ref()?.results.hit_stops();
        if stops.is_empty() {
            return None;
        }
        let place = stops.iter().position(|stop| Some(stop.note) == shown);
        if let Some(place) = place {
            let hits = self.stop_hits(&stops[place]);
            let inside = self.cursor.current.min(hits - 1);
            let next_inside = if forward {
                (inside + 1 < hits).then_some(inside + 1)
            } else {
                inside.checked_sub(1)
            };
            if let Some(next_inside) = next_inside {
                self.cursor.note = shown;
                self.cursor.current = next_inside;
                return Some(HitStep::InsideNote);
            }
        }
        let count = stops.len();
        let target = match (place, forward) {
            (Some(place), true) => (place + 1) % count,
            (Some(place), false) => (place + count - 1) % count,
            (None, true) => 0,
            (None, false) => count - 1,
        };
        let stop = stops[target];
        let current = if forward { 0 } else { stop.hits - 1 };
        self.cursor.start_at(Some(stop.note), current);
        Some(HitStep::ToNote {
            note: stop.note,
            current,
        })
    }

    /// Takes the hits the reading view drew in `shown`: another note starts
    /// at its first hit, and the current hit stays in range.
    pub fn record_drawn_hits(&mut self, shown: Option<NoteId>, drawn: usize) {
        let cursor = &mut self.cursor;
        if shown != cursor.note {
            cursor.start_at(shown, 0);
        }
        cursor.drawn = Some(drawn);
        if drawn > 0 && cursor.current >= drawn {
            cursor.current = drawn - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use jiff::Timestamp;

    use super::*;
    use trailkit_memo_shared::note_tree::{Note, ResetSchedule, TreeNode};

    /// A document of notes with `bodies`, in order, and their ids.
    fn document(bodies: &[&str]) -> (NotesDocument, Vec<NoteId>) {
        let notes: Vec<Note> = bodies
            .iter()
            .enumerate()
            .map(|(number, body)| {
                let mut note = Note::new(
                    format!("Note {number}"),
                    ResetSchedule::Never,
                    Timestamp::UNIX_EPOCH,
                );
                (*body).clone_into(&mut note.body);
                note
            })
            .collect();
        let ids = notes.iter().map(|note| note.id).collect();
        let document = NotesDocument {
            root: notes.into_iter().map(TreeNode::Note).collect(),
            trash: Vec::new(),
            unknown_fields: Default::default(),
        };
        (document, ids)
    }

    #[test]
    fn steps_wrap_through_every_hit_of_every_note() {
        let (document, ids) = document(&["ore ore", "nothing", "ore"]);
        let mut search = ListSearch::new(SearchScope::Notes);
        assert_eq!(
            search.update("ore", &document, 1, Some(ids[0])),
            SearchUpdate::NewQuery
        );
        let position = |search: &ListSearch, shown| search.position(Some(shown));
        assert_eq!(
            position(&search, ids[0]),
            Some(HitPosition {
                current: Some(1),
                total: 3
            })
        );
        assert_eq!(search.step(Some(ids[0]), true), Some(HitStep::InsideNote));
        assert_eq!(position(&search, ids[0]).and_then(|p| p.current), Some(2));
        // Past the last hit of the note: on to the next matching note.
        assert_eq!(
            search.step(Some(ids[0]), true),
            Some(HitStep::ToNote {
                note: ids[2],
                current: 0
            })
        );
        assert_eq!(position(&search, ids[2]).and_then(|p| p.current), Some(3));
        // Past the last hit of all: back to the first (Q505).
        assert_eq!(
            search.step(Some(ids[2]), true),
            Some(HitStep::ToNote {
                note: ids[0],
                current: 0
            })
        );
        // Backwards from the first: the last hit of the last note.
        search.record_drawn_hits(Some(ids[0]), 2);
        assert_eq!(
            search.step(Some(ids[0]), false),
            Some(HitStep::ToNote {
                note: ids[2],
                current: 0
            })
        );
    }

    #[test]
    fn backwards_into_a_note_lands_on_its_last_hit() {
        let (document, ids) = document(&["ore ore ore", "ore"]);
        let mut search = ListSearch::new(SearchScope::Notes);
        search.update("ore", &document, 1, Some(ids[1]));
        assert_eq!(
            search.step(Some(ids[1]), false),
            Some(HitStep::ToNote {
                note: ids[0],
                current: 2
            })
        );
        assert_eq!(
            search.position(Some(ids[0])),
            Some(HitPosition {
                current: Some(3),
                total: 4
            })
        );
    }

    #[test]
    fn a_shown_note_without_hits_has_no_place() {
        let (document, ids) = document(&["ore", "nothing"]);
        let mut search = ListSearch::new(SearchScope::Notes);
        search.update("ore", &document, 1, Some(ids[1]));
        assert_eq!(
            search.position(Some(ids[1])),
            Some(HitPosition {
                current: None,
                total: 1
            })
        );
        // From outside the stops, forward starts at the first hit.
        assert_eq!(
            search.step(Some(ids[1]), true),
            Some(HitStep::ToNote {
                note: ids[0],
                current: 0
            })
        );
    }

    #[test]
    fn drawn_hits_replace_the_index_count_of_the_shown_note() {
        let (document, ids) = document(&["ore ore"]);
        let mut search = ListSearch::new(SearchScope::Notes);
        search.update("ore", &document, 1, Some(ids[0]));
        search.step(Some(ids[0]), true);
        // The view drew only one hit: the cursor stays in range.
        search.record_drawn_hits(Some(ids[0]), 1);
        assert_eq!(
            search.position(Some(ids[0])),
            Some(HitPosition {
                current: Some(1),
                total: 1
            })
        );
    }

    #[test]
    fn updates_report_what_changed_and_end_on_an_empty_field() {
        let (document, ids) = document(&["ore"]);
        let mut search = ListSearch::new(SearchScope::Notes);
        assert_eq!(
            search.update("", &document, 1, None),
            SearchUpdate::Unchanged
        );
        assert_eq!(
            search.update("ore", &document, 1, None),
            SearchUpdate::NewQuery
        );
        assert_eq!(
            search.update("ore", &document, 1, None),
            SearchUpdate::Unchanged
        );
        assert_eq!(
            search.update("ore", &document, 2, None),
            SearchUpdate::Refreshed
        );
        assert_eq!(
            search.update(" ORE ", &document, 2, None),
            SearchUpdate::Refreshed,
            "same query, other text"
        );
        assert_eq!(search.update("  ", &document, 2, None), SearchUpdate::Ended);
        assert!(search.current().is_none());
        assert_eq!(search.step(Some(ids[0]), true), None);
    }
}
