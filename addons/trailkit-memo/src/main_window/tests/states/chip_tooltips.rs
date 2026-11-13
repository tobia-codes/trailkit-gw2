//! A chat-link chip's tooltip in the note view and in the editor (ROADMAP
//! R104, Q356): the editor once crashed the game when a chip tooltip took a
//! second draw list. With `survive_navigation` each state also runs with
//! keyboard navigation planned while the tooltip shows.

use super::*;

/// Over the "Map point #56" chip in the note view of [`chips_note`].
const VIEW_CHIP: [f32; 2] = [340.0, 200.0];
/// Center of the note toolbar's "Edit" button.
const EDIT_BUTTON: [f32; 2] = [598.0, 64.0];
/// Over the "Map point #56" chip in the editor of [`chips_note`].
const EDITOR_CHIP: [f32; 2] = [125.0, 185.0];

#[test]
fn chip_tooltips() {
    assert_state("chip-tooltip", WIDE, chips_note, &hover(VIEW_CHIP));
    let mut editing = click(EDIT_BUTTON);
    editing.extend(hover(EDITOR_CHIP));
    assert_state("editor-chip-tooltip", WIDE, chips_note, &editing);
}
