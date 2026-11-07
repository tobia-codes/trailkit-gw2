# Token guideline

Binding rules for every agent (Claude Code, Codex) and every human who touches
addon UI. The token set is listed in `SKILL.md` §2; the values live only in
`crates/trailkit-ui/src/theme.rs` (palette ROADMAP R71). The visual reference
is the "Trailkit" design system in the user's Claude Design account; when it
and `theme.rs` disagree, `theme.rs` wins and the difference is a bug to report.

## 1. Where colors and sizes come from

- **Only tokens.** Every color, radius, spacing, border width and size comes
  from the active `Theme`. No hex values, no `[f32; 4]` literals, no
  `ImColor32::from_rgb`, no magic pixel numbers in addon or component code.
  The only literal colors in the workspace are the token values in
  `theme.rs` (and `Color::TRANSPARENT`).
- **Derived colors are rare and documented.** `with_alpha` on a token is
  allowed for overlays and selections (text selection, chip fill of invalid
  links); write a comment saying why. Never mix two tokens into a new color.
- **Sizes scale.** Token sizes are pixels at 100 %; anything that sits in a
  text line derives from the current font size (as chips and badges do).

## 2. Which token for what

| Need | Token | Never |
| --- | --- | --- |
| Window frame, gaps between panes | `bg.window` | text directly on it |
| Content region | `bg.panel` via `pane::draw_pane` | a bare `ChildWindow` |
| Text field, search | `bg.input` | |
| Hovered row or ghost button | `bg.hover` | a brass hover |
| Hovered row inside a popup | `bg.popup_hover` via `menu::draw_popup` | `bg.hover`, invisible on `bg.popup` |
| Selected or pressed list row | `bg.active` + selection bar | a brass fill |
| Pressed regular button, dropdown, menu row | `bg.pressed` | the same color as its hover |
| Body text, headings | `text.primary` | |
| Counts, ids, hints, placeholders | `text.secondary` | `text.disabled` for readable text |
| Bullets, chevrons, disabled controls | `text.disabled` | |
| Primary button fill, selection bar | `accent` | `accent` as text or a thin line |
| Brass text, icon or line (active toggle, check mark, drop line, focus) | `accent.text` | `accent` |
| Text on a brass fill | `text.on_accent` | any other text color |
| Status | `success`, `warning`, `danger`, `info`, always with an icon | color alone |
| Chat-link chips | `chip.<category>` (§4) | `accent` |
| Reset badges | `reset.daily`, `reset.weekly` | |

## 3. The brass budget

Brass (`accent`, `accent.text`, `accent.highlight`) marks exactly four things:

1. **The primary action** of a view: one `draw_primary_button` per view
   (Save while editing, Edit while reading). Secondary actions use
   `draw_button`, icon actions `draw_icon_button`.
2. **Keyboard focus** (`border.focus`).
3. **The current selection**: the selection bar, check marks, the drop
   line while dragging.
4. **Search hits** (R73, memo M122): `accent.highlight` behind matching
   text, an `accent.text` frame around a matching chip.

Brass never fills large areas, never decorates, and never marks a category.
If a design needs a fifth use, it needs a decision first (grill-me).

**Toggles that keep a state are never brass (R79).** A button that switches
something on and off and stays that way (show tools, show the note list,
show the search field, compact view, …) is drawn with
`button::draw_icon_toggle`: while on it sits on `bg.active` with a
`text.primary` icon, while off it is a plain icon button (`text.secondary`
icon, `bg.hover` only on hover). Several such toggles side by side would
otherwise light up like a christmas tree (user decision 2026-11-04). The
editor's formatting buttons that light up for the formatting at the cursor
(memo M69) still use `accent.text` until that is decided.

## 4. Chat-link chips

`ChatLinkKind::category()` assigns every link kind to one of four groups;
`theme.chip.for_category()` returns its `text` and `background`.

| Category | Kinds | Color |
| --- | --- | --- |
| `Place` | map point, WvW objective | copper |
| `Character` | skill, trait, build | blue |
| `Item` | item, recipe, skin, outfit, coins | jade |
| `Progress` | achievement, NPC text, player | violet |

