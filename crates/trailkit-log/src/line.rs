//! The text format of one log line (ROADMAP R92, README "Log format").
//!
//! ```text
//! 2026-11-08T15:02:11+01:00 WARN addon=memo version=0.3.0 save_failed at=trailkit_memo_shared::storage_worker:318 file=notes.json error="Access is denied. (os error 5)"
//! ```
//!
//! Time with UTC offset, level, addon, version, event, code location, then
//! `key=value` fields. Values are quoted when needed and every value passes
//! through [`scrub_paths`], so an absolute path can never reach the file.

use std::{borrow::Cow, fmt::Write as _};

use jiff::{Timestamp, tz::Offset};
use trailkit_storage::AddonName;

/// Field name the logger uses for a message that is not an event name.
pub(crate) const TEXT_FIELD: &str = "text";

/// Everything that is the same on every line of one addon's log.
#[derive(Clone, Copy)]
pub(crate) struct LineIdentity {
    /// The addon writing the log.
    pub(crate) addon: AddonName,
    /// The addon's version, e.g. `0.3.0`.
    pub(crate) version: &'static str,
    /// The player's UTC offset at a given time (from the operating system).
    pub(crate) local_offset: fn(Timestamp) -> Offset,
}

/// One `key=value` field of a line.
pub(crate) type Field<'a> = (Cow<'a, str>, Cow<'a, str>);

/// What a single line says, apart from the identity and the time.
pub(crate) struct LineContent<'a> {
    /// Severity.
    pub(crate) level: log::Level,
    /// Event name in snake_case, e.g. `save_failed`.
    pub(crate) event: &'a str,
    /// Code location, `module::path:line` or `file.rs:line`.
    pub(crate) at: &'a str,
    /// The fields after `at=`.
    pub(crate) fields: &'a [Field<'a>],
}

/// Formats a complete line, including the trailing newline.
pub(crate) fn format_line(
    identity: &LineIdentity,
    time: Timestamp,
    content: &LineContent<'_>,
) -> String {
    let mut line = format_time(time, (identity.local_offset)(time));
    line.push(' ');
    line.push_str(level_name(content.level));
    line.push(' ');
    line.push_str(&format_body(identity, content));
    line.push('\n');
    line
}

/// Formats everything after the level: `addon= version= event at= fields…`.
/// This is also the text forwarded to `Nexus.log`, which adds its own time.
pub(crate) fn format_body(identity: &LineIdentity, content: &LineContent<'_>) -> String {
    let mut body = String::with_capacity(128);
    // Writing into a `String` cannot fail.
    let _ = write!(
        body,
        "addon={} version={} {}",
        identity.addon, identity.version, content.event
    );
    push_field(&mut body, "at", content.at);
    for (key, value) in content.fields {
        push_field(&mut body, key, value);
    }
    body
}

/// Appends ` key=value`, quoting and scrubbing the value.
fn push_field(line: &mut String, key: &str, value: &str) {
    line.push(' ');
    line.push_str(key);
    line.push('=');
    push_value(line, &scrub_paths(value));
}

/// Appends `value`, in double quotes (with `\"`, `\\`, `\n`, `\r`, `\t`
/// escaped) if it is empty or contains whitespace, `"`, `=` or `\`.
fn push_value(line: &mut String, value: &str) {
    let needs_quotes = value.is_empty()
        || value
            .chars()
            .any(|character| character.is_whitespace() || matches!(character, '"' | '=' | '\\'));
    if !needs_quotes {
        line.push_str(value);
        return;
    }
    line.push('"');
    for character in value.chars() {
        match character {
            '"' => line.push_str("\\\""),
            '\\' => line.push_str("\\\\"),
            '\n' => line.push_str("\\n"),
            '\r' => line.push_str("\\r"),
            '\t' => line.push_str("\\t"),
            other => line.push(other),
        }
    }
    line.push('"');
}

/// `time` in ISO 8601 with seconds and the UTC offset, e.g.
/// `2026-11-08T15:02:11+01:00`.
pub(crate) fn format_time(time: Timestamp, offset: Offset) -> String {
    time.to_zoned(jiff::tz::TimeZone::fixed(offset))
        .strftime("%Y-%m-%dT%H:%M:%S%:z")
        .to_string()
}

/// The level as written in the file.
const fn level_name(level: log::Level) -> &'static str {
    match level {
        log::Level::Error => "ERROR",
        log::Level::Warn => "WARN",
        log::Level::Info => "INFO",
        log::Level::Debug => "DEBUG",
        log::Level::Trace => "TRACE",
    }
}

/// Whether `text` is an event name: lowercase ASCII words joined by single
/// underscores, starting with a letter (`save_failed`, `session_start`).
pub(crate) fn is_event_name(text: &str) -> bool {
    let mut previous = '_';
    for (index, character) in text.chars().enumerate() {
        let allowed = match character {
            'a'..='z' => true,
            '0'..='9' => index > 0,
            '_' => index > 0 && previous != '_',
            _ => false,
        };
        if !allowed {
            return false;
        }
        previous = character;
    }
    !text.is_empty() && previous != '_'
}

