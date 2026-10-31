# AGENTS.md

Shared instructions for every coding agent working in this repository
(Claude Code reads this via `CLAUDE.md`, Codex reads it directly).

## Project

`trailpack-gw2` is a monorepo for Guild Wars 2 tooling:

- **Raidcore Nexus addons** (the main focus) — DLLs loaded by the
  [Raidcore Nexus](https://raidcore.gg/Nexus) addon loader and published to the
  Nexus addon library.
- **Data pipeline** — a Rust CLI run by a daily scheduled job that publishes
  static data files and archives prices (see `docs/plans/data-pipeline.md`).
  A Rust backend server is planned for later (`docs/plans/backend.md`, deferred).

Planning documents live in [`docs/plans/`](docs/plans/README.md).

## Hard rules

1. **Never push without explicit approval.** Do not run `git push` (or any
   command that publishes commits, tags or releases to a remote) unless the
   user has approved that specific push in the current conversation. Approval
   for one push does not carry over to the next. Commit only when asked.
   *Exception:* the scheduled data-pipeline CI job may push the `gh-pages`
   branch and create price-archive releases on its own
   (`docs/plans/data-pipeline.md` D9); changing that workflow still needs
   approval.
2. **Decisions go through the grill-me skill.** Whenever a decision is needed
   (architecture, scope, libraries, naming, UX, data model, …) that is not
   already settled in `docs/plans/` or by the user, run the `grilling` skill
   (`.agents/skills/grilling`, entry point `grill-me`): ask numbered questions
   in rounds with a recommended answer each, until the decision tree is fully
   resolved. Do not silently assume. Record settled decisions in the relevant
   plan document.
3. **Every project has a README.** Every crate, addon, app or package directory
   gets a `README.md` describing what it is, how to build it and how to use it.
   Keep it up to date when behavior changes.
4. **Every function is documented.** In Rust use rustdoc comments (`///` on
   items, `//!` for modules/crates) on *every* function, method, struct, enum,
   trait and module — public and private — describing what it does, and, where
   relevant, its errors (`# Errors`), panics (`# Panics`) and safety
   requirements (`# Safety`). Other languages use their equivalent
   (JSDoc/TSDoc for TypeScript, docstrings for Python).
5. **English only** for code comments, doc comments, READMEs, plans, commit
   messages and identifiers. (Conversation with the user may be in German.)
   User-facing addon strings are localized (English + German), never
   hard-coded.
6. **Help pages stay current.** Every addon has an in-game help page
   (`addons/<name>/help/{en,de}.md`). Any change to user-visible behavior
   updates the help page in the same change.
7. **Avoid code duplication — but extract only on real reuse.** Before writing
   a helper, component or type, search the workspace (`crates/` and other
   addons) for an existing one. Code moves into a shared crate as soon as a
   **second** place needs it, not speculatively. When you extract, replace
   all copies in the same change. UI components (popover, dialogs, buttons…)
   always come from `crates/trailpack-ui` once shared, so every addon looks
   identical.
8. **Type everything.** Prefer newtypes (`struct NoteId(Uuid)`), enums over
   strings/booleans, `serde`-derived structs over untyped JSON
   (`serde_json::Value` only at boundaries), compile-time checked SQL
   (`sqlx::query!`). Invalid states should be unrepresentable.
9. **Performance and security are requirements, not polish.** Follow the
   threading rules in the `nexus-addon-dev` skill; treat API keys and user
   data as secrets (see the same skill, §6).
10. **GDPR (DSGVO) compliance is mandatory.** Any service, provider, log,
    backup, CDN or website must be GDPR-compliant. Collect as little personal
    data as possible (IP addresses count as personal data), prefer EU
    providers with a data processing agreement (AVV/DPA), no tracking or
    analytics without consent. Flag any GDPR impact of a change to the user.

## Recurring checks

- **GW2 wiki availability:** the wiki is behind a login since 2026-10-31, so
  event schedules come from the Wayback Machine. Re-check regularly — see
  [`data/events/README.md`](data/events/README.md) for the rule.

- **EU-US Data Privacy Framework:** core's data downloads from GitHub Pages
  rely on it (`docs/plans/data-pipeline.md`, GDPR assessment). At the start
  of every session that touches planning, hosting or core — and at least at
  the start of every phase — check the status of CJEU appeal **C-703/25 P
  (Latombe v Commission)**. If the framework is annulled or suspended, tell
  the user immediately and plan the move to an EU host (D11 base-URL switch).

## Development environment

- Development happens in **WSL** (Linux). Addons are cross-compiled with
  `cargo-xwin` to `x86_64-pc-windows-msvc` and copied into the Windows GW2
  `addons` folder by a deploy script; CI additionally builds on a Windows
  runner.

## Architecture decisions

Fundamental design decisions are recorded so any agent can pick them up:

- Addon runtime, threading, inter-addon protocol, i18n, help pages, API key
  handling → `.agents/skills/nexus-addon-dev/SKILL.md`
- Theme, design tokens, shared components, UX rules →
  `.agents/skills/trailpack-ui/SKILL.md`
- Per-component decisions → the "Decisions" table in `docs/plans/**`

When a new fundamental decision is made, add it to the matching place above
in the same change.

## Skills

Skills live in [`.agents/skills/`](.agents/skills) (read by Codex);
`.claude/skills` is a symlink to the same directory (read by Claude Code).
Add new skills only under `.agents/skills/`.

| Skill | Use for |
|-------|---------|
| `grill-me` / `grilling` | All decisions and plan refinement (mandatory, see rule 2) |
| `rust-best-practices` | Writing/reviewing Rust (Apollo handbook, compact) |
| `rust-skills` | Detailed Rust rule lookup (265 rules incl. unsafe/FFI, async, API design) |
| `nexus-addon-dev` | **Project skill:** Nexus API facts, threading/performance, inter-addon protocol, i18n, help pages, API keys |
| `trailpack-ui` | **Project skill:** design system, themes, shared components, full UX rule set for in-game ImGui overlays (`references/ux-rules.md`) — the only UI/UX authority |

Third-party skills keep their upstream `LICENSE` file; see
[`.agents/skills/README.md`](.agents/skills/README.md) for sources.

## Rust conventions

- Cargo workspace at the repo root; lints configured once in the workspace
  `Cargo.toml` (`missing_docs`, `clippy::missing_docs_in_private_items`,
  `clippy::undocumented_unsafe_blocks`) so rule 4 is enforced by the compiler.
- Before considering work done: `cargo fmt --all`, `cargo clippy --all-targets
  -- -D warnings`, `cargo test`.
