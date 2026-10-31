# trailkit-ui

Shared design system for all trailkit Nexus addons. Rules and token
documentation: the [`trailkit-ui` skill](../../.agents/skills/trailkit-ui/SKILL.md).

> **Status:** phase 0 — theme tokens + style scope only. Shared components
> move in here once a second addon needs them (AGENTS.md rule 7, ROADMAP R47).

## Contents

| Module | Content | Platform |
|--------|---------|----------|
| `color` | `Color` (RGBA) + WCAG contrast ratio | any |
| `theme` | `Theme` tokens, built-in `Theme::DARK` (default) / `Theme::LIGHT`, `ThemeKind` | any |
| `style_scope` | `StyleScope` — RAII guard that pushes a theme onto the shared ImGui context and pops it on drop (no per-frame allocation) | Windows |

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
cargo test -p trailkit-ui   # includes the 4.5:1 text contrast check for every theme
```
