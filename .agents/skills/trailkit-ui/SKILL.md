---
name: trailkit-ui
description: Design system and UI rules for all trailkit-gw2 Nexus addons (ImGui) — theme tokens, dark/light themes, user customization, shared components, layout and UX best practices. Use whenever building, changing or reviewing any addon UI.
---

# trailkit UI design system

All trailkit addons must look and feel like **one product**. This skill is
the **single source of truth** for UI/UX in this repo. The detailed rule set
(game overlay, accessibility, interaction, forms, layout, navigation, states,
charts) lives in [`references/ux-rules.md`](references/ux-rules.md) — read it
before designing a new window or component and when reviewing UI. Do not apply
generic web/mobile UI guidelines (touch targets, breakpoints, CSS, …) here.

## 1. Principles

1. **Consistency over novelty** — same component, same look, same place, in
   every addon.
2. **Modern, flat, square** — `border radius = 0` everywhere (windows, frames,
   buttons, popups, scrollbars, tabs). Clear hierarchy through spacing,
   typography and color, not decoration.
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
| Accent | `accent`, `accent.hover`, `accent.active` |
| Status | `success`, `warning`, `danger`, `info` |
| Border | `border`, `border.focus`, `border.width` |
| Coin | `coin.gold`, `coin.silver`, `coin.copper` (coin amounts in chat-link chips; graphics, not text) |
| Event | `event.default`, `event.highlight`, `event.active`, `event.upcoming`, `event.custom` (per category overrides allowed) |
| Spacing | `space.xs` 4, `space.sm` 8, `space.md` 12, `space.lg` 16, `space.xl` 24 (px at 100 %) |
| Lists | `list.item_spacing`, `list.indent` |
| Panes | `pane.padding` 8 (inner padding), `pane.gap` 8 (space between panes) — see `lay-pane-padding` |
| Shape | `radius` = 0 (user-overridable, default 0) |
| Typography | `font.size.small`, `font.size.body`, `font.size.heading`, `font.line_height` (running text: 1.5 × font size, text centered in its line; markdown view and editor) |

- **Built-in themes:** `Dark` (default) and `Light`. Both are full token sets.
  Values: `crates/trailkit-ui/src/theme.rs` (palette R51: anthracite surfaces,
  GW2-like gold accent). A unit test enforces ≥ 4.5:1 for every text color on
  every background — add new text/background tokens to that test.
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

- `Window` wrapper (title bar with menu: Settings, Help, About)
- `SettingsWindow` scaffold (left navigation list, right content, search)
- `HelpWindow` (renders bundled markdown help in active language)
- `Button` variants: primary, secondary, ghost, danger
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
  platform) + `chat_link_chip` (pill: icon, kind, id; coins as amounts with
  coin dots; `!` warning chip for broken codes). The addon supplies the texts
  through `ChipContext` (trailkit-ui has no strings). Used by the markdown
  view and the editor, where a chip is one atomic unit (memo M75)
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
- Lists: consistent row height, hover highlight, selected state, context menu
  on right-click with the same actions as visible buttons. Item names start
  flush left; per-item icons and badges sit right-aligned at the row end
  (the name is truncated before them); only structural controls such as a
  group's expand arrow stay on the left.
- Drag & drop: show a drop indicator line between items and a highlight on
  groups; Escape cancels.
- Text: sentence case for labels, title case never; no trailing colons on
  section headers.
- Every window remembers position/size; Escape closes (register via
  `GUI_RegisterCloseOnEscape`).
- Text buttons: always `button::draw_button` / `draw_primary_button`, never
  `ui.button` with a text label (Open Sans sits low in ImGui's centering,
  memo M85).
- Tooltips: plain ImGui tooltips; `StyleScope` moves them right of the real
  cursor (GW2's cursor is large, memo M84) — never position them by hand.
- Overlays inside a window (scrims): ImGui renders a child window's
  *background* into the parent's draw list, i.e. below every sibling child
  drawn earlier. Never dim with `ChildBg`; draw the dimming rectangle into
  the overlay child's own draw list (see memo `draw_drawer`).

## 5. Review checklist

- [ ] Rules in `references/ux-rules.md` checked (cite rule ids in reviews)
- [ ] Overlay rules: compact, no focus stealing, auto-hide, frame budget
- [ ] Only tokens used, no raw colors/sizes
- [ ] Works and is readable in Dark *and* Light
- [ ] Empty / loading / error / disabled states present
- [ ] Every string localized (en + de)
- [ ] Help page updated
- [ ] Style push/pop balanced
