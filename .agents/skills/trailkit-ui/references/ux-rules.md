# trailkit UX rules (ImGui overlays inside Guild Wars 2)

Full rule set for building and reviewing trailkit addon UIs. `SKILL.md` holds
the principles, tokens and components; this file holds the detailed rules.
Read it when designing a new window/component or doing a UI review.

Context that shapes every rule: our UI is a set of **small Dear ImGui windows
drawn over a running game**, operated with **mouse and keyboard**, on desktop
monitors, rendered **every frame**. There is no web, no touch, no mobile, no
screen reader. Rules from web/mobile guidelines that do not survive this
context were deliberately left out (see "Not applicable" at the end).

> Adapted from the rule set of
> [nextlevelbuilder/ui-ux-pro-max-skill](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill)
> (MIT License, © its authors), filtered and rewritten for in-game ImGui
> overlays. Sections 1 and 8 are trailkit-specific.

Each rule has an id (`area-name`) so reviews can cite it.

## 1. Game overlay (CRITICAL — trailkit-specific)

- `overlay-occlusion` — Cover as little of the game as possible. Default
  window sizes are compact; content-heavy windows (settings, help, editors)
  are opened on demand and closed afterwards.
- `overlay-combat` — Never demand attention during combat: no auto-opening
  windows, no focus stealing, no modal dialogs triggered by timers or data
  updates. Reminders are non-blocking toasts.
- `overlay-auto-hide` — Persistent overlays (event tables, pinned panels)
  hide during loading screens, cutscenes and character select (MumbleLink /
  Nexus state); "hide in combat" is offered as an option where it makes sense.
- `overlay-no-focus-steal` — Opening or updating a window never takes keyboard
  focus from the game unless the user clicked into a text field.
- `overlay-lock` — Every persistent window can lock position and size, and
  remembers both per window.
- `overlay-ui-scale` — Respect the user's GW2 interface size / Nexus font
  scale; all sizes come from tokens multiplied by the current scale, never
  hard-coded pixels.
- `overlay-frame-budget` — The UI is drawn every frame: no I/O, allocation
  bursts, sorting of large lists or string formatting of unchanged data in
  the render callback (see `nexus-addon-dev` threading rules). Cache derived
  display data and rebuild it only on change.
- `overlay-motion` — No decorative animation. Allowed: instant state changes,
  a progress spinner, and at most a short fade for toasts. Countdowns update
  at most once per second.
- `overlay-sleep` — Closed windows cost nothing: skip all work for hidden
  windows (no layout, no text formatting).
- `overlay-click-through` — Purely informational overlays may offer a
  click-through mode so they never block game clicks.
- `overlay-escape` — Escape closes the focused trailkit window
  (`GUI_RegisterCloseOnEscape`); it never closes something with unsaved
  changes without asking.

## 2. Accessibility & readability (CRITICAL)

- `a11y-contrast` — Text ≥ 4.5:1 against its background, large text / icons /
  data marks ≥ 3:1, in **both** Dark and Light themes. Remember the game
  scene behind semi-transparent windows: window background opacity must keep
  contrast valid on bright and dark scenes.
- `a11y-color-not-only` — Never convey meaning by color alone; add an icon,
  text or shape (e.g. event "running" = color **and** label/icon).
  Status colors must stay distinguishable for red-green color blindness.
- `a11y-icon-tooltip` — Icon-only buttons always have a tooltip naming the
  action (and the keybind if any).
- `a11y-keyboard` — Tab order follows visual order; Enter confirms, Escape
  cancels in dialogs; every drag & drop action has a non-drag alternative
  (context menu "Move to…", up/down buttons).
- `a11y-focus-visible` — The keyboard-focused control is visibly marked
  (`border.focus` token).
- `a11y-text-scaling` — Layouts survive larger font scales: labels wrap or
  truncate with tooltip, nothing overlaps.
- `a11y-escape-routes` — Every modal and multi-step flow has a visible
  cancel/close.

## 3. Interaction & feedback (CRITICAL)

- `ix-states` — Every interactive element has distinct hover, active,
  disabled and (where relevant) selected states, all from tokens.
- `ix-target-size` — Mouse targets ≥ 24 px tall at 100 % scale with ≥ 4 px
  gap; no pixel-precise targets (thin edges, tiny icons without padding).
- `ix-hover-not-only` — Never hide essential actions behind hover alone; the
  right-click context menu repeats the visible actions, it does not replace
  them.
