# Next session — handoff

Last updated: 2026-11-10 (session 22). Current state and
next steps only; decisions belong in plans, lasting facts in skills. Keep under 100 lines.

## Where we are

- Phase 0 + 0b complete; daily archives sometimes run hours late. Memo standalone: storage, tree, editor/preview, chips, resets, drag & drop, search,
  trash/undo and UX/keyboard improvements through M178 are implemented and tested in game.
- Decisions: `addons/trailkit-memo.md`, `ROADMAP.md`. Appearance: trailkit-ui `current-design.md`; review: tokens.md §10.

## ▶ Resume here

**Session 22 (2026-11-10) — M180 save errors and R93 done; PR `feature/memo-save-errors` → `develop`** (auto-merge armed after
the user's merge OK; check it merged, fix CI if not). Tested in game: retries, give-up, manual retry, recovery
(`save_recovered`), dialog, disabled buttons. Release DLL 2,420,224 bytes vs. 2,344,960 before (+3.2 %, measured before R93).
New in `trailkit-ui`: error/success toast, `Button` with `ButtonKind` + `ButtonState` (`Ready`/`Busy`/`Disabled { reason }`,
busy/disabled are no ImGui buttons), `BusyState` (≥ 0.6 s), reason tooltip with neutral `ban`. R93: every failure case and
disabled control is designed. Splits done; `actions.rs` 413 waits for its rebuild (Q134). Remote branches of #99–#102 deleted.
**Next:** PR #96 (below), then R89 (help `?`, export, problem page; `PRIVACY.md`, Q170). Start a fresh session.

Next ids: **M181+, R94+, questions from 222.** R93 (failure cases, busy buttons) added and implemented 2026-11-10. Still deferred: GW2 build and Nexus version in `session_start` (need MumbleLink → core,
C12). If ever needed: limit repeated log events once core adds network paths.

- **PR #96 (B22, backend before the first release):** Q210 yes: after M180 is merged close it and redo B22 on a new branch from `origin/develop`
  (no force push); turn off its armed auto-merge first. B22's design needs its own grilling round (endpoint, GDPR, offline, lock UX, R49).
- **Housekeeping (Q211) done 2026-11-10:** remote branches of PRs before #99 deleted. Still there: remote branches of merged
  #99–#102 and local `chore/contributor-docs`, `chore/xtask-token-split-and-docs`, `chore/handoff-session-20` (old handoffs,
  unmerged commits) need the user's OK to delete; `feature/memo-save-error-design` may go once M180 is merged (Q141).

## Review improvements — one at a time, agreed order, solutions still proposals

1. **Storage failures:** done (memo M180, 2026-11-10).
2. **Pre-push protection:** `.githooks/pre-push` checks protected destination refs only
   for deletions; normal pushes check the current branch via `xtask/src/check_branch.rs`.
   Validate every destination ref. Server-side protection was not inspected in the review.
3. **WvW snapshot validity:** `tools/trailkit-datagen/src/wvw.rs::write_reset_snapshot`
   saves current matches without checking the intended pre-reset time/match period.
   A delayed job can succeed with post-reset data. Check the actual Nov 6 EU run as well.
4. **Search cache:** `addons/trailkit-memo/src/main_window/search.rs::update_search`
   calls `SearchQuery::parse` before checking the cache, allocating for unchanged queries
   every frame. Check unchanged inputs first. No performance benchmark was run.
5. **Docs/skills accuracy:** READMEs done (R91). Still wrong: `nexus-addon-dev/references/addon-structure.md`
   says Cargo forbids sibling-slice dependencies (it only rejects cycles; the rule is enforced by review);
   `rust-best-practices/references/chapter_09.md` marks `&mut T` as not Send (it is Send when T is Send).
6. **Performance contract:** reconcile the Nexus skill's blanket render-thread parsing
   ban with existing change-triggered Markdown parsing. Clarify allowed work and caching
   before proposing an architectural rewrite; keep actual performance/security guarantees.
7. **Decision/documentation workflow:** broad token-loading fixes are done (R88).
   Relaxing grilling for routine implementation choices or rustdoc for trivial getters
   was only suggested, not approved. Preserve current rules until explicitly settled.

## Earlier implementation queue — after the review work

- Approved splits (R77): Q91
  `note_list.rs`/`draw_group`, Q95 `draw_dropdown`, Q96 `editor/view.rs`, Q105 `markdown_view.rs`.
  Q92 `Text::get`: recommendation no, no objection recorded. Grill remaining hints for
  `Editor::draw`, `draw_lines` and the large main-window test file before splitting.
- Q134: split `actions.rs` into tree/editor actions at its next rebuild, not beforehand.
- Continue M28: settings/options → export/import → backups → help → release preparation
  (full tests, M67 screenshots, docs, manually measure release DLL size, R72).
- Separate grill round: hide Markdown outside the active editor line. Optional once:
  Light `reset.daily` on pressed tinted dropdown is 4.16:1 (R85 scope).

## Recurring checks and other open items

Triggers/consequences: [agent reference](../agent-reference.md#recurring-checks).
- **Wiki:** Nov 6 review got HTTP 403; public recovery/login status not confirmed; keep Wayback.
- **C-703/25 P:** Nov 6 CURIA check: pending, order dated June 4, 2026 (ECLI:EU:C:2026:465).
  Previous “sources only from 2025” note was stale. Recheck when triggered; source:
  https://juris.curia.europa.eu/juris/liste.jsf?language=en&num=C-703%2F25
- **WvW reset snapshots (checked 2026-11-07):** both Nov 6 EU (ran 22:12 UTC, +4.8 h) and Nov 7 NA (07:16 UTC,
  +5.9 h) assets hold the **post-reset** matches; CI was green. Confirms review item 3. Also Nov 6 14:13 catch-up failed on GemExchange.
- **DPF cron:** Nov 1 skipped, issue #11 opened manually. If Dec 1 skips, grill shifting off :00.
- **Flipping volumes (BL9):** DataWars2 vs GW2BLTC differs 3–10×; Silver allowed free use/
  archiving Nov 1 and confirmed suspicious `calculate_sold` branch; user sent the fix.
  GW2BLTC reply unknown; 24 h comparison needs an always-on machine (BL5). `flip_out` unreviewed.
- Nexus hover cursor: drafted Discord message remains with the user.
- User to-dos: verify public GitHub email; TMview/DPMA name check; C-703/25 Google Alert.
  Optional: cargo env in bashrc; GitHub “Automatically delete head branches”.
