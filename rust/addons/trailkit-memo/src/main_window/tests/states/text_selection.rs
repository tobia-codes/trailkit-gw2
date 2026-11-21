//! Snapshots of selected text in the reading view (M217, mock
//! https://claude.ai/artifact/1FeMLueyes6L11SiUyFbgi P1–P6): a selection over
//! lines and a list's markers, its right-click menu alone and with a chip's
//! entries, wrapped German text at the narrowest window and a selection over
//! find hits.

use super::*;

/// The narrowest window.
const MINIMUM: [f32; 2] = [320.0, 360.0];

/// Inside "Globs" of the sample note, before " of Ectoplasm".
const BEFORE_OF: [f32; 2] = [300.0, 195.0];
/// After "first." on the sample note's last line.
const AFTER_FIRST: [f32; 2] = [372.0, 243.0];

/// A drag from `from` to `to`, released there.
fn select(from: [f32; 2], to: [f32; 2]) -> Vec<FrameInput> {
    let mut steps = drag(from, to);
    steps.extend(hover(to));
    steps
}

/// A selection from `from` to `to`, then a right click at `menu_at`.
fn select_and_open_menu(from: [f32; 2], to: [f32; 2], menu_at: [f32; 2]) -> Vec<FrameInput> {
    let mut steps = select(from, to);
    steps.extend(right_click(menu_at));
    steps
}

/// German text that wraps at the narrowest window, with a link.
const WRAPPING_GERMAN: &str = "## Ektos\n- [x] **250** Kugeln aus Ektoplasma\n\nErst im [Wiki](https://wiki.guildwars2.com) nachsehen, dann beim Handelsposten kaufen.";

#[test]
fn selection_states() {
    assert_state("selection", WIDE, selected, &select(BEFORE_OF, AFTER_FIRST));
    assert_state(
        "selection-menu",
        WIDE,
        selected,
        &select_and_open_menu(BEFORE_OF, AFTER_FIRST, [270.0, 219.0]),
    );
    // "Copy" chosen: the toast confirms it like a chip click.
    let mut copied = select_and_open_menu(BEFORE_OF, AFTER_FIRST, [270.0, 219.0]);
    copied.extend(click([440.0, 240.0]));
    assert_state("selection-copied", WIDE, selected, &copied);
    // The chip of the first item lies inside the selection.
    assert_state(
        "selection-chip-menu",
        WIDE,
        chips_note,
        &select_and_open_menu([243.0, 195.0], [330.0, 219.0], [500.0, 195.0]),
    );
}

#[test]
fn selection_in_german_at_the_minimum_width() {
    let make = || with_body(WRAPPING_GERMAN, |_| {});
    let (from, to) = ([230.0, 218.0], [250.0, 314.0]);
    assert_state_in(
        Language::German,
        "min-de-selection",
        MINIMUM,
        make,
        &select(from, to),
    );
    assert_state_in(
        Language::German,
        "min-de-selection-menu",
        MINIMUM,
        make,
        &select_and_open_menu(from, to, [150.0, 314.0]),
    );
}

#[test]
fn selection_over_find_hits() {
    let make = || {
        let mut fixture = selected();
        fixture.0.find_text("ecto");
        fixture
    };
    assert_state(
        "selection-find",
        WIDE,
        make,
        // From inside the filled first hit to inside the framed second one.
        &select([232.0, 200.0], [330.0, 244.0]),
    );
}
