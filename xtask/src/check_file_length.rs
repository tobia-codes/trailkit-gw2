//! `cargo xtask check-file-length [--all]`: hints for coding agents that a
//! Rust file or function has grown long enough to consider splitting it
//! (ROADMAP R77).
//!
//! Advisory only: it prints one line per hint and nothing when all is fine,
//! and it never fails, so it costs almost no tokens. By default it looks at
//! the Rust files changed on the current branch (against `develop`,
//! uncommitted and untracked files included); `--all` checks every file.
//!
//! Counted are code lines: no blank lines, no comments or doc comments
//! (rule 4 makes docs mandatory, they must not count against a file) and no
//! `#[cfg(test)]` blocks. Pure test files have their own, higher limit.

use std::{collections::BTreeSet, fs, process::Command};

use anyhow::{Context, Result, bail};

use crate::workspace;

/// Code lines from which a source file gets a hint.
const FILE_HINT_CODE_LINES: usize = 400;

/// Code lines from which a pure test file gets a hint.
const TEST_FILE_HINT_CODE_LINES: usize = 800;

/// Code lines from which a function gets a hint.
const FUNCTION_HINT_CODE_LINES: usize = 80;

/// The branch whose merge base marks "changed on this branch".
const BASE_BRANCH: &str = "develop";

/// Which files are checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Rust files changed on the current branch (the default).
    Changed,
    /// Every Rust file in the repository.
    All,
}

/// One classified source line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceLine<'source> {
    /// The line without its line break.
    text: &'source str,
    /// Whether the line holds code (not blank, not a comment).
    is_code: bool,
    /// Whether the line belongs to a `#[cfg(test)]` item.
    is_test: bool,
}

/// Prints a hint for every long file and function in `scope`.
///
/// # Errors
/// Returns an error if `git` cannot be run.
pub fn run(scope: Scope) -> Result<()> {
    let paths = match scope {
        Scope::Changed => changed_rust_files()?,
        Scope::All => git_paths(&["ls-files", "--cached", "--others", "--exclude-standard"])?,
    };
    let root = workspace::root();
    for path in paths.iter().filter(|path| path.ends_with(".rs")) {
        // Deleted or unreadable files have nothing to report.
        let Ok(source) = fs::read_to_string(root.join(path)) else {
            continue;
        };
        for hint in hints(path, &source) {
            println!("{hint}");
        }
    }
    Ok(())
}

/// The Rust files changed on this branch against [`BASE_BRANCH`], including
/// uncommitted and untracked ones.
///
/// # Errors
/// Returns an error if `git` cannot be run.
fn changed_rust_files() -> Result<BTreeSet<String>> {
    let base = git_paths(&["merge-base", "HEAD", BASE_BRANCH])
        .ok()
        .and_then(|lines| lines.into_iter().next())
        .unwrap_or_else(|| "HEAD".to_owned());
    let mut paths = git_paths(&["diff", "--name-only", &base])?;
    paths.extend(git_paths(&["ls-files", "--others", "--exclude-standard"])?);
    Ok(paths)
}

/// Runs `git` with `args` in the repository root and returns its output
/// lines.
///
/// # Errors
/// Returns an error if `git` cannot be run or fails.
fn git_paths(args: &[&str]) -> Result<BTreeSet<String>> {
    let output = Command::new("git")
        .current_dir(workspace::root())
        .args(args)
        .output()
        .with_context(|| format!("failed to run `git {}`", args.join(" ")))?;
    if !output.status.success() {
        bail!("`git {}` failed ({})", args.join(" "), output.status);
    }
    let listing = String::from_utf8(output.stdout).context("non-UTF-8 git output")?;
    Ok(listing.lines().map(str::to_owned).collect())
}

/// The hints for the file at `path` with `source`: the file itself, then
/// its long functions (outside tests).
fn hints(path: &str, source: &str) -> Vec<String> {
    let lines = classify(source);
    let is_test_file = path.ends_with("/tests.rs") || path.contains("/tests/");
    let mut hints = Vec::new();
    let (counted, limit) = if is_test_file {
        (
            lines.iter().filter(|line| line.is_code).count(),
            TEST_FILE_HINT_CODE_LINES,
        )
    } else {
        let code = lines.iter().filter(|line| line.is_code && !line.is_test);
        (code.count(), FILE_HINT_CODE_LINES)
    };
    if counted >= limit {
        hints.push(format!("{path}: {counted} code lines (hint at {limit})"));
    }
    if !is_test_file {
        hints.extend(function_hints(path, &lines));
    }
    hints
}

