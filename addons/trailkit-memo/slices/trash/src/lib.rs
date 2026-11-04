//! Trash slice of trailkit-memo (ROADMAP R76, memo plan M12, M133–M151):
//! the trash column (header, two-line rows, context menu, empty state), the
//! preview toolbar and group preview of the note pane, and the texts of
//! deleting, the dialogs and the undo toast.
//!
//! Depends only on `trailkit-memo-shared`, never on another slice. The
//! trash operations and the undo history live in shared, because the app
//! applies them to the note tree. The views compile for Windows.

pub mod i18n;
pub mod labels;

#[cfg(windows)]
pub mod trash_view;
