//! Indent guides in the note list (ROADMAP R156): a line below each expanded
//! group's chevron down to its last descendant, with the fills of nested
//! rows starting right of the lines.

use super::*;

/// The row of the nested group "Wing 1" inside "Raids".
const WING_ROW: [f32; 2] = [100.0, 217.0];
/// The row of "Sabetha", the last note inside "Wing 1".
const SABETHA_ROW: [f32; 2] = [100.0, 273.0];

/// The sample with a second level: "Raids" › "Wing 1" › "Gorseval",
/// "Sabetha"; the entry titled `selected` is selected.
fn nested(selected: &'static str) -> impl Fn() -> Fixture {
    move || {
        let mut document = sample_document();
        let mut wing = Group::new("Wing 1".to_owned());
        for title in ["Gorseval", "Sabetha"] {
            wing.children
                .push(Node::Note(note(title, "", ResetSchedule::Never)));
        }
        let mut chosen = (selected == wing.title).then_some(NodeId::Group(wing.id));
        for child in &wing.children {
            if let Node::Note(entry) = child
                && entry.title == selected
            {
                chosen = Some(NodeId::Note(entry.id));
            }
        }
        if let Some(Node::Group(raids)) = document.root.last_mut() {
            raids.children.push(Node::Group(wing));
        }
        let ui_state = UiState {
            selected: chosen,
            ..selected_first(&document)
        };
        memo_window(document, ui_state)
    }
}

#[test]
fn guides_leave_the_lines_free_of_fills() {
    assert_state("indent-guides", WIDE, nested("Gorseval"), &hover(NOWHERE));
    assert_state(
        "indent-guides-hover",
        WIDE,
        nested("Gorseval"),
        &hover(SABETHA_ROW),
    );
    assert_state(
        "indent-guides-group",
        WIDE,
        nested("Wing 1"),
        &hover(NOWHERE),
    );
    assert_state(
        "indent-guides-drag",
        WIDE,
        nested("Gorseval"),
        &drag(WING_ROW, DAILIES_ROW),
    );
    assert_state(
        "min-indent-guides",
        MINIMUM_WINDOW,
        nested("Gorseval"),
        &hover(NOWHERE),
    );
}