/// Removes the folders of every absolute Windows path in `value`, keeping
/// the file name and whatever follows it:
/// `moved to C:\Users\Jo\addons\notes.json` → `moved to notes.json`.
///
/// A path starts at a drive letter (`C:\`, `C:/`) that does not follow a
/// letter or digit, or at a UNC prefix (`\\`). As a path may contain spaces,
/// it is taken to end at the last `\` or `/` of the value. This is a safety
/// net only: code must log file names (`trailkit_storage::FileName`), never
/// paths.
pub(crate) fn scrub_paths(value: &str) -> Cow<'_, str> {
    let Some(start) = find_path_start(value) else {
        return Cow::Borrowed(value);
    };
    let last_separator = value.rfind(['\\', '/']).unwrap_or(start);
    let mut scrubbed = String::with_capacity(value.len());
    scrubbed.push_str(&value[..start]);
    // Everything up to the last separator is gone, so a second path in the
    // same value ("from C:\a to D:\b") is removed with the first.
    scrubbed.push_str(&value[last_separator + 1..]);
    Cow::Owned(scrubbed)
}

/// Byte index where the first absolute Windows path in `value` starts.
fn find_path_start(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    (0..bytes.len()).find(|&index| {
        let follows_word = index > 0 && bytes[index - 1].is_ascii_alphanumeric();
        let is_drive = bytes[index].is_ascii_alphabetic()
            && bytes.get(index + 1) == Some(&b':')
            && matches!(bytes.get(index + 2), Some(b'\\' | b'/'));
        let is_unc = bytes[index] == b'\\' && bytes.get(index + 1) == Some(&b'\\');
        !follows_word && (is_drive || is_unc)
    })
}

/// The time at the start of a line written by [`format_line`].
pub(crate) fn parse_time(line: &str) -> Option<Timestamp> {
    line.split(' ').next()?.parse().ok()
}

/// The event of a line written by [`format_line`] (the fifth word).
pub(crate) fn parse_event(line: &str) -> Option<&str> {
    line.split(' ').nth(4).map(str::trim_end)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed +01:00 offset, as in Central European winter time.
    fn plus_one_hour(_time: Timestamp) -> Offset {
        Offset::constant(1)
    }

    /// The identity used by the tests.
    const IDENTITY: LineIdentity = LineIdentity {
        addon: AddonName::new("memo"),
        version: "0.3.0",
        local_offset: plus_one_hour,
    };

    /// 2026-11-08 14:02:11 UTC.
    fn sample_time() -> Timestamp {
        Timestamp::from_second(1_794_146_531).unwrap_or(Timestamp::UNIX_EPOCH)
    }

    #[test]
    fn formats_time_level_identity_event_location_and_fields() {
        let fields = [
            (Cow::Borrowed("file"), Cow::Borrowed("notes.json")),
            (
                Cow::Borrowed("error"),
                Cow::Borrowed("Access is denied. (os error 5)"),
            ),
        ];
        let line = format_line(
            &IDENTITY,
            sample_time(),
            &LineContent {
                level: log::Level::Warn,
                event: "save_failed",
                at: "trailkit_memo_shared::storage_worker:318",
                fields: &fields,
            },
        );
        assert_eq!(
            line,
            "2026-11-08T15:02:11+01:00 WARN addon=memo version=0.3.0 save_failed \
             at=trailkit_memo_shared::storage_worker:318 file=notes.json \
             error=\"Access is denied. (os error 5)\"\n"
        );
        assert_eq!(parse_time(&line), Some(sample_time()));
        assert_eq!(parse_event(&line), Some("save_failed"));
    }

    #[test]
    fn quotes_and_escapes_values_that_would_break_the_line() {
        let mut line = String::new();
        push_value(&mut line, "a \"b\"\nc=d");
        assert_eq!(line, r#""a \"b\"\nc=d""#);

        let mut empty = String::new();
        push_value(&mut empty, "");
        assert_eq!(empty, r#""""#);
    }

    #[test]
    fn keeps_only_the_file_name_of_absolute_paths() {
        assert_eq!(
            scrub_paths(r"moved to C:\Users\Jo Doe\Guild Wars 2\addons\notes.json"),
            "moved to notes.json"
        );
        assert_eq!(
            scrub_paths(r"\\server\share\notes.json (busy)"),
            "notes.json (busy)"
        );
        assert_eq!(scrub_paths("Z:/home/jo/notes.json"), "notes.json");
    }

    #[test]
    fn leaves_text_without_paths_alone() {
        for text in [
            "Access is denied. (os error 5)",
            "https://example.com/a/b",
            "ratio 3:4",
            "notes.json",
        ] {
            assert!(matches!(scrub_paths(text), Cow::Borrowed(_)), "{text}");
        }
    }

    #[test]
    fn scrubs_paths_inside_formatted_fields() {
        let fields = [(
            Cow::Borrowed("error"),
            Cow::Borrowed(r"cannot open C:\Users\Jo\x.json"),
        )];
        let body = format_body(
            &IDENTITY,
            &LineContent {
                level: log::Level::Error,
                event: "load_failed",
                at: "x:1",
                fields: &fields,
            },
        );
        assert!(!body.contains("Users"), "{body}");
        assert!(body.ends_with("error=\"cannot open x.json\""), "{body}");
    }

    #[test]
    fn recognises_event_names() {
        for name in ["save_failed", "session_start", "panic", "load_v2"] {
            assert!(is_event_name(name), "{name}");
        }
        for text in [
            "",
            "Save",
            "save failed",
            "_save",
            "save_",
            "save__failed",
            "2fa",
            "notes loaded",
        ] {
            assert!(!is_event_name(text), "{text:?}");
        }
    }
}
