# Next session — handoff

Last updated: 2026-11-06 (session 18). This file holds only the current state and
the next steps. Keep it under 100 lines: decisions go into `docs/plans/`, lasting
facts into the skills, and the history stays in git.

## Where we are

- **Phase 0 + 0b complete.** The daily archive runs, sometimes hours late.
- **Phase 1 (trailkit-memo standalone):** merged into `develop` and tested in game:
  storage, tree and note view, own editor with preview, chat-link chips, reset kinds,
  drag & drop, search, trash & undo, and the UX round with the window menu (M157-M171).
- Decisions: `docs/plans/addons/trailkit-memo.md`, `docs/plans/ROADMAP.md`. Current
  look: [`current-design.md`](../../.agents/skills/trailkit-ui/references/current-design.md).
  Snapshots and testing: `tokens.md` §10. In-game facts: `nexus-addon-dev` skill §8.

## ▶ Resume here

Next ids: **M172+, R87+, questions from 129.** Work on a new branch from `develop`.

1. **Splits (R77), each a small own step:** Q91 `note_list.rs` + `draw_group`, Q95
   `draw_dropdown`, Q96 `editor/view.rs`, Q105 `markdown_view.rs`. All answered yes.
   Q92 `Text::get`: the recommendation is no, the user has not objected. `Editor::draw`
   (101 code lines) is a hint not yet put to the user: propose it with Q96.
2. **Then the M28 order continues:** settings window (a row in the window menu),
   export/import, backups, help page, release preparation (full test, screenshots M67,
   docs, measure the release DLL size by hand, R72).
3. **Its own grill round, the biggest change:** an editor that hides markdown outside
   the active line ("row 4" of the UX round).
4. Optional, offer once: Light contrast of `reset.daily` on `bg.pressed` is 4.16:1 while
   a tinted dropdown is held (see R85's test scope).

**Agent's choices, disclosed to the user and easy to turn back:** Ctrl+F does nothing in
the editor (M167); the menu marks the area shown as selected (M166); the Hide row is also
offered in the trash, as in the approved mockup.

**Leftover:** the merged remote branch `feature/memo-ux-round` still exists, because
`git push --delete` fails in the pre-push hook while `develop` is checked out. The user
can delete it on GitHub. The hook could also let branch deletions through.

## Open with the user

- **Flipping addon, sales volumes:** DataWars2 (api.datawars2.ie, Hetzner DE, polls every
  4 min) and GW2BLTC differ by factor 3-10; one hour of own 1-minute collection matched
  Mithril roughly, not ecto or mystic coin. Silver (DataWars2) answered 2026-11-01: free
  use and archiving allowed, attribution up to us, nginx may log IPs (BL9). He confirmed
  the `calculate_sold` branch looks wrong (the untested `-total > cutoff` case overstates
  `listed`), and the user sent him the fix. GW2BLTC e-mail drafted, no answer known. A
  24 h own collection needs an always-on machine (BL5). PieOrCake also has a `flip_out`
  addon, not looked at.
- **Nexus hover cursor:** the user may post the drafted Discord message.

## Recurring checks

- **GW2 wiki** (2026-10-31): still behind Cognito login, the Wayback workaround stays.
- **C-703/25 P** (checked 2026-11-06: no judgment, opinion or hearing found, sources only
  from 2025; check again next session): pending, DPF in force. Microsoft intervened for
  the Commission; noyb asked for repeal (2026-06-30) but has not sued yet.
- **DPF reminder workflow:** the 2026-11-01 08:00 cron was silently skipped by GitHub and
  triggered by hand (issue #11). If it skips again on 2026-12-01, grill moving the cron off
  the full hour.
- **Daily archive:** the WvW EU snapshot of Fri 2026-11-06 17:23 UTC is the real test of the
  cron delay margin (D29). Not yet checked after that time.

## User to-dos

- Add and verify `tobiacodes@gmail.com` on GitHub (Settings → Emails).
- Verify "trailkit" in TMview and DPMAregister.
- Google Alert for C-703/25 (D18).
- Optional: add `. "$HOME/.cargo/env"` to `~/.bashrc`.