- Icon and kind label use the category `text` color, the id (`#825`) uses
  `text.secondary`, coin amounts `text.primary` with `coin.*` dots.
- Invalid links (unknown type and damaged code alike) are a **warning**:
  neutral fill, `warning` frame, icon and label.
- A new link kind joins an existing category. A fifth category needs a
  decision first.

## 5. Shape and edges

- Radii: `radius.sm` 4 for chips, badges and scrollbar grabs; `radius.md` 6
  for buttons, inputs, dropdowns, rows, menu items and the drop highlight;
  `radius.lg` 10 for windows, panes, popups and dialogs. Custom-drawn shapes
  pick the radius of the thing they look like.
- **Nothing is square (R81, all addons).** Every filled or outlined surface
  has rounded corners, including every hover, pressed and selected state.
  Only lines too thin to show a radius (separators, the text caret) are
  exempt. ImGui 1.80 draws the highlight of `Selectable`, `MenuItem`,
  `BeginMenu` and tree nodes with radius 0, whatever `FrameRounding` says, so
  never use its highlight: rows use `selection_bar::hide_square_row_highlight`
  + `draw_row_highlight`, popups use `menu::draw_popup` with the `menu` entries
  (never a bare `MenuItem` or `ui.popup`).
- Every window, pane, popup and dialog has the 1 px `border.subtle` outline.
- Regular buttons and dropdowns have a 1 px `border` frame (R85): their fill
  alone can vanish on `bg.window`. Primary and `danger` fills stand out alone.
