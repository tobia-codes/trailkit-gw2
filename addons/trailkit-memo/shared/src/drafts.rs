//! Drafts: uncommitted edits kept across an unload, stored as `drafts.json`
//! (memo plan M54, M55).
//!
//! A draft is written only when the addon is unloaded (hot reload, game exit)
//! while a note has unsaved changes. The next time that note is opened, memo
//! offers to restore or discard it. Drafts are cleaned up so they never pile
//! up: one per note, removed on restore, discard or regular save, and pruned
//! at start when their note is gone, when older than [`DRAFT_RETENTION`] or
//! beyond [`MAXIMUM_DRAFTS`]. An empty draft list deletes the file.
//!
//! Platform-independent so it can be unit-tested on any host (ROADMAP R23).

use jiff::{SignedDuration, Timestamp};
use serde::{Deserialize, Serialize};
use trailkit_storage::{SchemaVersion, VersionedDocument};

use crate::note_tree::NoteId;

/// Drafts older than this are deleted at start (same as the trash, M12).
pub const DRAFT_RETENTION: SignedDuration = SignedDuration::from_hours(30 * 24);

/// At most this many drafts are kept; the oldest go first (safety net).
pub const MAXIMUM_DRAFTS: usize = 20;

/// Uncommitted title and body of one note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Draft {
    /// The note the draft belongs to.
    pub note: NoteId,
    /// The edited title.
    pub title: String,
    /// The edited markdown body.
    pub body: String,
    /// When the draft was written.
    pub saved_at: Timestamp,
    /// The note's `updated_at` when the draft was written; if the note
    /// changed since, the restore hint says so.
    pub note_updated_at: Timestamp,
}

/// All drafts, persisted as `drafts.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DraftsDocument {
    /// The drafts, at most one per note.
    pub drafts: Vec<Draft>,
}

impl VersionedDocument for DraftsDocument {
    const FILE_STEM: &'static str = "drafts";
    const CURRENT_SCHEMA_VERSION: SchemaVersion = SchemaVersion::FIRST;
}

impl DraftsDocument {
    /// The draft of `note`, if any.
    #[must_use]
    pub fn get(&self, note: NoteId) -> Option<&Draft> {
        self.drafts.iter().find(|draft| draft.note == note)
    }

    /// Stores `draft`, replacing an older draft of the same note, and drops
    /// the oldest drafts beyond [`MAXIMUM_DRAFTS`].
    pub fn upsert(&mut self, draft: Draft) {
        self.drafts.retain(|existing| existing.note != draft.note);
        self.drafts.push(draft);
        self.enforce_maximum();
    }

    /// Removes and returns the draft of `note`.
    pub fn take(&mut self, note: NoteId) -> Option<Draft> {
        let index = self.drafts.iter().position(|draft| draft.note == note)?;
        Some(self.drafts.remove(index))
    }

    /// Deletes drafts whose note no longer exists, drafts older than
    /// [`DRAFT_RETENTION`] at `now`, and the oldest beyond [`MAXIMUM_DRAFTS`].
    /// Returns whether anything was removed.
    pub fn prune(&mut self, now: Timestamp, note_exists: impl Fn(NoteId) -> bool) -> bool {
        let before = self.drafts.len();
        self.drafts.retain(|draft| {
            note_exists(draft.note) && now.duration_since(draft.saved_at) <= DRAFT_RETENTION
        });
        self.enforce_maximum();
        self.drafts.len() != before
    }

    /// Keeps only the newest [`MAXIMUM_DRAFTS`] drafts.
    fn enforce_maximum(&mut self) {
        if self.drafts.len() > MAXIMUM_DRAFTS {
            self.drafts.sort_by_key(|draft| draft.saved_at);
            let excess = self.drafts.len() - MAXIMUM_DRAFTS;
            self.drafts.drain(..excess);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A draft of a new note, saved `hours_ago` hours before `now`.
    fn draft(now: Timestamp, hours_ago: i64) -> Draft {
        Draft {
            note: NoteId::new_unique(),
            title: "Title".to_owned(),
            body: "Body".to_owned(),
            saved_at: now - SignedDuration::from_hours(hours_ago),
            note_updated_at: now,
        }
    }

    #[test]
    fn one_draft_per_note() {
        let now = Timestamp::now();
        let mut drafts = DraftsDocument::default();
        let first = draft(now, 1);
        let note = first.note;
        drafts.upsert(first);
        drafts.upsert(
            Draft {
                body: "Newer".to_owned(),
                ..draft(now, 0)
            }
            .with_note(note),
        );
        assert_eq!(drafts.drafts.len(), 1);
        assert_eq!(
            drafts.get(note).map(|draft| draft.body.as_str()),
            Some("Newer")
        );
        assert!(drafts.take(note).is_some());
        assert!(drafts.drafts.is_empty());
    }

    #[test]
    fn prune_removes_orphaned_expired_and_excess_drafts() {
        let now = Timestamp::now();
        let orphan = draft(now, 1);
        let orphan_note = orphan.note;
        let mut drafts = DraftsDocument {
            drafts: vec![orphan, draft(now, 31 * 24)],
        };
        for hours_ago in 0..25 {
            drafts.drafts.push(draft(now, hours_ago));
        }
        assert!(drafts.prune(now, |note| note != orphan_note));
        assert_eq!(drafts.drafts.len(), MAXIMUM_DRAFTS);
        assert!(drafts.get(orphan_note).is_none());
        assert!(
            drafts
                .drafts
                .iter()
                .all(|draft| now.duration_since(draft.saved_at)
                    < SignedDuration::from_hours(MAXIMUM_DRAFTS as i64))
        );
        assert!(!drafts.prune(now, |_| true), "nothing left to prune");
    }

    impl Draft {
        /// The same draft for another note (test helper).
        fn with_note(self, note: NoteId) -> Self {
            Self { note, ..self }
        }
    }
}