- `ix-feedback-fast` — Every click produces visible feedback in the same
  frame; long operations (file import, downloads) show progress and keep the
  UI responsive (work on the worker thread).
- `ix-busy-buttons` — A button that starts an async action is disabled with a
  spinner until it completes.
- `ix-drag-threshold` — Drag starts only after a few pixels of movement;
  show a drop indicator line between items and highlight target groups;
  Escape cancels the drag.
- `ix-confirm-destructive` — Destructive actions ask for confirmation stating
  what will be lost (e.g. "Delete group with 12 notes?"), or are undoable.
- `ix-undo` — Prefer undo over confirmation for frequent actions ("Deleted —
  Undo" toast).
- `ix-success-feedback` — Completed actions that have no visible result get a
  short toast ("Waypoint copied").
- `ix-toasts` — Toasts auto-dismiss after 3–5 s, never steal focus, stack
  without covering the window they refer to, and can be dismissed by click.

## 4. Forms & settings (MEDIUM)

- `form-labels` — Every input has a visible label; placeholders are hints,
  never the only label.
- `form-helper-text` — Non-obvious options have one line of helper text below
  them (`text.secondary`).
- `form-errors` — Errors appear directly at the field, state cause **and**
  fix ("Time must be HH:MM, e.g. 20:30"), and use `danger` + icon. Validate
  when the field loses focus, not on every keystroke.
- `form-required` — Required fields are marked.
- `form-live-apply` — Settings apply live (no "Apply" button), except
  destructive or import actions, which need explicit confirmation.
- `form-unsaved` — Closing an editor with unsaved changes asks: save /
  discard / cancel.
- `form-grouping` — Related settings are grouped under a section heading;
  advanced options are collapsed by default (progressive disclosure).
- `form-disabled-why` — Disabled controls say *why* and *how to enable* them
  (tooltip or `DependencyHint`), e.g. "Requires trailkit-core".
- `form-read-only` — Read-only values look different from disabled ones.
- `form-secret` — Secret inputs (API key) are masked with a show/hide toggle
  and never logged.
- `form-reset` — Every customizable value can be reset to default
  individually and all at once.

## 5. Layout, typography & color (MEDIUM)

- `lay-tokens-only` — Colors, spacing, sizes and font sizes come from theme
  tokens only — no raw hex or pixel numbers in components.
- `lay-spacing-scale` — Use the 4/8 px spacing scale (`space.*`).
- `lay-hierarchy` — Hierarchy through size, weight, spacing and contrast,
  not through color alone or decoration. Radius stays 0.
- `lay-one-primary` — At most one primary button per window/dialog;
  secondary actions are visually subordinate; destructive actions use the
  `danger` variant and sit apart from the primary action.
- `lay-truncation` — Prefer wrapping; when truncating use an ellipsis and
  show the full text in a tooltip.
- `lay-tabular-numbers` — Timers, countdowns, prices and quantities use
  right-aligned, fixed-width digits so they do not jitter while updating.
- `lay-line-length` — Long text (help, notes view) wraps at a readable width
  (~60–80 characters), not at the full window width on wide windows.
- `lay-scroll` — Avoid nested scroll regions; one scrollable area per window
  where possible; restore scroll position when reopening.
- `lay-text-case` — Sentence case for labels and headings, no trailing colons
  on section headings.
- `lay-icons` — One consistent icon set across all addons; no emoji as icons.
- `lay-dark-light` — Design and check Dark and Light together; Dark uses
  desaturated tones, not inverted colors.

## 6. Navigation & windows (HIGH)

- `nav-consistent-placement` — The same things live in the same places in
  every addon: window menu (Settings, Help, About), settings navigation on
  the left, primary action bottom-right in dialogs.
- `nav-active-state` — The current section in a settings/navigation list is
  clearly highlighted.
- `nav-state-preservation` — Windows reopen with their last section, scroll
  position, filters and expanded tree nodes.
- `nav-overflow-menu` — When actions do not fit, use a "…" overflow menu
  instead of cramming buttons.
- `nav-empty-destination` — If a section or feature is unavailable, explain
  why instead of hiding it silently.
- `nav-search` — Long lists (notes, events, settings) offer a search field at
  the top; matches are highlighted and their parents expanded.
- `nav-modal-use` — Modals only for decisions that block progress
  (confirmations, conflicts); never for navigation.

## 7. States & data (MEDIUM)

- `state-designed` — Empty, loading, error, disabled and offline states are
  designed for every view — never a blank area.
- `state-empty-action` — Empty states say what belongs here and offer the
  action to create it ("No notes yet — Create note").
- `state-error-recovery` — Errors state what happened and offer a recovery
  (retry, open settings, help link).
- `state-stale-data` — Data older than its validity shows its age
  ("Data from 2026-10-29") — see trailkit-core C8.
- `data-tables` — Tables have sortable columns with a visible sort
  indicator, aligned numbers, and keep the header visible while scrolling.
- `data-large-lists` — Lists with many rows (items, achievements) use
  clipped rendering (`ImGuiListClipper`) — only visible rows are drawn.
- `data-charts` (later, price features) — Line charts for trends, bars for
  comparisons, no pie charts beyond 5 slices; labeled axes with units and
  time granularity; subtle grid lines; hover tooltip with exact values;
  legend near the chart; no color-only series distinction; empty/error state
  instead of an empty frame; aggregate large series (daily OHLC) instead of
  plotting every point.
- `data-formatting` — Numbers, dates and times formatted per active language
  (en/de) and shown in the user's local time zone; gold values as
  gold/silver/copper with icons.

## 8. GW2 conventions (researched 2026-10-31)

Sources: source code + screenshots of popular open-source Nexus addons
(TaimiHUD, Hoard & Seek, Event Timers, kp_sync), the Nexus source
(v2026.10.25), the arcdps readme, community rarity color tables
(discretize-ui, gw2stacks, gw2inventory; the GW2 wiki is login-only).
Rules marked **(pending Q19–Q22)** await the user's decision — see
`docs/plans/NEXT_SESSION.md`.

### 8.1 Observed conventions (facts)

- **Window chrome:** title bar with the addon/window name, close `X` top
  right, Escape closes (`GUI_RegisterCloseOnEscape`). Tables with a header
  row; trees with ▸/▾ disclosure arrows.
- **Quick access icons:** 64×64 RGBA PNG in the Nexus style — parchment/beige
  fill with a dark outline — plus a lighter `_Hover` variant; tooltip names
  the action. Several addons offer a "show quick access icon" setting; a
  right-click context menu is possible (`add_quick_access_context_menu`).
- **Quick access bar position** is not draggable: Nexus sets it every frame
  from *Location* (Extend / Under / Bottom / Custom) + *Offset* (pixels from
  the game window's top-left). Overlays never leave the game window.
- **Nexus options tab:** most addons render their full settings there.
- **Item rarity:** item names and icon borders use rarity colors — Junk
  `#AAAAAA`, Basic `#FFFFFF`, Fine `#62A4DA`, Masterwork `#1A9306`, Rare
  `#FCD00B`, Exotic `#FFA405`, Ascended `#FB3E8D`, Legendary `#4C139D`
  (on dark backgrounds tools commonly use `#974EFF` for Legendary).
- **Gameplay state:** `NexusLink.IsGameplay` is false during loading
  screens, cutscenes and character select; overlays hide then (TaimiHUD).
- **arcdps habits:** hotkeys default to Alt+Shift modifiers; "moving requires
  modifiers" option; right-click context menu per window; option to hide in
  PvP.
- **Shared author themes** exist (Hoard & Seek: "Use Pie UI theme (if
  available)") — same idea as the trailkit theme.

### 8.2 trailkit rules derived from 8.1

- G1 Every window: name in the title bar, close `X`, Escape closes, position
  and size remembered (already §4).
- G2 Nexus options tab content **(pending Q19)** — proposal: header (name +
  version) + "Open settings" + "Open help" buttons only.
- G3 Rarity colors as `rarity.*` theme tokens, Legendary `#974EFF` in the dark
  theme, contrast ≥ 3:1 **(pending Q20)**.
- G4 Quick access icon in Nexus style, generated from SVG sources in the
  repo; "show quick access icon" setting, default on **(pending Q21)**.
- G5 All trailkit windows auto-hide while `IsGameplay == false`; per-addon
  setting "also show outside gameplay", default off **(pending Q22)**.
- G6 Default keybinds stay unbound (memo M14) to avoid clashes with the game
  and other addons; help pages explain where to bind them.

## Not applicable (deliberately excluded)

Web and mobile-only guidance from the upstream rule set: touch targets
44 pt / 48 dp, gestures and haptics, safe areas, mobile-first layout and
breakpoints, viewport/CSS/HTML/ARIA specifics, web performance (images,
fonts, bundles, CLS), framework stacks (React, Angular, Vue, …), landing
pages, font pairing and product palettes (our palette is the trailkit theme),
GSAP and decorative animation, deep links and browser navigation.
