//! Notes, groups, tree entries and trash entries (memo plan M12, M13,
//! M23–M25).

use jiff::Timestamp;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};
use serde_json::Value;
use trailkit_storage::UnknownFields;

use super::ids::{GroupId, NodeId, NoteId};

/// Stored key of [`Note::reset_schedule`] (M203).
const RESET_SCHEDULE_KEY: &str = "reset_schedule";

/// Stored key of the reset state of a `daily`/`weekly` note.
const LAST_RESET_APPLIED_AT_KEY: &str = "last_reset_applied_at";

/// How often a reset note is reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResetPeriod {
    /// Every day at 00:00 UTC.
    Daily,
    /// Every Monday at 07:30 UTC.
    Weekly,
}

impl ResetPeriod {
    /// Both periods, in the order the kind dropdown lists them.
    pub const ALL: [Self; 2] = [Self::Daily, Self::Weekly];
}

/// What happens to a note's checkboxes at the game's reset times (M13).
///
/// Stored flat in the note as `"reset_schedule"` (`never`, `daily`,
/// `weekly`, M203) plus, for `daily`/`weekly`, `"last_reset_applied_at"`; a
/// note that is never reset cannot carry a reset time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetSchedule {
    /// A plain note; nothing is reset.
    Never,
    /// All checkboxes are unchecked at the daily reset (00:00 UTC).
    Daily {
        /// The reset that was last applied to this note.
        last_reset_applied_at: Timestamp,
    },
    /// All checkboxes are unchecked at the weekly reset (Monday 07:30 UTC).
    Weekly {
        /// The reset that was last applied to this note.
        last_reset_applied_at: Timestamp,
    },
    /// A schedule a newer version added (M202). Behaves like [`Self::Never`];
    /// the stored values stay in [`Note::unknown_fields`] and are written
    /// back unchanged until the user picks another schedule.
    Unrecognized,
}

/// A markdown note.
///
/// Stored through [`StoredNote`] so that an unknown `reset_schedule` value
/// survives a save (M202).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "StoredNote")]
pub struct Note {
    /// Stable unique id.
    pub id: NoteId,
    /// Title shown in the note list.
    pub title: String,
    /// The note content as markdown.
    pub body: String,
    /// Whether and when the note's checkboxes are reset, with the reset
    /// state (M13, M203).
    pub reset_schedule: ResetSchedule,
    /// When the note was created.
    pub created_at: Timestamp,
    /// When the note's title or content was last changed by the user; reset
    /// changes and resets do not count (M86, M88).
    pub updated_at: Timestamp,
    /// Fields a newer version added, kept on save (M202).
    pub unknown_fields: UnknownFields,
}

/// The stored form of a [`Note`] with the reset schedule still untyped.
#[derive(Deserialize)]
struct StoredNote {
    /// See [`Note::id`].
    id: NoteId,
    /// See [`Note::title`].
    title: String,
    /// See [`Note::body`].
    body: String,
    /// `never`, `daily`, `weekly` or a value of a newer version.
    reset_schedule: Value,
    /// The reset state of `daily`/`weekly`.
    #[serde(default)]
    last_reset_applied_at: Option<Value>,
    /// See [`Note::created_at`].
    created_at: Timestamp,
    /// See [`Note::updated_at`].
    updated_at: Timestamp,
    /// See [`Note::unknown_fields`].
    #[serde(flatten)]
    unknown_fields: UnknownFields,
}

impl TryFrom<StoredNote> for Note {
    type Error = String;

