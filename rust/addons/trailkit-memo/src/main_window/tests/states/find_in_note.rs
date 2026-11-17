//! Snapshots of find in the open note (M198, mockup
//! https://claude.ai/artifact/S2rNsD4TnkoemgMULKerJc variant A): the bar in
//! the reading view and the editor, with hits, without and empty, also at the
//! narrowest window in German.

use super::*;

/// The sample's first note with the find bar open on `text`, after `steps`
/// to the next hit.
fn finding(text: &str, steps: usize) -> Fixture {
    let mut fixture = selected();
    fixture.0.find_text(text);
    // The hits are known after the first frame; stepping uses their count.
    fixture.0.find.hit_count = 2;
    for _ in 0..steps {
        fixture.0.step_find(true);
    }
    fixture
}

/// The editor with the find bar open on `text`.
fn finding_in_editor(text: &str) -> Fixture {
    let mut fixture = editing();
    fixture.0.find_text(text);
    fixture
}

#[test]
fn find_bar_states() {
    assert_state("find-empty", WIDE, || finding("", 0), &[]);
    assert_state("find-first-hit", WIDE, || finding("ecto", 0), &[]);
    assert_state("find-second-hit", WIDE, || finding("ecto", 1), &[]);
    assert_state("find-no-results", WIDE, || finding("xyzzy", 0), &[]);
    assert_state("find-editor", WIDE, || finding_in_editor("ecto"), &[]);
}

#[test]
fn find_bar_in_german_at_the_minimum_width() {
    let window = MINIMUM_WINDOW;
    let german = Language::German;
    assert_state_in(
        german,
        "min-de-find-hits",
        window,
        || finding("ecto", 1),
        &[],
    );
    assert_state_in(
        german,
        "min-de-find-no-results",
        window,
        || finding("xyzzy", 0),
        &[],
    );
    assert_state_in(
        german,
        "min-de-find-editor",
        window,
        || finding_in_editor("ecto"),
        &[],
    );
}
