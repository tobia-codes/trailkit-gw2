//! trailkit-ui — the shared design system of all trailkit Nexus addons.
//!
//! Source of truth for the rules: the `trailkit-ui` skill
//! (`.agents/skills/trailkit-ui/SKILL.md`). This crate provides:
//!
//! - [`chat_link`]: finding and decoding GW2 chat links (`chat_link_chip`
//!   draws them, Windows only);
//! - [`color`]: an RGBA color type with WCAG contrast helpers;
//! - [`theme`]: the semantic theme tokens and the built-in `Dark` / `Light`
//!   themes;
//! - `pane` and `separator` (Windows only): padded content regions and a
//!   separator that respects that padding (`lay-pane-padding`);
//! - `style_scope` (Windows only): an RAII guard that applies a theme to the
//!   shared ImGui context and restores the previous style on drop.
//!
//! Shared components move in here only once a second addon needs them
//! (AGENTS.md rule 7, ROADMAP R47).

pub mod chat_link;
pub mod color;
pub mod editor;
pub mod fonts;
pub mod icons;
pub mod markdown;
pub mod text_search;
pub mod theme;

#[cfg(windows)]
pub mod button;
#[cfg(windows)]
pub mod chat_link_chip;
#[cfg(windows)]
pub mod cursor_extent;
#[cfg(windows)]
pub mod dropdown;
#[cfg(windows)]
pub mod font_registry;
#[cfg(windows)]
pub mod glyph_metrics;
#[cfg(windows)]
pub mod markdown_view;
#[cfg(windows)]
pub mod menu;
#[cfg(windows)]
pub mod pane;
#[cfg(windows)]
pub mod selection_bar;
#[cfg(windows)]
pub mod separator;
#[cfg(windows)]
pub mod splitter;
#[cfg(windows)]
pub mod style_scope;
#[cfg(windows)]
pub mod text_layout;
#[cfg(windows)]
pub mod title_bar;
#[cfg(windows)]
pub mod toast;
#[cfg(windows)]
pub mod tool_rail;
#[cfg(windows)]
pub mod toolbar;
#[cfg(windows)]
pub mod tree_arrow;
