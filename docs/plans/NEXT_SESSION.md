# Next session — handoff

Last updated: 2026-11-12 (session 29, after PR #110). Current state and
next steps only; decisions belong in plans, lasting facts in skills. Keep under 100 lines.

## Where we are

- Phase 0 + 0b complete; daily archives sometimes run hours late. Memo standalone: storage, tree, editor/preview, chips, resets, drag & drop, search,
  trash/undo and UX/keyboard improvements through M178 are implemented and tested in game.
- Decisions: `addons/trailkit-memo.md`, `ROADMAP.md`. Appearance: trailkit-ui `current-design.md`; review: tokens.md §10.

## ▶ Resume here

**Session 29 (2026-11-11/12):** PR #109 (R96 shared text styles and components) and PR #110
(`chore/privacy-and-issue-form`: R98, R99, `PRIVACY.md`/`.de.md`, issue form, CONTRIBUTING, `trailkit-logs` skill)
are merged into `develop`. R98/B25: in-game problem reports through the backend, switched by a backend feature
flag, with the R95 GitHub form as fallback; open points Q288–Q293 in `backend.md`.
Q286 (tooltip looks like a button, 2–3 variants for all addons) stays a later design round.
Before the first release: confirm GitHub's DPF listing on `dataprivacyframework.gov` (R99 check).

**In progress: `feature/help-problem-page`** (local, from `origin/develop`, not pushed). Done and committed: `trailkit-log::export`
(`build_log_export`, `export_file_name`, `has_logs`; newest 3 days, 5 MiB, header with trims, tests, README) and
`trailkit-ui::help_topics` (`parse_help_topics`, `subsection_counts`). Also PR #112 (component snapshots) was opened with auto-merge.
**The approved mockup v3 (R95) could not be read** in this account (artifact 404); build from the R95 text and let the user review
the snapshots, or ask them to re-share the mockup as an image. Remaining steps, in order:
1. `draw_status_line` and `draw_list_row` in `trailkit-ui` (R96; list row for note list, trash and help topic list, drag/indent/arrow
   stay in the slice; existing snapshots stay identical) + component snapshots.
2. Help window in `trailkit-ui` (title `<window title> · Help`, tabs "Report a problem" start page and "Help", problem page with
   numbered steps, Small texts, status line states none/success/failure/no logs, topic list 112 px at minimum width) + snapshots
   (Dark/Light, minimum width, German).
3. memo: title bar `?` (Q171), Nexus "Open help", export action on a worker thread (read logs, write to Downloads with
   `export_file_name`, open Explorer with the file selected), GitHub issue link with addon and version, `has_logs` check.
4. Help rewrite M181 (nine topics in `help/{en,de}.md`, privacy topic embedded from `PRIVACY.md`/`.de.md`, parity test with
   `subsection_counts`), icons in `icons.rs` (circle-help, download, external-link, circle-check, circle-alert, info) + `cargo xtask subset-fonts`.
5. In-game test, merge OK (R74), PR into `develop`.

**Queue after that:** B23/B24/B25: tech-stack grilling, Q288–Q293, `trailkit-i18n` for shared lock texts, icon `wrench`; server stays local (Q223).

Next ids: **M182+, R100+, B26+, questions from 294.** Deferred: GW2 build/Nexus version in `session_start` (C12).

- **Housekeeping (Q211/Q240):** the user deletes the approved old local branches by hand (`contributor-docs`,
  `xtask-token-split-and-docs`, `handoff-session-20`, `addon-status-design`, `addon-status-lock-design`,
  `handoff-session-26`, `handoff-branch-hint`, `help-window-design`, all `chore/`; merged PR branches).

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
- **C-703/25 P:** Nov 11 CURIA check: still pending (case in progress); issues #11/#20 closed. Order of June 4, 2026 (ECLI:EU:C:2026:465).
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
