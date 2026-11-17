# design

Design sources shared by the addons and the coming website: the design
tokens, the only place where colors and sizes are written down.

## Status

Token files per ROADMAP R118 (Q453–Q458); the CSS output follows with
`apps/web`.

## Contents

| File | Tokens | Section |
|------|--------|---------|
| [`tokens/colors.toml`](tokens/colors.toml) | Every color of the Dark and Light themes: surfaces, text, accent, status, borders, shadows, chat-link chips, coins, events | `[shared]` |
| [`tokens/border.toml`](tokens/border.toml) | Border widths | `[shared]` |
| [`tokens/radius.toml`](tokens/radius.toml) | Corner radii | `[shared]` |
| [`tokens/spacing.toml`](tokens/spacing.toml) | Spacing scale | `[addon]`, later `[web]` |
| [`tokens/components.toml`](tokens/components.toml) | Layout of list, pane, button and menu | `[addon]` |
| [`tokens/shadow.toml`](tokens/shadow.toml) | Shadow offset (its color is in `colors.toml`) | `[addon]`, later `[web]` |
| [`tokens/typography.toml`](tokens/typography.toml) | Font sizes and line height | `[addon]`, later `[web]` |

`[shared]` tokens are used by the addons and the web, `[addon]` and `[web]`
tokens by one of them only.

## Usage

```sh
cd rust
cargo xtask tokens   # generates trailkit-ui/src/theme/generated{.rs,/} and token-values.md
```

A test fails in `cargo test` while a generated file is out of date.

## Rules

- A token is one inline table: its `name` (as in the docs, e.g. `bg.window`),
  a fixed `dark` and `light` color or one `value` with its `unit` (`px` by
  default, `font_size`, `body_size`), and its `purpose`. Values are never
  derived (Q454); a comment above a token explains a non-obvious value.
- A color is `"#RRGGBB"`, `"#RRGGBBAA"` or `{ hex = "#RRGGBB", alpha = 0.5 }`.
- Adding or changing a token is a decision first; how:
  [`tokens.md` §8](../.agents/skills/trailkit-ui/references/tokens.md).

## Links

ROADMAP R112, R118; the generated reference:
[`token-values.md`](../.agents/skills/trailkit-ui/references/token-values.md).