    /// Types the reset schedule; an unknown value becomes
    /// [`ResetSchedule::Unrecognized`] and keeps its stored fields.
    ///
    /// # Errors
    ///
    /// Returns a reason if a `daily`/`weekly` note has no valid
    /// `last_reset_applied_at`.
    fn try_from(stored: StoredNote) -> Result<Self, Self::Error> {
        let StoredNote {
            id,
            title,
            body,
            reset_schedule,
            last_reset_applied_at,
            created_at,
            updated_at,
            mut unknown_fields,
        } = stored;
        let reset_time = || -> Result<Timestamp, String> {
            let value = last_reset_applied_at
                .clone()
                .ok_or_else(|| format!("note {id:?}: missing {LAST_RESET_APPLIED_AT_KEY}"))?;
            serde_json::from_value(value).map_err(|error| format!("note {id:?}: {error}"))
        };
        let reset_schedule = match reset_schedule.as_str() {
            Some("never") => ResetSchedule::Never,
            Some("daily") => ResetSchedule::Daily {
                last_reset_applied_at: reset_time()?,
            },
            Some("weekly") => ResetSchedule::Weekly {
                last_reset_applied_at: reset_time()?,
            },
            _ => {
                unknown_fields.insert(RESET_SCHEDULE_KEY.to_owned(), reset_schedule);
                if let Some(value) = last_reset_applied_at {
                    unknown_fields.insert(LAST_RESET_APPLIED_AT_KEY.to_owned(), value);
                }
                ResetSchedule::Unrecognized
            }
        };
        Ok(Self {
            id,
            title,
            body,
            reset_schedule,
            created_at,
            updated_at,
            unknown_fields,
        })
    }
}

impl Serialize for Note {
    /// Writes the note flat: known fields, the reset schedule, then the
    /// unknown fields. A known schedule replaces stored values of an
    /// unrecognized one, so they are never written twice.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("title", &self.title)?;
        map.serialize_entry("body", &self.body)?;
        let (schedule_name, reset_time) = match self.reset_schedule {
            ResetSchedule::Never => (Some("never"), None),
            ResetSchedule::Daily {
                last_reset_applied_at,
            } => (Some("daily"), Some(last_reset_applied_at)),
            ResetSchedule::Weekly {
                last_reset_applied_at,
            } => (Some("weekly"), Some(last_reset_applied_at)),
            ResetSchedule::Unrecognized => (None, None),
        };
        if let Some(name) = schedule_name {
            map.serialize_entry(RESET_SCHEDULE_KEY, name)?;
        }
        if let Some(time) = reset_time {
            map.serialize_entry(LAST_RESET_APPLIED_AT_KEY, &time)?;
        }
        map.serialize_entry("created_at", &self.created_at)?;
        map.serialize_entry("updated_at", &self.updated_at)?;
        for (key, value) in self.unknown_fields.iter() {
            let replaced = schedule_name.is_some()
                && (key == RESET_SCHEDULE_KEY || key == LAST_RESET_APPLIED_AT_KEY);
            if !replaced {
                map.serialize_entry(key, value)?;
            }
        }
        map.end()
    }
}

/// A group of notes and subgroups.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    /// Stable unique id.
    pub id: GroupId,
    /// Title shown in the note list.
    pub title: String,
    /// Notes and subgroups in display order.
    pub children: Vec<TreeNode>,
    /// Whether the group is expanded in the note list (kept across sessions).
    pub expanded: bool,
    /// Fields a newer version added, kept on save (M202).
    #[serde(flatten)]
    pub unknown_fields: UnknownFields,
}

/// One entry of the tree: a note or a group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TreeNode {
    /// A note.
    Note(Note),
    /// A group with its own children.
    Group(Group),
}

/// A deleted note or group waiting in the trash (M12, M25).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrashEntry {
    /// The deleted note or group, including all its children.
    pub node: TreeNode,
    /// When it was deleted; it is purged 30 days later (M12).
    pub deleted_at: Timestamp,
    /// The group it was in, or `None` if it was on the root level.
    pub former_parent: Option<GroupId>,
    /// Its position within the former parent's children.
    pub former_index: usize,
    /// Fields a newer version added, kept on save (M202).
    #[serde(flatten)]
    pub unknown_fields: UnknownFields,
}

impl ResetSchedule {
    /// A reset schedule with `period` (`None` = never reset) whose reset state
    /// starts at `now`, so checks made before `now` stay until the next
    /// reset (M90).
    #[must_use]
    pub const fn with_period(period: Option<ResetPeriod>, now: Timestamp) -> Self {
        match period {
            None => Self::Never,
            Some(ResetPeriod::Daily) => Self::Daily {
                last_reset_applied_at: now,
            },
            Some(ResetPeriod::Weekly) => Self::Weekly {
                last_reset_applied_at: now,
            },
        }
    }

