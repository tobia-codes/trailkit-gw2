//! Repository task runner, invoked as `cargo xtask <command>`.
//!
//! Commands:
//! - `build-icons [--check]` — render the addons' SVG icons to the committed
//!   PNGs (ROADMAP R57).
//! - `deploy <addon> [--release] [--features <list>]` — cross-compile an addon with cargo-xwin and
//!   copy the DLL into the Guild Wars 2 `addons` folder (ROADMAP R32).
//! - `check-branch [--head <branch>] [--base <branch>]` — verify branch names
//!   (ROADMAP R14); defaults to the current branch.
//! - `install-hooks` — enable the versioned git hooks in `.githooks/` (R48).
//! - `check-file-length [--all]` — advisory hints for long Rust files and
//!   functions changed on the branch (ROADMAP R77); never fails.
//! - `check-names` — verify file and directory names against the naming
//!   conventions in AGENTS.md (ROADMAP R46).
//! - `sync-versions [--check]` — copy each addon's `package.json` version
//!   (owned by Changesets) into its `Cargo.toml` (ROADMAP R16/R17).
//! - `subset-fonts [--check]` — cut the embedded fonts down to the characters
//!   trailkit can show (M62).
//! - `tokens [--check]` — write the design token reference from the themes
//!   (ROADMAP R84).
//! - `ui-test [--update]` — build the Windows UI tests and run them (from WSL
//!   through interop); `--update` rewrites the snapshot references (M65).

mod checks;
mod dev;
mod generators;
mod workspace;

use anyhow::{Result, bail};

/// Usage text printed for `help` and on invalid arguments.
const USAGE: &str = "\
usage: cargo xtask <command>

commands:
  build-icons [--check]        render addon SVG icons to PNG (--check: only verify)
  deploy <addon> [--release] [--features <list>]
                               build an addon for Windows and copy it into the GW2 addons folder
  check-branch [--head B] [--base B]  verify the branch name (default: current branch)
  install-hooks                enable the git hooks in .githooks/
  check-file-length [--all]    hint at long Rust files/functions changed on this branch (advisory)
  check-names                  verify file/directory names against the naming conventions
  sync-versions [--check]      copy package.json versions into Cargo.toml (--check: only verify)
  subset-fonts [--check]       cut the embedded fonts to the shown characters (--check: only verify)
  tokens [--check]             write the token reference from the themes (--check: only verify)
  ui-test [--update]           build and run the Windows UI tests (--update: rewrite snapshots)
  help                         show this text";

/// Entry point: parses the command line and dispatches to a command.
///
/// # Errors
/// Returns an error if the arguments are invalid or the command fails.
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["deploy", addon, rest @ ..] => {
            let (profile, features) = parse_deploy_options(rest)?;
            dev::deploy::run(addon, profile, features)
        }
        ["build-icons"] => generators::build_icons::run(generators::build_icons::Mode::Write),
        ["build-icons", "--check"] => {
            generators::build_icons::run(generators::build_icons::Mode::Check)
        }
        ["check-file-length"] => {
            checks::check_file_length::run(checks::check_file_length::Scope::Changed)
        }
        ["check-file-length", "--all"] => {
            checks::check_file_length::run(checks::check_file_length::Scope::All)
        }
        ["check-names"] => checks::check_names::run(),
        ["check-branch", options @ ..] => {
            let (head, base) = parse_branch_options(options)?;
            checks::check_branch::run(head, base)
        }
        ["install-hooks"] => dev::install_hooks::run(),
        ["sync-versions"] => generators::sync_versions::run(generators::sync_versions::Mode::Write),
        ["sync-versions", "--check"] => {
            generators::sync_versions::run(generators::sync_versions::Mode::Check)
        }
        ["subset-fonts"] => generators::subset_fonts::run(generators::subset_fonts::Mode::Write),
        ["subset-fonts", "--check"] => {
            generators::subset_fonts::run(generators::subset_fonts::Mode::Check)
        }
        ["tokens"] => generators::token_docs::run(generators::token_docs::Mode::Write),
        ["tokens", "--check"] => generators::token_docs::run(generators::token_docs::Mode::Check),
        ["ui-test"] => dev::ui_test::run(dev::ui_test::Mode::Compare),
        ["ui-test", "--update"] => dev::ui_test::run(dev::ui_test::Mode::Update),
        [] | ["help" | "--help" | "-h"] => {
            println!("{USAGE}");
            Ok(())
        }
        _ => bail!("unknown command: {args:?}\n\n{USAGE}"),
    }
}

/// Parses `[--head <branch>] [--base <branch>]` for `check-branch`.
///
/// # Errors
/// Returns an error on unknown options or a missing value.
fn parse_branch_options<'a>(options: &[&'a str]) -> Result<(Option<&'a str>, Option<&'a str>)> {
    let mut head = None;
    let mut base = None;
    let mut remaining = options.iter();
    while let Some(option) = remaining.next() {
        let value = remaining.next().copied();
        match (*option, value) {
            ("--head", Some(value)) => head = Some(value),
            ("--base", Some(value)) => base = Some(value),
            _ => bail!("invalid check-branch option {option:?}\n\n{USAGE}"),
        }
    }
    Ok((head, base))
}

/// Parses the options after `deploy <addon>`: `--release` and
/// `--features <list>`, each at most once, in any order.
///
/// # Errors
/// Returns an error for an unknown, repeated or incomplete option.
fn parse_deploy_options<'a>(
    options: &[&'a str],
) -> Result<(dev::deploy::Profile, Option<&'a str>)> {
    let mut profile = dev::deploy::Profile::Debug;
    let mut features = None;
    let mut rest = options;
    loop {
        rest = match rest {
            [] => return Ok((profile, features)),
            ["--release", tail @ ..] if profile == dev::deploy::Profile::Debug => {
                profile = dev::deploy::Profile::Release;
                tail
            }
            ["--features", list, tail @ ..] if features.is_none() && !list.starts_with("--") => {
                features = Some(*list);
                tail
            }
            _ => bail!("unexpected arguments: {options:?}\n\n{USAGE}"),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deploy_options_in_any_order() {
        let parsed = parse_deploy_options(&["--features", "fault-injection", "--release"]).ok();
        assert_eq!(
            parsed,
            Some((dev::deploy::Profile::Release, Some("fault-injection")))
        );
        assert_eq!(
            parse_deploy_options(&[]).ok(),
            Some((dev::deploy::Profile::Debug, None))
        );
    }

    #[test]
    fn deploy_options_reject_repeats_and_gaps() {
        assert!(parse_deploy_options(&["--release", "--release"]).is_err());
        assert!(parse_deploy_options(&["--features"]).is_err());
        assert!(parse_deploy_options(&["--features", "--release"]).is_err());
        assert!(parse_deploy_options(&["--debug"]).is_err());
    }
}
