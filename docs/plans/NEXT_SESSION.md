# Next session — handoff

Last updated: 2026-10-31. Delete or rewrite this file once the open round is
answered.

## Where we are

- Grilling (`grill-me` skill) is in progress. Rounds 1–11 are settled and
  recorded in the "Decisions" tables of:
  - [`ROADMAP.md`](ROADMAP.md) (R1–R8)
  - [`data-pipeline.md`](data-pipeline.md) (D1–D5) — **current**, replaces the server for now
  - [`backend.md`](backend.md) (B1–B21) — deferred target design
  - [`addons/trailkit-core.md`](addons/trailkit-core.md) (C1–C5)
  - [`addons/trailkit-memo.md`](addons/trailkit-memo.md) (M1–M6)
  - [`addons/trailkit-event.md`](addons/trailkit-event.md) (E1–E7)
  - [`BACKLOG.md`](BACKLOG.md) (BL1–BL4)
- Fundamental rules live in `AGENTS.md` (rules 1–10) and the project skills
  `nexus-addon-dev` and `trailkit-ui`.
- Nothing is committed yet (`git init` done, all files untracked). Never push
  without approval.
- The user converses in German; all repo text stays English.

## Design change 2026-10-31: no backend server for now

User decided: no Rust server yet; static data files + daily price archive
instead, free hosting. Recorded as D1–D5 in
[`data-pipeline.md`](data-pipeline.md); `backend.md` is marked deferred.

## Data pipeline: rounds 12a/12b settled (D1–D18 in `data-pipeline.md`)

User to-do: create the Google Alert for C-703/25 (D18).

Delivery settled: GitHub-only, large files as release assets (D19).

## Round 12 (memo) settled → M7–M15, overlay idea → BL6

## Round 13 (core) settled → C6–C13 (C8 form of the outdated message to confirm)

## Round 16 (UI skill) settled 2026-10-31

`ui-ux-pro-max` removed; adapted rules in
`.agents/skills/trailkit-ui/references/ux-rules.md`. Phase 0 to-do: research
GW2 UI + popular Nexus/arcdps addon conventions → fill section 8 of that file.

## Round 17 (roadmap) settled → R8–R13 in `ROADMAP.md`

R14 (branch-name check) + R15 (hotfix branches) settled. R16: Changesets chosen. R17 xtask version sync, R18 oxlint later. R19 cargo-deny in CI. Round 18 → R20–R26, D25–D26.
Project renamed trailpack → **trailkit** (R20), domain trailkit.de (R27).
User to-do: verify "trailkit" in TMview + DPMAregister, (trailkit.de is registered since
2026-10-31). **Planning complete → next: phase 0.** Local folder renamed by the user to
`/home/dev/projects/trailkit-gw2` (was `trailpack-gw2`).

## Environment notes

- Bash tool works again (verified 2026-10-31).
- Rust is not installed in WSL yet.
- GW2 addons folder: `C:\Guild Wars 2\addons` (WSL: `/mnt/c/Guild Wars 2/addons`).
  The deploy script reads it from a git-ignored local config / env var, never
  hard-coded.
- GW2 wiki was behind a login on 2026-10-31 → check again (see
  `data/events/README.md`).
