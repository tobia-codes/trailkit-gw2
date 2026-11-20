//! A click on an empty spot of the note list (M216).

use super::*;

/// A spot of the list below its last row.
const BELOW_THE_ROWS: [f32; 2] = [100.0, 300.0];

#[test]
fn a_click_below_the_rows_selects_nothing_and_keeps_the_note() {
    assert_state("click-empty-spot", WIDE, selected, &click(BELOW_THE_ROWS));
}
