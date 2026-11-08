//! File names for log lines, without the folders around them (ROADMAP R92).

use std::{fmt, path::Path};

/// Displays only the last component of a path, e.g. `notes.json`.
///
/// Log lines never contain absolute paths, because they hold the Windows user
/// name (GDPR, ROADMAP R89/R92). Log a path as `file:% = FileName::of(&path)`.
#[derive(Debug, Clone, Copy)]
pub struct FileName<'a>(&'a Path);

impl<'a> FileName<'a> {
    /// Wraps `path` for display.
    #[must_use]
    pub const fn of(path: &'a Path) -> Self {
        Self(path)
    }
}

impl fmt::Display for FileName<'_> {
    /// Writes the file name (lossily converted to UTF-8), or `?` if the path
    /// has no file name (e.g. `..` or a root).
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.file_name() {
            Some(name) => write!(formatter, "{}", name.to_string_lossy()),
            None => formatter.write_str("?"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_only_the_last_component() {
        let path = Path::new("/home/someone/addons/trailkit/memo/notes.json");
        assert_eq!(FileName::of(path).to_string(), "notes.json");
    }

    #[test]
    fn shows_a_question_mark_without_a_file_name() {
        assert_eq!(FileName::of(Path::new("..")).to_string(), "?");
    }
}
