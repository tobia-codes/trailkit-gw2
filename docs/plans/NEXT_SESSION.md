# Next session — handoff

Last updated: 2026-11-01 (end of session 4).

## Where we are

- **Phase 0 + 0b complete.** First scheduled archive run 2026-11-01 OK
  (release `archive-2026-11`, all 4 data sets `complete`) — but ~6 h late.
- **Phase 1 (trailkit-memo standalone) in progress** — decisions M16–M46 in
  `docs/plans/addons/trailkit-memo.md`, implementation order M28 (11 steps).
  - **Step 1 (storage & data model): done**, PR #8 open
    (`feature/memo-storage`): new crate `crates/trailkit-storage`, note tree
    model, settings persistence, storage worker thread.
  - **Step 2 (tree & note view): done, not pushed** — branch
    `feature/memo-tree-view` (based on `feature/memo-storage`): shared fonts
    (Noto Sans) + Lucide icons + markdown parser/view in `trailkit-ui`,
    memo main window (two panes, tree, D/W·T/W badges, rename, empty state,
    `ui_state.json`).
  - **Neither step is tested in game yet.** Riskiest spots: badge/title
    position in note rows, inline rename input, font loading (glyph trick).
- **New working rule (R69, AGENTS.md rule 1):** agents push work branches and
  merge PRs into `develop` on their own once **all CI checks are green**.
  `main`, releases, tags, force pushes, CI-workflow / branch-protection
  changes: **always ask**. Committed on `chore/agent-merge-develop`
  (`03b77d2`), not pushed yet.
- Rules unchanged: no AI attribution (R68); conversation German, repo English.

## ▶ Resume here

1. **Permissions (user, before restart):**
   - `.claude/settings.local.json` — done (autoMode allow for develop merges).
   - `.claude/settings.json` — remove `"Bash(git push:*)"` and
     `"Bash(git push)"` from `ask` (still present at end of session 4).
   - Restart Claude Code. If a push/merge is still blocked by the auto-mode
     classifier, tell the user; do not work around it.
2. **Land the open work (no need to ask, R69):** for each, wait for green
   checks, then `gh pr merge --merge --delete-branch`:
   1. PR #8 `feature/memo-storage` → `develop`.
   2. `chore/agent-merge-develop` → push, PR, merge (include the
      `settings.json` change if the user made it in the working tree).
   3. `feature/memo-tree-view` → rebase/merge onto updated `develop`, push,
      PR, merge.
3. **In-game test with the user:** `cargo xtask deploy trailkit-memo`, then
   check: fonts (bold/italic/headings, umlauts, "…"), icons in the toolbar,
   new note/group + rename, badges, checkbox click, link tooltip, splitter,
   single-pane below ~440 px, empty state, restart keeps selection. Fix on
   `bugfix/` branches.
4. **Phase 1 step 3: editing & toolbar** (M3, M4 without events, "cancel
   with unsaved changes asks", M28). Start with a grill round (e.g. edit/view
   toggle placement, toolbar buttons, unsaved-changes dialog, `/e` deferred
   to core, autosave vs. explicit save).
5. **Checks to keep doing:**
   - Cron delays: watch `gh run list --workflow daily-archive.yml`; the
     06:17 catch-up of 2026-11-01 never ran; the **WvW EU snapshot Fri
     2026-11-06 17:23 UTC** is the real test (D29 margin ~35 min). If late →
     grill round on mitigation.
   - **DPF reminder issue did not appear** on 2026-11-01 (due 08:00 UTC, no
     run of `dpf-reminder.yml` by 09:50 UTC). Check `gh run list --workflow
     dpf-reminder.yml`; if it never ran, investigate (scheduled workflows on
     a new default branch? disabled?) and trigger it once via
     `workflow_dispatch` — changing the workflow needs approval.
6. Next release PR `develop` → `main` (needs approval!) will show "out of
   date" because `main` has the merge commit of PR #2: "Update branch" first.

## Session 4 notes

- `trailkit-storage`: atomic JSON saves, `schema_version` + step-wise
  migration, read-only on newer versions, unreadable files moved to
  `backups/` (max 3), migration copy (newest only, 30 days). memo reads
  `settings.json` synchronously in `load` (quick access icon needs it); notes
  and `ui_state.json` load on the worker.
- Nexus font atlas facts (verified in Nexus' source, now in the
  nexus-addon-dev skill): font data is copied; same identifier = shared;
  atlas = Latin + Latin-1 + Latin Ext-A/B + characters of all registered
  localization texts → icons and UI strings are registered as glyph texts.
  Emoji/CJK in notes render as `?` (accepted).
- DLL size grows by ~2.7 MB per addon (embedded fonts), accepted.
- New Nexus identifier kind `font` (AGENTS.md naming table).
- Lucide 1.66.0 icons; custom icons follow Lucide style (`lay-icons`);
  quick access bar icons stay colored (R59).

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
- C-703/25 P (checked 2026-11-01, start of phase 1): still pending, no
  hearing date or judgment reported; DPF in force. Microsoft intervened for the
  Commission; noyb asked for repeal (2026-06-30) but has not sued yet.

## Environment notes

- GW2 addons folder: `C:\Guild Wars 2\addons` (WSL: `/mnt/c/Guild Wars 2/addons`).
- Node 24 via fnm, pnpm 11.17.0 already installed.
- `gh` is logged in as `tobia-codes` (ssh).
