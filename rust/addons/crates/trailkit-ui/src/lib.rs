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
//! - [`text`]: the text roles (`TextStyle`) through which addons draw
//!   text and icons (R96), plus fonts, icons, text layout and search;
//! - [`layout`], [`controls`], [`lists`], [`feedback`], [`help`]: the shared
//!   components, grouped by concern (ROADMAP R134; mostly Windows only);
//! - [`markdown`] and [`editor`]: the read-only markdown view and the text
//!   editor;
//! - [`drawing`]: low-level ImGui support such as the theme style scope.
//!
//! Shared components move in here only once a second addon needs them
//! (AGENTS.md rule 7, ROADMAP R47).

pub mod chat_link;
pub mod color;
pub mod controls;
pub mod drawing;
pub mod editor;
pub mod feedback;
pub mod help;
pub mod layout;
pub mod lists;
pub mod markdown;
pub mod text;
pub mod theme;
