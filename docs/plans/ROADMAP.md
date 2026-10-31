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
   prices, condensed order book, WvW, gem exchange, build id, item-id diff
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

- Nexus update provider in a monorepo with several addons (GitHub provider
  may only look at the repo's *latest* release) — research in phase 0.

## Plans

- [Data pipeline](data-pipeline.md) (current)
- [Backend](backend.md) (deferred)
- [trailkit-core](addons/trailkit-core.md)
- [trailkit-memo](addons/trailkit-memo.md)
- [trailkit-event](addons/trailkit-event.md)
