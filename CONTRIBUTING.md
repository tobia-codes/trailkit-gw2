# Contributing to trailkit-gw2

Thank you for your interest! This guide explains how the project is built and
**why** it follows the rules it does, so your change fits in on the first try.
It is the long form of the rules; binding details live in the documents it
links (naming table, design tokens, skills) and are not repeated here.

## Contents

1. [Before you start](#1-before-you-start)
2. [Setup](#2-setup)
3. [How the repository is organised](#3-how-the-repository-is-organised)
4. [Rust code rules](#4-rust-code-rules)
5. [Addon rules (Nexus, in game)](#5-addon-rules-nexus-in-game)
6. [UI rules](#6-ui-rules)
7. [Privacy and security](#7-privacy-and-security)
8. [Tests and checks](#8-tests-and-checks)
9. [Git workflow and pull requests](#9-git-workflow-and-pull-requests)
10. [What the maintainer does](#10-what-the-maintainer-does)

## 1. Before you start

- **Open an issue first.** Bug reports and ideas are always welcome. For a
  pull request, agree on the change in an issue before you write code
  (ROADMAP R26): decisions about scope, architecture, libraries, naming and
  UX are made by the maintainer and recorded in [`docs/plans/`](docs/plans/README.md).
  Read the relevant plan first; settled decisions are not reopened in a PR.
- **Language.** Everything in the repository is English: code, identifiers,
  comments, docs, plans and commit messages. Issues and PRs are English too;
  issues in German are welcome.
- **License.** Contributions are licensed under the same terms as the files
  they change ("inbound = outbound"): code under [MIT](LICENSE), event data in
  `data/events/` under [CC BY-NC-SA 2.5](data/events/LICENSE). No CLA or
  sign-off is needed.
- **Security problems** are never reported in a public issue: see
  [`SECURITY.md`](SECURITY.md).

## 2. Setup

Development happens in **WSL** (Linux). Addons are cross-compiled to Windows
(`x86_64-pc-windows-msvc`) with `cargo-xwin` and copied into the game's addons
folder. Follow the [setup in the README](README.md#setup-wsl), then enable the
git hooks with `cargo xtask install-hooks`: pre-commit runs `cargo fmt` and the
naming check, pre-push checks the branch name, Clippy and the tests.

To try an addon in game you need Guild Wars 2 with
[Raidcore Nexus](https://raidcore.gg/Nexus); `cargo xtask deploy <addon>`
builds it and hot-reloads it into the running game
([xtask](xtask/README.md#configuring-the-gw2-addons-folder)).

## 3. How the repository is organised

One monorepo, one Cargo workspace (R2). Every project has its own README;
start from [Find your way](README.md#find-your-way).

| Folder | What lives there |
|--------|------------------|
| `addons/<addon>/` | One Nexus addon (a Windows DLL), split into slices |
| `crates/` | Libraries shared by several projects, flat (R90) |
| `services/` | Server-side programs (the deferred REST API) |
| `tools/` | Developer tools: the daily data job and a [Bruno collection](tools/bruno-gw2-api/README.md) for trying the GW2 API by hand |
| `xtask/` | Build and check commands: `cargo xtask <command>` |
| `data/` | Curated source data |
| `docs/plans/` | Roadmap and decisions; each decision has an id (R…, M…, B…) |

### Addons are built from slices (R76)

An addon is an **app crate** (`addons/<addon>/`, builds the DLL and wires
everything), **slice crates** (`addons/<addon>/slices/<slice>/`, one feature
each) and one **shared crate** (`addons/<addon>/shared/`).

- **A slice never depends on another slice.** When slice A needs something of
  slice B, the app hands it in as a neutral type from `shared` (a trait, a
  plain data type or a closure). Why: a feature can be understood, tested and
  removed by deleting one folder, one dependency and its wiring.
- A slice owns its state, its `draw` function, its text enum and its section
  of the help page. It returns what the user did as an action (`XyzAction`);
  the app decides what happens.
- A slice never writes files itself: persisted values come in as `&mut` to
  the fields it needs, the slice reports a change, the app saves.
- Full blueprint, slice contract and checklist for a new addon:
  [`addon-structure.md`](.agents/skills/nexus-addon-dev/references/addon-structure.md).

### Share on the second real use (rule 7)

Before you add a helper or a type, search the workspace. Code is extracted
into `shared/` (within an addon) or `crates/` (across projects) only when a
**second** real user appears, and then all copies are replaced in the same
change. Nothing is shared "in case we need it later". Shared UI always comes
from [`crates/trailkit-ui`](crates/trailkit-ui/README.md).

## 4. Rust code rules

- **Type everything (rule 8).** Prefer newtype ids (`NoteId`, `ItemId`),
  enums and serde structs over strings, integers and loose JSON. Make invalid
  states unrepresentable: if two fields only make sense together, they are one
  enum variant. Untyped JSON is allowed only at boundaries (an API response
  before it is converted).
- **Document every item (rule 4)**: modules (`//!`), functions, methods,
  structs, enums, variants, fields and traits, public **and** private.
  Name the errors (`# Errors`), panics (`# Panics`) and safety contracts
  (`# Safety`) where they apply. The workspace lints (`missing_docs`,
  `missing_docs_in_private_items`) report what is missing.
- **Readable names.** No abbreviations beyond widely known ones (`id`, `url`,
  `api`, `ui` …), no catch-all names (`Manager`, `Utils`, `Data`, `Info`).
  Rust files `snake_case.rs`, module folders as `foo.rs` + `foo/` (no
  `mod.rs`), other paths `kebab-case`, crates `trailkit-*`, data keys
  `snake_case`. The binding table:
  [naming conventions](docs/agent-reference.md#naming-conventions), checked
  by `cargo xtask check-names`.
- **Small files and functions.** `cargo xtask check-file-length` hints at
  files with ≥ 400 code lines and functions with ≥ 80 (R77). Split by concern,
  and mention a hint in the PR if you think the file should stay as it is.
- **Errors, not panics.** Return `Result` with context: libraries use typed
  errors (`thiserror`, e.g. `trailkit-storage`), tools use `anyhow`.
  `unwrap()` and `expect()` are lint errors outside tests; a panic must never
  cross the FFI boundary into the game.
- **Lints are errors.** Clippy runs with `-D warnings`; fix the cause.
  [`clippy.toml`](clippy.toml) also bans a few methods and names the
  replacement (e.g. tooltips go through `trailkit_ui::tooltip`).
- **Performance beats DLL size** (R72), and dependencies are checked by
  `cargo deny` (licenses, advisories).

## 5. Addon rules (Nexus, in game)

An addon runs inside the game's process. A slow frame or a crash is felt by
the player, so these rules are strict. The reference with the details:
[`nexus-addon-dev` skill](.agents/skills/nexus-addon-dev/SKILL.md).

- **The render thread only draws.** No file or network I/O, no blocking locks,
  no parsing and no steady-state allocations in `draw`. Expensive work runs on
  the addon's one worker thread, which blocks on a channel instead of polling;
  the render thread reads finished results (e.g. through `arc-swap`).
- **Closed means asleep.** When no window is visible, the render callback
  returns after one atomic load.
- **Event and keybind callbacks** copy what they need, send it to a channel or
  flip an atomic, and return.
- **All files go through [`trailkit-storage`](crates/trailkit-storage/README.md)**
  (R22): versioned, crash-safe writes with safety copies. Never hand-roll
  file I/O.
- **Log through [`trailkit-log`](crates/trailkit-log/README.md)** (R92): the
  message is a snake_case event listed in its README, details are `key = value`
  pairs. Log file names, never paths, and never user content (GDPR); a test
  checks the event list.
- **Every text is localized in English and German** (rule 5): a typed text
  enum per slice, never a string literal in UI code. A test checks that every
  text is translated.
- **The help page stays current** (rule 6): a change to user-visible behavior
  updates `addons/<addon>/help/en.md` and `de.md` in the same PR. Use only
  characters the bundled fonts contain (a test checks this).
- **Nexus identifiers** (keybinds, textures, events …) are typed constants
  named `trailkit.<addon>.<kind>.<name>`, never inline strings.

## 6. UI rules

All addons share one look: calm, flat, rounded, a brass accent used sparingly,
Dark and Light themes. Read the [`trailkit-ui` skill](.agents/skills/trailkit-ui/SKILL.md)
and [`current-design.md`](.agents/skills/trailkit-ui/references/current-design.md)
before changing anything visible.

- **Propose before you build (R87).** For any visible change, also a small
  one (layout, wording, color, icon, a new state), post a visual proposal in
  the issue first: every affected state in Dark and Light, plus the minimum
  window width and German text where the layout changes. Implement after the
  maintainer approves it.
- **Use tokens, never raw values.** Colors, sizes, radii and spacing come from
  the theme (`theme.spacing.small`, `theme.colors…`). A new token is a
  decision; how to add one: [`tokens.md`](.agents/skills/trailkit-ui/references/tokens.md),
  then `cargo xtask tokens`. Generated token values are never edited by hand.
- **Text has roles (R96).** Every text is a title, section, body, info,
  fine print or status text and is drawn through `TextStyle` in exactly that
  role: headings and content in `text.primary`, info and hints in
  `text.secondary`, never greyed out to show a state. The table and the icon
  and empty-state rules are in
  [`tokens.md` §7.1](.agents/skills/trailkit-ui/references/tokens.md#71-text-roles-r96-q284).
- **Every state has a snapshot (R82).** Each element gets a snapshot of every
  state it can be in (hover, pressed, empty, error, minimum width, German …),
  in Dark and Light, in the same change. `cargo xtask ui-test --update` writes
  the missing and changed ones; open and review every new image before you
  open the PR. Without a local Windows build, CI uploads the changed images
  as the artifact `snapshot-updates` when the snapshot tests fail (R97). How:
  [`tokens.md` §10](.agents/skills/trailkit-ui/references/tokens.md#10-every-state-has-a-snapshot-every-snapshot-gets-reviewed-r82).
- **Design the failure case (R93).** If an action can fail (a file, the
  network, an API), the proposal also shows what the user sees then and how
  they recover. Default: retry automatically a few times, then an error toast
  with "Try again", as memo does for saving (M180). Buttons whose action runs
  show a spinner for at least 0.6 s and cannot be clicked twice
  (`trailkit_ui::busy::BusyState`). A control that cannot be used now is
  disabled and says why in its tooltip (`ButtonState::Disabled { reason }`).
- **Accessibility**: text contrast ≥ 4.5:1 in both themes, meaning never by
  color alone, icons always with a tooltip, destructive actions ask first.
- **New icons**: add them to `Icon::ALL` and run `cargo xtask subset-fonts`.

## 7. Privacy and security

The GDPR applies to every part of the project (rule 10).

- **No telemetry, no tracking, nothing sent automatically.** Errors go to the
  local log; users decide what to send in an issue (R25).
- **Logs hold no user content** (no note text, no API keys) and no absolute
  paths.
- **API keys and user data are secrets**: never logged, never committed, never
  sent anywhere except where the user asked.
- Any new service, provider or website needs a privacy review: minimal data
  (IP addresses count), EU providers preferred, and the maintainer decides.
- **The privacy texts stay current** (`PRIVACY.md`, `PRIVACY.de.md`, R99).
  A change to what data leaves the player's PC, where it goes or who receives
  it (a new endpoint, server, provider, log field, report form, the backend)
  updates both texts in the same pull request. Before every release they are
  checked against the real data flows.

## 8. Tests and checks

Run these before you push (`source ~/.cargo/env` first if cargo is not found):

| Command | What it checks |
|---------|----------------|
| `cargo fmt --all` | Formatting |
| `cargo clippy --all-targets -- -D warnings` | Lints, Linux build |
| `cargo xwin clippy --target x86_64-pc-windows-msvc --all-targets -- -D warnings` | Lints of Windows-only addon code, which the Linux run skips |
| `cargo test` | Unit tests, translations, generated docs up to date, README links |
| `cargo xtask ui-test` | UI snapshots (Windows test binaries run through WSL, about 4 minutes) |
| `cargo xtask check-names` | Paths follow the naming conventions |
| `cargo xtask check-file-length` | Advisory size hints for changed files |
| `cargo deny check` | Dependency licenses and advisories |

Tests never call live services; tools use committed fixtures. Every command:
[`xtask/README.md`](xtask/README.md).

## 9. Git workflow and pull requests

- **Branch** from a freshly fetched `develop`:
  `git fetch origin && git switch -c feature/<topic> origin/develop`. Prefixes:
  `feature/`, `bugfix/`, `chore/`, `hotfix/`, then `kebab-case` (R14, checked
  by CI). `main` holds released code; nobody pushes to `develop` or `main`
  directly.
- **Commits** follow [Conventional Commits](https://www.conventionalcommits.org)
  (`feat(memo): …`, `fix: …`, `docs: …`). Keep each commit one logical step.
  Do not rewrite history that reviewers have seen; add commits instead.
- **Versioning** uses Changesets (R16): add `pnpm changeset` for a
  user-visible change, unless [`.changeset/README.md`](.changeset/README.md)
  lists an exception.
- **Pull request** into `develop`, linked to its issue. Describe what changed
  and why, list the snapshots you added and, for addon changes, what you tested
  in game. Docs (README, help page, plan) change in the same PR as the
  behavior.
- **CI must pass**: formatting, Clippy and tests on Windows, branch name,
  `cargo deny`. Fix failures; never bypass a hook or a check.

## 10. What the maintainer does

The maintainer reviews, decides open questions, records decisions in the
plans, merges into `develop`, and alone handles merges into `main`, releases,
tags, deploys and repository settings.
