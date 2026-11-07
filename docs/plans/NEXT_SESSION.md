# Next session — handoff

Last updated: 2026-11-07 (session 19, branch integration). Current state and
next steps only; decisions belong in plans, lasting facts in skills. Keep under 100 lines.

## Where we are

- Phase 0 + 0b complete; daily archives sometimes run hours late. Memo standalone: storage, tree, editor/preview, chips, resets, drag & drop, search,
  trash/undo and UX/keyboard improvements through M178 are implemented and tested in game.
- Decisions: `addons/trailkit-memo.md`, `ROADMAP.md`. Appearance: trailkit-ui `current-design.md`; review: tokens.md §10.

## ▶ Resume here

Next ids: **M181+, R90+, questions from 172.** Order: save errors (review item 1), then R89 (help `?`).
**Everything decided about save errors is in memo M180** (toasts, retry, dialog; mockup
`docs/plans/addons/memo-save-error-toast-design.html`, same as the artifact
https://claude.ai/artifact/DyA5k7uYVKGhuTARYTzyJ7). **Open, ask first (recommendation yes unless noted):**
Q151 restart a dead worker with "Erneut versuchen"; Q164 close button on the error toast;
Q165 shorten the text to "Speichern fehlgeschlagen" (not wrap); Q166 log format;
Q167–Q171 (R89/M179): own log per addon, export button, help window, problem link, `?` placement.
Then: get **visual approval of the mockup (R87)**, including the new `danger.subtle` token, busy/status-line
dialog, error toast; only then implement (outline below). No runtime code, deploy or approval exists yet.

Implementation outline: typed worker → UI result channel; retry state (3 writes, 5 s, parked); immediate write for
"Übernehmen"; toast error kind, counter, spinner, close; dialog busy button + status line; `danger.subtle` via
`cargo xtask tokens` + contrast tests; en/de texts, help, READMEs; tests (no endless retry, no content in logs).

- Context cleanup merged (PR #92, R87/R88): read references narrowly; keep Claude's shared import.
  Q141 yes (token tables stay together, R77); Q142 yes (one memo changeset, R16).
- **Branch integration (session 19, 2026-11-07):** PR A #94 merged (docs). **PR B #95** (token split Q140, docs
  refresh, 3 commits) is open with auto-merge armed; check it merged. Dropped from `cd17a9b`: its stale handoff part.
  Then, each with the user's OK: delete local-only `chore/codex-handoff` (`git branch -D`, commits were
  cherry-picked) and the merged remote branches. Keep `feature/memo-save-error-design` until M180 ships.
  Disclosed: `token_groups.rs` has 419 code lines (hint 400, kept together by Q141); `editor/view.rs` hints remain.

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
5. **Docs/skills accuracy:** root and memo READMEs have stale progress summaries.
   `nexus-addon-dev/references/addon-structure.md` wrongly says Cargo forbids sibling-slice
   dependencies (it only rejects cycles). `rust-best-practices/references/chapter_09.md`
   wrongly marks `&mut T` as not Send; it is Send when T is Send. These remain unfixed.
6. **Performance contract:** reconcile the Nexus skill's blanket render-thread parsing
   ban with existing change-triggered Markdown parsing. Clarify allowed work and caching
   before proposing an architectural rewrite; keep actual performance/security guarantees.
7. **Decision/documentation workflow:** broad token-loading fixes are done (R88).
   Relaxing grilling for routine implementation choices or rustdoc for trivial getters
   was only suggested, not approved. Preserve current rules until explicitly settled.

Review evidence (Nov 6): format, host/Windows Clippy and 205 host tests passed;
normal note snapshots inspected in Dark/Light. No complete audit or UI-suite rerun.

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
- Disclosed choices: Ctrl+F does nothing in editor (M167); menu selects shown area (M166);
  Hide is also offered in trash. Merged remote branches may need manual deletion;
  `bugfix/ui-theme-in-game-fixes` is obsolete and awaits user deletion (Q138).

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
