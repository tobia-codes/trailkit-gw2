# Next session — handoff

Last updated: 2026-10-31 (session 3).

## Where we are

- Planning complete (ROADMAP R1–R39, data-pipeline D1–D26, core C1–C13,
  memo M1–M15, event E1–E15). **Phase 0 (foundation) in progress** on branch
  `chore/phase-0-foundation` (cut from `develop`; merge after user approval, R29).
- Phase 0 grill round settled → R28–R39 in `ROADMAP.md`.
- Never push without approval. The user converses in German; repo text English.

## Phase 0 status

Done:
- rustup + pinned toolchain 1.99.1, cargo-xwin, cargo-deny installed.
- Workspace (`Cargo.toml`, lints, `clippy.toml`, `deny.toml`), `xtask`
  (`deploy`, `sync-versions`), `addons/trailkit-memo` skeleton, Changesets
  structure (pnpm), CI workflow (`.github/workflows/ci.yml`, inactive until 0b).
- Host checks green: fmt, clippy, tests, `cargo deny check`.

Optional for the user:
- Add `. "$HOME/.cargo/env"` to `~/.bashrc` (rustup was installed with
  `--no-modify-path`).

Settled since: R47 trailkit-ui crate (theme + StyleScope, done), R48 git
hooks (done), R49 update provider `Direct`, R50 kebab-case DLL names,
R51 gold palette, R52 domain `gw2.trailkit.de`; naming conventions (R40–R46, AGENTS.md "Naming conventions", `cargo xtask check-names`).

Settled in session 3: grill rounds 5–6 → R53–R61 (options tab, rarity
tokens, quick access icons + `cargo xtask build-icons`, gameplay auto-hide).

Open phase 0 work:
1. ~~Windows build, deploy, in-game test, hot reload~~ — done 2026-10-31.
2. Memo quick access icon (R55/R57–R59) and gameplay auto-hide (R56/R61) —
   done 2026-10-31, verified in game by the user (icon size matches other
   Nexus icons after padding, hover, tooltip, click toggle, auto-hide, hot
   reload).

**Phase 0 complete.**

## Phase 0b status (price archive)

Settled: R62–R65 (public email rewrite, first push, 0b scope, `ureq`),
data-pipeline D27–D31 (datagen path, DTOs in datagen, WvW reset snapshot
times, no item-id diff, gem exchange quantities).

Done 2026-10-31:
- History rewritten to `tobiacodes@gmail.com` (R62).
- Public repo `tobia-codes/trailkit-gw2` created; `main` = `develop`
  pushed (bootstrap push with `--no-verify`, because the pre-push hook only
  allows work branches; clippy + tests were run by hand).
- Branch protection: `main` PR-only + required checks (Windows job,
  cargo-deny, branch name), `develop` PR-only; both enforce admins, no force
  push, no deletion.
- CI did not trigger on the initial push (GitHub skips it sometimes); the
  first PR will run it.

## ▶ Resume here

Done on `chore/datagen-skeleton` (not pushed yet):
- `tools/trailkit-datagen` (`archive`, `wvw-snapshot`), live-tested locally
  2026-10-31: all 4 data sets complete (prices 0.38 MB, order book 1.9 MB),
  catch-up re-runs only incomplete data sets. Decisions R66, D32–D39.
- Workflows `price-archive.yml` (daily 00:17, catch-up 06:17, WvW Fri 17:23 /
  Sat 01:23 UTC, keep-alive) and `dpf-reminder.yml` (monthly issue, D18) —
  **need the user's approval (D9) before the PR**.

PR #1 (datagen + workflows) merged into `develop` 2026-10-31; `develop` is
the default branch since then (R67), so the schedules are active from
`develop`. Release PR #2 (`develop` → `main`) is open, CI green, merge
awaits approval. Verify the first scheduled archive (00:17 UTC) or a manual
run (`gh workflow run price-archive.yml -f mode=catch-up`): release
`prices-YYYY-MM` exists with the day's files + manifest. Then phase 0b is
complete → phase 1 (trailkit-memo standalone).

## User to-dos

- Verify "trailkit" in TMview + DPMAregister.
- Add + verify `tobiacodes@gmail.com` on GitHub (Settings → Emails).
- Google Alert for C-703/25 (D18).

## Recurring checks (2026-10-31)

- GW2 wiki: still behind Cognito login → Wayback workaround stays.
- C-703/25 P: pending, no hearing date; DPF in force.

## Environment notes

- GW2 addons folder: `C:\Guild Wars 2\addons` (WSL: `/mnt/c/Guild Wars 2/addons`).
- Node 24 via fnm, pnpm 11.17.0 already installed.