- **Dividers (R83)** come from `separator`, never `ui.separator()`, and have the
  same gap above and below, measured to the visible content (letters, an icon,
  a button edge); the fill of a hovered row may come closer:
  - **Section, `spacing.medium` 12:** `draw_separator` (buttons, fields),
    `draw_separator_above_rows` (fill of the first row `list.item_spacing` away),
    `draw_separator_above_text` (the text's own leading counts as gap).
  - **Compact, `spacing.small` 8:** `draw_compact_separator`, inside a tight
    group: menus, a rule in a note's body.
- Text buttons are at least `button.min_width` × font size wide (M131);
  measure rows with `button_width` / `icon_label_button_width`, never with
  `calc_text_size` + padding, or right-aligned rows drift.
  No highlight edges and no shadows (a top highlight was tried and dropped
  after the in-game test, R71).
- Lines (separators, outlines) sit on whole pixels: a 1 px line drawn
  between two pixel rows blurs into a soft 2 px band. `draw_separator`
  snaps; custom-drawn lines must do the same.

## 6. Selection

The selected row keeps the neutral `bg.active` fill (ImGui `Header`) and gets
`selection_bar::draw_selection_bar` right after its tree node or selectable:
an `accent` bar `list.selection_bar_width` (3 px) wide with round ends,
inset `list.selection_bar_inset` (6 px) from the top and bottom of the row,
on the row's left edge. Hover shows `bg.hover` only, no bar.

## 7. Typography

Open Sans in Regular, Bold, Italic and Bold Italic (`fonts`, subset by
`cargo xtask subset-fonts`, memo M62). Inter was tried for R71 and dropped
(larger DLL, wider, less soft). Use only characters the subset covers; a
test fails otherwise.

## 8. Changing or adding a token

1. Decide it first (grill-me) and record it in `docs/plans/ROADMAP.md`.
2. Add it to `theme.rs` with a value for **both** themes and a doc comment
   naming its token id (`chip.place`, `radius.md`, …).
3. Text tokens join `text_meets_contrast_minimum_in_every_theme` (≥ 4.5:1 on
   every surface); text on a new fill gets its own contrast test. Graphics
   and focus rings need ≥ 3:1 against their ground.
4. Document it in `xtask/src/token_docs/token_groups.rs` (the crate does not compile until
   you do), run `cargo xtask tokens` to update
   [`token-values.md`](token-values.md), and add any rule for it here.
5. Run `cargo xtask ui-test`; review changed snapshots before accepting them
   with `--update`.

## 9. Review checklist for tokens

- [ ] No literal colors or sizes outside `theme.rs`
- [ ] `accent` only as a fill; `accent.text` for brass text, icons, lines
- [ ] At most one primary (brass) button per view
- [ ] Selected rows use the selection bar, not a colored fill
- [ ] Chips use their category colors; invalid links are warnings
- [ ] Radius matches the element, nothing is square, hover states included (§5, R81)
- [ ] Readable in Dark and Light; new text tokens are in the contrast test

## 10. Every state has a snapshot, every snapshot gets reviewed (R82)

A UI change is not done, and not handed to the user for an in-game test,
until its snapshots have been reviewed against this file.

**Coverage.** Every element of every addon has a snapshot of each state it can
be in, in Dark **and** Light:

- Interaction: rest, hover, pressed, selected, on/off, disabled, focused,
  open, dragged.
- Content: empty, filled, long/scrolled, error or problem hint.
- Size and language: every view, menu, dialog and hint also at the window's
  minimum width, and the longest texts in German, where nothing may run past or
  be clipped by its pane (`lay-responsive`; hovers excepted, they do not change
  the layout). A snapshot shows clipped text but does not fail on it, so look.

Each state is covered once per element; the cross product of independent
states is not needed. Shared components live in
`crates/trailkit-ui/tests/component_states.rs` (`ui-<component>-<state>-<theme>`),
addon views in `addons/<addon>/src/main_window/tests/states.rs`
(`<addon>-state-<name>-<theme>`), the minimum width in
`tests/states/minimum_width.rs` (`min-*`, German `min-de-*`). A new element or
state adds its snapshot in the same change.

**Adding a snapshot.**

1. Write a fixture that brings the window into the state: a function returning
   a `Fixture`, shared if several tests need it.
2. Call `assert_state(name, size, fixture, &steps)`. The steps come from `hover`,
   `press`, `click`, `right_click`, `drag` and `typing`. It runs Dark and Light.
   `assert_state_in` takes a language; a dialog's question built in the fixture
   stays in the fixture's language.
3. Run `cargo xtask ui-test --update` (about 4 minutes), which writes the new
   reference. Without `--update` every differing pixel fails, and the images land
   in `tests/snapshots/failures/`. A refactoring must pass without `--update`:
   that proves it changed no pixel.
4. Open each new image and review it as below. Delete references of removed
   states by hand; nothing reports orphans.

**Coordinates break silently.** Steps use fixed positions, so any layout shift
moves them, and `--update` stays green while a hover misses:

- At 640 px wide and test fonts the list spans x 12-204, the note pane 212-628.
- After `--update`, group the images by md5 and compare the groups: `hover-group =
  hover-group-arrow = hover-badge-count` and `note = hover-selected-row =
  hover-task-text = memo-wide` are the known identical sets. A state joining a set
  means a missed hover or click.
- A click fires on release over the item: release there before moving away.
- A killed `ui-test` can leave a test process running that overwrites images with
  old code. Stop it with `pkill -f '^/init .*debug/deps/'`; a pattern that also
  matches the calling shell's command line kills that shell.

**Review.** Look at every new or changed snapshot, the whole element and not
only what the change was about, and go through §9:

- [ ] Every corner rounded with the right radius, hover states included (§5)
- [ ] Tokens: colors, spacing and borders as in §2 and §3
- [ ] Alignment and equal gaps between neighbors
- [ ] The state is visible at all, in both themes
- [ ] Texts fit and are not clipped

Check the snapshot actually shows the state: a hover that misses its element
looks like a rest state and passes silently. Fix or report what the review
finds before accepting with `--update`.

**Limits.** The test renderer uses 16 px fonts (18 px in game), samples
textures nearest-neighbour and shows no game behind the window. All its fonts
are the same size, while in game the window font, body and icons differ, so a
size taken from the wrong font looks right in every snapshot (M170). Sizes that
depend on the font, and anything depending on the real game, stay in-game
checks.

**Measuring in game.** When the user reports a few pixels off, measure before
changing anything; guessing took three rounds on M132 and M170.

1. Decode the user's screenshot with a small stdlib script (`zlib`, `struct`) and
   print a brightness map of the region: exact rows and columns, faint ones too.
2. For the real sizes, add a one-off `log::info!` (guarded to log once), deploy,
   and read `Nexus.log`: it appears seconds later while the window is open. Remove
   it before committing.
3. Compare the map with the logged numbers, and fix what does not match.
