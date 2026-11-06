# Next session — handoff

Last updated: 2026-11-06 (session 18: window menu replaces the tool rail, on the unmerged, unpushed branch `feature/memo-ux-round`; the user continues with Codex).

## Where we are

- **Phase 0 + 0b complete.** Daily archive runs (sometimes hours late).
- **Phase 1 (trailkit-memo standalone):** steps 1 (storage), 2 (tree & note
  view), 3 (editing), 3b (own editor with preview), 3c (chat-link chips),
  reset kinds and **drag & drop (M106–M117)** are merged into `develop` and
  tested in game; **search (M118–M129)** is merged and deployed, not yet
  confirmed in game. Decisions in `docs/plans/addons/trailkit-memo.md`.
- Order from here (M28 as amended by M58, M62, M65):
  ~~visual & interaction tests~~ → ~~font subsetting~~ → ~~3c chips~~ →
  ~~reset kinds~~ → ~~drag & drop~~ → ~~search~~ → **trash & undo
  (delete, M12, next)** → settings window → export/import → backups → help page →
  **release preparation** (full test, screenshots M67, docs, measure the
  release DLL size by hand, R72).
- **Session 9 (PRs #65–#69): new "Trailkit" theme, ROADMAP R71** (supersedes
  R51), all deployed; the user called the result "richtig gut":
  - Dark/Light (named just that): ink-tinted dark and parchment-warm light
    surfaces; **brass** accent `#E2A740` spent only on the primary action,
    focus and selection; `accent.text` for brass text, icons and lines.
  - Selection = neutral `bg.active` + short inset 3 px brass bar
    (`trailkit_ui::selection_bar`); tree expand arrows are Lucide chevrons
    (`tree_arrow`); no folder/note icons (M49 stays).
  - Chat-link chips in four categories (`ChatLinkKind::category()`,
    `theme.chip.*`): place copper, character blue, item jade, progress
    violet; id in `text.secondary`; invalid links stay warnings.
  - Radii 4/6/10 (`radius.sm/md/lg`); buttons filled with `bg.hover`
    (never darker than the pane); separators in `border.subtle`, drawn on a
    whole pixel; no pane highlight edge, no shadows.
  - **One dropdown** (`draw_dropdown(ui, theme, tint, …)`): button height,
    `bg.hover` fill, `border` frame, optional tint; the chip variant is gone.
  - **Own title bar** (`title_bar::draw_window`): no band, Lucide chevron
    collapses, bold title, Lucide `x` closes; memo's Edit is a brass button
    with a pencil (`draw_primary_button_with_icon`); `draw_icon_button`
    moved into trailkit-ui.
  - Inter was tried and **dropped** (≈130 KB bigger, wider, less soft) —
    Open Sans stays (M62).
  - **Binding token guideline** for Claude and Codex:
    `.agents/skills/trailkit-ui/references/tokens.md` (linked from the
    trailkit-ui skill).
  - Visual reference in the user's Claude account: design system
    "Trailkit" (https://claude.ai/artifact/G7sNVZe1JYTgxbUE5mhS33) and the
    mockup canvas (https://claude.ai/artifact/Dp8BGiKczKw1ysiBmA8acj). Both
    are **out of date** in places (still mention Inter, the pane highlight
    and a selection bar at a fixed x); `theme.rs` is the source of truth.
- Session 8 (PRs #63–#64): drag & drop and "Move to…" for notes and groups
  (M106–M115), gold drag-label border (M116), text size steps of 6.25 %
  (M117).
- Session 7 (PRs #52–#61), all deployed and checked in game by the user:
  - Reset kinds: `reset_schedule` (daily 00:00 UTC, weekly Mon 07:30 UTC),
    due resets untick all tasks on open/while open, wait for edited notes
    and drafts (M88–M90); kind via chip dropdown right behind the note
    title (view mode only, M102) or right-click › Reset type; countdown
    only in the badge tooltip (M95); colors `reset.daily` gold /
    `reset.weekly` violet for badge and chip (M104); bold, filled,
    ink-centered badges (M105).
  - Editor: one toolbar `[◧] | H1 H2 | B I | lists | link event | 👁 …
    Cancel Save` with group separators (M94, M100, M102); Save/Speichern
    instead of Done (M100); Ctrl+S = Save (M93).
  - Note list toggle = sidebar icon `panel-left-close/open` (M101).
  - Right-click: groups and empty list area offer New note/New group
    (M96, M97); submenus open flush and use Lucide check/chevron
    (`trailkit_ui::menu`, M98, M99).
  - All addons: radius 4 px and `border.subtle` outline (6 % white dark /
    10 % black light) for windows, panes, popups, dialogs (M103).
  - trailkit-ui gained `dropdown` (`Field`, `Chip { tint }`), `menu`,
    `glyph_ink_box`; harness `right_button_down`.
- Late session 5 (PRs #26–#30): bold/italic/link toggle off from anywhere
  inside the span (M68, M71, parsed with `markdown::inline_ranges`), toolbar
  buttons light up for the formatting at the cursor (M69), ≡ is gold while the
  note list is visible (M71), roomier unsaved-changes dialog with a border,
  right-aligned primary and stacked buttons in narrow windows (M70).
  Snapshot tests are strict now (no differing pixels; CI renders identically).
- Session 6 (PRs #33–#50), all tested in game by the user:
  - Layout: drawer scrim really dims (#33), editor and preview always side
    by side (M72), narrower note list + earlier drawer while previewing
    (M73), fixed note-list width (M74), note title stays at the top while
    the content scrolls (M81).
  - Dev loop (R70): auto-merge with required checks on `develop` (user set
    branch protection + `allow_auto_merge`), docs-only PRs skip the
    Windows job (verified, ~40 s), CI runs on push only for `main`,
    pre-push also runs `cargo xwin clippy` for the Windows target.
  - **3c chat-link chips** (M75–M77, M80–M82): `trailkit_ui::chat_link`
    decodes every link type locally, `chat_link_chip` draws them (icon at
    85 % of the text size, centered on the capitals; 3 px radius; slim);
    click copies in view/preview, atomic in the editor; warning chip for
    broken codes. Names later via core (C15); slash commands `/w`, `/e`
    with core (M76); never touch the game chat (C16); "open in wiki" → BL10.
  - Text: line height 1.5 × (token `font.line_height`, M78); whole memo
    window uses the body font (list and note had different sizes); text
    size setting `text_scale` (default 112.5 %, Ctrl+mouse wheel, Ctrl+0,
    percentage tooltip, fonts rebuilt only after the wheel rests; M79/M80).
  - Fixes: keyboard sink stays in view in scrolled editors (keys went to
    the game in long notes, M83); tooltips start right of GW2's large
    cursor (`cursor_extent`, M84); text-button labels centered by their
    capitals (`draw_button`, M85).
- Session 5 also: Open Sans replaces Noto Sans (M62), panes with padding
  (`lay-pane-padding`), drawer + menu button (M48), titles flush left with
  right-aligned badges (M49), bold groups with note count (M50), in-window
  dialogs (`overlay-dialog-scope`), Escape via Nexus registration.

## ▶ Resume here

Next ids: **M170+, R87+, questions from 123.** The new look is described in
[`current-design.md`](../../.agents/skills/trailkit-ui/references/current-design.md).

1. **Finish branch `feature/memo-ux-round`** (local only, nothing pushed; work
   continues here, on no other branch). Decisions M157-M169 are in
   `docs/plans/addons/trailkit-memo.md`.
   - **The user tests in game, then says whether it may merge.** Deployed: M157-M169.
     Not yet confirmed in game: the window menu and its rows, the backdrop, the
     minimum width (at 18 px text), the list hiding and returning around editing,
     the search field's `x`, the `⋯` menu, Rename from it (title selected), toast
     buttons, delete and undo. Follow-ups stay on this branch and are redeployed.
   - **Merge only after the user's OK in that conversation** (R74): then push, open
     the PR into `develop`, `gh pr merge --auto --merge --delete-branch`. No AI
     attribution in commits or PRs (R68).
   - **Open questions, not yet answered:**
     - Q112: split `draw_toast` (103 code lines, hint at 80) by moving the layout
       calculation into its own function? Recommendation: yes, a small commit here.
     - Q113: keep the right-click menu's new order (Reset type below Move to), because
       it shares its code with the `⋯` menu (M160)? Recommendation: yes.
     - Q121: split `SearchField::draw` (91 code lines), a new hint from M169, by moving
       the icon placement out? Recommendation: yes, a small commit here.
     - Q122: `render_main_window` (83 code lines) grew slightly past the hint.
       Recommendation: leave it until the settings step changes that function anyway.
   - **Choices the agent made, disclosed to the user and easy to turn back:** Ctrl+F does
     nothing in the editor (M167); the selected row of the menu (M166); the Hide row is
     also offered in the trash, as in the approved mockup.
   - Row 4 (editor that hides markdown outside the active line) is **not** part of
     the branch: it needs its own grill round, it is the biggest change.
2. **Splits after that (R77), each a small own step:** Q91 `note_list.rs` +
   `draw_group`, Q95 `draw_dropdown`, Q96 `editor/view.rs`, Q105 `markdown_view.rs`.
   All answered yes. Q92 `Text::get`: my recommendation is no, the user has not
   objected. `Editor::draw` (101 code lines) is a new hint, not yet put to the user:
   proposed to take it along with Q96 or the editor round.
3. **Then the M28 order continues:** settings window (a row in the window menu),
   export/import, backups, help page, release preparation.
4. Optional, offer once: Light contrast of `reset.daily` on `bg.pressed` is
   4.16:1 while a tinted dropdown is held (see R85's test scope).

7. Working rules that proved useful:
   - UI tests: `cargo xtask ui-test [--update]`; look at PNGs only when
     they change. Tests with fixed list coordinates use
     `selected_first_with_search` (search field shown, M138).
   - Characters in help pages must exist in the subset fonts (`⋮` does not;
     the test `every_character_is_in_the_subset_fonts` catches it).
   - Lucide codepoints by name: parse the source TTF `cmap` + `post`
     (a stdlib Python script did it in session 15).
   - Test fonts are 16 px, in game 18 px: verify in-game screenshots.
   - Deploy after every UI change (`cargo xtask deploy trailkit-memo`);
     check `Nexus.log` for "Loaded addon".
   - In this WSL shell plain `grep` may fail; use `command grep`. Run
     `source ~/.cargo/env` before cargo.
   - Never leave an `eprintln!` debug line before `cargo fmt`.

## Session 18 summary (2026-11-06, branch `feature/memo-ux-round`)

- Window menu instead of the tool rail (M166), the editor hides the note list
  (M167), a minimum width instead of the responsive layout (M168), the search
  field's `x` closes it (M169); R86 amends R78. `trailkit_ui::tool_rail` removed.
- Mockups (private to the user's Claude account):
  https://claude.ai/artifact/GzpgDiBgz5VrzFMdtvTYWN.
- Knowledge that only Claude Code's memory held moved into `AGENTS.md` ("Working
  with the user", commands) and `current-design.md`, so Codex can continue.

## Facts learned in session 18

- Removing the rail moved everything left of the note pane's right edge by 40 px:
  list items and the note's left-anchored items in `states.rs` shifted with it;
  right-anchored buttons stayed. The md5 groups below confirmed no hover missed.
- Window menu rows are at y 67, 99, 131 in the 640 px snapshots, x from 20 to 147.

## Session 17 summary (2026-11-05, branch `feature/memo-ux-round`)

- Commits: `4b5c1f2` UX round, `69a479e` drawer on right click, `1eb1595` drawer
  on delete and undo, `d1a983f` toast above the drawer.
- M157 rail always visible, `⋮` and `tool_rail_visible` gone. M158 toast "Moved to
  the trash" with Undo and Open trash, no count on the icon (Q108). M159 toast
  buttons below the message or stacked (Q110). M160 `⋯` menu, shared with the
  context menu through `note_list::EntryMenu`. M161 Rename from `⋯` opens the
  editor with the title selected, the sidebar stays (Q109, Q111). M162 tooltips
  name shortcuts.
- **Arrow keys in the note list were dropped** (Q107, M162): Nexus would send them
  to the game.
- M163/M164 (user reports): only a left click on a note closes the drawer. M165:
  toast card in its own draw list.
- Shared: `menu::place_next_popup`, `ToastAction`, `Icon::ELLIPSIS` (U+E0B6).

## Facts learned in session 17

- Any layout shift silently breaks fixed coordinates in `states.rs`: `--update`
  stays green while hovers miss. After `ui-test --update`, group the snapshots by
  md5 and compare with the groups at `HEAD`: `hover-badge-count = hover-group-arrow =
  hover-group` and `hover-selected-row = hover-task-text = note = memo-wide` are the
  known identical sets (`collapsed-group` joining one exposed a missed click).
- The columns start at: list x 12-204, note pane 212-628 (window 640 wide, test
  fonts).
- A child window's background lands in the parent's draw list, so a later sibling
  (the drawer) covers it and the parent's other children can show through. Overlays
  draw their card in their own list (`draw_background(false)`, as the scrim does); a
  borderless child has no window padding and is not clipped at its edge.
- Selecting through `select` closes the drawer; use `change_selected` when the
  list must stay open.
- Nexus logs "Loaded addon" 1–2 s after the deploy returns: check the log again
  before calling a reload failed.

## Session 16 summary (2026-11-05)

- Merged branch `feature/memo-sidebar-layout`: layout M136-M144, trash & undo
  M133-M152, sidebar toggle M153, back arrow M154, neutral editor toolbar M155,
  chip "Copied" on the toast M156.
- Rules: R80 `bg.popup_hover`, R81 nothing is square, **R82 every state has a
  snapshot and every changed snapshot is reviewed against `tokens.md` §10**, R83
  two kinds of dividers, R84 generated token reference, R85 control states with
  `bg.pressed` and a `border` frame on regular buttons.
- `crates/trailkit-ui/tests/component_states.rs` and
  `addons/trailkit-memo/src/main_window/tests/states.rs` hold about 240 snapshots;
  `UiHarness::step` skips rasterizing, the run takes about 3.5 minutes.
- Mockup canvases: dividers https://claude.ai/artifact/98pE7YeQzXxxugr1kW5VEa,
  sidebar toggle and way back https://claude.ai/artifact/34t732EdC4hpsY3Tjp1aaz,
  tokens and states https://claude.ai/artifact/Hn23Uh5RkchXZ57NaX1MP5.

## Facts learned in session 16

- Host `cargo clippy` does not compile the addon's Windows-only code: run
  `cargo xwin clippy --target x86_64-pc-windows-msvc --all-targets` as well.
- A killed `ui-test` can leave its test process running under `/init`, which
  overwrites snapshots with old code. Stop it with
  `pkill -f '^/init .*debug/deps/'`; a pattern that also matches the calling
  shell's own command line kills that shell.
- A hover that misses its element looks like a rest state: compare the hover
  snapshot with the rest snapshot by pixels (0 differing pixels = missed or no
  hover). Coordinates in `states.rs` shift when the layout above them changes.
- ImGui 1.80: `Selectable`/`MenuItem` highlights have radius 0; their item rect
  reaches half the item spacing outwards; disabled items draw their label in
  `TextDisabled`, and `IsItemHovered` is true for them once the item has ended.

## Session 15 summary (2026-11-04, PR #83 + branch)

- PR #83 (addon blueprint R76, length hints R77) merged.
- Trash & undo grill round 1 (Q63–Q70) settled; the trash entry point led
  to a whole layout round (Q68–Q88) with ~30 mockups on a Claude Design
  canvas: tool rail like VS Code, both layout toggles in the title bar,
  hideable search, compact note view, neutral toggles (user: "perfekt").
- Implemented and deployed on `feature/memo-sidebar-layout`; then the
  trash round (Q74–Q80, mockups on the same canvas) and the whole trash &
  undo step on the same branch.

## Session 14 summary (2026-11-04, PRs #81–#82)

- R76 questions 48–58: note-list slice (`ListFilter` trait, closures for
  reset extras, empty search result in the search slice), `note_tree`
  split, `trailkit_ui::text_layout`, `main_window` split with an `apply`
  dispatcher.
- **M132** row badge letters pixel-centred: visible ink measured from the
  font atlas (`glyph_metrics::glyph_visible_ink`), letter drawn at the
  sub-pixel centre (`draw_glyph_at`), all badges one width, height by
  ink-centre parity (rule in the trailkit-ui skill). Took three in-game
  rounds — measure in-game screenshots, the test renderer is 16 px and
  samples textures nearest-neighbour.
- **R77** advisory length hints (`cargo xtask check-file-length`).
- Addon structure blueprint (`references/addon-structure.md`).
- Nexus hot reload stopped once in a game session (see below); a game
  restart fixed it. If a deploy shows no "Loaded addon" line in
  `Nexus.log`, ask the user to reload or restart.
- The user is learning Rust (asked about `self.0`, derives, tuple structs,
  test placement, closures): explain with TypeScript comparisons.

## Session 11 summary (2026-11-03, PRs #77–#80)

- **M130/M131** (#77, #78): primary toolbar buttons carry a Lucide icon
  (Save = floppy `save`, Edit = pencil); text buttons are at least
  `button.min_width` = 4 × font size wide, icon buttons stay square.
- **R74** (#79): merge into `develop` only after the user's in-game OK.
- **R75** (#79): three type layers — `trailkit-gw2-api-types` (boundary
  only), `trailkit-model` (our refined domain types, shared by addons,
  datagen, backend), `trailkit-protocol` (`#[repr(C)]`).
- **#80**: `main_window.rs` split into `main_window/{note_list,note_pane,
  dialogs,layout,tests}.rs`; check-names allows snake_case module dirs
  below `src/`.
- **R76** (local branch): addons are built from slice crates; step 1 done.

## Session 10 summary (2026-11-03, PRs #71–#75)

- **R72 DLL size:** jiff without default features (no bundled tzdb),
  local time via `SystemTimeToTzSpecificLocalTime` (Wine maps it under
  Linux/Proton), clippy bans `jiff::tz::TimeZone::system`. Memo release
  DLL 2 315 264 → 1 964 032 bytes. User: **performance beats size** —
  `opt-level` 3 and `panic = "unwind"` stay; measured `s` −11 %, `z`
  −14 %, abort −14 %. Compared with installed addons memo is mid-range
  (Killproofme 14.8 MB, Event Timers 3.7 MB, DRF 2.4 MB, ArcDPS 1.4 MB).
- **Search M118–M127** (`note_search`, `search_field`,
  `trailkit_ui::text_search`, `markdown::plain_text`,
  `markdown_view::SearchHighlight`), token **`accent.highlight` R73**
  (brass budget now has four uses, tokens.md §3), Lucide `search` icon.
- **M128** list header (equal full-width buttons that stack, taller
  field, clear `x` mirrors the magnifier), **M129** fixed list header.
- **#74** `xtask subset-fonts` fixes Lucide glyph boxes (see facts).

## Facts learned in session 10

- The Lucide source font stores `xMin`/`yMin` = 0 for every glyph, so
  ImGui glyph quads were far larger than the ink and icons could not be
  centered by `glyph_ink_box`. `cargo xtask subset-fonts` now rewrites the
  boxes from the outline points (`fix_glyph_bounds`); drawing positions do
  not change.
- Test fonts are 16 px, in game 18 px: anything sized from an icon
  button's natural width can overflow in game while tests look fine —
  size inner controls from the ink or a fixed square.

## Facts learned in session 9

- ImGui's `AddLine` already adds half a pixel; add none yourself or a 1 px
  line straddles two rows and shows as a dim 2 px band in game (the test
  renderer does not show it).
- ImGui draws a tree node's triangle in the node's text color: push a
  transparent `Text` color while pushing the node, then draw chevron and
  title yourself.
- Windows with `NO_TITLE_BAR` can be moved from any empty area even when
  `ConfigWindowsMoveFromTitleBarOnly` is set.
- Lucide glyph codepoints: parse the source TTF's `cmap` + `post` tables in
  Python (e.g. `x` = U+E1B2); no fontTools available.

## Facts learned in session 7

- ImGui submenus overlap their parent by `ItemInnerSpacing.x` (read in
  `BeginMenu`); `trailkit_ui::menu::begin_submenu` sets it to 0.
- A `Selectable`/button click fires on release over the item: UI tests
  must release on the item before moving the mouse away.
- Python heredoc edits: never `open(p,'w')` and read the same file in one
  expression (it truncated `main_window.rs` once; restored from git).

## Facts learned in session 6 (also in the skills)

- The keyboard sink must stay **in view**: ImGui does not activate an
  input scrolled out of its window, and Nexus then sends every key to the
  game (M83, nexus-addon-dev skill).
- Open Sans sits low in ImGui's line-box centering; center by the capitals
  (`glyph_metrics::capital_middle`) — text buttons via `draw_button` (M85).
  Lucide icons fill the whole line box: a 16 px "H" is 9 px tall, the icon
  square 15 px.
- `DrawListMut` is a singleton in imgui-rs: shared components draw through
  the raw `ImDrawList_*` calls when a caller may hold one (dialog card).
- Tooltip position = mouse + 24 px × `style.MouseCursorScale`; `StyleScope`
  sets the scale from the measured cursor (`cursor_extent`).
- Nexus font resizes (`resize_font`) flicker; debounce them (text size).
- The test harness fonts are 16 px; in game the body is FontUI × 1.125
  (18 px) — measure in-game screenshots when sizes matter
  (`png_measure`-style pixel checks worked well).
- Auto mode blocks: changing branch protection (user ran it) and
  `git commit --amend` (use a follow-up commit).

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

## Nexus hot reload stopped once (session 14, 2026-11-04)

- In the game session started 13:40 no deploy was hot-reloaded; manual
  unload/load in the Nexus addons menu works. Tested: WSL rename-over,
  WSL move-aside + copy, and a native PowerShell rename + copy — Nexus logged
  nothing for any of them, so its directory watcher did not fire; not a
  deploy-script bug. Nexus 2026.10.25.60 (installed 2026-10-25, hot reload
  still worked on 2026-11-03). After a game restart the watcher fired again
  on the next deploy ("Processing changes" in the log): a Nexus hiccup in
  that game session. If it recurs: restart the game; if it persists, ask
  in the Raidcore Discord.

## Recurring checks

- GW2 wiki (2026-10-31): still behind Cognito login → Wayback workaround stays.
- C-703/25 P (checked 2026-11-06, session 18: search found no judgment, opinion or hearing, sources only from 2025; check again next session): still
  pending, no hearing date or judgment reported; DPF in force. Microsoft intervened for the
  Commission; noyb asked for repeal (2026-06-30) but has not sued yet.
- DPF reminder workflow: the 2026-11-01 08:00 cron was silently skipped by
  GitHub; triggered manually (issue #11). If it skips again on 2026-12-01,
  grill round on moving the cron off the full hour.
- Daily archive: WvW EU snapshot Fri 2026-11-06 17:23 UTC is the real test of
  the cron delay margin (D29). Not yet checked after that time.

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
