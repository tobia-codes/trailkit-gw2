//! What the slices of trailkit-memo share (ROADMAP R76): the note tree and
//! its types, the persisted documents (settings, UI state, drafts), the
//! storage worker that writes them, localization and the neutral types slices
//! hand each other through the app (`list_filter`, `row_badge`).
//!
//! Slices depend on this crate, never on each other.

pub mod drafts;
pub mod i18n;
pub mod list_filter;
pub mod note_tree;
pub mod row_badge;
pub mod save_text;
pub mod settings;
pub mod storage_worker;
pub mod ui_state;
pub mod undo;
