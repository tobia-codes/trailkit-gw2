//! Build script: embeds the short Git commit of the build as `TRAILKIT_COMMIT`
//! for the `session_start` log line (ROADMAP R92). Without Git (e.g. a source
//! archive) the value is `unknown`. Uncommitted changes are not marked.

use std::{path::PathBuf, process::Command};

/// Emits `TRAILKIT_COMMIT` and asks Cargo to rerun when `HEAD` moves.
fn main() {
    let commit = git(&["rev-parse", "--short=7", "HEAD"]).unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=TRAILKIT_COMMIT={commit}");

    if let Some(git_directory) = git(&["rev-parse", "--absolute-git-dir"]).map(PathBuf::from) {
        println!(
            "cargo:rerun-if-changed={}",
            git_directory.join("HEAD").display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            git_directory.join("packed-refs").display()
        );
        if let Some(reference) = git(&["symbolic-ref", "-q", "HEAD"]) {
            println!(
                "cargo:rerun-if-changed={}",
                git_directory.join(reference).display()
            );
        }
    }
}

/// Runs `git` with `arguments` and returns its trimmed output, or `None` if Git
/// is missing, fails or prints nothing.
fn git(arguments: &[&str]) -> Option<String> {
    let output = Command::new("git").args(arguments).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?.trim().to_owned();
    (!text.is_empty()).then_some(text)
}
