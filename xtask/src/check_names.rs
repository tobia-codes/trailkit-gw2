//! `cargo xtask check-names`: verifies that every file and directory in the
//! repository follows the naming conventions in AGENTS.md ("Naming
//! conventions", ROADMAP R40–R46).
//!
//! Rules per path segment:
//! - directories: `kebab-case` (a leading `.` is allowed, e.g. `.github`);
//! - Rust sources (`*.rs`): `snake_case`;
//! - files below `data/`: `snake_case` or `kebab-case` (files named after a
//!   data id keep the id's `snake_case`);
//! - well-known meta files and tool-mandated files: exact names from
//!   [`FIXED_FILE_NAMES`];
//! - everything else: `kebab-case` stem with lowercase extensions.
//!
//! Vendored third-party content keeps its upstream names ([`VENDORED_PREFIXES`]).

use std::process::Command;

use anyhow::{Context, Result, bail};

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
    "PRIVACY.md",
    "README.md",
    "ROADMAP.md",
    "SKILL.md",
    "Cargo.lock",
    "Cargo.toml",
];

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
    let output = Command::new("git")
        .current_dir(workspace::root())
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ])
        .output()
        .context("failed to run `git ls-files`")?;
    if !output.status.success() {
        bail!("`git ls-files` failed ({})", output.status);
    }
    let listing = String::from_utf8(output.stdout).context("non-UTF-8 path in repository")?;
    let violations: Vec<String> = listing
        .split('\0')
        .filter(|path| !path.is_empty())
        .filter_map(check_path)
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
    for directory in directories {
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
            "docs/plans/data-pipeline.md",
            ".github/workflows/ci.yml",
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
            "assets/icon.PNG",
        ] {
            assert!(check_path(path).is_some(), "{path}");
        }
    }
}
