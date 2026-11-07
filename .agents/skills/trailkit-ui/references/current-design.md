# How the addons look today

The starting point for any agent that has not seen the addon. It tells you where
the current look is recorded and how to see it. The rules for changing it are in
[`tokens.md`](tokens.md) and [`ux-rules.md`](ux-rules.md).

## Where the look is recorded

Use these in this order. The first one that answers the question wins.

| Source | What it tells you |
|--------|-------------------|
| Snapshot PNGs | What each state really looks like, rendered by the addon's own code |
| `crates/trailkit-ui/src/theme.rs` | The token values; [`token-values.md`](token-values.md) is generated from it |
| [`tokens.md`](tokens.md) | Which token to use when (binding), with the review checklist in §10 |
| [`../SKILL.md`](../SKILL.md), [`ux-rules.md`](ux-rules.md) | Shared components and UX rules |
| `docs/plans/addons/trailkit-memo.md` | Why memo looks the way it does (decisions M1 onwards) |

**The Claude Design canvases linked in the plans are not a source for you.** They
are private `claude.ai` pages, so other agents cannot open them, and several are
out of date. Each decision they led to is written down in the plans. The oldest,
for the user: design system https://claude.ai/artifact/G7sNVZe1JYTgxbUE5mhS33 and
mockups https://claude.ai/artifact/Dp8BGiKczKw1ysiBmA8acj (they still show Inter and
a pane highlight).

## How to see the addon

The snapshots are committed, so no game and no build are needed to look at them:

- Memo window: `addons/trailkit-memo/tests/snapshots/`
- Shared components: `crates/trailkit-ui/tests/snapshots/`

Open the PNGs with your image tool. If you have none, ask the user to attach the
files named below. To render them again, run `cargo xtask ui-test`; add `--update`
to rewrite them after an intended change. Failing comparisons land in
`tests/snapshots/failures/`.

Names follow `memo-state-<state>-<dark|light>.png` and
`ui-<component>-<state>-<dark|light>.png`. States are defined in
`addons/trailkit-memo/src/main_window/tests/states.rs` and
`crates/trailkit-ui/tests/component_states.rs`.

## The memo window

```text
+--------------------------------------------------+
| [=] Notes                                  v   x |  title bar
+--------------+-----------------------------------+
| [Search..  x]|  [<>]                ...   [Edit] |  search field only while shown
| [ + ]  [ + ] |  Shopping list     [Normal  v]    |
|--------------|-----------------------------------|
| | Shopping   |  Ectos                            |  note list, fixed width
|   Dailies  D |  [x] 250 Globs of Ectoplasm       |  note gets all extra room
| v Raids    1 |  [ ] Mystic Coins                 |
|   Weekly   W |  Check the wiki first.            |
+--------------+-----------------------------------+
```

| To see | Open `memo-state-…` |
|--------|---------------------|
| Regular page | `note-dark`, `note-light` |
| Window menu, over the full height with a backdrop | `window-menu-dark`, `window-menu-light` |
| Window menu while hovering, in the trash, without notes | `window-menu-hover-row-dark`, `window-menu-in-trash-dark`, `window-menu-no-notes-dark` |
| Narrowest window, list and note side by side | `minimum-width-dark` |
| Search field shown, with hits | `search-shown-dark`, `search-hits-dark` |
| Trash in place of the list, with its back arrow | `trash-nothing-selected-dark`, `trash-selected-dark` |
| Note list hidden by the user | `list-hidden-dark` |
| Editor (the list is hidden while editing) | `editor-dark`, `editor-preview-dark` |
| Menu while editing | `window-menu-editing-dark` |
| Compact note view | `compact-dark` |
| Dialogs, toast | `dialog-unsaved-dark`, `toast-dark` |
| Popup menus, submenus, dropdown | `menu-note-dark`, `menu-note-reset-submenu-dark`, `kind-open-dark` |
| A group collapsed by a click, the note staying | `group-clicked-dark` |
| Any view at the narrowest window | `min-<state>-dark`, in German `min-de-<state>-dark` |

## How the layout behaves

- **No layout depends on the window width.** The window has a minimum width
  instead (M168), so the note list never folds away by itself. Only rows of
  controls still wrap (`lay-responsive`).
- **The `menu` button in the title bar** opens the window menu (M166): Notes,
  Search, Trash, then Hide/Show note list. "Notes" always leads back to the
  regular page.
- **The editor hides the note list** and brings it back when it ends (M167). A
  list the user hid stays hidden.
- **A click on a group's row toggles it** and selects it; the note pane keeps its note
  (M172, M173).
- **Ctrl+F toggles the search.** The field's `x` clears text first and closes an empty
  field (M176).
- **Popup menus have room** (`menu.*` tokens, M177): an icon in every command row, none in
  a row that picks a value.
- **A click into the game hands the keyboard back** to the game, also from the editor
  (M178).
- **At the narrowest window the list is barely readable by choice** (M168): nested
  notes show only their badge, and the search hint is cut to "Sea". The note stays
  fully usable. Everything else wraps or shortens with "…".

## Character of the design

"Trailkit" (R71): ink-tinted dark and parchment-warm light surfaces, with Open Sans
and Lucide icons. Brass `#E2A740` is rare and only marks the primary action, focus,
selection and search hits. Toggles are neutral (R79), nothing is square (R81), and
there are no shadows or highlight edges.

## What the snapshots cannot show

- **Text size:** test fonts are 16 px, the game draws the body at 18 px. Anything
  sized from text can fit in a test and overflow in game.
- **Textures:** the test renderer samples nearest-neighbour, and it does not show
  half-pixel line bands. Measure in-game screenshots when pixels matter.
- **Input focus:** the game routes keys through Nexus (`nexus-addon-dev` skill), so
  focus can behave differently there. Only the user's in-game test confirms it.

## Showing the user a visual decision

The user decides from images, not descriptions, and wants every state shown. Claude
Code draws mockups on a Claude Design canvas. Without that tool, build the state
behind a test and show the rendered PNG, or write a standalone HTML page with the
values from [`token-values.md`](token-values.md) and give the user its path. Say
clearly when an image is a mockup and not rendered by the addon.

What the proposal must show and when addon implementation may start is binding
in AGENTS.md rule 2 (R87).
