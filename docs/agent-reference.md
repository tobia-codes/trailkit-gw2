# Agent reference

Shared reference for Codex and Claude Code. Read only the section needed for
the current task; AGENTS.md contains the always-applicable rules. These
conventions remain binding when working in their scope.

## Naming conventions

Binding for every language and every file (ROADMAP R40–R46). Checked by the
compiler/clippy (Rust) and `cargo xtask check-names` (paths, CI).

**Readable over short.** Names must be understandable by any developer without
context: prefer a long, descriptive name (`package_json_version`,
`render_main_window`) over abbreviations (`pkg_ver`, `rmw`). Only widely known
abbreviations are allowed (`id`, `url`, `api`, `http`, `json`, `ui`, `dll`,
`gw2`). No single-letter names except trivial index loops. No catch-all names
(`Manager`, `Helper`, `Utils`, `Misc`, `Data`, `Info`).

| What | Convention | Example |
|------|-----------|---------|
| Rust types, traits, enum variants | `UpperCamelCase`, acronyms as words | `NoteTree`, `ApiKey`, `HttpClient` |
| Rust functions, methods, variables, modules | `snake_case` | `load_notes`, `mod sync_versions` |
| Rust constants, statics | `SCREAMING_SNAKE_CASE` | `KEYBIND_TOGGLE` |
| Rust source files | `snake_case.rs`; module dirs as `foo.rs` + `foo/` (no `mod.rs`, lint `clippy::mod_module_files`) | `src/note_tree.rs` |
| Crates, packages, addon directories | `kebab-case`, prefix `trailkit-` | `trailkit-memo`, `trailkit-ui` |
| All other directories and files (Rust module directories below `src/` are `snake_case` like their module) | `kebab-case`, lowercase extensions | `data-pipeline.md`, `plan-template.md` |
| Fixed upper-case meta files | only these: `README.md`, `AGENTS.md`, `CLAUDE.md`, `LICENSE`, `CHANGELOG.md`, `CONTRIBUTING.md`, `PRIVACY.md`, `SECURITY.md`, `ROADMAP.md`, `BACKLOG.md`, `NEXT_SESSION.md`, `SKILL.md` | |
| Tool-mandated files | as the tool requires | `Cargo.toml`, `package.json` |
| Data identifiers (ids, slugs, JSON/TOML keys, settings) | `snake_case`; files named after a data id keep it | `world_bosses`, `schema_version`, `world_bosses.toml` |
| Nexus runtime identifiers (global namespace shared with all addons) | `trailkit.<addon>.<kind>.<name>`, `snake_case` segments, `<kind>` ∈ `keybind`, `texture`, `shortcut`, `event`, `datalink`, `localization`, `font`; cross-addon: `trailkit.shared.…`; always typed constants, never inline | `trailkit.memo.keybind.toggle`, `trailkit.core.event.data_updated` |
| TypeScript/React (later) | files `kebab-case` (also components); components/types `PascalCase`, hooks `useXyz`, else `camelCase` | `note-tree.tsx` → `NoteTree` |
| Git branches | `feature/`, `chore/`, `bugfix/`, `hotfix/` + `kebab-case` (R9, R14) | `feature/memo-tree` |

Vendored third-party skills keep their upstream names.

## README outline

Every project README (rule 3, ROADMAP R91) follows this order; a section that
has nothing to say is left out, slices stay short:

1. **Purpose**: one or two sentences, what it is and who uses it.
2. **Status**: one line, linking the plan. No feature or progress lists; they
   go stale. Behavior is described in the plan, the help page and the code.
3. **Contents**: modules, files or sub-projects, each with one line; a parent
   README links every child README (checked by a test).
4. **Build and test**: the exact commands.
5. **Usage** or **API**: how to run it or how other code uses it.
6. **Rules**: what applies to this part only (allowed dependencies, threading,
   what does not belong here), linking `CONTRIBUTING.md` for general rules.
7. **Links**: plans, decisions, upstream docs.

## Development environment

- Development happens in **WSL** (Linux). Addons are cross-compiled with
  `cargo-xwin` to `x86_64-pc-windows-msvc` and copied into the Windows GW2
  `addons` folder by a deploy script; CI additionally builds on a Windows
  runner.
