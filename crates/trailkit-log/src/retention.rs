//! What survives of an existing log file when an addon starts (ROADMAP R92):
//! entries older than the retention period go, and the rest is cut down to a
//! start budget so the session has room below the size cap.

use jiff::Timestamp;

use crate::{
    events::{EVENT_SESSION_END, EVENT_SESSION_START},
    line::{parse_event, parse_time},
};

/// The part of an existing log that is kept, plus what the old content says
/// about the previous session.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Retained {
    /// The kept lines, each ending with a newline.
    pub(crate) content: String,
    /// Whether `content` differs from the file, so it must be rewritten.
    pub(crate) changed: bool,
    /// Whether the last session started but never wrote `session_end`.
    pub(crate) previous_session_unclean: bool,
}

/// Keeps the lines of `existing` written at or after `cutoff`, then drops
/// the oldest of those until at most `budget` bytes remain.
///
/// Lines are chronological, so everything before the first line at or after
/// `cutoff` goes, including lines without a readable time. A last line
/// without its newline (a write cut off by a crash) gets one.
pub(crate) fn retain(existing: &str, cutoff: Timestamp, budget: usize) -> Retained {
    let previous_session_unclean = existing
        .lines()
        .rev()
        .find_map(|line| {
            parse_event(line)
                .filter(|event| *event == EVENT_SESSION_START || *event == EVENT_SESSION_END)
        })
        .is_some_and(|event| event == EVENT_SESSION_START);

    let first_recent = line_starts(existing)
        .find(|&start| parse_time(&existing[start..]).is_some_and(|time| time >= cutoff))
        .unwrap_or(existing.len());
    let mut kept = &existing[first_recent..];
    if kept.len() > budget {
        let excess_end = kept.len() - budget;
        let cut = line_starts(kept)
            .find(|&start| start >= excess_end)
            .unwrap_or(kept.len());
        kept = &kept[cut..];
    }

    let mut content = kept.to_owned();
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    Retained {
        changed: content != existing,
        content,
        previous_session_unclean,
    }
}

/// Byte offsets where the lines of `text` start.
fn line_starts(text: &str) -> impl Iterator<Item = usize> + '_ {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(index, _)| index + 1))
        .filter(move |&start| start < text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line at day `day` of November 2026 with `event`.
    fn line(day: u8, event: &str) -> String {
        format!("2026-11-{day:02}T12:00:00+01:00 INFO addon=memo version=0.3.0 {event} at=x:1\n")
    }

    /// Midnight UTC at day `day` of November 2026.
    fn november(day: u8) -> Timestamp {
        format!("2026-11-{day:02}T00:00:00Z")
            .parse()
            .unwrap_or(Timestamp::UNIX_EPOCH)
    }

    #[test]
    fn drops_lines_older_than_the_cutoff() {
        let existing = [
            line(1, "session_start"),
            line(1, "session_end"),
            line(9, "session_start"),
            line(9, "session_end"),
        ]
        .concat();
        let retained = retain(&existing, november(5), usize::MAX);
        assert_eq!(
            retained.content,
            [line(9, "session_start"), line(9, "session_end")].concat()
        );
        assert!(retained.changed);
        assert!(!retained.previous_session_unclean);
    }

    #[test]
    fn keeps_a_recent_log_untouched() {
        let existing = [line(9, "session_start"), line(9, "session_end")].concat();
        let retained = retain(&existing, november(5), usize::MAX);
        assert_eq!(retained.content, existing);
        assert!(!retained.changed);
    }

    #[test]
    fn cuts_the_oldest_lines_down_to_the_budget() {
        let existing = [line(6, "first"), line(7, "second"), line(8, "third")].concat();
        let budget = line(8, "third").len() + 5;
        let retained = retain(&existing, november(5), budget);
        assert_eq!(retained.content, line(8, "third"));
    }

    #[test]
    fn detects_a_session_without_end() {
        let existing = [line(9, "session_start"), line(9, "save_failed")].concat();
        assert!(retain(&existing, november(5), usize::MAX).previous_session_unclean);
    }

    #[test]
    fn detects_an_unclean_session_even_when_it_is_too_old_to_keep() {
        let existing = line(1, "session_start");
        let retained = retain(&existing, november(5), usize::MAX);
        assert!(retained.previous_session_unclean);
        assert!(retained.content.is_empty());
    }

    #[test]
    fn completes_a_line_cut_off_by_a_crash() {
        let existing = format!(
            "{}{}",
            line(9, "session_start"),
            "2026-11-09T12:00:01+01:00 WARN addon=memo"
        );
        let retained = retain(&existing, november(5), usize::MAX);
        assert!(retained.content.ends_with("addon=memo\n"));
        assert!(retained.changed);
    }

    #[test]
    fn an_empty_file_stays_empty() {
        assert_eq!(
            retain("", november(5), usize::MAX),
            Retained {
                content: String::new(),
                changed: false,
                previous_session_unclean: false,
            }
        );
    }
}
