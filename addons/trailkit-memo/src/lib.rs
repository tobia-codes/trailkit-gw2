//! trailkit-memo — a notes addon for Guild Wars 2, loaded by Raidcore Nexus.
//!
//! This crate is the app (ROADMAP R76): it wires the slices into the main
//! window toggled by a keybind (unbound by default, memo plan M14) or the
//! quick access icon (R55) and builds the DLL. What slices share lives in
//! `trailkit-memo-shared`. Platform-independent logic stays testable on any
//! host (R23); the Nexus glue in `addon` only compiles for Windows.

pub mod note_editing;
pub mod tree_drag;

#[cfg(windows)]
mod addon;
#[cfg(windows)]
mod main_window;

use trailkit_memo_drafts::i18n::DraftText;
use trailkit_memo_reset_schedule::i18n::ResetText;
use trailkit_memo_search::i18n::SearchText;
use trailkit_memo_shared::i18n::Text;

/// Every user-facing string of the app and all its slices in all languages,
/// for registering their characters with Nexus' font atlas
/// (nexus-addon-dev skill §1).
#[must_use]
pub fn all_texts() -> String {
    Text::all_texts() + &DraftText::all_texts() + &ResetText::all_texts() + &SearchText::all_texts()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_character_is_in_the_subset_fonts() {
        use trailkit_ui::fonts::{
            OPEN_SANS_BOLD, OPEN_SANS_BOLD_ITALIC, OPEN_SANS_ITALIC, OPEN_SANS_REGULAR, covers,
        };
        // UI strings plus the help pages (rendered with the same fonts).
        let texts = all_texts() + include_str!("../help/en.md") + include_str!("../help/de.md");
        for font in [
            OPEN_SANS_BOLD,
            OPEN_SANS_BOLD_ITALIC,
            OPEN_SANS_ITALIC,
            OPEN_SANS_REGULAR,
        ] {
            let missing: String = texts
                .chars()
                .filter(|character| !character.is_control() && !covers(font, *character))
                .collect();
            assert!(
                missing.is_empty(),
                "missing in the subset fonts (run `cargo xtask subset-fonts`): {missing}"
            );
        }
    }
}
