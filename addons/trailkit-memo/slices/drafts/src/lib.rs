//! Drafts slice of trailkit-memo (ROADMAP R76, memo plan M54, M55): the
//! hint above a note with a kept draft, offering to restore or discard it.
//!
//! Depends only on `trailkit-memo-shared`, never on another slice. The
//! drafts document itself lives in shared, because the storage worker
//! writes it and the editor creates drafts. The hint compiles for Windows.

pub mod i18n;

#[cfg(windows)]
pub mod draft_hint;