- Start a new branch from a fresh `develop`: `git fetch origin` and branch from
  `origin/develop`. After a merge the local `develop` is stale until pulled, and a
  branch from it silently drops what was just merged (it undid M170 once, seen only
  in game).
- Run `source ~/.cargo/env` before cargo. Plain `grep` may fail in this shell;
  use `command grep`.
- Commands used daily:
  | Command | Purpose |
  |---------|---------|
  | `cargo xtask deploy trailkit-memo` | Build and hot-reload into the running game; `addons/Nexus/Nexus.log` in the game folder logs "Loaded addon" 1-2 s later |
  | `cargo xtask ui-test [--update]` | Snapshot tests, about 4 minutes. Adding one, coordinates, reviewing, measuring in game: [`tokens.md` §10](../.agents/skills/trailkit-ui/references/tokens.md). `--update` rewrites only differing references |
  | CI artifact `snapshot-updates` | Without a local Windows build (cloud sessions): push, let the Windows job fail, download the artifact (changed references + `failures/` diffs, R97), review, copy the references into `tests/snapshots/`, commit |
  | `cargo xwin clippy --target x86_64-pc-windows-msvc --all-targets -- -D warnings` | Needed besides host clippy, which skips Windows-only code |
  | `cargo xtask subset-fonts` | After adding an icon to `Icon::ALL` |
  | `cargo xtask tokens` | After changing a token |
  | `cargo xtask check-names` | Naming conventions of paths |
- Game folder `C:\Guild Wars 2\addons` (WSL: `/mnt/c/Guild Wars 2/addons`), log
  `addons/Nexus/Nexus.log`. Node 24 via fnm, pnpm 11.17.0. `gh` is logged in as
  `tobia-codes` over ssh.

## Recurring checks

Check the current status in `docs/plans/NEXT_SESSION.md` first. Repeat a
check when its trigger applies or the recorded next check is due; do not
repeat a completed check during the same session without new evidence.

- **GW2 wiki:** regularly re-check public availability, whenever working on
  event data and at the start of each phase. The availability test, Wayback
  workaround and return-to-live rule are in `data/events/README.md`.
- **EU-US Data Privacy Framework:** at the start of every session touching
  planning, hosting or core, and at least at the start of every phase,
  check CJEU appeal **C-703/25 P (Latombe v Commission)**. If the framework
  is annulled or suspended, tell the user immediately and plan the switch
  to an EU host (data-pipeline D11). Core's planned GitHub Pages downloads
  rely on the framework; do not treat an old handoff entry as a current check.
- **Scheduled jobs:** follow due reminders and archive checks in
  `NEXT_SESSION.md`. Report an unverified or blocked check explicitly.

## Operational facts and instruction maintenance

- Cargo is a root workspace. Every crate opts into workspace lints from
  `Cargo.toml`, including documentation and unsafe-block checks.
- `gh` is configured for `tobia-codes` over SSH; verify actual tool access
  before relying on it. A tool allowlist is not workflow approval to publish.
- Some Claude Code modes have refused branch-protection changes and
  `git commit --amend`. No agent amends commits; use a follow-up commit.
- In Codex, the observed filesystem sandbox made `.git` read-only and blocked
  `gh` via snap-confine. Fetching and switching branches required tool approval.
  Deployment to `/mnt/c` has not yet been verified in Codex. Follow the tool's
  supported approval flow; never bypass a refusal with another tool or flag.
- Avoid unnecessary environment setup for a design review: a local HTML
  mockup using the existing tokens is acceptable (current-design.md). Check
  existing rendering tools first; agree before expanding the setup.
- `rust-best-practices`, `rust-skills` and `grilling` retain their upstream
  licenses and names, with local workflow adaptations documented in
  `.agents/skills/README.md`. Preserve those adaptations when updating upstream.
- Decisions live in `docs/plans/`; lasting project facts live in the relevant
  skill. `NEXT_SESSION.md` holds only current state and next steps, under
  100 lines. Update these in the same change so either agent can resume.
