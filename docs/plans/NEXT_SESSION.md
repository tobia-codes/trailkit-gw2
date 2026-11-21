# Next session — handoff

Last updated: 2026-11-21 (phone session 54; **active work branches: `feature/trash-group-tree`** (M218, built, `child_order` OK given, waits for in-game test 38), **`feature/shared-search`** (BL30, BL31, M219 on top of it, Q587 A; built, waits for test 39 after 38) and **`feature/windows-cursor-prototype`** (R159 prototype on top of M218, waits for an in-game check; not for merging)). Current state and next steps only; decisions belong in plans, lasting facts in skills. Keep under 100 lines.

## Where we are

- Phase 0 + 0b complete; daily archives sometimes run hours late. Memo standalone: storage, tree, editor/preview, chips, resets, drag & drop, both searches,
  trash/undo, UX/keyboard improvements, help window, status cards and crash protection (R104) are implemented and tested in game (through M199).
- Decisions: `addons/trailkit-memo.md`, `ROADMAP.md`. Appearance: trailkit-ui `current-design.md`; review: tokens.md §10.

## ▶ Resume here

**First, if [`in-game-tests.md`](in-game-tests.md) lists tests, ask the user: can you test in game now?** Remove each item once
the user confirms it (user requests 2026-11-13, 2026-11-16); an empty list means no question. A failed test becomes a `bugfix/` branch.

**On the Mac or phone** ("…auf dem mac/handy auf"): no game, so skip that question and add every needed game check to
[`in-game-tests.md`](in-game-tests.md) in the same commit (2026-11-16). Phone: grilling asks **one question per message**.
**Cloud sessions:** auto mode may refuse `git commit`: ask the user, never work around it. PRs via `gh api .../pulls`, auto-merge
via `PUT .../pulls/{n}/ccr/auto_merge` (`merge_method=merge`); branch deletion is blocked (the user deletes). Push does not set
the upstream (narrow fetch refspec): set `branch.<b>.remote/merge` by hand for the stop hook. `cargo xwin clippy` works.

**On the Mac (no game, no WSL):** `git fetch origin && git switch develop && git pull --ff-only`; deploy, `ui-test` and in-game tests wait for the Windows PC; check `source ~/.cargo/env` and a first `cargo test` there; snapshots via the PR's Windows CI.

