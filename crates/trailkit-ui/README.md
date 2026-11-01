# trailkit-ui

Shared design system for all trailkit Nexus addons. Rules and token
documentation: the [`trailkit-ui` skill](../../.agents/skills/trailkit-ui/SKILL.md).

> **Status:** phase 1 — theme tokens, style scope, shared fonts, icons and
> the markdown view. Other components move in here once a second addon
> needs them (AGENTS.md rule 7, ROADMAP R47).

Third-party assets: `assets/fonts/open-sans/` (SIL Open Font License) and
`assets/fonts/lucide/` (ISC), each with its `LICENSE`.

## Contents

| Module | Content | Platform |
|--------|---------|----------|
| `color` | `Color` (RGBA) + WCAG contrast ratio | any |
| `theme` | `Theme` tokens, built-in `Theme::DARK` (default) / `Theme::LIGHT`, `ThemeKind` | any |
| `fonts` | shared font roles (body, bold, italic, headings, icons), embedded Open Sans + Lucide | any |
| `icons` | Lucide icon glyphs used by trailkit (add new ones here) | any |
| `markdown` | markdown → blocks and styled spans; task toggling; `http(s)`-only links | any |
| `button` | `draw_primary_button` (accent, `lay-one-primary`), `button_width` | Windows |
| `pane` | `draw_pane` — a content region on `bg.panel` with `pane.padding` on every side (`lay-pane-padding`) | Windows |
| `splitter` | `draw_vertical_splitter` — resizable gap between two panes (accent line on hover/drag) | Windows |
| `separator` | `draw_separator` — horizontal line that ends at the window/pane padding | Windows |
| `style_scope` | `StyleScope` — RAII guard that pushes a theme onto the shared ImGui context and pops it on drop (no per-frame allocation) | Windows |
| `font_registry` | registers the shared fonts and glyphs with Nexus, follows the `FontUI` size, `FontScope` guard | Windows |
| `markdown_view` | draws a parsed markdown document with word wrapping; returns task/link clicks | Windows |

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

Snapshot references live in `tests/snapshots/`; see `crates/trailkit-ui-testing`.
