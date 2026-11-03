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
| Selected row, pressed button | `bg.active` + selection bar | a brass fill |
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

Brass (`accent`, `accent.text`) marks exactly three things:

1. **The primary action** of a view: one `draw_primary_button` per view
   (Save while editing, Edit while reading). Secondary actions use
   `draw_button`, icon actions `draw_icon_button`.
2. **Keyboard focus** (`border.focus`).
3. **The current selection or active state**: the selection bar, active
   toggles, check marks, the drop line while dragging.

Brass never fills large areas, never decorates, and never marks a category.
If a design needs a fourth use, it needs a decision first (grill-me).

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
- Every window, pane, popup and dialog has the 1 px `border.subtle` outline.
  Panes add the `border.highlight` line along their top inner edge
  (`draw_pane` does it). No shadows: ImGui has none, and fakes look cheap.

## 6. Selection

The selected row keeps the neutral `bg.active` fill (ImGui `Header`) and gets
`selection_bar::draw_selection_bar` right after its tree node or selectable:
an `accent` bar `list.selection_bar_width` (3 px) wide with round ends,
inset `list.selection_bar_inset` (6 px) from the top and bottom of the row,
on the row's left edge. Hover shows `bg.hover` only, no bar.

## 7. Typography

Inter in Regular, SemiBold, Italic and SemiBold Italic (`fonts`, subset by
`cargo xtask subset-fonts`). Bold text and headings use SemiBold. Use only
characters the subset covers; a test fails otherwise.

## 8. Changing or adding a token

1. Decide it first (grill-me) and record it in `docs/plans/ROADMAP.md`.
2. Add it to `theme.rs` with a value for **both** themes and a doc comment
   naming its token id (`chip.place`, `radius.md`, …).
3. Text tokens join `text_meets_contrast_minimum_in_every_theme` (≥ 4.5:1 on
   every surface); text on a new fill gets its own contrast test. Graphics
   and focus rings need ≥ 3:1 against their ground.
4. Add it to the token table in `SKILL.md` §2 and, if it has a rule, here.
5. Update the "Trailkit" design system (tokens and README) so the visual
   reference stays in step, and tell the user.
6. Run `cargo xtask ui-test`; review changed snapshots before accepting them
   with `--update`.

## 9. Review checklist for tokens

- [ ] No literal colors or sizes outside `theme.rs`
- [ ] `accent` only as a fill; `accent.text` for brass text, icons, lines
- [ ] At most one primary (brass) button per view
- [ ] Selected rows use the selection bar, not a colored fill
- [ ] Chips use their category colors; invalid links are warnings
- [ ] Radius matches the element (§5)
- [ ] Readable in Dark and Light; new text tokens are in the contrast test
