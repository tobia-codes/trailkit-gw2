//! `cargo xtask check-branch`: enforces branch names (ROADMAP R9, R14, R15).
//!
//! Allowed: `feature/<name>`, `chore/<name>`, `bugfix/<name>`,
//! `hotfix/<name>` with a `kebab-case` name, and the release PR
//! `develop` → `main`. Used by CI (`--head`/`--base` from the pull request)
//! and by the local pre-push hook (current branch, no base).

use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::{check_names::is_kebab_case, workspace};

/// Branch prefixes for work branches.
const WORK_BRANCH_PREFIXES: [&str; 4] = ["feature/", "chore/", "bugfix/", "hotfix/"];

/// Checks `head` (default: the current branch) against the rules; `base` is
/// the pull request target, if known.
///
/// # Errors
/// Returns an error if the current branch cannot be determined or the name
/// is not allowed.
pub fn run(head: Option<&str>, base: Option<&str>) -> Result<()> {
    let head = match head {
        Some(head) => head.to_owned(),
        None => current_branch()?,
    };
    if is_allowed(&head, base) {
        println!("branch `{head}` is allowed");
        Ok(())
    } else {
        bail!(
            "branch `{head}`{} is not allowed: use feature/, chore/, bugfix/ or hotfix/ followed by \
             a kebab-case name (or develop → main for releases)",
            base.map(|base| format!(" → `{base}`")).unwrap_or_default()
        )
    }
}

/// Returns whether a branch named `head` may be pushed / merged into `base`.
fn is_allowed(head: &str, base: Option<&str>) -> bool {
    if head == "develop" && base == Some("main") {
        return true;
    }
    WORK_BRANCH_PREFIXES
        .iter()
        .any(|prefix| head.strip_prefix(prefix).is_some_and(is_kebab_case))
}

/// Returns the name of the checked-out branch.
///
/// # Errors
/// Returns an error if `git` fails or `HEAD` is detached.
fn current_branch() -> Result<String> {
    let output = Command::new("git")
        .current_dir(workspace::root())
        .args(["symbolic-ref", "--short", "HEAD"])
        .output()
        .context("failed to run `git symbolic-ref`")?;
    if !output.status.success() {
        bail!("cannot determine the current branch (detached HEAD?)");
    }
    Ok(String::from_utf8(output.stdout)
        .context("non-UTF-8 branch name")?
        .trim()
        .to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_work_branches_and_release_pr() {
        assert!(is_allowed("feature/memo-tree", Some("develop")));
        assert!(is_allowed("chore/phase-0-foundation", None));
        assert!(is_allowed("hotfix/crash-on-unload", Some("main")));
        assert!(is_allowed("develop", Some("main")));
    }

    #[test]
    fn rejects_other_branches() {
        assert!(!is_allowed("main", None));
        assert!(!is_allowed("develop", None));
        assert!(!is_allowed("develop", Some("develop")));
        assert!(!is_allowed("feature/", None));
        assert!(!is_allowed("feature/Memo_Tree", None));
        assert!(!is_allowed("fix/memo", None));
    }
}
