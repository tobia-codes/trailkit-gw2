---
name: trailkit-ui
description: Design system and UI rules for all trailkit-gw2 Nexus addons (ImGui) — theme tokens, dark/light themes, user customization, shared components, layout and UX best practices. Use whenever building, changing or reviewing any addon UI.
---

# trailkit UI design system

All trailkit addons must look and feel like **one product**. This skill is
the **single source of truth** for UI/UX in this repo. The detailed rule set
(game overlay, accessibility, interaction, forms, layout, navigation, states,
charts) lives in [`references/ux-rules.md`](references/ux-rules.md) — read it
before designing a new window or component and when reviewing UI. How to use
the theme tokens (which token for what, the brass budget, chips, selection,
adding a token) is binding in [`references/tokens.md`](references/tokens.md) —
read it before touching any color, size or radius, and review every new or
changed snapshot against it before handing a UI change over (§10, R82). Do not apply
generic web/mobile UI guidelines (touch targets, breakpoints, CSS, …) here.


How the addons look today, with the snapshots to open: [`references/current-design.md`](references/current-design.md).

**Design approval first (R87):** AGENTS.md rule 2 applies to every visible
change, including small changes and states. How to show a proposal is in
[`current-design.md`](references/current-design.md#showing-the-user-a-visual-decision).
Read only reference sections relevant to the task, rather than every linked file.

## 1. Principles

1. **Consistency over novelty** — same component, same look, same place, in
   every addon.
2. **Calm, flat, always rounded (nothing is square, hover states included: R81)** — ink-tinted dark and parchment-warm
   light surfaces with one brass accent spent sparingly (R71); radii 4 / 6 /
   10 by element size; windows, panes, popups and dialogs get a faint 1 px
   `border.subtle` outline (memo M103); no highlight edges, no shadows —
   except tooltips: `border` outline, `radius.sm` and a hard 2 px
   `shadow.popup` below, so they stand out from panes and rows (Q327).
   Clear hierarchy through spacing, typography and color, not decoration.
3. **Clarity** — every control has a visible label; icons always have a
   tooltip; destructive actions ask for confirmation; unsaved changes warn
   before being discarded.
4. **Feedback** — loading, empty, error and disabled states are always
   designed (never a blank area); every action that can fail has a designed
   failure case with a recovery (`state-failure-designed`). Disabled features say *why* and *how to
   enable* them.
5. **Accessibility** — text contrast ≥ 4.5:1 against its background in both
   themes; never encode meaning by color alone (add icon/text); minimum
   comfortable click target ≈ 24 px height at 100 % scaling.
6. **Performance** — UI code follows the render-thread rules of the
   `nexus-addon-dev` skill.

## 2. Theme tokens

Components never use raw colors or numbers; they read **semantic tokens** from
the active `Theme` (crate `crates/trailkit-ui`). Every token with its value in
both themes and its purpose is listed in
[`references/token-values.md`](references/token-values.md), generated from
`theme.rs` by `cargo xtask tokens` (R84); when to use which is binding in
[`references/tokens.md`](references/tokens.md).

- **Built-in themes:** `Dark` (default) and `Light`. Both are full token sets.
  Values: `crates/trailkit-ui/src/theme.rs` (palette R71, supersedes R51:
  ink-tinted dark and parchment-warm light surfaces, brass accent). The
  "Trailkit" design system in Claude Design is no longer a reference (R84). The themes are
  called just "Dark" and "Light", in code and in the UI. Unit tests enforce
  ≥ 4.5:1 for every text color on every surface and for chip text on its
  fill — add new text/background tokens to them (`references/tokens.md` §8).
- **User customization:** the user can override any token in the settings
  (color pickers, sliders) and save as a custom theme; reset-to-default per
  token and for the whole theme. Overrides are stored as a diff over the base
  theme.
- **Shared across addons:** the active theme lives in a shared file under the
  Nexus common directory so all trailkit addons use the same theme; an addon
  may offer per-addon overrides only for addon-specific tokens (e.g. event
  colors). (Exact mechanism: see core plan.)
- Applying a theme = RAII guard that pushes ImGui style colors/vars and pops
  them on drop. Never touch the global ImGui style (shared with other addons).

## 3. Shared components (crate `trailkit-ui`)

Build once, use everywhere. Planned:

- Title bar — **exists**: `title_bar::draw_window` replaces ImGui's title
  bar on every trailkit window (R71, R78): on the window background; on the
  left the addon's own toggles (`TitleBarToggle`, e.g. memo's note list) and the bold title, on the right the Lucide chevron to
  collapse and the Lucide `x` to close. The row uses the window padding, so
  the first and the last button line up with the panes below. The addon
  passes texts and toggles, keeps a `TitleBarState` and applies the clicked
  toggle index it gets back. A menu (Settings, Help, About) joins it later