**Decided, not implemented** (M212, R140, mock https://claude.ai/artifact/2DJ1oU6jPCpswHgnaSm3W1): find arrows active (empty: focus the field; no hits: "No results" flashes),
log export active (click re-checks, then the info), event button **hidden** until the event picker exists (variant A). The mock lacks the export hint sentence. **R140 buttons, after the bugfixes below (one question at a time):** 1. **Q2 open:** tokens §5.1 rule "a hint of an element sits `space.sm` directly below it" (recommended yes; then the export info moves directly under the button, the hint sentence after it) → update the mock (R87 OK) → implement the R140 buttons + help texts + in-game test. 2. shared input core for `SearchField` and `TextField`? Recommended yes. 3. slash menu BL23: check Notion's behavior, then mock. 4. splits for the length hints (`markdown.rs`, `editor.rs`, `i18n.rs`, `actions.rs`, `buffer.rs`, `states.rs`).
**Sessions 44–49 (2026-11-18/20), all merged (PRs #152–#165):** memo data model M201–M216, R139–R156 (TextField, no disabled buttons, dialog close icon, one spacing scale and `Stack`, one icon size, shared `MenuItem` and list row, indent guides); details in the plans and the git log. Lesson: md5 groups miss a hover that lands on a wrong row, so measure positions in the PNG after layout changes.
**Sessions 50–51 (2026-11-20/21, phone + PC), merged:** Q538 (topic without sections: no chevron, R155) and **R157** (Q539–Q545): help split into `help/{en,de}/NN-name.md`, read and joined by `help_assembly.rs` from memo's `build.rs` with the privacy policy, build errors name file, line and fix; output byte-identical; 13 tests; in-game test 34 passed. **R155 c** (Q546 A, Q547): the help topic list shows the selected topic's sections, bold topics, `RowChevron::Blank` for a topic without sections, a clicked section is marked and scrolled to; in-game test 35 passed. BL27 marketplace and BL28 raid guide addon added. Escape closes memo's main window again while a pane has the focus (memo catches it in its `WndProc`; in-game test 36 passed). **M217 decided** (Q548–Q550): select and copy text in the note's read view; next an R87 proposal on `feature/read-view-text-selection`. Test gaps found (own `chore/test-gaps` after R155 c): the nested-row padding fix 7790ea3 (R156) has no automated test, M172 (double-click without toggle) only a snapshot; plus the known `render.rs`/window action/key gaps below.
**Session 52 (2026-11-21, PC), merged via the PR of `feature/read-view-text-selection`:** M217 (Q551–Q556, mock P1–P6): select and copy in the read view, the trash preview and the help; Ctrl+C/Ctrl+A over `WndProc`; the "Copied" toast for every copy (read view, menu, editor body; not the title field, not Ctrl+X; not yet in the help window, which has no toast); any left click ends a selection; R158 windows move only by their title bar. In-game test 37 passed. Lesson: probes in the log found both in-game bugs fast (keys caught before the frame lost a pushed action; the user had copied in the editor).
**Phone session 54 (2026-11-21), on `feature/shared-search`:** the user is away from the PC for two weeks (until about 2026-12-05) and tests everything afterwards; until then work goes on in the cloud and every game check goes straight into `in-game-tests.md` (user request). M219 mock approved (Q585, https://claude.ai/artifact/6ijSByZw8m9PS2hkPXcm5i); shared search and M219 built together (Q586) on a branch from `feature/trash-group-tree` (Q587 A; merge order: M218, then the search). Built: `controls::search` in trailkit-ui (field + count/arrows, Below/Beside with wrap at 12 characters), note list with "x of y" over all notes (current hit filled, others framed), the trash search (also trashed group children, Ctrl+F in the preview), the help window search (filtered topics/sections, counts, title marks, click to first hit, Ctrl+F below the head, F3, Shift twice → note search), `text::search_steps` shared by all three, review item 4 (text compared before parsing) fixed in `ListSearch`; help texts en/de, in-game test 39. **Open:** Q588 (push the branch and open a draft PR so the Windows CI renders the snapshots; no snapshot was rendered yet, the new help snapshots and the shifted hover coordinates (+49 px) are unverified). **Not in the mock, show the user for an R87 OK with the first rendered snapshots:** the list count row hides while nothing matches (the list's own empty state speaks); the trash search hides while the trash is empty; the help's no-result line is a plain "No results"; "– of 4" while the shown note is no stop; the trash search empties when the trash reopens. Escape in the help window still closes it, also with text in its field (test 39.9). New length hints to grill: `help_window.rs` (421), `draw_in` (94), `trash.rs` (413), `draw_main_window` (123).
**▶ Next step (start here):** Q588, then the snapshot review against mock v9 and the M219 mock (tokens.md §10); on the PC tests 38, 39 and the cursor check. Branches are pushed; plans of M218 (Q561–Q571), R159 (Q557–Q560) and the backend notes (Q572–Q574) live on `feature/trash-group-tree` until it is merged: read them there (`git fetch origin && git switch feature/trash-group-tree`).
**Phone session 53 (2026-11-21), done:** the user's OK to `child_order` (M218, recorded on `feature/trash-group-tree`); **BL30/BL31 grilling done** (Q575–Q583, BACKLOG): one shared search component in `trailkit-ui` (today's `SearchField` and the find bar M198 merged, Q581) for the note list, the trash and the help, field above the list, "x of y" with up/down over all entries, wrapping to a second row when narrow, `x` at the field end (clears; closes only where closable), a divider with `space.sm` under the search area and the Ctrl+F bar, Ctrl+F finds in the open content everywhere, Shift twice always opens the note search (also from the help window, Q582), privacy topic searched (Q579), every addon's help gets it (Q583); mock v9 approved per R87 https://claude.ai/artifact/DQRhxQtb4zyUxKB1hBtEVy. M219: the menu row becomes "Search notes" (R87 mock first). Release DLL of memo measured in the cloud: 2.80 MiB (2,933,760 bytes, `feature/trash-group-tree`). Note: `feature/trash-group-tree` carries an older copy of this file; take develop's when merging it.
1. **Shared search + M219** on `feature/shared-search`: built (session 54); still to do: memo window snapshot states for the new search (list count row, trash search, find bar wrap, menu "Search notes" in German at minimum width), every snapshot checked against the approved mocks, spacing included (user request 2026-11-21: edge cases empty field, no hits, one hit, wrap at the last hit, counter row wrap, closable vs not, Escape, Shift twice from trash and help, trashed group children). Then test 39 on the PC.
2. (M219 is part of 1.)
3. **"Copied" toast in the help window:** R87 mock first (the help window has no toast yet).
4. Optional in the cloud: `chore/` splits for the length hints (chips and link icon of `inline_text.rs` into `inline_text/links.rs`, the help window's key handling out of `draw_in`), user OK given 2026-11-21; snapshots via the PR's Windows CI.
**Back on the PC:** in-game test 38 (`in-game-tests.md` on `feature/trash-group-tree`), then the M218 PR; deploy `feature/windows-cursor-prototype` and check: Windows arrow, text cursor and hand over memo, no game gear behind it, game cursor back outside. If it works, build R159 properly on its own branch (trailkit windows only, through `trailkit-nexus-bridge`); if not, search another way.
Also open: length hints for split grilling: `draw_spans` (`inline_text.rs`, 80), `states.rs` tests (863), `editor.rs` (481), `dialogs.rs` (403), `buffer.rs` (413); one unreproduced debug crash `imgui.h:1629` while deleting in a link label (if it recurs: Ignore, note cursor placement, key, preview, tooltip).
**Next: data pipeline** (`rust/tools/trailkit-datagen`): the failing GemExchange job below (review item 3 done, D42); no more datagen unit tests, the backend tests cover it later (R152). Coverage 2026-11-18 before #146: addons 89 % of lines, memo app 70 %, datagen 52 %.
**Then test gaps:** extract and test the Escape/close logic of `nexus_bridge/render.rs` like `key_routing.rs`; window actions (new group, format, preview, compact view, open folder, clear search); Ctrl+Y/Ctrl+Shift+Z keys. `xtask` last. Prove new tests with injected faults.
**Coverage tooling (R152, option A):** build `cargo xtask coverage` from the manual steps below and document it in agent-reference/CONTRIBUTING. **Coverage on demand, not in CI** (`llvm-tools` installed): `RUSTFLAGS="-C instrument-coverage" CARGO_TARGET_DIR=target/coverage cargo xwin test --no-run`, run each binary from its crate with `LLVM_PROFILE_FILE`/`TRAILKIT_SNAPSHOT_ROOT` via `WSLENV` (`/p`), `llvm-profdata merge`, `llvm-cov export`; then delete `rust/*.profraw` and the 2.8 GB build.

**Open, in this order:**
1. **In-game tests:** tests 25–33 after deploying the branch above. After that pipeline work: Q501 toast visibility needs an R87 mockup; BL18 text size in core settings needs grilling.
   Propose splits via grilling for length hints: `shared/i18n.rs`, `get`, `slices/editor/editor.rs`, `draw_find_bar`, `draw_main_window`, `actions.rs`.
   Q436 (menu divider) stays open. The user kept the current find marks (current filled, others framed; section 5 of the mockup was declined).
   **Favorites** on `feature/memo-favorites` (M188–M191 decided): open Q407, Q408, Q416–Q419, then mockup (R87) and implementation.
2. **Remaining wishes of 2026-11-13:** memo struck-through checked items, then the second window. event:
   filters ("used in notes", a meta train order: research trains, timing, a source with licence/GDPR first), favorite events.
   All addons: more languages without growing the DLLs (BACKLOG BL15; reworks the `get` tables kept whole in Q392).
3. Last: chat translation addon feasibility (BL16, research first); then the queue below and the review items, one at a time.

**Open (grill):** an unknown tree node `type` (e.g. a future `guild_note`) still stops older builds from loading `notes.json`; `settings.json`/`ui_state.json` do not keep unknown fields (M202 covers notes only). **Decided, not scheduled:** Q340 backups with restore after settings and export/import (M28); Q329 `cargo bloat` at release prep. **Lessons:** measure before fixing; remove probes by editing; mockups first (R87); snapshot checks need a (draft) PR.

**Deferred by the user (2026-11-12): one theme character for addons and the coming React website.** No code yet. Mockups:
https://claude.ai/artifact/SFFrXawiqYD6HouhdihGdv (latest: Papier A–D). Wanted: GW2-reminiscent, own character, dark, very subtle Session 50 exploration, no decision: an ImGui-only paint layer (vertex-color fans with noisy rims and a faded fringe, gradients, Bézier strokes, points clamped into the rounded window; ~6300 triangles, rebuilt only on resize; no blur, grain or overlay) https://claude.ai/artifact/48jmBfK1ZQfsL6wFE27oeB, after the user's canvas https://claude.ai/artifact/4m5MTgaXdWPJXga4RzAC5a; its brass corners break "Brass never decorates". Alternative: PNG from an `xtask` (resvg), 9-slice.
(background about `#060709`–`#0d0e17`), crisp edges. Rejected: mountains, waypoint markers, header band/spine, web-template look. Closest:
Papercraft Q–T and Flecken U–X; recommended Papier B Rissrand. ImGui draws flat fills, polygons, one-direction gradients, offset shapes
as shadow, no blur or noise: mottling needs a PNG from a `cargo xtask`. Brass never decorates. Before code: grilling plus R87. ArenaNet's
Content Terms for fan content are not read yet; never copy game or Blish HUD art, never look official.

**Queue after that:** B23/B24/B25: tech-stack grilling, Q288–Q293, `trailkit-i18n` for shared lock texts, icon `wrench`; server stays local (Q223).
Next ids: **M220+, R160+, B46+, BL32+, questions from 586**. Deferred: GW2 build/Nexus version in `session_start` (C12).
## Review improvements — one at a time, agreed order, solutions still proposals

4. **Search cache:** `rust/addons/trailkit-memo/src/main_window/search.rs::update_search` calls `SearchQuery::parse` before checking the
   cache, allocating for unchanged queries every frame. Check unchanged inputs first; no benchmark was run.
5. **Docs/skills accuracy:** READMEs done (R91). Still wrong: `nexus-addon-dev/references/addon-structure.md`
   says Cargo forbids sibling-slice dependencies (it only rejects cycles; the rule is enforced by review);
   `rust-best-practices/references/chapter_09.md` marks `&mut T` as not Send (it is Send when T is Send).
6. **Performance contract:** reconcile the Nexus skill's blanket render-thread parsing ban with the change-triggered Markdown parsing;
   clarify allowed work and caching before any rewrite; keep the performance/security guarantees.
7. **Decision/documentation workflow:** broad token-loading fixes are done (R88).
   Relaxing grilling for routine implementation choices or rustdoc for trivial getters
   was only suggested, not approved. Preserve current rules until explicitly settled.

## Earlier implementation queue — after the review work

- Q134: split `actions.rs` into tree/editor actions at its next rebuild, not beforehand.
- Continue M28: settings/options → export/import → backups → help → release preparation
  (full tests, M67 screenshots, docs, manually measure release DLL size, R72).
- Separate grill round: hide Markdown outside the active editor line. Optional once:
  Light `reset.daily` on pressed tinted dropdown is 4.16:1 (R85 scope).

## Recurring checks and other open items

Triggers/consequences: [agent reference](../agent-reference.md#recurring-checks).
- **Dependencies (R130):** checked and updated 2026-11-17 (26 patch updates on `chore/dependency-patch-updates`, in-game test 19; jiff 1.0 open; nexus-rs current; `cargo audit`: no vulnerabilities, unmaintained `instant` and `paste` via nexus-rs). Next due 2026-11-24.
- **Wiki:** public again on Nov 18 (`Event_timers` HTTP 200, no login redirect). Next event-data work: switch back from Wayback per `data/events/README.md`.
- **C-703/25 P:** Nov 15 check on CURIA: still pending, no hearing, no AG opinion, DPF in force. Order of June 4, 2026 (ECLI:EU:C:2026:465). Recheck on CURIA when triggered:
  https://juris.curia.europa.eu/juris/liste.jsf?language=en&num=C-703%2F25
- **WvW reset snapshots (checked 2026-11-07):** both Nov 6 EU (ran 22:12 UTC, +4.8 h) and Nov 7 NA (07:16 UTC,
  +5.9 h) assets hold the **post-reset** matches; CI was green. Confirms review item 3. GemExchange fails daily since ≤ Nov 10 (HTTP 400, guess: gem > 1 gold).
- **DPF cron:** Nov 1 skipped, issue #11 opened manually. If Dec 1 skips, grill shifting off :00.
- **Flipping volumes (BL9):** DataWars2 vs GW2BLTC differs 3–10×; Silver allowed free use/
  archiving Nov 1 and confirmed suspicious `calculate_sold` branch; user sent the fix.
  GW2BLTC reply unknown; 24 h comparison needs an always-on machine (BL5). `flip_out` unreviewed.
- Nexus hover cursor: drafted Discord message remains with the user.
- User to-dos: verify public GitHub email; TMview/DPMA name check; C-703/25 Google Alert.
  Optional: cargo env in bashrc; GitHub “Automatically delete head branches”.
