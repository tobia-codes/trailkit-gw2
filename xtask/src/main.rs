//! Repository task runner, invoked as `cargo xtask <command>`.
//!
//! Commands:
//! - `build-icons [--check]` — render the addons' SVG icons to the committed
//!   PNGs (ROADMAP R57).
//! - `deploy <addon> [--release]` — cross-compile an addon with cargo-xwin and
//!   copy the DLL into the Guild Wars 2 `addons` folder (ROADMAP R32).
//! - `check-branch [--head <branch>] [--base <branch>]` — verify branch names
//!   (ROADMAP R14); defaults to the current branch.
//! - `install-hooks` — enable the versioned git hooks in `.githooks/` (R48).
//! - `check-names` — verify file and directory names against the naming
//!   conventions in AGENTS.md (ROADMAP R46).
//! - `sync-versions [--check]` — copy each addon's `package.json` version
//!   (owned by Changesets) into its `Cargo.toml` (ROADMAP R16/R17).
//! - `subset-fonts [--check]` — cut the embedded fonts down to the characters
//!   trailkit can show (M62).
//! - `ui-test [--update]` — build the Windows UI tests and run them (from WSL
//!   through interop); `--update` rewrites the snapshot references (M65).

mod build_icons;
mod check_branch;
mod check_names;
mod deploy;
mod install_hooks;
mod subset_fonts;
mod sync_versions;
mod ui_test;
mod workspace;

use anyhow::{Result, bail};

/// Usage text printed for `help` and on invalid arguments.
const USAGE: &str = "\
usage: cargo xtask <command>

commands:
  build-icons [--check]        render addon SVG icons to PNG (--check: only verify)
  deploy <addon> [--release]   build an addon for Windows and copy it into the GW2 addons folder
  check-branch [--head B] [--base B]  verify the branch name (default: current branch)
  install-hooks                enable the git hooks in .githooks/
  check-names                  verify file/directory names against the naming conventions
  sync-versions [--check]      copy package.json versions into Cargo.toml (--check: only verify)
  subset-fonts [--check]       cut the embedded fonts to the shown characters (--check: only verify)
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
            let profile = match rest {
                [] => deploy::Profile::Debug,
                ["--release"] => deploy::Profile::Release,
                _ => bail!("unexpected arguments: {rest:?}\n\n{USAGE}"),
            };
            deploy::run(addon, profile)
        }
        ["build-icons"] => build_icons::run(build_icons::Mode::Write),
        ["build-icons", "--check"] => build_icons::run(build_icons::Mode::Check),
        ["check-names"] => check_names::run(),
        ["check-branch", options @ ..] => {
            let (head, base) = parse_branch_options(options)?;
            check_branch::run(head, base)
        }
        ["install-hooks"] => install_hooks::run(),
        ["sync-versions"] => sync_versions::run(sync_versions::Mode::Write),
        ["sync-versions", "--check"] => sync_versions::run(sync_versions::Mode::Check),
        ["subset-fonts"] => subset_fonts::run(subset_fonts::Mode::Write),
        ["subset-fonts", "--check"] => subset_fonts::run(subset_fonts::Mode::Check),
        ["ui-test"] => ui_test::run(ui_test::Mode::Compare),
        ["ui-test", "--update"] => ui_test::run(ui_test::Mode::Update),
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