    /// The reset period, or `None` if the note is never reset.
    #[must_use]
    pub const fn period(self) -> Option<ResetPeriod> {
        match self {
            Self::Never | Self::Unrecognized => None,
            Self::Daily { .. } => Some(ResetPeriod::Daily),
            Self::Weekly { .. } => Some(ResetPeriod::Weekly),
        }
    }

    /// The reset state, or `None` if the note is never reset.
    #[must_use]
    pub const fn last_reset_applied_at(self) -> Option<Timestamp> {
        match self {
            Self::Never | Self::Unrecognized => None,
            Self::Daily {
                last_reset_applied_at,
            }
            | Self::Weekly {
                last_reset_applied_at,
            } => Some(last_reset_applied_at),
        }
    }
}

impl Note {
    /// Creates a new, empty note with `reset_schedule` created at `now`.
    ///
    /// For daily/weekly schedules, the reset state starts at `now`, so the first reset
    /// after creation is applied.
    #[must_use]
    pub fn new(title: String, reset_schedule: ResetSchedule, now: Timestamp) -> Self {
        Self {
            id: NoteId::new_unique(),
            title,
            body: String::new(),
            reset_schedule,
            created_at: now,
            updated_at: now,
            unknown_fields: UnknownFields::default(),
        }
    }
}

impl Group {
    /// Creates a new, empty and expanded group.
    #[must_use]
    pub fn new(title: String) -> Self {
        Self {
            id: GroupId::new_unique(),
            title,
            children: Vec::new(),
            expanded: true,
            unknown_fields: UnknownFields::default(),
        }
    }

    /// Number of notes in this group, including those in nested groups
    /// (shown next to the group title, M50).
    #[must_use]
    pub fn note_count(&self) -> usize {
        self.children
            .iter()
            .map(|child| match child {
                TreeNode::Note(_) => 1,
                TreeNode::Group(group) => group.note_count(),
            })
            .sum()
    }

    /// Whether a note in this group, nested groups included, satisfies
    /// `predicate` (e.g. a collapsed group with unsaved changes, M185).
    pub fn any_note(&self, predicate: &impl Fn(NoteId) -> bool) -> bool {
        self.children.iter().any(|child| match child {
            TreeNode::Note(note) => predicate(note.id),
            TreeNode::Group(group) => group.any_note(predicate),
        })
    }
}

impl TreeNode {
    /// The id of this note or group.
    #[must_use]
    pub const fn id(&self) -> NodeId {
        match self {
            Self::Note(note) => NodeId::Note(note.id),
            Self::Group(group) => NodeId::Group(group.id),
        }
    }

    /// The title shown in the note list.
    #[must_use]
    pub fn title(&self) -> &str {
        match self {
            Self::Note(note) => &note.title,
            Self::Group(group) => &group.title,
        }
    }

    /// Whether this is the note `id` or a group holding it, nested groups
    /// included.
    #[must_use]
    pub fn contains_note(&self, id: NoteId) -> bool {
        match self {
            Self::Note(note) => note.id == id,
            Self::Group(group) => group.children.iter().any(|child| child.contains_note(id)),
        }
    }

    /// Calls `visit` with the id of this note or of every note inside this
    /// group, nested groups included.
    pub fn for_each_note_id(&self, visit: &mut impl FnMut(NoteId)) {
        match self {
            Self::Note(note) => visit(note.id),
            Self::Group(group) => {
                for child in &group.children {
                    child.for_each_note_id(visit);
                }
            }
        }
    }

    /// The note `id` if it is this note or inside this group.
    #[must_use]
    pub fn find_note(&self, id: NoteId) -> Option<&Note> {
        match self {
            Self::Note(note) => (note.id == id).then_some(note),
            Self::Group(group) => group.children.iter().find_map(|child| child.find_note(id)),
        }
    }
}

/// Tests of the node types; `noon` is shared with the document tests.
#[cfg(test)]
pub(super) mod tests {
    use super::*;

    /// A fixed point in time.
    pub(in crate::note_tree) fn noon() -> Timestamp {
        "2026-11-01T12:00:00Z".parse().unwrap_or_default()
    }

