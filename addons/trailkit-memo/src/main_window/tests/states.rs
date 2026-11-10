//! Snapshots of every view, menu, dialog and hover state of the window in both
//! themes (ROADMAP R82): the reference images are reviewed against the design
//! system (`tokens.md` §10). Behavior is tested elsewhere; this file only
//! brings the window into each state.

use trailkit_memo_shared::{drafts::Draft, i18n::local_utc_offset, note_tree::TrashEntry};
use trailkit_memo_trash::trash_view::TrashAction;
use trailkit_ui::markdown_view::MarkdownAction;

use super::{trash::document_with_trash, *};

mod minimum_width;
mod save_errors;

/// A window with the storage worker and directory that must outlive it.
type Fixture = (MemoWindow, StorageWorker, tempfile::TempDir);
/// A state's name and how to bring a window into it.
type NamedFixture = (&'static str, fn() -> Fixture);

/// The usual window size of these snapshots.
const WIDE: [f32; 2] = [640.0, 360.0];
/// A spot on the window menu's rows, by row (see [`window_menu`]).
const MENU_ROW_X: f32 = 60.0;

/// Center of the "Shopping list" row (selected in most fixtures).
const SHOPPING_ROW: [f32; 2] = [100.0, 106.0];
/// Center of the "Dailies" row.
const DAILIES_ROW: [f32; 2] = [100.0, 138.0];
/// Center of the "Raids" group row.
const RAIDS_ROW: [f32; 2] = [100.0, 170.0];
/// An empty spot of the note list.
const EMPTY_LIST_SPOT: [f32; 2] = [100.0, 300.0];
/// The kind dropdown of the selected note.
const KIND_FIELD: [f32; 2] = [400.0, 114.0];
/// A spot that shows nothing on hover.
const NOWHERE: [f32; 2] = [420.0, 330.0];

/// Vertical distances of the rows of a context menu from where it opened; the
/// fourth row sits below the separator of a note's menu.
const MENU_ROWS: [f32; 4] = [21.0, 53.0, 85.0, 127.0];

/// The mouse resting at `position`.
fn hover(position: [f32; 2]) -> Vec<FrameInput> {
    vec![FrameInput {
        mouse_position: position,
        ..FrameInput::default()
    }]
}

/// The mouse holding the left button at `position`.
fn press(position: [f32; 2]) -> Vec<FrameInput> {
    let mut steps = hover(position);
    steps.push(FrameInput {
        mouse_position: position,
        left_button_down: true,
        ..FrameInput::default()
    });
    steps
}

/// A left click at `position`, then the mouse rests there.
fn click(position: [f32; 2]) -> Vec<FrameInput> {
    let mut steps = press(position);
    steps.extend(hover(position));
    steps
}

/// A right click at `position`, opening its context menu.
fn right_click(position: [f32; 2]) -> Vec<FrameInput> {
    let mut steps = hover(position);
    steps.push(FrameInput {
        mouse_position: position,
        right_button_down: true,
        ..FrameInput::default()
    });
    steps.extend(hover(position));
    steps
}

/// A right click at `position`, then the mouse over the menu row `row`.
fn menu_row_hover(position: [f32; 2], row: usize) -> Vec<FrameInput> {
    let mut steps = right_click(position);
    steps.extend(hover([position[0] + 30.0, position[1] + MENU_ROWS[row]]));
    steps
}

/// A drag from `from` to `to` that stays pressed.
fn drag(from: [f32; 2], to: [f32; 2]) -> Vec<FrameInput> {
    let mut steps = press(from);
    for step in 1..=4_u8 {
        let fraction = f32::from(step) / 4.0;
        steps.push(FrameInput {
            mouse_position: [
                from[0] + (to[0] - from[0]) * fraction,
                from[1] + (to[1] - from[1]) * fraction,
            ],
            left_button_down: true,
            ..FrameInput::default()
        });
    }
    steps
}

/// A click at `position`, then typing `text` there.
fn typing(position: [f32; 2], text: &str) -> Vec<FrameInput> {
    let mut steps = click(position);
    steps.push(FrameInput {
        mouse_position: position,
        characters: text.to_owned(),
        ..FrameInput::default()
    });
    steps
}

/// Brings a window from `make` into the state after `steps` in both themes
/// and compares each with the reference `memo-state-<name>-<theme>`.
fn assert_state(name: &str, size: [f32; 2], make: impl Fn() -> Fixture, steps: &[FrameInput]) {
    assert_state_in(Language::English, name, size, make, steps);
}

/// Like [`assert_state`] with the window's texts in `language`.
fn assert_state_in(
    language: Language,
    name: &str,
    size: [f32; 2],
    make: impl Fn() -> Fixture,
    steps: &[FrameInput],
) {
    for (theme, theme_name) in [(ThemeKind::Dark, "dark"), (ThemeKind::Light, "light")] {
        let (mut memo, _worker, _directory) = make();
        let mut harness = UiHarness::new(size);
        let mut draw_frame = |ui: &Ui| draw_in_language(ui, &mut memo, size, theme, language);
        for _ in 0..2 {
            harness.step(&FrameInput::default(), &mut draw_frame);
        }
        for step in steps {
            harness.step(step, &mut draw_frame);
        }
        // Hold the last mouse state; typed text and wheel steps apply once.
        let hold = steps
            .last()
            .map_or_else(FrameInput::default, |last| FrameInput {
                mouse_position: last.mouse_position,
                left_button_down: last.left_button_down,
                right_button_down: last.right_button_down,
                ..FrameInput::default()
            });
        let image = harness.settle(4, &hold, &mut draw_frame);
        assert_snapshot(
            env!("CARGO_MANIFEST_DIR"),
            &format!("memo-state-{name}-{theme_name}"),
            &image,
        );
    }
}

/// The sample with its first note selected.
fn selected() -> Fixture {
    let document = sample_document();
    let ui_state = selected_first(&document);
    memo_window(document, ui_state)
}

/// The sample with `change` applied to the state that selects its first note.
fn selected_with(change: impl Fn(&mut UiState)) -> Fixture {
    let document = sample_document();
    let mut ui_state = selected_first(&document);
    change(&mut ui_state);
    memo_window(document, ui_state)
}

/// The sample with the root note titled `title` selected.
fn selected_titled(title: &str) -> Fixture {
    let document = sample_document();
    let selected = document.root.iter().find_map(|node| match node {
        Node::Note(note) if note.title == title => Some(NodeId::Note(note.id)),
        Node::Note(_) | Node::Group(_) => None,
    });
    let ui_state = UiState {
        selected,
        ..UiState::default()
    };
    memo_window(document, ui_state)
}

/// The first note open in the editor, with the body not focused.
fn editing() -> Fixture {
    let mut fixture = selected();
    fixture
        .0
        .apply(&WindowAction::StartEditing, Language::English);
    if let Some(editor) = fixture.0.editor.as_mut() {
        editor.unfocus_body();
    }
    fixture
}

/// The sample with the first note's body replaced by `body`.
fn with_body(body: &str, ui_state_change: impl Fn(&mut UiState)) -> Fixture {
    let mut document = sample_document();
    if let Some(Node::Note(first)) = document.root.first_mut() {
        first.body = body.to_owned();
    }
    let mut ui_state = selected_first(&document);
    ui_state_change(&mut ui_state);
    memo_window(document, ui_state)
}

/// The trash view with entries, the first entry selected if `select`.
fn trash_view(document: NotesDocument, select: bool) -> Fixture {
    let selected = document.trash.first().map(TrashEntry::id);
    let mut fixture = memo_window(document, UiState::default());
    fixture.0.apply(&WindowAction::OpenTrash, Language::English);
    if let (true, Some(selected)) = (select, selected) {
        fixture.0.apply(
            &WindowAction::Trash(TrashAction::Select(selected)),
            Language::English,
        );
    }
    fixture
}

/// The first note with a body long enough to scroll, scrolled down.
fn scrolled_note() -> Fixture {
    let long_body = (1..=30)
        .map(|line| format!("- Line {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    with_body(&long_body, |state| state.note_scroll = 200.0)
}

/// The first note with a chat link chip of every category.
fn chips_note() -> Fixture {
    with_body(
        "## Route\n- Start at [&BDgAAAA=], buy [&AvoJTQAA]\n- Budget: [&AdsnAAA=]\n- Build: [&DQc=]\n- Gear: [&CwEAAAA=] [&BnMVAAA=] [&CgEAAAA=] [&DgEAAAA=]\n- Broken: [&KgAAAAA=]",
        |_| {},
    )
}

/// The first note open in the editor beside its preview.
fn editing_with_preview() -> Fixture {
    let document = sample_document();
    let ui_state = UiState {
        preview_visible: true,
        ..selected_first(&document)
    };
    let mut fixture = memo_window(document, ui_state);
    fixture
        .0
        .apply(&WindowAction::StartEditing, Language::English);
    if let Some(editor) = fixture.0.editor.as_mut() {
        editor.unfocus_body();
    }
    fixture
}

/// The unsaved changes dialog over the editor.
fn unsaved_dialog() -> Fixture {
    let mut fixture = editing();
    if let Some(editor) = fixture.0.editor.as_mut() {
        editor.title_mut().push_str(" (changed)");
    }
    assert!(!fixture.0.request_close(), "closing must ask first");
    fixture
}

/// The question before deleting a group with notes.
fn delete_group_dialog() -> Fixture {
    let document = sample_document();
    let raids = document.root.iter().find_map(|node| match node {
        Node::Group(group) => Some(NodeId::Group(group.id)),
        Node::Note(_) => None,
    });
    let ui_state = selected_first(&document);
    let mut fixture = memo_window(document, ui_state);
    if let Some(raids) = raids {
        fixture
            .0
            .apply(&WindowAction::Delete(raids), Language::English);
    }
    fixture
}

/// The question before deleting a trash entry for good.
fn delete_for_good_dialog() -> Fixture {
    let document = document_with_trash([0, 25]);
    let first = document.trash[0].id();
    let mut fixture = trash_view(document, false);
    fixture.0.apply(
        &WindowAction::Trash(TrashAction::DeletePermanently(first)),
        Language::English,
    );
    fixture
}

/// The question before emptying the trash.
fn empty_trash_dialog() -> Fixture {
    let mut fixture = trash_view(document_with_trash([0, 25]), false);
    fixture.0.apply(
        &WindowAction::Trash(TrashAction::EmptyTrash),
        Language::English,
    );
    fixture
}

/// The first note with a draft kept from an earlier session.
fn with_draft() -> Fixture {
    let mut fixture = selected();
    if let Some(NodeId::Note(note)) = first_note(&fixture.0.document) {
        fixture.0.drafts.drafts.push(Draft {
            note,
            title: "Shopping list (draft)".to_owned(),
            body: "- [ ] A draft line".to_owned(),
            saved_at: same_local_time_everywhere(),
            note_updated_at: fixed_time(),
        });
    }
    fixture
}

/// A time that local time shows as the same wall clock in every time zone, so
/// a snapshot with a formatted time matches on the CI runner too.
fn same_local_time_everywhere() -> Timestamp {
    let offset = local_utc_offset(fixed_time());
    fixed_time() - jiff::SignedDuration::from_secs(i64::from(offset.seconds()))
}

/// The sample with a storage `problem` shown.
fn with_problem(problem: NotesProblem) -> Fixture {
    let mut fixture = selected();
    fixture.0.problem = Some(problem);
    fixture
}

#[test]
fn views() {
    assert_state(
        "no-selection",
        WIDE,
        || memo_window(sample_document(), UiState::default()),
        &[],
    );
    assert_state(
        "empty",
        WIDE,
        || memo_window(NotesDocument::default(), UiState::default()),
        &[],
    );
    assert_state("note", WIDE, selected, &[]);
    assert_state("daily-note", WIDE, || selected_titled("Dailies"), &[]);
    assert_state(
        "search-shown",
        WIDE,
        || selected_with(|state| state.search_visible = true),
        &[],
    );
    assert_state(
        "list-hidden",
        WIDE,
        || selected_with(|state| state.note_list_hidden = true),
        &[],
    );
    assert_state(
        "compact",
        WIDE,
        || selected_with(|state| state.compact_note_view = true),
        &[],
    );
    assert_state("minimum-width", MINIMUM_WINDOW, selected, &[]);
    assert_state("scrolled-note", WIDE, scrolled_note, &[]);
    assert_state("chips", WIDE, chips_note, &[]);
}

#[test]
fn window_chrome_hovers() {
    for (name, position) in [
        ("hover-menu-button", [28.0, 25.0]),
        ("hover-collapse", [576.0, 25.0]),
        ("hover-close", [612.0, 25.0]),
        ("hover-new-note", [62.0, 65.0]),
        ("hover-new-group", [153.0, 65.0]),
        ("hover-compact", [236.0, 65.0]),
        ("hover-edit", [590.0, 65.0]),
    ] {
        assert_state(name, WIDE, selected, &hover(position));
    }
    assert_state("pressed-edit", WIDE, selected, &press([590.0, 65.0]));
    assert_state("pressed-new-note", WIDE, selected, &press([62.0, 65.0]));
}

#[test]
fn list_rows() {
    for (name, position) in [
        ("hover-selected-row", SHOPPING_ROW),
        ("hover-row", DAILIES_ROW),
        ("hover-group", RAIDS_ROW),
        ("hover-group-arrow", [36.0, RAIDS_ROW[1]]),
        ("hover-child-row", [100.0, 202.0]),
        ("hover-badge-count", [184.0, RAIDS_ROW[1]]),
    ] {
        assert_state(name, WIDE, selected, &hover(position));
    }
    assert_state("pressed-row", WIDE, selected, &press(DAILIES_ROW));
    // A click on the row collapses the group; the note stays (M172, M173).
    assert_state("group-clicked", WIDE, selected, &{
        let mut steps = click(RAIDS_ROW);
        steps.extend(hover(NOWHERE));
        steps
    });
    assert_state("collapsed-group", WIDE, selected, &{
        let mut steps = click([36.0, RAIDS_ROW[1]]);
        steps.extend(hover(NOWHERE));
        steps
    });
    let long_list = || {
        let mut document = sample_document();
        for number in 1..=20 {
            document.root.push(Node::Note(note(
                &format!("Note {number}"),
                "",
                NoteKind::Normal,
            )));
        }
        let ui_state = selected_first(&document);
        memo_window(document, ui_state)
    };
    assert_state(
        "scrolled-list",
        WIDE,
        long_list,
        &[FrameInput {
            mouse_position: [100.0, 250.0],
            mouse_wheel: -5.0,
            ..FrameInput::default()
        }],
    );
    assert_state(
        "drag-into-group",
        WIDE,
        selected,
        &drag(SHOPPING_ROW, [100.0, 177.0]),
    );
    assert_state(
        "drag-between",
        WIDE,
        selected,
        &drag(SHOPPING_ROW, [100.0, 154.0]),
    );
    assert_state(
        "drag-impossible",
        WIDE,
        selected,
        &drag(RAIDS_ROW, [100.0, 202.0]),
    );
}

#[test]
fn note_content_hovers() {
    for (name, position) in [
        ("hover-checkbox", [228.0, 225.0]),
        ("hover-task-text", [300.0, 225.0]),
        ("hover-link", [310.0, 249.0]),
    ] {
        assert_state(name, WIDE, selected, &hover(position));
    }
    assert_state("pressed-checkbox", WIDE, selected, &press([228.0, 225.0]));
}

#[test]
fn context_menus() {
    assert_state("menu-note", WIDE, selected, &right_click(DAILIES_ROW));
    for (name, row) in [
        ("menu-note-hover-rename", 0),
        ("menu-note-move-submenu", 1),
        ("menu-note-reset-submenu", 2),
        ("menu-note-hover-delete", 3),
    ] {
        assert_state(name, WIDE, selected, &menu_row_hover(DAILIES_ROW, row));
    }
    assert_state("menu-group", WIDE, selected, &right_click(RAIDS_ROW));
    assert_state("menu-list", WIDE, selected, &right_click(EMPTY_LIST_SPOT));
    assert_state(
        "menu-list-hover",
        WIDE,
        selected,
        &menu_row_hover(EMPTY_LIST_SPOT, 0),
    );
    let mut renaming = right_click(DAILIES_ROW);
    renaming.extend(click([
        DAILIES_ROW[0] + 30.0,
        DAILIES_ROW[1] + MENU_ROWS[0],
    ]));
    renaming.extend(hover(NOWHERE));
    assert_state("rename", WIDE, selected, &renaming);
}

#[test]
fn kind_dropdown() {
    assert_state("kind-hover", WIDE, selected, &hover(KIND_FIELD));
    let mut open = click(KIND_FIELD);
    open.extend(hover(NOWHERE));
    assert_state("kind-open", WIDE, selected, &open);
    let mut option = click(KIND_FIELD);
    option.extend(hover([KIND_FIELD[0], KIND_FIELD[1] + 56.0]));
    assert_state("kind-hover-option", WIDE, selected, &option);
}

#[test]
fn editor() {
    assert_state("editor", WIDE, editing, &[]);
    assert_state("editor-focused", WIDE, selected_editing_focused, &[]);
    assert_state("editor-selection", WIDE, editing_with_selection, &[]);
    assert_state("editor-preview", [720.0, 360.0], editing_with_preview, &[]);
}

/// The first note open in the editor with the keyboard focus in its body.
fn selected_editing_focused() -> Fixture {
    let mut fixture = selected();
    fixture
        .0
        .apply(&WindowAction::StartEditing, Language::English);
    fixture
}

/// The editor with a selection over several lines (R81: rounded).
fn editing_with_selection() -> Fixture {
    let mut fixture = editing();
    if let Some(editor) = fixture.0.editor.as_mut() {
        let end = editor.body_text().len().saturating_sub(12);
        editor.select_in_body(5..end);
        editor.unfocus_body();
    }
    fixture
}

#[test]
fn search() {
    let searching = || selected_with(|state| state.search_visible = true);
    let field = [80.0, 68.0];
    assert_state("search-focused", WIDE, searching, &click(field));
    assert_state("search-hover-field", WIDE, searching, &hover(field));
    assert_state("search-hits", WIDE, searching, &typing(field, "ecto"));
    assert_state("search-no-hits", WIDE, searching, &typing(field, "xyzzy"));
    assert_state("search-hover-close", WIDE, searching, &hover([185.0, 68.0]));
    assert_state("search-hover-clear", WIDE, searching, &{
        let mut steps = typing(field, "ecto");
        steps.extend(hover([185.0, 68.0]));
        steps
    });
}

#[test]
fn dialogs() {
    assert_state("dialog-unsaved", WIDE, unsaved_dialog, &[]);
    assert_state("dialog-delete-group", WIDE, delete_group_dialog, &[]);
    assert_state("dialog-delete-for-good", WIDE, delete_for_good_dialog, &[]);
    assert_state("dialog-empty-trash", WIDE, empty_trash_dialog, &[]);
}

#[test]
fn trash() {
    let entries = || trash_view(document_with_trash([0, 25]), false);
    let chosen = || trash_view(document_with_trash([0, 25]), true);
    assert_state(
        "trash-nothing-in-it",
        WIDE,
        || trash_view(sample_document(), false),
        &[],
    );
    assert_state("trash-nothing-selected", WIDE, entries, &[]);
    assert_state("trash-selected", WIDE, chosen, &[]);
    for (name, position) in [
        ("trash-hover-note", [108.0, 160.0]),
        ("trash-hover-group", [108.0, 112.0]),
        ("trash-hover-empty", [163.0, 65.0]),
        ("trash-hover-back", [32.0, 65.0]),
    ] {
        assert_state(name, WIDE, entries, &hover(position));
    }
    for (name, position) in [
        ("trash-hover-restore", [580.0, 65.0]),
        ("trash-hover-delete", [515.0, 65.0]),
    ] {
        assert_state(name, WIDE, chosen, &hover(position));
    }
    assert_state("trash-menu", WIDE, entries, &right_click([108.0, 160.0]));
    let mut menu_hover = right_click([108.0, 160.0]);
    menu_hover.extend(hover([138.0, 180.0]));
    assert_state("trash-menu-hover", WIDE, entries, &menu_hover);
}

#[test]
fn toast() {
    let deleted = || {
        let mut fixture = selected();
        if let Some(first) = first_note(&fixture.0.document) {
            fixture
                .0
                .apply(&WindowAction::Delete(first), Language::English);
        }
        fixture
    };
    assert_state("toast", WIDE, deleted, &[]);
    assert_state("toast-minimum-width", MINIMUM_WINDOW, deleted, &[]);
    let copied = || {
        let mut fixture = selected();
        fixture.0.apply(
            &WindowAction::Markdown(MarkdownAction::ChatLinkCopied("[&BDgAAAA=]".to_owned())),
            Language::English,
        );
        fixture
    };
    assert_state("toast-copied", WIDE, copied, &[]);
}

/// Center of the note toolbar's `⋯` button (M160).
const NOTE_MENU_BUTTON: [f32; 2] = [532.0, 65.0];

#[test]
fn note_menu() {
    assert_state("note-menu-hover", WIDE, selected, &hover(NOTE_MENU_BUTTON));
    assert_state(
        "note-menu-pressed",
        WIDE,
        selected,
        &press(NOTE_MENU_BUTTON),
    );
    assert_state("note-menu", WIDE, selected, &click(NOTE_MENU_BUTTON));
}

/// The sample with the window menu open.
fn menu_open() -> Fixture {
    let mut fixture = selected();
    fixture.0.window_menu_open = true;
    fixture
}

#[test]
fn window_menu() {
    assert_state("window-menu", WIDE, menu_open, &hover(NOWHERE));
    assert_state(
        "window-menu-minimum-width",
        MINIMUM_WINDOW,
        menu_open,
        &hover(NOWHERE),
    );
    assert_state(
        "window-menu-hover-selected",
        WIDE,
        menu_open,
        &hover([MENU_ROW_X, 67.0]),
    );
    assert_state(
        "window-menu-hover-row",
        WIDE,
        menu_open,
        &hover([MENU_ROW_X, 99.0]),
    );
    assert_state(
        "window-menu-pressed-row",
        WIDE,
        menu_open,
        &press([MENU_ROW_X, 99.0]),
    );
    assert_state(
        "window-menu-button-open",
        WIDE,
        menu_open,
        &hover([28.0, 25.0]),
    );
    let in_trash = || {
        let mut fixture = trash_view(document_with_trash([0, 25]), false);
        fixture.0.window_menu_open = true;
        fixture
    };
    assert_state("window-menu-in-trash", WIDE, in_trash, &hover(NOWHERE));
    let list_hidden = || {
        let mut fixture = selected_with(|state| state.note_list_hidden = true);
        fixture.0.window_menu_open = true;
        fixture
    };
    assert_state(
        "window-menu-list-hidden",
        WIDE,
        list_hidden,
        &hover(NOWHERE),
    );
    let while_editing = || {
        let mut fixture = editing();
        fixture.0.window_menu_open = true;
        fixture
    };
    assert_state("window-menu-editing", WIDE, while_editing, &hover(NOWHERE));
    let without_notes = || {
        let mut fixture = memo_window(NotesDocument::default(), UiState::default());
        fixture.0.window_menu_open = true;
        fixture
    };
    assert_state("window-menu-no-notes", WIDE, without_notes, &hover(NOWHERE));
    // The toast lies above the menu (M165).
    let with_toast = || {
        let mut fixture = menu_open();
        if let Some(first) = first_note(&fixture.0.document) {
            fixture
                .0
                .apply(&WindowAction::Delete(first), Language::English);
        }
        fixture
    };
    assert_state("window-menu-toast", WIDE, with_toast, &hover(NOWHERE));
}

#[test]
fn problems_and_drafts() {
    assert_state(
        "problem-recovered",
        WIDE,
        || with_problem(NotesProblem::Recovered("notes.corrupt.json".to_owned())),
        &[],
    );
    assert_state(
        "problem-newer",
        WIDE,
        || with_problem(NotesProblem::NewerVersion),
        &[],
    );
    assert_state(
        "problem-inaccessible",
        WIDE,
        || with_problem(NotesProblem::Inaccessible),
        &[],
    );
    assert_state("draft-hint", WIDE, with_draft, &[]);
}
