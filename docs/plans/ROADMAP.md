# Roadmap (master plan)

- **Status:** phase 0 (foundation) in progress since 2026-10-31.

## Settled decisions

| # | Decision | Date |
|---|----------|------|
| R1 | Public GitHub repository, MIT license | 2026-10-31 |
| R2 | Monorepo, single Cargo workspace | 2026-10-31 |
| R3 | Never push without explicit approval | 2026-10-31 |
| R4 | Decisions via `grill-me`; recorded in plans / skills | 2026-10-31 |
| R5 | Every function documented (rustdoc), READMEs everywhere, English docs | 2026-10-31 |
| R6 | Addons: en + de, own settings window, help page, dark (default) + light theme, radius 0, user-customizable tokens | 2026-10-31 |
| R7 | Development in WSL, cross-compile with cargo-xwin, CI on Windows runner | 2026-10-31 |
| R8 | Phase order as listed below (memo standalone first; price archive as phase 0b) | 2026-10-31 |
| R9 | Branching: `main` (release branch, protected: PR only, **all CI checks must pass**, no force push, no deletion), `develop` (integration branch, protected: PR only, no required CI — tests run locally before the PR), work branches cut from `develop` with prefixes `feature/<name>`, `chore/<name>`, `bugfix/<name>`; releases via PR `develop` → `main` | 2026-10-31 |
| R10 | Phase definition of done: `cargo fmt`, `clippy -D warnings`, tests green; Windows CI build green; READMEs + help pages (en/de) current; tested in game (addons); new decisions recorded in the plans | 2026-10-31 |
| R11 | The repo goes public on GitHub (`tobia-codes/trailkit-gw2`) when phase 0b starts; the first push (like every push) needs explicit approval | 2026-10-31 |
| R12 | No website for now: GitHub Pages serves data files + a minimal index linking to the repo; privacy policy as `PRIVACY.md` in the repo, linked from core's first-start notice (D14); whether an Impressum is required is clarified **before** any real website exists | 2026-10-31 |
| R13 | Addon releases: memo is published as beta after phase 1 (GitHub Releases + Nexus addon library); SemVer per addon with changelog; domain **trailkit.de** bought now at INWX (R27) | 2026-10-31 |
| R14 | Branch names enforced: CI check on PRs to `develop` (and `main` for hotfixes) accepts only `feature/`, `chore/`, `bugfix/`, `hotfix/` prefixes (+ `develop` → `main` release PRs); same check as local git hook | 2026-10-31 |
| R16 | Versioning + changelogs via **Changesets** (`@changesets/cli`): every work branch adds a `.changeset/*.md` file; merges to `develop` create no versions; on `main` the Changesets action creates the version PR → versions bumped, `CHANGELOG.md` per addon, git tags (`<addon>@x.y.z`, via `changeset tag`), own release step builds DLLs and creates one GitHub release per addon; each Rust addon has a private placeholder `package.json`, a sync step copies versions into `Cargo.toml` and CI checks they match; `main` is merged back into `develop` after each release. (knope / release-please considered and rejected: knope too small, release-please needs commit-message-based workflow) | 2026-10-31 |
| R17 | The `package.json` → `Cargo.toml` version sync is a Rust `cargo xtask sync-versions` (using `toml_edit`), not a JS script → no own JS code in the repo | 2026-10-31 |
| R18 | No JS/TS linter for now; **oxlint** is added once real JS/TS code exists (e.g. a React app) | 2026-10-31 |
| R19 | **cargo-deny** runs in CI (`cargo deny check`, config `deny.toml`): security advisories (RustSec) fail the build, only MIT-compatible licenses allowed (allow-list), duplicate crate versions warn, crates only from crates.io | 2026-10-31 |
| R20 | Project/brand name **trailkit** (renamed from "trailpack" on 2026-10-31 because trailpack.de was taken). Brand is not GW2-specific: further projects (e.g. UI libraries) get their own `trailkit-<name>` repos. Checked 2026-10-31: no trademark, product or company "Trailkit" found (web search; user verifies TMview/DPMAregister); trailkit.com (2005, unused) and trailkit.net (parked) taken; **trailkit.de free**; GitHub name free. Spelling: "Trailkit" in prose, `trailkit` in code/ids | 2026-10-31 |
| R21 | **No monetization**: all addons free, no paid features; a donation link would need a separate decision first | 2026-10-31 |
| R22 | User data migration: every persisted file has `schema_version`; addons migrate forward automatically, **create a backup before migrating**, and never overwrite a file with an unknown newer schema (show "please update the addon" instead) | 2026-10-31 |
| R23 | Testing: logic (resets, time zones, schedules, import conflicts, parsers) lives in ImGui-free modules with unit tests; UI tested manually via an in-game checklist in the release PR template | 2026-10-31 |
| R24 | No code signing for now; every release publishes SHA-256 checksums and GitHub artifact attestations (build provenance) | 2026-10-31 |
| R25 | No automatic crash/telemetry reports (GDPR); errors go to the local Nexus log; support via GitHub issue forms (addon version, Nexus version, log excerpt); help pages explain where the log is | 2026-10-31 |
| R26 | Contributions: issues welcome; external PRs only after the base structure exists and after agreement in an issue; short `CONTRIBUTING.md` derived from `AGENTS.md` | 2026-10-31 |
| R27 | Domain: **trailkit.de** at INWX, **registered 2026-10-31** (DENIC status `connect`, INWX nameservers) to secure the name; DNS records are only added when core needs them; supersedes the timing in B13 | 2026-10-31 |
| R15 | `hotfix/<name>` branches for urgent fixes of released versions: cut from `main`, PR into `main` (all CI checks), then merged back into `develop` | 2026-10-31 |
| R28 | Rust via **rustup** (user-local), toolchain pinned **exactly** in `rust-toolchain.toml` (1.99.1, components rustfmt + clippy, target `x86_64-pc-windows-msvc`); edition 2024; bumps only via dedicated `chore/` branches | 2026-10-31 |
| R29 | Until the GitHub remote exists (0b): `develop` exists locally, work happens on `chore/`/`feature/` branches cut from it and is merged locally **after user approval**; the agent may commit on local work branches on its own (see AGENTS.md rule 1) | 2026-10-31 |
| R30 | Cross-compile with **cargo-xwin**; the user accepted the Microsoft CRT/SDK license (`XWIN_ACCEPT_LICENSE=1`, cache `~/.cache/cargo-xwin`). Needs LLVM in WSL (`clang`, `lld`, `llvm` from apt) because `arcdps-imgui-sys` compiles Dear ImGui (C++) with `clang-cl` | 2026-10-31 |
| R31 | `nexus-rs` is a git dependency pinned to a fixed commit (`rev`), declared once in `[workspace.dependencies]`; bumping re-verifies `nexus-addon-dev` §1 | 2026-10-31 |
| R32 | Deploy via `cargo xtask deploy <addon> [--release]` (Rust, consistent with R17): builds with cargo-xwin, copies the DLL (tmp + rename) into the GW2 addons folder taken from `TRAILKIT_GW2_ADDONS_DIR`, else `gw2_addons_dir` in git-ignored `xtask.local.toml`; never hard-coded | 2026-10-31 |
| R33 | Workspace lints: `missing_docs`, `clippy::missing_docs_in_private_items`, `clippy::undocumented_unsafe_blocks` + `unsafe_op_in_unsafe_fn = deny`, `clippy::{unwrap_used, expect_used, dbg_macro, todo} = warn` (unwrap/expect allowed in tests); **no** blanket `clippy::pedantic` — single pedantic lints may be enabled later deliberately | 2026-10-31 |
| R34 | CI workflow written in phase 0, active from the first push (0b): Windows job (fmt, clippy `-D warnings`, test, release DLL build, version-sync check) + Ubuntu job (cargo-deny, branch-name check R14) | 2026-10-31 |
| R35 | JS side of the repo uses **pnpm**, pinned via `packageManager` in the root `package.json`; `pnpm-workspace.yaml` covers `addons/*` (Changesets placeholders) and later `apps/*` | 2026-10-31 |
| R36 | Node via **fnm**, version pinned in `.node-version` (24, LTS line); CI reads the same file | 2026-10-31 |
| R37 | Changesets structure (root `package.json`, `.changeset/`, addon placeholder `package.json`) exists from phase 0; the release workflow (version PR, tags, GitHub releases) comes with the remote in 0b | 2026-10-31 |
| R38 | The phase 0 "hello" addon is the skeleton of `addons/trailkit-memo` (no throwaway addon) | 2026-10-31 |
| R39 | Addon crates are `cdylib` + `rlib`; Nexus/ImGui dependencies are `cfg(windows)`-only so platform-independent modules (R23) are unit-tested on the Linux host with plain `cargo test`; Windows-only code is linted with `cargo xwin clippy` locally and natively in CI | 2026-10-31 |
| R40 | Naming conventions live in **AGENTS.md** ("Naming conventions", always in context for every agent); the README carries a short summary + link (later also `CONTRIBUTING.md`) | 2026-10-31 |
| R41 | Rust naming strictly per the Rust API Guidelines (types `UpperCamelCase`, acronyms as words; functions/modules `snake_case`; constants `SCREAMING_SNAKE_CASE`); no catch-all names; modules as `foo.rs` + `foo/` (lint `clippy::mod_module_files`). **Readable over short:** descriptive names instead of abbreviations, only widely known abbreviations (`id`, `url`, `api`, `http`, `json`, `ui`, …) | 2026-10-31 |
| R42 | Directories and non-Rust files `kebab-case` with lowercase extensions; exceptions: Rust sources (`snake_case`), a fixed list of upper-case meta files, tool-mandated names; crates/addons prefixed `trailkit-`. `docs/plans/_TEMPLATE.md` renamed to `plan-template.md` | 2026-10-31 |
| R43 | Data identifiers (ids, slugs, JSON/TOML keys, settings) `snake_case`; files named after a data id keep it (`world_bosses.toml`) | 2026-10-31 |
| R44 | Nexus runtime identifiers: `trailkit.<addon>.<kind>.<name>` (`snake_case` segments; kinds `keybind`, `texture`, `shortcut`, `event`, `datalink`, `localization`; cross-addon `trailkit.shared.…`), always typed constants | 2026-10-31 |
| R45 | TypeScript/React (later): `kebab-case` file names incl. components (case-insensitive file systems); identifiers per TS convention (`PascalCase` components/types, `useXyz` hooks, `camelCase` otherwise) | 2026-10-31 |
| R46 | Path names enforced by `cargo xtask check-names` (all tracked + untracked-not-ignored paths, vendored skills exempt) in CI and later in the local git hook | 2026-10-31 |
| R47 | `crates/trailkit-ui` exists from phase 0 with **only** theme tokens (`Theme::DARK` default, `Theme::LIGHT`, WCAG contrast test) and the `StyleScope` RAII guard; components move in only on real reuse (rule 7) | 2026-10-31 |
| R48 | Local git hooks versioned in `.githooks/`, enabled via `cargo xtask install-hooks` (`core.hooksPath`): pre-commit = `cargo fmt --check` + `check-names`; pre-push = `check-branch` + clippy + tests. CI uses the same `cargo xtask check-branch` | 2026-10-31 |
| R49 | Addon updates use Nexus provider **`Direct`**: a fixed URL per addon serving the latest DLL + `<url>.md5`, published by the release workflow (0b) to GitHub Pages. Reason (Nexus source, v2026.10.25): the `GitHub` provider parses release tags as `v?X.Y.Z` (our `<addon>@x.y.z` tags are skipped) and takes the first `.dll` of the highest release across the whole repo → unusable in a monorepo; `Raidcore` provider is unimplemented. No pre-release channel (betas ship as 0.x). Exact URL/domain: see next decision | 2026-10-31 |
| R50 | Shipped/deployed DLLs are named after the `kebab-case` crate (`trailkit-memo.dll`); `xtask deploy` and the release workflow rename cargo's `trailkit_memo.dll` | 2026-10-31 |
| R51 | Theme palette: anthracite surfaces + **GW2-like gold accent** (dark `#D9A441` with dark text, light `#8C5E0A` with white text), green/yellow/red/blue status colors; all text pairs ≥ 4.5:1 (unit-tested); fine-tuning after the first in-game test. Values live in `crates/trailkit-ui/src/theme.rs` | 2026-10-31 |
| R52 | Domain of the `trailkit-gw2` GitHub Pages site: **`gw2.trailkit.de`** (one subdomain per repo/product line, since Pages allows one custom domain per repo; future `trailkit-<name>` repos get `<name>.trailkit.de`; apex `trailkit.de` stays free for a website). Addon update link (R49): `https://gw2.trailkit.de/addons/<addon>/<addon>.dll` + `.md5`; the served data files move there at the first public release (D11). INWX gets the CNAME `gw2` → `tobia-codes.github.io` at the first addon release (supersedes R27 timing). Addons switch from `UpdateProvider::None` to `Direct` with this link in the release that first publishes there. **GDPR:** Nexus update checks transmit the player's IP address to GitHub Pages (USA, DPF) without an in-addon consent step → must be covered in `PRIVACY.md` and the help pages before the first release | 2026-10-31 |
| R53 | Nexus options tab (ux-rules G2): **no full settings there** — only a header (addon name + version) and the buttons "Open settings" and "Open help". Avoids a duplicate settings UI; search and navigation live in the addon's own settings window | 2026-10-31 |
| R54 | Item rarity colors as `rarity.junk` … `rarity.legendary` theme tokens in `trailkit-ui` (G3): light theme = standard GW2 values, dark theme = standard values except Legendary `#974EFF`; contrast requirement relaxed to ≥ 3:1 for these game-identity colors | 2026-10-31 |
| R55 | Quick access icons (G4): Nexus style (64×64 RGBA, parchment fill, dark outline, lighter `_hover` variant), generated by a script from SVG sources in the repo (MIT, reproducible, no third-party art); every addon has a "show quick access icon" setting, default on | 2026-10-31 |
| R56 | Gameplay auto-hide (G5): all trailkit windows hide while `IsGameplay == false` (loading screens, cutscenes, character select); per-addon setting "also show outside gameplay", default off; further hide rules (combat, PvP, …) are decided per addon when needed | 2026-10-31 |
| R57 | Icon pipeline (R55): `cargo xtask build-icons` renders every `addons/<addon>/assets/icons/*.svg` with `resvg` to 64×64 PNGs; the PNGs are **committed** and CI verifies them with `build-icons --check`; addons embed them via `include_bytes!` + `load_texture_from_memory` (no extra files in the addons folder) | 2026-10-31 |
| R58 | Hover variant is derived, not hand-drawn: xtask replaces the parchment fill token in the SVG with a lighter hover fill and renders `<name>-hover.png`; one SVG per icon | 2026-10-31 |
| R59 | trailkit-memo quick access motif: note sheet with folded corner and lines, pencil across it | 2026-10-31 |
| R60 | Phase 0 settings: a typed `Settings` struct with defaults (icon shown, hidden outside gameplay) drives behavior; persistence and the settings window come in phase 1, together with the Nexus options tab (R53); rarity tokens (R54) are added when the first addon needs them | 2026-10-31 |
| R61 | Auto-hide (R56) only suppresses drawing: an open window reappears after the loading screen, the keybind toggles normally outside gameplay, and the Nexus quick access bar is left to Nexus | 2026-10-31 |
| R62 | Before the repo goes public, every commit's author/committer email is rewritten to the dedicated public address `tobiacodes@gmail.com` and the repo's `user.email` is set to it (the private address must never appear in public history) | 2026-10-31 |
| R63 | First push: `main` is fast-forwarded to `develop`, both are pushed, then branch protection (R9) is enabled; from then on changes reach `develop`/`main` only via PR | 2026-10-31 |
| R64 | Phase 0b scope and order: (1) repo public + first push + branch protection, (2) `tools/trailkit-datagen` + archive workflow (daily, catch-up, WvW reset snapshots), (3) monthly C-703/25 P reminder issue. The addon release workflow (R37/R49) moves to the end of phase 1 (first memo release; needs `gw2.trailkit.de`, R52) | 2026-10-31 |
| R65 | One HTTP client everywhere: **`ureq`** (blocking, `rustls`, no Tokio) for datagen and all addons, so GW2 API code (throttle, retries) can be shared. `reqwest` (most popular, but async/Tokio, also in its blocking mode) is not used in addons; it may be reconsidered for the deferred backend server | 2026-10-31 |
| R66 | One date/time library everywhere: **`jiff`** (Temporal-like API, built-in time zone database, DST-correct arithmetic — needed later for memo reset times and event schedules in local time); 0.x risk accepted because versions are pinned and bumped deliberately | 2026-10-31 |
| R67 | **`develop` is the GitHub default branch** (PRs target it automatically). Consequence: scheduled workflows (price archive, DPF reminder) run from `develop`, so datagen/workflow changes take effect when merged into `develop`; addon releases still go `develop` → `main` (R9) | 2026-10-31 |
| R68 | **No AI attribution** in commits or PRs (no `Co-Authored-By` trailer, no "Generated with …" footer), so only `tobia-codes` appears as contributor. The existing history was rewritten once on 2026-10-31 (branch protection lifted for the force push of `main`/`develop`, then restored) | 2026-10-31 |

