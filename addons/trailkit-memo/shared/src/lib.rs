//! What the slices of trailkit-memo share (ROADMAP R76): the note tree and
//! its types, the persisted documents (settings, UI state, drafts), the
//! storage worker that writes them, reset periods and localization.
//!
//! Slices depend on this crate, never on each other.

pub mod drafts;
pub mod i18n;
pub mod note_tree;
pub mod reset_schedule;
pub mod settings;
pub mod storage_worker;
pub mod ui_state;
