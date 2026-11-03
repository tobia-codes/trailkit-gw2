//! Reset-schedule slice of trailkit-memo (ROADMAP R76, memo plan M13,
//! M86–M105): the game's reset times, unticking the tasks of due reset
//! notes, the reset badge and the note kind dropdown with their colors and
//! texts.
//!
//! Depends only on `trailkit-memo-shared`, never on another slice. The
//! period type itself ([`ResetPeriod`]) belongs to the shared note model;
//! the ImGui-free logic is tested on any host, the dropdown compiles for
//! Windows.
//!
//! [`ResetPeriod`]: trailkit_memo_shared::note_tree::ResetPeriod

pub mod due_resets;
pub mod i18n;
pub mod note_kind;
pub mod schedule;
