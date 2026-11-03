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
read it before touching any color, size or radius. Do not apply
generic web/mobile UI guidelines (touch targets, breakpoints, CSS, …) here.

## 1. Principles

1. **Consistency over novelty** — same component, same look, same place, in
   every addon.
2. **Calm, flat, slightly rounded** — ink-tinted dark and parchment-warm
   light surfaces with one brass accent spent sparingly (R71); radii 4 / 6 /
   10 by element size; windows, panes, popups and dialogs get a faint 1 px
   `border.subtle` outline (memo M103); no highlight edges, no shadows.
   Clear hierarchy through spacing, typography and color, not decoration.
3. **Clarity** — every control has a visible label; icons always have a
   tooltip; destructive actions ask for confirmation; unsaved changes warn
   before being discarded.
4. **Feedback** — loading, empty, error and disabled states are always
   designed (never a blank area). Disabled features say *why* and *how to
   enable* them.
5. **Accessibility** — text contrast ≥ 4.5:1 against its background in both
   themes; never encode meaning by color alone (add icon/text); minimum
   comfortable click target ≈ 24 px height at 100 % scaling.
6. **Performance** — UI code follows the render-thread rules of the
   `nexus-addon-dev` skill.

## 2. Theme tokens

Components never use raw colors or numbers; they read **semantic tokens** from
the active `Theme` (crate `crates/trailkit-ui`). Token set (initial — extend
in the crate, document here):

| Group | Tokens |
|-------|--------|
| Surface | `bg.window`, `bg.panel`, `bg.input`, `bg.hover`, `bg.active`, `bg.popup`, `overlay.modal` |
| Text | `text.primary`, `text.secondary`, `text.disabled`, `text.link`, `text.on_accent` |
| Accent | `accent` (brass fill: primary button, selection bar), `accent.hover`, `accent.active`, `accent.text` (brass as text, icon or line) |
| Status | `success`, `warning`, `danger`, `info` |
| Border | `border` (inputs, dropdowns), `border.focus`, `border.subtle` (windows, panes, popups, dialogs; ~6 % white / ~8 % warm black), `border.width` |
| Reset | `reset.daily` (brass), `reset.weekly` (violet): reset note badges and kind chip (memo M104; text colors, contrast-tested) |
| Chip | `chip.place`, `chip.character`, `chip.item`, `chip.progress`, each with `text` and `background` (chat-link categories, R71) |
| Coin | `coin.gold`, `coin.silver`, `coin.copper` (coin amounts in chat-link chips; graphics, not text) |
| Event | `event.default`, `event.highlight`, `event.active`, `event.upcoming`, `event.custom` (per category overrides allowed) |
| Spacing | `space.xs` 4, `space.sm` 8, `space.md` 12, `space.lg` 16, `space.xl` 24 (px at 100 %) |
| Lists | `list.item_spacing`, `list.indent`, `list.selection_bar_width` 3, `list.selection_bar_inset` 6 |
| Panes | `pane.padding` 8 (inner padding), `pane.gap` 8 (space between panes) — see `lay-pane-padding` |
| Shape | `radius.sm` 4 (chips, badges, scrollbar), `radius.md` 6 (buttons, inputs, rows, menus), `radius.lg` 10 (windows, panes, popups, dialogs); user-overridable; custom-drawn frames use them too |
| Typography | `font.size.small`, `font.size.body`, `font.size.heading`, `font.line_height` (running text: 1.5 × font size, text centered in its line; markdown view and editor) |

- **Built-in themes:** `Dark` (default) and `Light`. Both are full token sets.
  Values: `crates/trailkit-ui/src/theme.rs` (palette R71, supersedes R51:
  ink-tinted dark and parchment-warm light surfaces, brass accent; visual
  reference: the "Trailkit" design system in Claude Design). The themes are
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
  bar on every trailkit window (R71): on the window background, Lucide
  chevron to collapse, bold title, Lucide `x` to close; the addon passes the
  texts and keeps a `TitleBarState`. A menu (Settings, Help, About) joins it
  later
- `SettingsWindow` scaffold (left navigation list, right content, search)
- `HelpWindow` (renders bundled markdown help in active language)
- Buttons — **exist**: `button::draw_button` (secondary, `bg.hover` fill),
  `draw_primary_button`, `draw_primary_button_with_icon` (Lucide icon +
  label, R71), `draw_icon_button` / `draw_icon_toggle` (borderless icon
  buttons with tooltip). Planned: danger
- `Popover` / `Tooltip`, `ConfirmDialog`, `UnsavedChangesDialog`
- `DependencyHint` (greyed feature + explanation + "Open core settings"/
  "Get trailkit-core" button)
- `Tree` with drag & drop (groups / subgroups / items)
- `ColorTokenEditor`, `KeybindHint`, `EmptyState`, `ErrorState`, `Spinner`
- Markdown renderer (read-only view) — **exists**: `markdown` (parser,
  any platform) + `markdown_view` (ImGui)
- Shared fonts (Open Sans regular/bold/italic/bold italic, headings; M62) and
  Lucide icons — **exist**: `fonts`, `font_registry` (`FontScope`), `icons`
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
  right after the selected row's tree node or selectable (R71)
- Tree arrow — **exists**: `tree_arrow::draw_tree_arrow`, the Lucide chevron
  (`chevron-down` / `chevron-right` in `text.disabled`) over ImGui's hidden
  triangle; never ImGui's filled triangle in lists (R71)
- Menu entries — **exist**: `menu::draw_menu_item` (check mark = Lucide
  `check` in `accent.text`) and `menu::begin_submenu` (Lucide `chevron-right`,
  flush beside the parent); never ImGui's own check mark/arrow (memo M99)
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
- Tooltips: plain ImGui tooltips; `StyleScope` moves them right of the real
  cursor (GW2's cursor is large, memo M84) — never position them by hand.
- Overlays inside a window (scrims): ImGui renders a child window's
  *background* into the parent's draw list, i.e. below every sibling child
  drawn earlier. Never dim with `ChildBg`; draw the dimming rectangle into
  the overlay child's own draw list (see memo `draw_drawer`).

## 5. Review checklist

- [ ] Rules in `references/ux-rules.md` checked (cite rule ids in reviews)
- [ ] Overlay rules: compact, no focus stealing, auto-hide, frame budget
- [ ] Only tokens used, no raw colors/sizes; token checklist in
      `references/tokens.md` §9 passed
- [ ] Works and is readable in Dark *and* Light
- [ ] Empty / loading / error / disabled states present
- [ ] Every string localized (en + de)
- [ ] Help page updated
- [ ] Style push/pop balanced
