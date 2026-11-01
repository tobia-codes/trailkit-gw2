# Next session — handoff

Last updated: 2026-11-01 (session 4).

## Where we are

- **Phase 0 (foundation) and phase 0b (daily archive) complete.** First
  scheduled run on 2026-11-01: release `archive-2026-11`, all 4 data sets
  `complete`. Repo public: <https://github.com/tobia-codes/trailkit-gw2>,
  `develop` is the default branch (R67); work on `feature/`/`chore/`
  branches → PR to `develop`; every push / merge needs explicit approval;
  no AI attribution (R68).
- **Phase 1 (trailkit-memo standalone) planned** — session 4 settled M16–M28
  in `docs/plans/addons/trailkit-memo.md` (storage, data model, markdown
  renderer in `trailkit-ui`, layout, drag & drop, character scope only with
  core, phase-1 settings, implementation order) and the UX rule
  `lay-responsive` (all addons: content adapts to the window, toolbars wrap,
  single-line items truncate with "…" + tooltip).
- Open PRs: #6 `chore/rustup-toolchain-install` (explicit toolchain install
  in CI + archive workflow). The decisions of session 4 are on
  `chore/phase-1-decisions`.

## ▶ Resume here

1. **Cron delays:** the 00:17 UTC archive run of 2026-11-01 started at
   06:14 UTC (~6 h late, GitHub load); the 06:17 catch-up had not run by
   07:30. Keep watching `gh run list --workflow daily-archive.yml`, especially
   the first **WvW EU snapshot Fri 2026-11-06 17:23 UTC** (D29 only has
   ~35 min margin). If it is late, run a grill round on mitigation
   (earlier/duplicate cron, external `workflow_dispatch` trigger — GDPR!).
2. Check the DPF reminder issue of 2026-11-01 (08:00 UTC) appeared.
3. **Phase 1, step 1 (M28): storage & data model** on `feature/memo-storage`
   — `notes.json`/`settings.json` per M16–M18, M23–M25, R22 (migration,
   newer-version read-only, atomic writes on the worker thread). Grill the
   step's open details first, then implement step by step through M28.
4. Next release PR `develop` → `main` will show "out of date" because `main`
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

## Recurring checks

- GW2 wiki (2026-10-31): still behind Cognito login → Wayback workaround stays.
- C-703/25 P (2026-11-01, start of phase 1): still pending, no hearing date
  or judgment reported; DPF in force. Microsoft intervened for the
  Commission; noyb asked for repeal (2026-06-30) but has not sued yet.

## Environment notes

- GW2 addons folder: `C:\Guild Wars 2\addons` (WSL: `/mnt/c/Guild Wars 2/addons`).
- Node 24 via fnm, pnpm 11.17.0 already installed.
- `gh` is logged in as `tobia-codes` (ssh).
