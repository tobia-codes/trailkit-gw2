//! Keeps the README's event table and the code in step (ROADMAP R92): every
//! event a `log::` call in `addons/` or `crates/` writes must be a snake_case
//! event listed in `crates/trailkit-log/README.md`, no call may log a path via
//! `.display()`, and every listed event must still occur in the code.

use std::{
    fs,
    path::{Path, PathBuf},
};

use trailkit_log::BUILT_IN_EVENTS;

/// The workspace root, two levels above this crate.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `.rs` file below `folder`, skipping build output and this crate's
/// own integration tests (they log free text on purpose).
fn rust_files(folder: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let name = entry.file_name();
        if path.is_dir() {
            let skipped = name == "target" || path.ends_with("trailkit-log/tests");
            if !skipped {
                rust_files(&path, files);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

/// The text between the parentheses of each `log::<level>!(…)` call.
fn log_invocations(source: &str) -> Vec<&str> {
    let mut invocations = Vec::new();
    for level in ["error", "warn", "info", "debug", "trace"] {
        let opener = format!("log::{level}!(");
        let mut rest = source;
        while let Some(start) = rest.find(&opener) {
            let body = &rest[start + opener.len()..];
            let end = closing_parenthesis(body).unwrap_or(body.len());
            invocations.push(&body[..end]);
            rest = &body[end..];
        }
    }
    invocations
}

/// Index of the `)` that closes the call, skipping string literals.
fn closing_parenthesis(body: &str) -> Option<usize> {
    let mut depth = 0_u32;
    let mut in_string = false;
    let mut escaped = false;
    for (index, character) in body.char_indices() {
        if in_string {
            match (escaped, character) {
                (false, '\\') => escaped = true,
                (false, '"') => in_string = false,
                _ => escaped = false,
            }
            continue;
        }
        match character {
            '"' => in_string = true,
            '(' => depth += 1,
            ')' if depth == 0 => return Some(index),
            ')' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// The event of a call: the first string literal after the top-level `;`, or
/// the first string literal if there are no `key = value` pairs.
fn event_of(invocation: &str) -> Option<&str> {
    let message_part = invocation
        .rsplit_once(';')
        .map_or(invocation, |(_, message)| message);
    let start = message_part.find('"')? + 1;
    let length = message_part[start..].find('"')?;
    Some(&message_part[start..start + length])
}

/// Whether `text` is lowercase words joined by single underscores.
fn is_event_name(text: &str) -> bool {
    !text.is_empty()
        && text.starts_with(|character: char| character.is_ascii_lowercase())
        && !text.ends_with('_')
        && !text.contains("__")
        && text.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
}

#[test]
fn every_logged_event_is_documented_and_every_documented_event_is_used() {
    let root = workspace_root();
    let readme = fs::read_to_string(root.join("crates/trailkit-log/README.md")).unwrap_or_default();
    let mut files = Vec::new();
    rust_files(&root.join("addons"), &mut files);
    rust_files(&root.join("crates"), &mut files);
    assert!(files.len() > 20, "found only {} Rust files", files.len());

    let events_section = readme.split_once("### Events").map_or("", |(_, rest)| {
        rest.split("\n## ").next().unwrap_or_default()
    });
    assert!(
        !events_section.is_empty(),
        "README has no \"### Events\" section"
    );
    let mut problems = Vec::new();
    let mut corpus = String::new();
    let mut call_count = 0;
    for file in &files {
        let source = fs::read_to_string(file).unwrap_or_default();
        for invocation in log_invocations(&source) {
            call_count += 1;
            let place = file.strip_prefix(&root).unwrap_or(file).display();
            if invocation.contains(".display()") {
                problems.push(format!("{place}: logs a path via .display()"));
            }
            match event_of(invocation) {
                Some(event) if !is_event_name(event) => {
                    problems.push(format!("{place}: {event:?} is not a snake_case event"));
                }
                Some(event) if !events_section.contains(&format!("| `{event}` |")) => {
                    problems.push(format!("{place}: event `{event}` is missing in the README"));
                }
                Some(_) => {}
                None => problems.push(format!("{place}: log call without event: {invocation}")),
            }
        }
        corpus.push_str(&source);
    }

    for row in events_section
        .lines()
        .filter(|line| line.starts_with("| `"))
    {
        let event = row
            .trim_start_matches("| `")
            .split('`')
            .next()
            .unwrap_or_default();
        if !corpus.contains(event) && !BUILT_IN_EVENTS.contains(&event) {
            problems.push(format!("README lists `{event}`, but no code writes it"));
        }
    }
    assert!(call_count > 15, "found only {call_count} log calls");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
