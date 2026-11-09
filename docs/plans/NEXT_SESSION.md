# Next session — handoff

Last updated: 2026-11-09 (session 21). Current state and
next steps only; decisions belong in plans, lasting facts in skills. Keep under 100 lines.

## Where we are

- Phase 0 + 0b complete; daily archives sometimes run hours late. Memo standalone: storage, tree, editor/preview, chips, resets, drag & drop, search,
  trash/undo and UX/keyboard improvements through M178 are implemented and tested in game.
- Decisions: `addons/trailkit-memo.md`, `ROADMAP.md`. Appearance: trailkit-ui `current-design.md`; review: tokens.md §10.

## ▶ Resume here

**Session 21 (2026-11-09) — logs as day files (R92, Q213–Q221), PR `feature/log-daily-files`** (auto-merge armed after the
user's merge OK; check it merged, fix CI if not). `trailkit/logs/<addon>/<addon>-<yyyymmdd>.log` plus `_1`–`_4`, 1 MiB each; newest
14 days, none older than 30; a normal session is three lines (`session_start`, one `load_finished`, `session_end`).
In game only start and file creation were seen; day change, parts and cleanup are unit-tested. User deletes old `trailkit/logs/memo.log`
and `TrailpackMemo/` by hand. Format: `crates/trailkit-log/README.md`.

**Next: implement M180 save errors.** Answered yes 2026-11-09: Q151, Q164, Q165, Q166 (amended by Q219), Q168–Q171, Q209–Q211.
Mockup V4 (`x` always top right, 8 px; https://claude.ai/artifact/DyA5k7uYVKGhuTARYTzyJ7) counts as visually approved (R87); use the
short title "Speichern fehlgeschlagen" (Q165), the mockup still shows the long one. M180 adds `save_gave_up` to `CRITICAL_EVENTS`.
Then R89 (help `?`, export, problem page; `PRIVACY.md` does not exist yet, Q170 needs it).

Next ids: **M181+, R93+, questions from 222.** Still deferred: GW2 build and Nexus version in `session_start` (need MumbleLink → core,
C12). Hints: `addon.rs` 416 code lines (limit 400), split only on request. If ever needed: limit repeated log events once core adds network paths.

Implementation outline: typed worker → UI result channel; retry state (3 writes, 5 s, parked); immediate write for
"Übernehmen"; toast error kind, counter, spinner, close; dialog busy button + status line; `danger.subtle` via
`cargo xtask tokens` + contrast tests; en/de texts, help, READMEs; tests (no endless retry, no content in logs).

- **PR #96 (B22, backend before the first release):** Q210 yes: after M180 close it and redo B22 on a new branch from `origin/develop`
  (no force push); turn off its armed auto-merge first. B22's design needs its own grilling round (endpoint, GDPR, offline, lock UX, R49).
- **Housekeeping (Q211 yes, not done yet):** delete remote branches of merged PRs (`agent-guidance-followups`, `backend-service-
  skeleton`, `xtask-token-split-and-docs`, `contributor-docs`), obsolete `bugfix/ui-theme-in-game-fixes` (Q138), local
  merged ones; pull stale local `develop` first. Keep `feature/memo-save-error-design` until M180 ships (Q141: keep `token_groups.rs`).

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
