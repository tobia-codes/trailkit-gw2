# trailkit-ui

Shared design system for all trailkit Nexus addons: theme tokens (Dark and
Light), fonts, icons and the ImGui components every addon draws with. Runs in
the game (addons only). Rules, tokens and the current look: the
[`trailkit-ui` skill](../../.agents/skills/trailkit-ui/SKILL.md).

> **Status:** used by `trailkit-memo`. Components move in here once a second
> addon needs them (rule 7, ROADMAP R47); until then some patterns stay in
> memo (see the skill §3).

Third-party assets: `assets/fonts/open-sans/` (SIL Open Font License) and
`assets/fonts/lucide/` (ISC), each with its `LICENSE`. The embedded `.ttf`
files are **subsets** (memo plan M62) generated from the originals in
`assets/fonts/<font>/source/` with `cargo xtask subset-fonts`: text fonts keep
Latin up to Extended-B plus punctuation, the icon font only the icons in
`icons::Icon::ALL`. After adding an icon or a text with new characters, run
`cargo xtask subset-fonts`; tests fail until the subsets cover them.

## Contents

| Module | Content | Platform |
|--------|---------|----------|
| `color` | `Color` (RGBA) + WCAG contrast ratio | any |
| `theme` | `Theme` tokens, built-in `Theme::DARK` (default) / `Theme::LIGHT`, `ThemeKind` | any |
| `fonts` | shared font roles (body, bold, italic, headings, icons), embedded Open Sans + Lucide | any |
| `icons` | Lucide icon glyphs used by trailkit (add new ones here) | any |
| `text` | `TextStyle` — the text roles of `tokens.md` §7.1 (font and color, any platform); `draw_text`, `draw_text_wrapped`, `draw_truncated_text` ("…" plus full-text tooltip), `draw_icon`, `TextStyle::push` only for measuring, centering and inputs (R96) | any / Windows |
| `empty_state` | `draw_empty_state` — the one empty-state layout: optional icon in `text.disabled`, Body title, Secondary hint, optional buttons with one primary (Q283) | Windows |
| `editor` | the word-wrapping text editor: platform-independent `buffer` (text, cursor, undo) and `layout`, plus `highlight` and the drawing `view` (Windows) (M58–M60) | any / Windows |
| `text_search` | `SearchQuery`: whitespace-separated terms, case-insensitive, shared by list filtering and hit highlighting (M118–M120) | any |
| `help_topics` | `parse_help_topics`: the `##` topics of an addon's `help/<language>.md`, `subsection_counts` for the language parity test (R95) | any |
| `markdown` | markdown → blocks and styled spans; task toggling; `http(s)`-only links | any |
| `button` | `draw_button` (label centered by its capitals, M85), `draw_primary_button` (accent, `lay-one-primary`), `draw_primary_button_with_icon` (icon + label), `draw_icon_button` / `draw_icon_toggle` (borderless, tooltip), `draw_popup_icon_button` (hover on `bg.active`), `draw_icon_button_with_state`; every text button is a `Button` with `ButtonKind` and `ButtonState` (`Ready` / `Busy` / `Disabled { reason }`, R93), the `draw_*` functions are short forms, `button_width`, `icon_button_width`, `icon_label_button_width`; text buttons are at least `button.min_width` wide (M131) | Windows |
| `menu` | `draw_menu_item` (Lucide check mark), `begin_submenu` (Lucide chevron, opens flush beside the parent, M98/M99) | Windows |
| `dropdown` | `draw_dropdown` (themed select box, popup list with check mark, M87; variants `Field` and compact `Chip`, M102), `dropdown_width`, `dropdown_height` | Windows |
| `pane` | `draw_pane` — a content region on `bg.panel` with `pane.padding` on every side (`lay-pane-padding`) | Windows |
| `selection_bar` | `draw_selection_bar` — the brass bar on the left edge of the selected row (R71) | Windows |
| `tree_arrow` | `draw_tree_arrow` — the Lucide chevron of an expandable tree row (R71) | Windows |
| `title_bar` | `draw_window` — a window with the trailkit title bar (collapse chevron, bold title, close `x`) instead of ImGui's (R71) | Windows |
| `tooltip` | `draw_tooltip`, `draw_tooltip_with`: every tooltip, text centered by its capitals (M171) | Windows |
| `toolbar` | `begin_toolbar_group` (thin separator line, wraps when the group does not fit), `move_to_toolbar_end` (right-aligned items, wrap in narrow windows; `lay-responsive`) | Windows |
| `busy` | `BusyState` — when a button is `ButtonState::Busy`: busy while its action runs and at least `MINIMUM_BUSY` (0.6 s) after the click (`ix-busy-buttons`, M180) | any |
| `toast` | one short message with optional actions at the bottom of the window; the timer pauses on hover (M142, `ix-toasts`); error tone without timer, with counter, spinner and close button, and a success tone (M180) | Windows |
| `spinner` | `draw_spinner` — a rotating Lucide `loader-circle` for running actions (M180) | Windows |
| `splitter` | `draw_vertical_splitter` — resizable gap between two panes (accent line on hover/drag) | Windows |
| `text_layout` | `centered_text`, `indent_to_center`, `truncate_to_width` ("…", `lay-truncation`), `format_count` (allocation-free count labels) | Windows |
| `separator` | `draw_separator` — horizontal line that ends at the window/pane padding | Windows |
| `style_scope` | `StyleScope` — RAII guard that pushes a theme onto the shared ImGui context and pops it on drop; `StyleScope::transparent_frame` for borderless inputs (no per-frame allocation) | Windows |
| `font_registry` | registers the shared fonts and glyphs with Nexus, follows the `FontUI` size, `FontScope` guard | Windows |
| `markdown_view` | draws a parsed markdown document with word wrapping; returns task/link clicks and copies chat-link chips | Windows |
| `chat_link` | finds and decodes GW2 chat codes (`[&…]`) | any |
| `chat_link_chip` | draws a chat-link chip and its tooltip; texts come from the addon (`ChipContext`) | Windows |
| `cursor_extent` | measures the mouse cursor (GW2 draws a large one) so `StyleScope` places tooltips right of it | Windows |
| `glyph_metrics` | where glyphs really sit in a line (centering by capital letters) | Windows |

## Usage

```rust
use trailkit_ui::{style_scope::StyleScope, theme::ThemeKind};

fn render(ui: &nexus::imgui::Ui) {
    let _theme = StyleScope::push(ui, &ThemeKind::default().theme());
    // draw windows here; the style is restored when `_theme` is dropped
}
```

## Test

```sh
cargo test -p trailkit-ui   # logic tests, incl. the 4.5:1 text contrast check for every theme
cargo xtask ui-test         # ImGui tests (Windows binaries via WSL): editor snapshots and input
```

Snapshot references live in `tests/snapshots/`; every state of a shared
component has one in `tests/component_states.rs` (Dark and Light), see
[`trailkit-ui-testing`](../trailkit-ui-testing/README.md).

## Rules

- No raw colors or sizes: everything comes from `Theme` tokens. A new token is
  a decision ([`tokens.md`](../../.agents/skills/trailkit-ui/references/tokens.md));
  `cargo xtask tokens` regenerates the token reference and a test fails while
  it is out of date.
- Drawing code runs on the render thread: no allocations in steady state, no blocking, no I/O.
- Every component state gets a snapshot in Dark and Light in the same change.
- Texts are passed in by the addon; this crate has no strings of its own.

