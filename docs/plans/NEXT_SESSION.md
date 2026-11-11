# Next session — handoff

Last updated: 2026-11-11 (session 28). Current state and
next steps only; decisions belong in plans, lasting facts in skills. Keep under 100 lines.

## Where we are

- Phase 0 + 0b complete; daily archives sometimes run hours late. Memo standalone: storage, tree, editor/preview, chips, resets, drag & drop, search,
  trash/undo and UX/keyboard improvements through M178 are implemented and tested in game.
- Decisions: `addons/trailkit-memo.md`, `ROADMAP.md`. Appearance: trailkit-ui `current-design.md`; review: tokens.md §10.

## ▶ Resume here

**Session 28 (2026-11-11):** R96 shared text styles and components are done on `chore/shared-ui-components`:
`trailkit-ui` `text` (`TextStyle`, `draw_text`, `draw_text_wrapped`, `draw_truncated_text`, `draw_icon`), `empty_state`,
`StyleScope::transparent_frame`, `font.size.*` factors, `FontRole::Small`; memo and markdown view converted (Q275–Q286).
PR #109 into `develop` is pushed with auto-merge armed (user OK). **First step next session:** if #109 is not merged,
it waits for the snapshots: on the user's PC run `cargo xtask ui-test --update`, review only the changed images
(Q280 trash preview titles, Q283 b empty states, Q282 header tooltip; everything else must stay identical, R82),
commit them on the branch, push; then `cargo xwin clippy`, deploy and a short in-game check. After the merge delete
the branch. Q286 (tooltip looks like a button, 2–3 variants for all addons) stays a later design round.

**Next (once #109 is merged): start `chore/privacy-and-issue-form` from freshly fetched `origin/develop`.**

**Queue, in this order (R95, R96, Q274):**
1. **`chore/privacy-and-issue-form`:** `PRIVACY.md` + `PRIVACY.de.md`, issue form, `CONTRIBUTING.md` (attachments),
   `trailkit-logs` skill (untrusted data, export format).
2. **`feature/help-problem-page`** (an empty local branch exists; restart it from `origin/develop`): help window,
   `draw_list_row`, `draw_status_line`, Small, export, `?`, help rewrite (M181). Icons possibly missing from
   `icons.rs`: circle-help, download, external-link, circle-check, circle-alert, info (`cargo xtask subset-fonts`).
3. Then B23/B24: tech-stack grilling, `trailkit-i18n` for shared lock texts, icon `wrench`; server stays local (Q223).

Next ids: **M182+, R97+, B25+, questions from 287.** Deferred: GW2 build/Nexus version in `session_start` (C12).

- **Housekeeping (Q211/Q240):** the user deletes the approved old local branches by hand (`contributor-docs`,
  `xtask-token-split-and-docs`, `handoff-session-20`, `addon-status-design`, `addon-status-lock-design`, all `chore/`).

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
