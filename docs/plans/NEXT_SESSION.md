# Next session — handoff

Last updated: 2026-11-08 (session 20). Current state and
next steps only; decisions belong in plans, lasting facts in skills. Keep under 100 lines.

## Where we are

- Phase 0 + 0b complete; daily archives sometimes run hours late. Memo standalone: storage, tree, editor/preview, chips, resets, drag & drop, search,
  trash/undo and UX/keyboard improvements through M178 are implemented and tested in game.
- Decisions: `addons/trailkit-memo.md`, `ROADMAP.md`. Appearance: trailkit-ui `current-design.md`; review: tokens.md §10.

## ▶ Resume here

**Session 20 — logging (R92, Q191–Q208) on `feature/trailkit-log`, not pushed:** crate `trailkit-log`
(one `trailkit/logs/<addon>.log`, 30 days, 1 MiB cap, panic hook, event catalog test), `TrailkitDirectory`/
`AddonName`/`FileName` in `trailkit-storage`, memo converted, help en/de, skill `trailkit-logs`. Deployed and
seen working in game (session start/end on hot reload, load times). Waiting for: user's in-game test and merge
OK; help wording OK; Q209 (split `addon.rs`, now 448 code lines, hint 400). Not in R92 scope yet: GW2 build and
Nexus version in `session_start` (not exposed without MumbleLink; with core, C12). Then M180 below; Q166 becomes
the save-error event fields on top of R92; Q167 settled, Q168 amended.

**Mobile, to review the save-error design (M180):** https://claude.ai/artifact/DyA5k7uYVKGhuTARYTzyJ7
— a visual mockup (R87, still unapproved) with the error toast, busy dialog and `danger.subtle`. Decide there:
Q151, Q164, Q165, Q166, Q167–Q171 (listed below), then the visual OK.

Next ids: **M181+, R93+, questions from 210.** Order: save errors (review item 1), then R89 (help `?`).
**Everything decided about save errors is in memo M180** (toasts, retry, dialog; mockup
`docs/plans/addons/memo-save-error-toast-design.html`, same as the artifact
https://claude.ai/artifact/DyA5k7uYVKGhuTARYTzyJ7). **Open, ask first (recommendation yes unless noted):**
Q151 restart a dead worker with "Erneut versuchen"; Q164 close button on the error toast;
Q165 shorten the text to "Speichern fehlgeschlagen" (not wrap); Q166 log format;
Q167–Q171 (R89/M179): own log per addon, export button, help window, problem link, `?` placement.
Then **visual approval of the mockup (R87)** (`danger.subtle`, busy/status-line dialog, error toast); then implement.

Implementation outline: typed worker → UI result channel; retry state (3 writes, 5 s, parked); immediate write for
"Übernehmen"; toast error kind, counter, spinner, close; dialog busy button + status line; `danger.subtle` via
`cargo xtask tokens` + contrast tests; en/de texts, help, READMEs; tests (no endless retry, no content in logs).

- **PR #96 is stuck** (B22 from another Claude session, 2026-11-07, auto-merge armed, no CI run,
  merge state unknown; touches this file → likely conflicts). B22: the backend must exist **before the
  first addon release** so an addon version can be locked until the user updates; design (endpoint, GDPR,
  offline, lock UX, R49) needs its own grilling round. Ask the user: rebase #96 onto develop or redo it.
  This changes the backend from "deferred" to a release prerequisite (affects R90 skeleton, B10, B19).
- **Housekeeping (ask first):** remote branches of merged PRs were not deleted (`agent-guidance-followups`,
  `backend-service-skeleton`, `xtask-token-split-and-docs`, `contributor-docs` after #98) plus obsolete
  `bugfix/ui-theme-in-game-fixes` (Q138); local: `chore/agent-context-efficiency` and the merged branches.
  `develop` locally is stale: `check-file-length` compares against it, so pull first. Keep
  `feature/memo-save-error-design` until M180 ships. Q141 keeps `token_groups.rs` (419 lines) together.

## Review improvements — one at a time, agreed order, solutions still proposals

1. **Storage failures (highest priority):** designed in memo M180 (`storage_worker.rs` drops failed
   writes today; the UI gets no result). Not implemented.
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