- Window menu — **pattern, in memo** (`main_window/window_menu.rs`, M166, R86): the
  title bar's `menu` toggle opens a panel over the full content height with an
  `overlay.modal` backdrop (`lay-drawer`). Moves to this crate when a second
  addon needs it (rule 7)
- `SettingsWindow` scaffold (left navigation list, right content, search)
- `HelpWindow` (renders bundled markdown help in active language)
- Buttons — **exist**: `button::draw_button` (secondary, `bg.hover` fill),
  `draw_primary_button`, `draw_primary_button_with_icon` (Lucide icon +
  label, R71; primary toolbar actions, `lay-button-icon`), all text
  buttons at least `button.min_width` wide (M131), `draw_icon_button` / `draw_icon_toggle` (borderless icon
  buttons with tooltip), delete buttons (R105, memo M146): `draw_danger_button`,
  `draw_danger_fill_button_with_size` (e.g. the confirm button of a delete
  question) and the icon-only `draw_danger_icon_button`, all `danger` fill
- Toast — **exists**: `toast::draw_toast`, one message with an optional
  action button at the bottom centre of the window, 5 s, the timer pauses
  while hovered, a new one replaces the old (memo M142, M148, `ix-toasts`).
  `ToastTone::Error` (only for real need, e.g. a failed save; memo M180):
  `danger.subtle` fill, `danger` frame, `circle-alert`, no timer; with
  `draw_toast_with_extras` a counter, a spinner and a close `x`. A busy
  action (`ToastAction::busy`) shows a spinner. `ToastTone::Success`:
  `circle-check` in `success`, 5 s
- Status card — **exists** (memo M184): `status_card::draw_status_card`,
  a state that needs attention in place (not a toast): `CardStatus`
  `Warning`/`Danger` with `warning.subtle`/`danger.subtle` fill and a frame
  in the status color, a status line, an optional `CardDetail` hanging under
  the text in a normal text role, buttons at the left edge (R103: only
  primary, regular and delete) and an optional close `x` (`draw_icon_button`).
  `fits_status_card_line` lets a caller break a long text on purpose
- Only two button styles (R103) plus delete (R105): brass primary and
  regular; every button that deletes, and every button that throws data
  away for good ("Discard", R106), is the `danger` fill
  (`ButtonKind::DangerFill`) and takes the brass button's place in a row;
  icon buttons like the close `x` look like every other one. A delete entry
  in a menu uses `MenuItemTone::Danger`. Hover and pressed fills follow one
  rule (R107, `tokens.md` §2 Button states)
- Button with kind and state — **exists** (R93): `button::Button` with
  `ButtonKind` (`Regular`, `Primary`, `DangerFill`) and `ButtonState`
  (`Ready`, `Busy` = spinner, `Disabled { reason }` = `text.disabled`
  label, no hover, reason tooltip with a neutral `ban` icon via
  `tooltip::draw_reason_tooltip`); `draw_button`, `draw_primary_button` …
  are short forms. Icon buttons: `draw_icon_button_with_state`. Disabled
  menu entries use the same reason tooltip. Drive `Busy` with
  `busy::BusyState` (at least 0.6 s); on popups and toasts
  `draw_popup_icon_button` (hover on `bg.active`)
- `Popover` / `Tooltip`, `ConfirmDialog`, `UnsavedChangesDialog`
- `DependencyHint` (greyed feature + explanation + "Open core settings"/
  "Get trailkit-core" button)
- `Tree` with drag & drop (groups / subgroups / items)
- Empty state — **exists** (Q283): `empty_state::draw_empty_state` with
  `EmptyState` (optional icon, title, hint, buttons with one primary)
- Text and icons — **exist** (R96): `text::draw_text`, `draw_text_wrapped`,
  `draw_truncated_text`, `draw_icon`; borderless inputs use
  `StyleScope::transparent_frame`
- `ColorTokenEditor`, `KeybindHint`, `ErrorState`, `Spinner`
- Markdown renderer (read-only view) — **exists**: `markdown` (parser,
  any platform) + `markdown_view` (ImGui)
- Shared fonts (Open Sans regular/bold/italic/bold italic, headings; M62) and
  Lucide icons — **exist**: `fonts`, `font_registry` (`FontScope`), `icons`;
  addon text uses the `TextStyle` roles of `references/tokens.md` §7.1 (R96)
- Chat-link chip — **exists**: `chat_link` (find/decode `[&…]`, any
  platform) + `chat_link_chip` (pill in its category color: icon and kind in
  `chip.<category>.text`, id in `text.secondary`; coins as amounts with coin
  dots; warning chip for broken codes; R71). The addon supplies the texts
  through `ChipContext` (trailkit-ui has no strings). Used by the markdown
  view and the editor, where a chip is one atomic unit (memo M75)