/// Classifies every line of `source` as code or not and as test or not.
fn classify(source: &str) -> Vec<SourceLine<'_>> {
    let mut lines = Vec::new();
    let mut test_block_indent: Option<usize> = None;
    let mut after_test_attribute = false;
    for text in source.lines() {
        let trimmed = text.trim_start();
        let indent = text.len() - trimmed.len();
        let mut is_test = test_block_indent.is_some();
        if test_block_indent.is_none() && after_test_attribute && !trimmed.starts_with("#[") {
            is_test = true;
            after_test_attribute = false;
            if trimmed.ends_with('{') {
                test_block_indent = Some(indent);
            }
        } else if test_block_indent == Some(indent) && trimmed.starts_with('}') {
            test_block_indent = None;
        }
        if trimmed == "#[cfg(test)]" {
            after_test_attribute = true;
            is_test = true;
        }
        let is_code = !trimmed.is_empty() && !trimmed.starts_with("//");
        lines.push(SourceLine {
            text,
            is_code,
            is_test,
        });
    }
    lines
}

/// One hint per function outside tests with at least
/// [`FUNCTION_HINT_CODE_LINES`] code lines.
fn function_hints(path: &str, lines: &[SourceLine<'_>]) -> Vec<String> {
    let mut hints = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if line.is_test || !line.is_code {
            continue;
        }
        let Some(name) = function_name(line.text) else {
            continue;
        };
        let indent = line.text.len() - line.text.trim_start().len();
        let closing = format!("{}}}", " ".repeat(indent));
        let Some(length) = lines[index..]
            .iter()
            .position(|candidate| candidate.text == closing)
        else {
            continue;
        };
        let code = lines[index..=index + length]
            .iter()
            .filter(|candidate| candidate.is_code)
            .count();
        if code >= FUNCTION_HINT_CODE_LINES {
            hints.push(format!(
                "{path}:{}: fn {name}: {code} code lines (hint at {FUNCTION_HINT_CODE_LINES})",
                index + 1
            ));
        }
    }
    hints
}

/// The function name if `text` starts a function with a body (`fn`, after
/// any of `pub`, `pub(…)`, `const`, `async`, `unsafe`); `None` otherwise.
fn function_name(text: &str) -> Option<&str> {
    let mut rest = text.trim_start();
    loop {
        if let Some(after) = rest.strip_prefix("fn ") {
            let end = after.find(['(', '<'])?;
            return Some(&after[..end]);
        }
        let word_end = rest.find(' ')?;
        let word = &rest[..word_end];
        let is_qualifier = matches!(word, "pub" | "const" | "async" | "unsafe")
            || (word.starts_with("pub(") && word.ends_with(')'));
        if !is_qualifier {
            return None;
        }
        rest = rest[word_end..].trim_start();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_code_without_comments_blanks_and_tests() {
        let source = "//! Module docs.\n\n/// Docs.\nfn one() {\n    two();\n}\n\n#[cfg(test)]\nmod tests {\n    fn helper() {}\n}\n";
        let lines = classify(source);
        let code: Vec<&str> = lines
            .iter()
            .filter(|line| line.is_code && !line.is_test)
            .map(|line| line.text)
            .collect();
        assert_eq!(code, ["fn one() {", "    two();", "}"]);
    }

    #[test]
    fn a_test_module_declaration_is_a_test_line() {
        let lines = classify("#[cfg(test)]\nmod tests;\nfn after() {}\n");
        let flags: Vec<bool> = lines.iter().map(|line| line.is_test).collect();
        assert_eq!(flags, [true, true, false]);
    }

    #[test]
    fn finds_function_names_after_qualifiers() {
        assert_eq!(
            function_name("    pub(super) fn draw(&mut self) {"),
            Some("draw")
        );
        assert_eq!(
            function_name("pub const fn imgui_id<T>(self)"),
            Some("imgui_id")
        );
        assert_eq!(function_name("    let fn_name = 1;"), None);
        assert_eq!(function_name("// fn not_code()"), None);
    }

    #[test]
    fn reports_long_files_and_functions_only() {
        let long_body = "    work();\n".repeat(FUNCTION_HINT_CODE_LINES);
        let source = format!("fn long() {{\n{long_body}}}\n\nfn short() {{\n    work();\n}}\n");
        let hints = hints("src/example.rs", &source);
        assert_eq!(hints.len(), 1);
        assert!(hints[0].starts_with("src/example.rs:1: fn long:"));
        assert!(super::hints("src/example.rs", "fn short() {}\n").is_empty());
    }

    #[test]
    fn test_files_have_their_own_limit_and_no_function_hints() {
        let long_body = "    work();\n".repeat(FUNCTION_HINT_CODE_LINES);
        let source = format!("fn long_test() {{\n{long_body}}}\n");
        assert!(hints("src/window/tests.rs", &source).is_empty());
    }
}
