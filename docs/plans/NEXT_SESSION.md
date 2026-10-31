# Next session — handoff

Last updated: 2026-10-31 (end of session 3).

## Where we are

- **Phase 0 (foundation) complete** — workspace, xtask, trailkit-ui, memo
  skeleton with quick access icon + gameplay auto-hide, CI, git hooks
  (ROADMAP R28–R61).
- **Phase 0b (price archive) complete except the first scheduled run** —
  `tools/trailkit-datagen`, workflows `daily-archive.yml` and
  `dpf-reminder.yml`, first manual run on GitHub succeeded (release
  `archive-2026-10`, all 4 data sets complete). Decisions R62–R68,
  data-pipeline D27–D41, core C14, BACKLOG BL7.
- Repo is public: <https://github.com/tobia-codes/trailkit-gw2>.
  **`develop` is the default branch** (R67) → scheduled workflows run from
  `develop`. `main` = last release (PR #2). PRs #1–#4 merged.
- Branch protection: `main` PR-only + required checks (Windows job,
  cargo-deny, branch name), `develop` PR-only; both enforce admins, no force
  push, no deletion. (Re-create from R9 if it is ever lost.)
- Rules that changed in session 3:
  - Commit email `tobiacodes@gmail.com` (R62); history rewritten.
  - **No AI attribution** in commits or PRs (R68, AGENTS.md rule 1).
  - Every push / PR merge still needs the user's explicit approval; work
    happens on `feature/`/`chore/` branches → PR to `develop`.
- The user converses in German; repo text English.

## ▶ Resume here

1. **Check the first scheduled archive runs** (2026-11-01):
   `gh run list --workflow daily-archive.yml` — the 00:17 UTC run should have
   created release `archive-2026-11` with `2026-11-01.*` files; the 06:17 UTC
   catch-up should be green (it fails only if gaps remain). Look at
   `2026-11-01.manifest.json` (all data sets `complete`?). If something
   failed: read the run log, fix on a `bugfix/` branch.
   Also check that the first DPF reminder issue appears on the 1st (08:00 UTC).
2. Then **phase 0b is complete** → start **phase 1: trailkit-memo standalone**
   (no core). Begin with a grill round on what `docs/plans/addons/trailkit-memo.md`
   (M1–M15) leaves open for implementation — e.g. settings persistence format
   and location, settings window layout, help window (markdown rendering),
   note tree data model + drag & drop, storage format of notes, import/export,
   backups (M11), reset timers (M13, with `jiff`, R66). Phase 1 also builds the
   shared UI components in `crates/trailkit-ui` and the Nexus options tab (R53).
3. Next release PR `develop` → `main` will show "out of date" because `main`
   has the merge commit of PR #2: use "Update branch" (or merge `main` back
   into `develop` via a PR) before merging.

## Notes from session 3

- The first manual archive run started before the history rewrite; its tag
  `prices-2026-10` was moved to the rewritten commit `98427aa` (same tree),
  then the release was renamed to `archive-2026-10` (D41).
- `archive-2026-10` shows as "Latest" on GitHub although created with
  `--latest=false` — probably because it is the only release. Re-check once
  the first addon release exists (end of phase 1).
- Live archive run takes ~4.5 min; sizes per day: prices ~0.38 MB, order
  book ~1.9 MB, WvW ~25 KB, gem exchange < 1 KB.
- The pre-push hook rejects pushing `develop`/`main` directly (by design);
  the one-time bootstrap/rewrite pushes used `--no-verify` with clippy +
  tests run by hand.

## User to-dos

- Add + verify `tobiacodes@gmail.com` on GitHub (Settings → Emails), so the
  commits are linked to the profile.
- Verify "trailkit" in TMview + DPMAregister.
- Google Alert for C-703/25 (D18).
- Optional: add `. "$HOME/.cargo/env"` to `~/.bashrc` (rustup was installed
  with `--no-modify-path`).

## Recurring checks (2026-10-31)

- GW2 wiki: still behind Cognito login → Wayback workaround stays.
- C-703/25 P: pending, no hearing date, no AG opinion; DPF in force
  (checked again at the start of phase 0b).

## Environment notes

- GW2 addons folder: `C:\Guild Wars 2\addons` (WSL: `/mnt/c/Guild Wars 2/addons`).
- Node 24 via fnm, pnpm 11.17.0 already installed.
- `gh` is logged in as `tobia-codes` (ssh).
