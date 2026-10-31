# Roadmap (master plan)

- **Status:** draft — phase order settled (R8), details per plan still being grilled.

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
| R11 | The repo goes public on GitHub (`tobia-codes/trailpack-gw2`) when phase 0b starts; the first push (like every push) needs explicit approval | 2026-10-31 |
| R12 | No website for now: GitHub Pages serves data files + a minimal index linking to the repo; privacy policy as `PRIVACY.md` in the repo, linked from core's first-start notice (D14); whether an Impressum is required is clarified **before** any real website exists | 2026-10-31 |
| R13 | Addon releases: memo is published as beta after phase 1 (GitHub Releases + Nexus addon library); SemVer per addon with changelog; own domain (B13) bought at the first **core** release (memo needs none) | 2026-10-31 |

## Proposed repository layout

```
trailpack-gw2/
├─ Cargo.toml                 # workspace + shared lints
├─ addons/
│  ├─ trailpack-core/
│  ├─ trailpack-memo/
│  └─ trailpack-event/
├─ crates/                    # shared libraries (only once reused, see AGENTS.md rule 7)
│  ├─ trailpack-ui/           # theme, tokens, shared ImGui components
│  ├─ trailpack-i18n/         # language resolution + shared strings
│  ├─ trailpack-protocol/     # repr(C) cross-addon contract + safe wrappers
│  └─ trailpack-api-types/    # data-file DTOs shared by datagen and core
├─ tools/trailpack-datagen/   # daily job: static data files + price archive
├─ backend/                   # (later) axum service, see backend.md
├─ data/events/               # curated event schedules (source of truth)
└─ docs/plans/
```

## Phases

0. **Foundation** — toolchain, workspace, lints, CI (Windows build), a
   minimal "hello" addon to validate build → deploy → hot reload in game,
   `trailpack-ui` theme skeleton (dark/light tokens, style guard); research GW2
   UI + popular Nexus/arcdps addon conventions → `trailpack-ui/references/ux-rules.md` §8.
0b. **Price archive** — minimal `trailpack-datagen` + scheduled workflow:
   prices, condensed order book, WvW, gem exchange, build id, item-id diff
   → monthly GitHub Releases (data-pipeline D4–D17); monthly C-703/25 P
   reminder issue (D18); branch protection (R9).
1. **trailpack-memo standalone** (no core) — fastest user value; forces the
   shared UI building blocks (settings window, help window, tree with
   drag & drop, markdown view, dialogs, import/export, i18n).
2. **Data pipeline, rest** (no server, see `data-pipeline.md`) — served data
   files (items, achievements, waypoints, prices, curated events from TOML),
   `meta.json`, Pages + release-asset delivery (D19).
3. **trailpack-core** — cache, data set registry, cross-addon protocol,
   API key (DPAPI), dependency hints.
4. **memo ↔ core** — `/e` autocomplete, live event chips.
5. **trailpack-event** — timeline, custom events, multiple tables, memo
   highlighting.
6. *(later)* **Backend server** — only when a feature needs dynamic queries
   (price history); seeded from the archive (`backend.md`, BL5).

## Open (round 17 follow-ups)

- Versioning/changelog tool: changesets workflow via **knope** (Rust-native,
  changeset files, per-package versions, tags `<package>/vX.Y.Z`, GitHub
  releases) — proposed, awaiting answer.
- Branch name enforcement + `hotfix/` branches — proposed, awaiting answer.
- Nexus update provider in a monorepo with several addons (GitHub provider
  may only look at the repo's *latest* release) — research in phase 0.

## Plans

- [Data pipeline](data-pipeline.md) (current)
- [Backend](backend.md) (deferred)
- [trailpack-core](addons/trailpack-core.md)
- [trailpack-memo](addons/trailpack-memo.md)
- [trailpack-event](addons/trailpack-event.md)
