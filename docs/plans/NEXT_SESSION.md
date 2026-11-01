# Next session — handoff

Last updated: 2026-11-01 (session 6: drawer scrim fix PR #33, faster dev loop R70).

## Where we are

- **Phase 0 + 0b complete.** Daily archive runs (sometimes hours late).
- **Phase 1 (trailkit-memo standalone):** steps 1 (storage), 2 (tree & note
  view), 3 (editing, M51–M57) and 3b (own word-wrapping editor with preview,
  M58–M64) are merged into `develop` and tested in game by the user.
  Decisions M16–M65 in `docs/plans/addons/trailkit-memo.md`.
- Order from here (M28 as amended by M58, M62, M65):
  ~~visual & interaction tests (M65)~~ (done, PR #21) → ~~font subsetting
  (M62)~~ (done, PR #25: release DLL 2.1 MB) → 3c chat-link chips (M61, own name lookup, no Decoder Ring) → reset kinds
  (M13) → drag & drop → search → trash & undo → settings window → export/import
  → backups → help page → **release preparation** (full test, screenshots
  M67, docs).
- Late session 5 (PRs #26–#30): bold/italic/link toggle off from anywhere
  inside the span (M68, M71, parsed with `markdown::inline_ranges`), toolbar
  buttons light up for the formatting at the cursor (M69), ≡ is gold while the
  note list is visible (M71), roomier unsaved-changes dialog with a border,
  right-aligned primary and stacked buttons in narrow windows (M70).
  Snapshot tests are strict now (no differing pixels; CI renders identically).
- Session 5 also: Open Sans replaces Noto Sans (M62), panes with padding
  (`lay-pane-padding`), drawer + menu button (M48), titles flush left with
  right-aligned badges (M49), bold groups with note count (M50), in-window
  dialogs (`overlay-dialog-scope`), Escape via Nexus registration.

## ▶ Resume here

1. **Grill round for step 3c (chat-link chips, M61).** Key question: memo
   works without trailkit-core in phase 1 (M1) and has no network access, but
   names (waypoint, item, …) need data. Likely phase-1 scope: decode
   `[&…]` locally (type + id), show a chip with type icon + label, click
   copies the code, right-click "Open in wiki"?; names later via core / our
   data pipeline (own lookup — the user does not want Decoder Ring).
   Magpie Memo (PieOrCake, MIT) is a good reference for chips as atomic
   editor cells (`src/ChipTextEdit.*`).
2. Then reset kinds (M13).
3. UI tests: `cargo xtask ui-test [--update]`; look at the PNGs in
   `tests/snapshots/` (and `failures/`) before deploying UI changes.
4. Token economy (user asked): keep tool output short (`| tail`/`grep`, also
   for `git push` — the pre-push hook prints all tests), look at snapshot
   PNGs only when they changed, avoid re-reading files.

## Nexus / ImGui facts learned in session 5 (also in the nexus-addon-dev skill)

- Nexus consumes **Escape** before ImGui → always `GUI_RegisterCloseOnEscape`.
- Nexus routes **key presses** (arrows, Backspace, Enter) to ImGui only while
  a real ImGui text input is active (`WantTextInput`); custom widgets keep an
  invisible input active ("keyboard sink" in `trailkit_ui::editor::view`).
- `IsWindowHovered` is false while any item is active → use
  `ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM`.
- ImGui 1.80: alpha 0 windows skip all items; `CalcTextSize` rounds up per
  call (measure glyphs with `ImFont_CalcTextSizeA`); no `BeginDisabled`;
  dragging where no item is moves the window (`movable(false)` on children);
  child backgrounds render into the parent's draw list, so a `ChildBg` scrim
  sits below earlier siblings — draw the dimming rectangle in the scrim's own
  draw list (PR #33, trailkit-ui skill §4).
- Nexus keeps fonts per identifier until the game restarts (font changes
  need a restart, not a hot reload).
- The game's hover cursor (gear, speech bubble) leaks through addon windows —
  Nexus issue; Discord message drafted for the user.

## Open with the user

- Sales volumes for a future flipping addon: DataWars2 (api.datawars2.ie,
  Hetzner DE, polls every 4 min) vs GW2BLTC differ by factor 3–10; one hour
  of own 1-minute collection matched Mithril roughly, not ecto/mystic coin.
  Silver (DataWars2) answered 2026-11-01: free use and archiving allowed,
  attribution up to us, nginx may log IPs (see BACKLOG BL9). He confirmed the
  `calculate_sold` branch looks wrong (untested `-total > cutoff` case
  overstates `listed`); the user sent him the fix (use the `else` formula). GW2BLTC e-mail
  drafted, no answer known.
  A 24 h own collection needs an always-on machine (backend, BL5).
  PieOrCake also has a `flip_out` addon (not looked at).
- Nexus hover-cursor issue: user may post the drafted Discord message.

## Recurring checks

- GW2 wiki (2026-10-31): still behind Cognito login → Wayback workaround stays.
- C-703/25 P (checked 2026-11-01, start of phase 1): still pending, no
  hearing date or judgment reported; DPF in force. Microsoft intervened for the
  Commission; noyb asked for repeal (2026-06-30) but has not sued yet.
- DPF reminder workflow: the 2026-11-01 08:00 cron was silently skipped by
  GitHub; triggered manually (issue #11). If it skips again on 2026-12-01,
  grill round on moving the cron off the full hour.
- Daily archive: WvW EU snapshot Fri 2026-11-06 17:23 UTC is the real test of
  the cron delay margin (D29).

## User to-dos

- Add + verify `tobiacodes@gmail.com` on GitHub (Settings → Emails).
- Verify "trailkit" in TMview + DPMAregister.
- Google Alert for C-703/25 (D18).
- Optional: add `. "$HOME/.cargo/env"` to `~/.bashrc`.

## Environment notes

- GW2 addons folder: `C:\Guild Wars 2\addons` (WSL: `/mnt/c/Guild Wars 2/addons`);
  Nexus log: `addons/Nexus/Nexus.log` (readable from WSL for diagnostics).
- `cargo xtask deploy trailkit-memo` hot-reloads the addon in a running game.
- Node 24 via fnm, pnpm 11.17.0 already installed.
- `gh` is logged in as `tobia-codes` (ssh).
