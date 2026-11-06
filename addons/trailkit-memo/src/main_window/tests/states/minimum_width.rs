//! Snapshots at the narrowest window (M168, Q126), which nothing may run past
//! or clip (`lay-responsive`), also with the longer German texts. Hovers are
//! left out: they do not change the layout.

use super::*;

/// Entries in the trash, none chosen.
fn trash_entries() -> Fixture {
    trash_view(document_with_trash([0, 25]), false)
}

/// Entries in the trash, the first one shown.
fn trash_entry_chosen() -> Fixture {
    trash_view(document_with_trash([0, 25]), true)
}

/// The sample with the search field shown.
fn searching() -> Fixture {
    selected_with(|state| state.search_visible = true)
}

/// Center of the search field at the minimum width.
const MINIMUM_SEARCH_FIELD: [f32; 2] = [55.0, 68.0];
/// Center of the kind dropdown at the minimum width.
const MINIMUM_KIND_FIELD: [f32; 2] = [180.0, 145.0];
/// Center of the note toolbar's `⋯` button at the minimum width.
const MINIMUM_NOTE_MENU_BUTTON: [f32; 2] = [212.0, 65.0];
/// Center of the "Dailies" row at the minimum width.
const MINIMUM_DAILIES_ROW: [f32; 2] = [60.0, 138.0];
/// Center of the "Raids" row at the minimum width.
const MINIMUM_RAIDS_ROW: [f32; 2] = [60.0, 170.0];

/// Every view and dialog at the narrowest window (M168), which nothing may run
/// past or clip (`lay-responsive`). Hovers are left out: they do not change the
/// layout.
#[test]
fn views_at_the_minimum_width() {
    let views: [NamedFixture; 24] = [
        ("no-selection", || {
            memo_window(sample_document(), UiState::default())
        }),
        ("empty", || {
            memo_window(NotesDocument::default(), UiState::default())
        }),
        ("daily-note", || selected_titled("Dailies")),
        ("search-shown", searching),
        ("list-hidden", || {
            selected_with(|state| state.note_list_hidden = true)
        }),
        ("compact", || {
            selected_with(|state| state.compact_note_view = true)
        }),
        ("scrolled-note", scrolled_note),
        ("chips", chips_note),
        ("editor", editing),
        ("editor-preview", editing_with_preview),
        ("dialog-unsaved", unsaved_dialog),
        ("dialog-delete-group", delete_group_dialog),
        ("dialog-delete-for-good", delete_for_good_dialog),
        ("dialog-empty-trash", empty_trash_dialog),
        ("trash-nothing-in-it", || {
            trash_view(sample_document(), false)
        }),
        ("trash-nothing-selected", trash_entries),
        ("trash-selected", trash_entry_chosen),
        ("problem-recovered", || {
            with_problem(NotesProblem::Recovered("notes.corrupt.json".to_owned()))
        }),
        ("problem-newer", || with_problem(NotesProblem::NewerVersion)),
        ("problem-inaccessible", || {
            with_problem(NotesProblem::Inaccessible)
        }),
        ("draft-hint", with_draft),
        ("window-menu-in-trash", || {
            let mut fixture = trash_entries();
            fixture.0.window_menu_open = true;
            fixture
        }),
        ("window-menu-editing", || {
            let mut fixture = editing();
            fixture.0.window_menu_open = true;
            fixture
        }),
        ("window-menu-no-notes", || {
            let mut fixture = memo_window(NotesDocument::default(), UiState::default());
            fixture.0.window_menu_open = true;
            fixture
        }),
    ];
    for (name, make) in views {
        assert_state(&format!("min-{name}"), MINIMUM_WINDOW, make, &[]);
    }
}

/// Menus and text input at the narrowest window (M168).
#[test]
fn menus_at_the_minimum_width() {
    let window = MINIMUM_WINDOW;
    let field = MINIMUM_SEARCH_FIELD;
    assert_state("min-search-hits", window, searching, &typing(field, "ecto"));
    assert_state(
        "min-search-no-hits",
        window,
        searching,
        &typing(field, "xyzzy"),
    );
    assert_state(
        "min-kind-open",
        window,
        selected,
        &click(MINIMUM_KIND_FIELD),
    );
    assert_state(
        "min-note-menu",
        window,
        selected,
        &click(MINIMUM_NOTE_MENU_BUTTON),
    );
    assert_state(
        "min-menu-note",
        window,
        selected,
        &right_click(MINIMUM_DAILIES_ROW),
    );
    assert_state(
        "min-menu-note-move-submenu",
        window,
        selected,
        &menu_row_hover(MINIMUM_DAILIES_ROW, 1),
    );
    assert_state(
        "min-menu-note-reset-submenu",
        window,
        selected,
        &menu_row_hover(MINIMUM_DAILIES_ROW, 2),
    );
    assert_state(
        "min-menu-group",
        window,
        selected,
        &right_click(MINIMUM_RAIDS_ROW),
    );
    assert_state(
        "min-menu-list",
        window,
        selected,
        &right_click([60.0, 300.0]),
    );
    assert_state(
        "min-trash-menu",
        window,
        trash_entries,
        &right_click([60.0, 160.0]),
    );
    let mut renaming = right_click(MINIMUM_DAILIES_ROW);
    renaming.extend(click([
        MINIMUM_DAILIES_ROW[0] + 30.0,
        MINIMUM_DAILIES_ROW[1] + MENU_ROWS[0],
    ]));
    renaming.extend(hover([300.0, 340.0]));
    assert_state("min-rename", window, selected, &renaming);
}

/// The longer German texts at the narrowest window (M168).
#[test]
fn german_at_the_minimum_width() {
    let views: [NamedFixture; 9] = [
        ("note", selected),
        ("empty", || {
            memo_window(NotesDocument::default(), UiState::default())
        }),
        ("draft-hint", with_draft),
        ("trash-nothing-selected", trash_entries),
        ("trash-selected", trash_entry_chosen),
        ("dialog-unsaved", unsaved_dialog),
        ("dialog-empty-trash", empty_trash_dialog),
        ("problem-recovered", || {
            with_problem(NotesProblem::Recovered("notes.corrupt.json".to_owned()))
        }),
        ("window-menu", menu_open),
    ];
    for (name, make) in views {
        assert_state_in(
            Language::German,
            &format!("min-de-{name}"),
            MINIMUM_WINDOW,
            make,
            &[],
        );
    }
    assert_state_in(
        Language::German,
        "min-de-search-no-hits",
        MINIMUM_WINDOW,
        searching,
        &typing(MINIMUM_SEARCH_FIELD, "xyzzy"),
    );
    assert_state_in(
        Language::German,
        "min-de-menu-note",
        MINIMUM_WINDOW,
        selected,
        &right_click(MINIMUM_DAILIES_ROW),
    );
    assert_state_in(
        Language::German,
        "min-de-kind-open",
        MINIMUM_WINDOW,
        selected,
        &click(MINIMUM_KIND_FIELD),
    );
}
