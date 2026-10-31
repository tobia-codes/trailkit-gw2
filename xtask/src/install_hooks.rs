//! `cargo xtask install-hooks`: activates the versioned git hooks in
//! `.githooks/` by setting `core.hooksPath` for this clone (ROADMAP R48).

use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::workspace;

/// Directory of the versioned hooks, relative to the repository root.
const HOOKS_DIRECTORY: &str = ".githooks";

/// Points git at the versioned hooks directory.
///
/// # Errors
/// Returns an error if `git config` fails.
pub fn run() -> Result<()> {
    let status = Command::new("git")
        .current_dir(workspace::root())
        .args(["config", "core.hooksPath", HOOKS_DIRECTORY])
        .status()
        .context("failed to run `git config`")?;
    if !status.success() {
        bail!("`git config core.hooksPath` failed ({status})");
    }
    println!("git hooks enabled from {HOOKS_DIRECTORY}/ (pre-commit, pre-push)");
    Ok(())
}
