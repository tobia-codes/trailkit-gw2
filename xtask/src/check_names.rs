//! `cargo xtask check-names`: verifies that every file and directory in the
//! repository follows the naming conventions in `docs/agent-reference.md`
//! ("Naming conventions", ROADMAP R40–R46).
//!
//! Rules per path segment:
//! - directories: `kebab-case` (a leading `.` is allowed, e.g. `.github`);
//!   below a `src/` directory also `snake_case`, because Rust module
//!   directories carry the module name (`main_window.rs` + `main_window/`);
//! - Rust sources (`*.rs`): `snake_case`;
//! - files below `data/`: `snake_case` or `kebab-case` (files named after a
//!   data id keep the id's `snake_case`);
//! - well-known meta files and tool-mandated files: exact names from
//!   [`FIXED_FILE_NAMES`]; tool-mandated directories: exact paths from
//!   [`FIXED_DIRECTORY_PATHS`];
//! - everything else: `kebab-case` stem with lowercase extensions.
//!
//! Vendored third-party content keeps its upstream names ([`VENDORED_PREFIXES`]).

use anyhow::{Result, bail};

use crate::workspace;

/// File names that are allowed regardless of the casing rules: established
/// upper-case meta files and names mandated by tools.
const FIXED_FILE_NAMES: &[&str] = &[
    "AGENTS.md",
    "BACKLOG.md",
    "CHANGELOG.md",
    "CLAUDE.md",
    "CONTRIBUTING.md",
    "LICENSE",
    "NEXT_SESSION.md",
    "PRIVACY.de.md",
    "PRIVACY.md",
    "README.md",
    "ROADMAP.md",
    "SECURITY.md",
    "SKILL.md",
    "Cargo.lock",
    "Cargo.toml",
];

/// Directory paths whose names a tool mandates (GitHub reads issue forms only
/// from `.github/ISSUE_TEMPLATE/`, R95).
const FIXED_DIRECTORY_PATHS: &[&str] = &[".github/ISSUE_TEMPLATE"];

/// Path prefixes of vendored third-party content that keeps upstream names.
const VENDORED_PREFIXES: &[&str] = &[
    ".agents/skills/grill-me/",
    ".agents/skills/grilling/",
    ".agents/skills/rust-best-practices/",
    ".agents/skills/rust-skills/",
];

/// Checks all tracked and untracked-but-not-ignored paths.
///
/// # Errors
/// Returns an error if `git` cannot be run or any path violates the rules.
pub fn run() -> Result<()> {
    let violations: Vec<String> = workspace::repository_paths()?
        .iter()
        .filter_map(|path| check_path(path))
        .collect();
    if !violations.is_empty() {
        bail!(
            "{} path(s) violate the naming conventions (see AGENTS.md):\n  {}",
            violations.len(),
            violations.join("\n  ")
        );
    }
    println!("all paths follow the naming conventions");
    Ok(())
}

/// Checks one repository-relative path; returns a description of the first
/// violation, or `None` if the path is fine.
fn check_path(path: &str) -> Option<String> {
    if VENDORED_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
    {
        return None;
    }
    let segments: Vec<&str> = path.split('/').collect();
    let (file_name, directories) = segments.split_last()?;
    for (index, directory) in directories.iter().enumerate() {
        let in_rust_sources = directories[..index].contains(&"src");
        if in_rust_sources && is_snake_case(directory) {
            continue;
        }
        if FIXED_DIRECTORY_PATHS.contains(&directories[..=index].join("/").as_str()) {
            continue;
        }
        if !is_kebab_case(directory.strip_prefix('.').unwrap_or(directory)) {
            return Some(format!(
                "{path}: directory `{directory}` must be kebab-case"
            ));
        }
    }
    let in_data = path.starts_with("data/");
    if is_valid_file_name(file_name, in_data) {
        None
    } else {
        Some(format!(
            "{path}: file name `{file_name}` violates the naming rules"
        ))
    }
}

/// Returns whether a file name is allowed (see the module docs for the rules).
fn is_valid_file_name(file_name: &str, in_data: bool) -> bool {
    if FIXED_FILE_NAMES.contains(&file_name) {
        return true;
    }
    // Dotfiles such as `.gitignore` / `.node-version`: check the part after the dot.
    let name = file_name.strip_prefix('.').unwrap_or(file_name);
    let (stem, extensions) = name.split_once('.').unwrap_or((name, ""));
    let extensions_valid = extensions.split('.').all(|extension| {
        extension
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    });
    if !extensions_valid {
        return false;
    }
    if extensions == "rs" {
        return is_snake_case(stem);
    }
    is_kebab_case(stem) || (in_data && is_snake_case(stem))
}

/// Returns whether `text` is non-empty `kebab-case` (`[a-z0-9]+(-[a-z0-9]+)*`).
pub fn is_kebab_case(text: &str) -> bool {
    is_separated_lowercase(text, '-')
}

/// Returns whether `text` is non-empty `snake_case` (`[a-z0-9]+(_[a-z0-9]+)*`).
fn is_snake_case(text: &str) -> bool {
    is_separated_lowercase(text, '_')
}

/// Returns whether `text` consists of non-empty lowercase ASCII alphanumeric
/// words joined by single `separator` characters.
fn is_separated_lowercase(text: &str, separator: char) -> bool {
    !text.is_empty()
        && text.split(separator).all(|word| {
            !word.is_empty()
                && word
                    .chars()
                    .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_conventional_paths() {
        for path in [
            "README.md",
            "addons/trailkit-memo/Cargo.toml",
            "addons/trailkit-memo/src/i18n.rs",
            "xtask/src/sync_versions.rs",
            "addons/trailkit-memo/src/main_window/note_list.rs",
            "docs/plans/data-pipeline.md",
            ".github/workflows/ci.yml",
            ".github/ISSUE_TEMPLATE/report-a-problem.yml",
            "PRIVACY.de.md",
            ".node-version",
            "pnpm-lock.yaml",
            "data/events/world_bosses.toml",
            ".agents/skills/rust-best-practices/references/chapter_01.md",
        ] {
            assert_eq!(check_path(path), None, "{path}");
        }
    }

    #[test]
    fn rejects_unconventional_paths() {
        for path in [
            "docs/plans/_TEMPLATE.md",
            "docs/Plans/roadmap.md",
            "addons/trailkit-memo/src/NoteTree.rs",
            "addons/trailkit-memo/src/note-tree.rs",
            "apps/web/NoteTree.tsx",
            "docs/my_notes.md",
            "docs/my_plans/roadmap.md",
            "addons/trailkit-memo/src/MainWindow/note_list.rs",
            "assets/icon.PNG",
            "docs/ISSUE_TEMPLATE/notes.md",
            "docs/README.de.md",
        ] {
            assert!(check_path(path).is_some(), "{path}");
        }
    }
}