- Dropdown (select box) — **exists**: `dropdown::draw_dropdown`, **one
  dropdown for every place** (R71): button height, filled like a button
  (`bg.hover`) with a `border` frame, chevron, popup list, check mark on the
  chosen option; fixed width of the widest option (memo M87); an optional
  `tint` colors frame, text, chevron and check mark (e.g. the reset kind,
  M104). No compact or other variants. Never ImGui's plain combo
- Selection bar — **exists**: `selection_bar::draw_selection_bar`, called
  right after the selected row's tree node (R71), with the rounded row fill
  `draw_row_highlight`; the `_in` variants take the row rect for
  selectables, whose item rect reaches beyond the row
- Tree arrow — **exists**: `tree_arrow::draw_tree_arrow`, the Lucide chevron
  (`chevron-down` / `chevron-right` in `text.disabled`) over ImGui's hidden
  triangle; never ImGui's filled triangle in lists (R71)
- Menu entries — **exist**: `menu::draw_menu_item` (check mark = Lucide
  `check` in `accent.text`) and `menu::begin_submenu` (Lucide `chevron-right`,
  flush beside the parent); never ImGui's own check mark/arrow (memo M99);
  `menu::draw_menu_item_with_icon` (Lucide icon before the label, regular
  or `danger` tone, disabled with a tooltip, memo M133, M141). A row that runs a
  command has an icon, so all labels start on one line; a row that picks a value
  (dropdown options, a submenu's targets) has none, only the check mark. The
  `menu.*` tokens give all of them their room (memo M177)
- `Pane` and inset separator — **exist**: `pane::draw_pane`,
  `separator::draw_separator`; every sub-region of a window and every
  separator uses them (`lay-pane-padding`, decided 2026-11-01)

Rule: a component starts life inside one addon. When a second addon needs it,
move it into `crates/trailkit-ui` (see AGENTS.md "Code sharing").

## 4. Layout & interaction

- Panes: content regions inside a window are padded panes on `bg.panel`,
  separated by a gap instead of a line; nothing touches an edge
  (`lay-pane-padding`).
- Settings windows: grouped sections with headings, short help text under
  non-obvious options, changes apply live (no "Apply" button) except
  destructive/import actions.
- Lists: consistent row height, hover highlight, selected state (`bg.active`
  plus the brass selection bar, never a colored fill), context menu
  on right-click with the same actions as visible buttons. Item names start
  flush left; per-item icons and badges sit right-aligned at the row end
  (the name is truncated before them); only structural controls such as a
  group's expand arrow (a Lucide chevron) stay on the left. Rows have no
  type icons (no folder or note icons; R71).
- Drag & drop: show a drop indicator line (`accent.text`) between items and a
  highlight on groups; the dragged item's title floats next to the cursor
  in a label with an `accent.text` border (`danger` plus Lucide `ban` over an
  impossible target, memo M109/M116); Escape cancels.
- Text: sentence case for labels, title case never; no trailing colons on
  section headers.
- Every window remembers position/size; Escape closes (register via
  `GUI_RegisterCloseOnEscape`).
- Text buttons: always `button::draw_button` / `draw_primary_button`, never
  `ui.button` with a text label (text fonts sit low in ImGui's centering,
  memo M85). One `draw_primary_button` per view at most (brass budget,
  `references/tokens.md` §3).
- A single glyph centered in a box (badge letters, later icons in boxes):
  measure its *visible* ink with `glyph_metrics::glyph_visible_ink` (font
  atlas pixels; the quad from `glyph_ink_box` is widened unevenly by
  ImGui's oversampling). Horizontally draw it with `draw_glyph_at` at the
  exact sub-pixel centre (`add_text` floors to whole pixels); vertically
  give the box the parity whose centre matches the ink centre's fraction
  (never round the ink size). Boxes of one kind keep one width so they line
  up (memo M132; `badge_geometry` in the memo note-list slice).
- Tooltips: always `tooltip::draw_tooltip` or `draw_tooltip_with`, which center the
  text by its capitals (M171; clippy bans the ImGui calls); `StyleScope` moves them right of the real
  cursor (GW2's cursor is large, memo M84) — never position them by hand.
- Overlays inside a window (scrims): ImGui renders a child window's
  *background* into the parent's draw list, i.e. below every sibling child
  drawn earlier. Never dim with `ChildBg`; draw the dimming rectangle into
  the overlay child's own draw list (see memo `draw_window_menu`).

## 5. Review checklist

- [ ] Rules in `references/ux-rules.md` checked (cite rule ids in reviews)
- [ ] Overlay rules: compact, no focus stealing, auto-hide, frame budget
- [ ] Only tokens used, no raw colors/sizes; token checklist in
      `references/tokens.md` §9 passed
- [ ] Works and is readable in Dark *and* Light
- [ ] Empty / loading / error / disabled states present
- [ ] Every action that can fail shows its failure case and recovery
      (`state-failure-designed`, default pattern memo M180)
- [ ] Every string localized (en + de)
- [ ] Help page updated
- [ ] Style push/pop balanced
