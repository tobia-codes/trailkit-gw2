//! The formatted texts of the trash (memo M145, M146, M150), kept apart
//! from the drawing so they are tested on any host.

use jiff::Timestamp;
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText},
    note_tree::{TrashEntry, TreeNode},
};

use crate::i18n::TrashText;

/// Below this many days the "days left" turn `danger` (M150).
pub const URGENT_DAYS: i64 = 7;

/// "29 days left" for `entry` at `now`.
#[must_use]
pub fn days_left(entry: &TrashEntry, now: Timestamp, language: Language) -> String {
    match entry.days_left(now) {
        1 => TrashText::OneDayLeft.get(language).to_owned(),
        days => with_count(TrashText::DaysLeft, days, language),
    }
}

/// Whether the purge of `entry` is close enough to show it in `danger`.
#[must_use]
pub fn is_urgent(entry: &TrashEntry, now: Timestamp) -> bool {
    entry.days_left(now) < URGENT_DAYS
}

/// The info line below a trashed entry's title (M145): "In the trash ·
/// deleted today · 29 days left".
#[must_use]
pub fn info_line(entry: &TrashEntry, now: Timestamp, language: Language) -> String {
    let deleted = match entry.days_since_deleted(now) {
        0 => TrashText::DeletedToday.get(language).to_owned(),
        1 => TrashText::DeletedYesterday.get(language).to_owned(),
        days => with_count(TrashText::DeletedDaysAgo, days, language),
    };
    format!(
        "{} · {deleted} · {}",
        TrashText::InTrash.get(language),
        days_left(entry, now, language)
    )
}

/// "3 notes" for a count of notes.
#[must_use]
pub fn note_count(count: usize, language: Language) -> String {
    match count {
        1 => TrashText::OneNote.get(language).to_owned(),
        count => TrashText::NoteCount
            .get(language)
            .replace("{count}", &count.to_string()),
    }
}

/// Title and text of the dialog that deletes `node` for good (M146).
#[must_use]
pub fn delete_permanently_question(node: &TreeNode, language: Language) -> (String, String) {
    let title = TrashText::DeletePermanentlyTitle
        .get(language)
        .replace("{title}", node.title());
    let text = match node {
        TreeNode::Note(_) => TrashText::DeleteNotePermanentlyText
            .get(language)
            .to_owned(),
        TreeNode::Group(group) => TrashText::DeleteGroupPermanentlyText
            .get(language)
            .replace("{count}", &group.note_count().to_string()),
    };
    (title, text)
}

/// Title and text of the dialog that empties the trash (M146).
#[must_use]
pub fn empty_trash_question(trash: &[TrashEntry], language: Language) -> (String, String) {
    let notes: usize = trash
        .iter()
        .map(|entry| match &entry.node {
            TreeNode::Note(_) => 1,
            TreeNode::Group(group) => group.note_count(),
        })
        .sum();
    let text = TrashText::EmptyTrashText
        .get(language)
        .replace("{entries}", &trash.len().to_string())
        .replace("{notes}", &notes.to_string());
    (TrashText::EmptyTrashTitle.get(language).to_owned(), text)
}

/// Title and text of the dialog before a group with notes goes to the
/// trash (M134).
#[must_use]
pub fn delete_group_question(title: &str, notes: usize, language: Language) -> (String, String) {
    (
        TrashText::DeleteGroupTitle
            .get(language)
            .replace("{title}", title),
        TrashText::DeleteGroupText
            .get(language)
            .replace("{count}", &notes.to_string()),
    )
}

/// The toast message after `title` was deleted (M148).
#[must_use]
pub fn deleted_message(title: &str, language: Language) -> String {
    TrashText::Deleted.get(language).replace("{title}", title)
}

/// `text` with `{count}` replaced by `count`.
fn with_count(text: TrashText, count: i64, language: Language) -> String {
    text.get(language).replace("{count}", &count.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::SignedDuration;
    use trailkit_memo_shared::note_tree::{Note, NoteKind};

    /// A note deleted at the Unix epoch.
    fn entry() -> TrashEntry {
        TrashEntry {
            node: TreeNode::Note(Note::new(
                "Old list".to_owned(),
                NoteKind::Normal,
                Timestamp::UNIX_EPOCH,
            )),
            deleted_at: Timestamp::UNIX_EPOCH,
            former_parent: None,
            former_index: 0,
        }
    }

    #[test]
    fn the_info_line_names_the_day_and_the_days_left() {
        let now = Timestamp::UNIX_EPOCH + SignedDuration::from_hours(25);
        assert_eq!(
            info_line(&entry(), now, Language::German),
            "Im Papierkorb · gelöscht gestern · noch 29 Tage"
        );
    }

    #[test]
    fn the_last_week_is_urgent() {
        let now = Timestamp::UNIX_EPOCH + SignedDuration::from_hours(24 * 24);
        assert!(is_urgent(&entry(), now));
        assert!(!is_urgent(&entry(), Timestamp::UNIX_EPOCH));
        let last_day = Timestamp::UNIX_EPOCH + SignedDuration::from_hours(24 * 29 + 1);
        assert_eq!(
            days_left(&entry(), last_day, Language::English),
            "1 day left"
        );
    }

    #[test]
    fn the_empty_trash_question_counts_entries_and_notes() {
        let (_, text) = empty_trash_question(&[entry(), entry()], Language::English);
        assert!(text.starts_with("2 entries (2 notes)"));
    }
}
