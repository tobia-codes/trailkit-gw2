//! The right-click menu of links and chat-link chips (M210).

use super::*;

/// The narrowest window.
const MINIMUM: [f32; 2] = [320.0, 360.0];

/// A web link, a mail link and more text in the note body.
const LINKS: &str = "A [web link](https://a.de), [mail](mailto:a@b.de) and text.";

/// How far below the opening click the menu's second row lies.
const SECOND_ROW: f32 = 53.0;

/// The sample with links in its note.
fn links_note() -> Fixture {
    with_body(LINKS, |_| {})
}

/// A right click on `position`, with the mouse then on the menu's second row.
fn on_second_row(position: [f32; 2]) -> Vec<FrameInput> {
    let mut steps = right_click(position);
    steps.extend(hover([position[0] + 30.0, position[1] + SECOND_ROW]));
    steps
}

#[test]
fn link_menu_states() {
    let web_link = [265.0, 147.0];
    let mail_link = [335.0, 147.0];
    assert_state("link-menu", WIDE, links_note, &right_click(web_link));
    assert_state(
        "link-menu-hover",
        WIDE,
        links_note,
        &on_second_row(web_link),
    );
    assert_state("link-menu-mail", WIDE, links_note, &right_click(mail_link));
    let map_point = [340.0, 200.0];
    let coins = [335.0, 225.0];
    assert_state("chip-menu", WIDE, chips_note, &right_click(map_point));
    assert_state("chip-menu-coins", WIDE, chips_note, &right_click(coins));
    assert_state_in(
        Language::German,
        "min-de-link-menu",
        MINIMUM,
        selected,
        &right_click([233.0, 267.0]),
    );
    assert_state_in(
        Language::German,
        "min-de-chip-menu",
        MINIMUM,
        chips_note,
        &right_click([205.0, 288.0]),
    );
}

/// Links of several words, the second one long enough to wrap when narrow.
const WORD_LINKS: &str = "Vorher im [das ist ein tester](https://wiki.guildwars2.com) nachsehen. Siehe [Guide für die wöchentlichen Raid-Bosse](https://example.com/raids) und danach weiter.";

/// The sample with multi-word links in its note.
fn word_links_note() -> Fixture {
    with_body(WORD_LINKS, |_| {})
}

/// A link is underlined in one line through its spaces, broken only where it
/// wraps (R149).
#[test]
fn links_of_several_words_have_one_underline() {
    assert_state("link-words", WIDE, word_links_note, &[]);
    assert_state_in(
        Language::German,
        "min-de-link-words",
        MINIMUM,
        word_links_note,
        &[],
    );
}

#[test]
fn copying_from_the_menu_confirms_it() {
    let web_link = [265.0, 147.0];
    let row = [web_link[0] + 30.0, web_link[1] + SECOND_ROW];
    let mut steps = right_click(web_link);
    steps.extend(click(row));
    steps.extend(hover(NOWHERE));
    assert_state("link-menu-copied", WIDE, links_note, &steps);
}