## Proposed repository layout

```
trailkit-gw2/
├─ Cargo.toml                 # workspace + shared lints
├─ addons/
│  ├─ trailkit-core/
│  ├─ trailkit-memo/
│  └─ trailkit-event/
├─ crates/                    # shared libraries (only once reused, see AGENTS.md rule 7)
│  ├─ trailkit-ui/           # theme, tokens, shared ImGui components
│  ├─ trailkit-i18n/         # language resolution + shared strings
│  ├─ trailkit-protocol/     # repr(C) cross-addon contract + safe wrappers
│  └─ trailkit-api-types/    # data-file DTOs shared by datagen and core
├─ tools/trailkit-datagen/   # daily job: static data files + price archive
├─ backend/                   # (later) axum service, see backend.md
├─ data/events/               # curated event schedules (source of truth)
└─ docs/plans/
```

## Phases

0. **Foundation** — toolchain, workspace, lints, CI (Windows build), a
   minimal "hello" addon to validate build → deploy → hot reload in game,
   `trailkit-ui` theme skeleton (dark/light tokens, style guard); research GW2
   UI + popular Nexus/arcdps addon conventions → `trailkit-ui/references/ux-rules.md` §8.
0b. **Price archive** — minimal `trailkit-datagen` + scheduled workflow:
   prices, condensed order book, WvW, gem exchange, build id (no item-id
   diff, D30)
   → monthly GitHub Releases (data-pipeline D4–D17); monthly C-703/25 P
   reminder issue (D18); branch protection (R9).
1. **trailkit-memo standalone** (no core) — fastest user value; forces the
   shared UI building blocks (settings window, help window, tree with
   drag & drop, markdown view, dialogs, import/export, i18n).
2. **Data pipeline, rest** (no server, see `data-pipeline.md`) — served data
   files (items, achievements, waypoints, prices, curated events from TOML),
   `meta.json`, Pages + release-asset delivery (D19).
3. **trailkit-core** — cache, data set registry, cross-addon protocol,
   API key (DPAPI), dependency hints.
4. **memo ↔ core** — `/e` autocomplete, live event chips.
5. **trailkit-event** — timeline, custom events, multiple tables, memo
   highlighting.
6. *(later)* **Backend server** — only when a feature needs dynamic queries
   (price history); seeded from the archive (`backend.md`, BL5).

## Open

- ~~Nexus update provider in a monorepo~~ → settled in R49.

## Plans

- [Data pipeline](data-pipeline.md) (current)
- [Backend](backend.md) (deferred)
- [trailkit-core](addons/trailkit-core.md)
- [trailkit-memo](addons/trailkit-memo.md)
- [trailkit-event](addons/trailkit-event.md)