    #[test]
    fn note_is_stored_flat_with_reset_schedule_and_type_tags() -> serde_json::Result<()> {
        let note = Note::new(
            "Dailies".to_owned(),
            ResetSchedule::Daily {
                last_reset_applied_at: noon(),
            },
            noon(),
        );
        let value = serde_json::to_value(TreeNode::Note(note))?;
        assert_eq!(value["type"], "note");
        assert_eq!(value["reset_schedule"], "daily");
        assert_eq!(value["last_reset_applied_at"], "2026-11-01T12:00:00Z");
        assert_eq!(value["created_at"], "2026-11-01T12:00:00Z");
        Ok(())
    }
    #[test]
    fn never_reset_note_has_no_reset_time() -> serde_json::Result<()> {
        let note = Note::new("Ideas".to_owned(), ResetSchedule::Never, noon());
        let value = serde_json::to_value(note)?;
        assert_eq!(value["reset_schedule"], "never");
        assert!(value.get("last_reset_applied_at").is_none());
        Ok(())
    }
    #[test]
    fn keeps_fields_of_a_newer_version() -> serde_json::Result<()> {
        let stored = serde_json::json!({
            "type": "note", "id": "0190a9d4-0000-7000-8000-000000000001",
            "title": "A", "body": "", "reset_schedule": "daily",
            "last_reset_applied_at": "2026-11-01T12:00:00Z",
            "created_at": "2026-11-01T12:00:00Z", "updated_at": "2026-11-01T12:00:00Z",
            "color": "red"
        });
        let node: TreeNode = serde_json::from_value(stored.clone())?;
        assert_eq!(serde_json::to_value(&node)?, stored);
        let group = serde_json::json!({
            "type": "group", "id": "0190a9d4-0000-7000-8000-000000000002",
            "title": "G", "children": [], "expanded": true, "pinned": true
        });
        let node: TreeNode = serde_json::from_value(group.clone())?;
        assert_eq!(serde_json::to_value(&node)?, group);
        Ok(())
    }

    #[test]
    fn keeps_an_unknown_reset_schedule_until_changed() -> serde_json::Result<()> {
        let stored = serde_json::json!({
            "id": "0190a9d4-0000-7000-8000-000000000001",
            "title": "A", "body": "", "reset_schedule": "monthly",
            "last_reset_applied_at": "2026-11-01T12:00:00Z",
            "created_at": "2026-11-01T12:00:00Z", "updated_at": "2026-11-01T12:00:00Z",
            "reset_day": 3
        });
        let mut note: Note = serde_json::from_value(stored.clone())?;
        assert_eq!(note.reset_schedule, ResetSchedule::Unrecognized);
        assert_eq!(note.reset_schedule.period(), None);
        assert_eq!(serde_json::to_value(&note)?, stored);

        note.reset_schedule = ResetSchedule::Never;
        let saved = serde_json::to_value(&note)?;
        assert_eq!(saved["reset_schedule"], "never");
        assert!(saved.get("last_reset_applied_at").is_none());
        assert_eq!(saved["reset_day"], 3, "other new fields stay");
        Ok(())
    }

    #[test]
    fn rejects_daily_note_without_reset_time() {
        let stored = serde_json::json!({
            "id": "0190a9d4-0000-7000-8000-000000000001",
            "title": "A", "body": "", "reset_schedule": "daily",
            "created_at": "2026-11-01T12:00:00Z", "updated_at": "2026-11-01T12:00:00Z"
        });
        assert!(serde_json::from_value::<Note>(stored).is_err());
    }

    #[test]
    fn note_count_includes_nested_groups() {
        let mut outer = Group::new("Outer".to_owned());
        assert_eq!(outer.note_count(), 0);
        let mut inner = Group::new("Inner".to_owned());
        inner.children.push(TreeNode::Note(Note::new(
            "A".to_owned(),
            ResetSchedule::Never,
            noon(),
        )));
        inner
            .children
            .push(TreeNode::Group(Group::new("Empty".to_owned())));
        outer.children.push(TreeNode::Group(inner));
        outer.children.push(TreeNode::Note(Note::new(
            "B".to_owned(),
            ResetSchedule::Never,
            noon(),
        )));
        assert_eq!(outer.note_count(), 2);
    }
}
