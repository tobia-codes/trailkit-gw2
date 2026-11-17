# trailkit-ui

Shared design system for all trailkit Nexus addons: theme tokens (Dark and
Light), fonts, icons and the ImGui components every addon draws with. Runs in
the game (addons only). Rules, tokens and the current look: the
[`trailkit-ui` skill](../../../../.agents/skills/trailkit-ui/SKILL.md).

> **No Nexus API here (ROADMAP R136):** the crate draws with ImGui only (it
> takes ImGui from the `nexus` crate's re-export, the same version as the host)
> and never calls Nexus; everything that talks to Nexus is in
> [`trailkit-nexus-bridge`](../trailkit-nexus-bridge/README.md) or in each
> addon's `nexus_bridge` module.

> **Status:** used by `trailkit-memo`. Components move in here once a second
> addon needs them (rule 7, ROADMAP R47); until then some patterns stay in
> memo (see the skill §3).

Third-party assets: `assets/fonts/open-sans/` (SIL Open Font License) and
`assets/fonts/lucide/` (ISC), each with its `LICENSE`. The embedded `.ttf`
files are **subsets** (memo plan M62) generated from the originals in
`assets/fonts/<font>/source/` with `cargo xtask subset-fonts`: text fonts keep
Latin up to Extended-B plus punctuation, the icon font only the icons in
`text::icons::Icon::ALL`. After adding an icon or a text with new characters, run
`cargo xtask subset-fonts`; tests fail until the subsets cover them.

## Contents

Modules are grouped by concern (ROADMAP R134): `text`, `layout`, `controls`, `lists`,
`feedback`, `markdown`, `chat_link`, `help` and `drawing` (low-level ImGui support); `color`,
`theme` and `editor` stand alone.

| Module | Content | Platform |
|--------|---------|----------|
| `color` | `Color` (RGBA) + WCAG contrast ratio | any |
| `theme` | `Theme` and `ThemeKind`; the token types in `theme/colors.rs` (color groups, R107 darkening steps), `theme/layout.rs` (`Spacing`, `Radii`, `ListLayout`, `PaneLayout`, `ButtonLayout`, `MenuLayout`) and `theme/typography.rs`; `Theme::DARK` (default) / `Theme::LIGHT` in the generated `theme/generated.rs`, assembled from one module per token file in `theme/generated/` (R118, R133) | any |
| `editor` | the word-wrapping text editor: platform-independent `buffer` (text, cursor, undo) and `layout`, plus `highlight` and the drawing `view` (Windows; `keyboard` handles keys, `lines` draws the visible lines) (M58–M60) | any / Windows |
| `text` | `TextStyle` — the text roles of `tokens.md` §7.1 (font and color, any platform); `draw_text`, `draw_text_wrapped`, `draw_truncated_text` ("…" plus full-text tooltip), `draw_icon`, `draw_icon_text` (icon in front of a text in any role, icon and capitals centered on each other, one line or wrapped; Q325), `TextStyle::push` only for measuring, centering and inputs (R96) | any / Windows |
| `text::fonts` | shared font roles (body, bold, italic, headings, icons), embedded Open Sans + Lucide | any |
| `text::font_registry` | which `ImFont` each font role uses (`install_font`, filled by the host's bridge) and the `FontScope` guard; registering with Nexus is in [`trailkit-nexus-bridge`](../trailkit-nexus-bridge/README.md) | Windows |
| `text::glyph_metrics` | where glyphs really sit in a line (centering by capital letters); `icon_visible_box` — where an icon's pixels are, for buttons, menus and icon texts | Windows |
| `text::text_layout` | `centered_text`, `indent_to_center`, `truncate_to_width` ("…", `lay-truncation`), `format_count` (allocation-free count labels), `body_characters_width` (a width in body characters, e.g. a window's minimum width) | Windows |
| `text::text_search` | `SearchQuery`: whitespace-separated terms, case-insensitive, shared by list filtering and hit highlighting (M118–M120) | any |
| `text::icons` | Lucide icon glyphs used by trailkit (add new ones here) | any |
| `layout::pane` | `draw_pane` — a content region on `bg.panel` with `pane.padding` on every side (`lay-pane-padding`); its size comes from the caller | Windows |
| `layout::pane_columns` | `draw_resizable_pane_columns` — panes side by side with a splitter in each gap: fixed columns with widths in body characters, the last one fills; memo's main window and the help window (R101) | Windows |
| `layout::splitter` | `draw_vertical_splitter` — resizable gap between two panes (accent line on hover/drag) | Windows |
| `layout::separator` | `draw_separator` — horizontal line that ends at the window/pane padding | Windows |
| `layout::toolbar` | `begin_toolbar_group` (thin separator line, wraps when the group does not fit), `move_to_toolbar_end` (right-aligned items, wrap in narrow windows; `lay-responsive`) | Windows |
| `layout::page` | `draw_page` — a page head inside a pane: Heading1 title with an optional `TitleTrailing` widget, optional Secondary line, separator, scrolling content; memo's note view and the help window (R101) | Windows |
| `layout::title_bar` | `draw_window` — a window with the trailkit title bar (collapse chevron, bold title, close `x`) instead of ImGui's (R71) | Windows |
| `layout::empty_state` | `draw_empty_state` — the one empty-state layout: optional icon in `text.disabled`, Body title, Secondary hint, optional buttons with one primary (Q283) | Windows |
| `controls::button` | `draw_button` (label centered by its capitals, M85), `draw_primary_button` (accent, `lay-one-primary`), `draw_primary_button_with_icon` (icon + label), `draw_icon_button` / `draw_icon_toggle` (borderless, tooltip), `draw_popup_icon_button` (hover on `bg.active`), `draw_icon_button_with_state`; every text button is a `Button` with `ButtonKind` and `ButtonState` (`Ready` / `Busy` / `Disabled { reason }`, R93), the `draw_*` functions are short forms, `button_width`, `icon_button_width`, `icon_label_button_width`; `Button::opens_link` adds `external-link` after the label (`lay-button-icon-position`, R100); text buttons are at least `button.min_width` wide (M131) | Windows |
| `controls::busy` | `BusyState` — when a button is `ButtonState::Busy`: busy while its action runs and at least `MINIMUM_BUSY` (0.6 s) after the click (`ix-busy-buttons`, M180) | any |
| `controls::spinner` | `draw_spinner` — a rotating Lucide `loader-circle` for running actions (M180) | Windows |
| `controls::dropdown` | `draw_dropdown` (themed select box: the selected content in a button, the option list with a check mark below it, M87, R71), `dropdown_width` | Windows |
| `controls::menu` | `draw_menu_item` (Lucide check mark), `begin_submenu` (Lucide chevron, opens flush beside the parent, M98/M99) | Windows |
| `controls::tooltip` | `draw_tooltip`, `draw_tooltip_with`: every tooltip, text centered by its capitals (M171) | Windows |
| `lists::list_row` | `draw_list_row` — a full-width selectable row: truncated title (Body or BodyStrong) with tooltip, trailing text, optional detail line; trash and help topics (R96, Q305) | Windows |
| `lists::selection_bar` | `draw_row_state` / `draw_row_state_in` — the one function for a row's hover, pressed and selection fill and the accent bar on its left edge; `hide_square_row_highlight` for the row's item (R71, Q306) | Windows |
| `lists::tree_arrow` | `draw_tree_arrow` — the Lucide chevron of an expandable tree row (R71) | Windows |
| `feedback::toast` | one short message with optional actions at the bottom of the window; the timer pauses on hover (M142, `ix-toasts`); error tone without timer, with counter, spinner and close button, and a success tone (M180) | Windows |
| `feedback::status_line` | `draw_status_line` — `draw_icon_text` with the status icon and a wrapping text in the status color (`Status`: Success, Warning, Danger, Info), Body in dialogs, Small in the log export (R96) | Windows |
| `markdown` | markdown → blocks and styled spans; task toggling; `http(s)`-only links | any |
| `markdown::markdown_view` | draws a parsed markdown document block by block; `inline_text` wraps words, links and chat-link chips and marks search hits; returns task/link clicks and copies chat-link chips | Windows |
| `chat_link` | finds and decodes GW2 chat codes (`[&…]`) | any |
| `chat_link::chat_link_chip` | draws a chat-link chip and its tooltip; texts come from the addon (`ChipContext`) | Windows |
| `help::help_window` | `HelpWindow` — the help window of every addon: Help tab (topic list + topic as resizable pane columns) and Report a problem tab (export, status line, GitHub), each page with the page head; the addon passes texts, help markdown and the export state, carries out the returned `HelpAction` and saves the `HelpWindowState` (tab, topic, list width; R95, R100, R101) | Windows |
| `help::help_topics` | `parse_help_topics`: the `##` topics of an addon's `help/<language>.md`, `subsection_counts` for the language parity test (R95) | any |
| `drawing::style_scope` | `StyleScope` — RAII guard that pushes a theme onto the shared ImGui context and pops it on drop; `StyleScope::transparent_frame` for borderless inputs (no per-frame allocation) | Windows |
| `drawing::cursor_extent` | measures the mouse cursor (GW2 draws a large one) so `StyleScope` places tooltips right of it | Windows |

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
component has one in `tests/component-states/` (Dark and Light), see
[`trailkit-ui-testing`](../trailkit-ui-testing/README.md).

## Rules

- No raw colors or sizes: everything comes from `Theme` tokens. A new token is
  a decision ([`tokens.md`](../../../../.agents/skills/trailkit-ui/references/tokens.md));
  Values live in [`design/tokens/`](../../../../design/tokens/) (R118);
  `cargo xtask tokens` generates `src/theme/generated.rs`, `src/theme/generated/`
  and the token reference, and a test fails while any of them is out of date.
- Drawing code runs on the render thread: no allocations in steady state, no blocking, no I/O.
- Every component state gets a snapshot in Dark and Light in the same change.
- Texts are passed in by the addon; this crate has no strings of its own.

