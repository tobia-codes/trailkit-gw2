//! Snapshots of trashed groups as a tree (M218, mock
//! https://claude.ai/artifact/3F7P7A1uxLhVhMGjwZ6b54): collapsed with a note
//! deleted out of the group, a child note selected, a nested group open, and
//! the narrowest window in German.

use jiff::Timestamp;
use trailkit_memo_trash::trash_view::TrashAction;

use super::*;

/// The narrowest window.
const MINIMUM: [f32; 2] = [320.0, 360.0];

/// The ids the tree states select and open.
struct TreeIds {
    /// The trashed group "Raids".
    raids: GroupId,
    /// The group "Archive" inside it.
    archive: GroupId,
    /// The note "Tester" inside "Raids".
    tester: NodeId,
    /// The note "Raid plan" inside "Archive".
    plan: NodeId,
}

/// The sample with "Raids" holding "Tester", "New note" and "Archive" with
/// "Raid plan"; "Weekly clears" was deleted on its own first, then the
/// group. The trash is open with nothing selected.
fn trash_tree() -> (Fixture, TreeIds) {
    let mut document = sample_document();
    let tester = note(
        "Tester",
        "- [ ] Mithril ore\n- [x] Ectos",
        ResetSchedule::Never,
    );
    let plan = note("Raid plan", "", ResetSchedule::Never);
    let mut archive = Group::new("Archive".to_owned());
    let Some(Node::Group(raids)) = document.root.last_mut() else {
        unreachable!("the sample ends with the group Raids")
    };
    let raids_id = raids.id;
    let ids = TreeIds {
        raids: raids_id,
        archive: archive.id,
        tester: NodeId::Note(tester.id),
        plan: NodeId::Note(plan.id),
    };
    archive.children.push(Node::Note(plan));
    let weekly = raids.children[0].id();
    raids.children.push(Node::Note(tester));
    raids
        .children
        .push(Node::Note(note("New note", "", ResetSchedule::Never)));
    raids.children.push(Node::Group(archive));
    let now = Timestamp::now();
    assert!(document.move_to_trash(weekly, now));
    assert!(document.move_to_trash(NodeId::Group(raids_id), now));
    let mut fixture = memo_window(document, UiState::default());
    fixture.0.apply(&WindowAction::OpenTrash, Language::English);
    (fixture, ids)
}

/// The tree with "Raids" opened by a click (Q563) and `then` applied.
fn opened(then: impl Fn(&mut MemoWindow, &TreeIds)) -> Fixture {
    let (mut fixture, ids) = trash_tree();
    let toggle = WindowAction::Trash(TrashAction::ToggleGroup(ids.raids));
    fixture.0.apply(&toggle, Language::English);
    then(&mut fixture.0, &ids);
    fixture
}

/// "Raids" open and its note "Tester" selected.
fn child_selected() -> Fixture {
    opened(|memo, ids| {
        memo.apply(
            &WindowAction::Trash(TrashAction::Select(ids.tester)),
            Language::English,
        );
    })
}

/// "Raids" and "Archive" open and "Raid plan" selected.
fn nested_child_selected() -> Fixture {
    opened(|memo, ids| {
        let toggle = WindowAction::Trash(TrashAction::ToggleGroup(ids.archive));
        memo.apply(&toggle, Language::English);
        let select = WindowAction::Trash(TrashAction::Select(ids.plan));
        memo.apply(&select, Language::English);
    })
}

#[test]
fn trash_tree_states() {
    assert_state("trash-tree", WIDE, || trash_tree().0, &[]);
    assert_state("trash-tree-group-open", WIDE, || opened(|_, _| {}), &[]);
    assert_state("trash-tree-child", WIDE, child_selected, &[]);
    assert_state("trash-tree-nested", WIDE, nested_child_selected, &[]);
    assert_state_in(
        Language::German,
        "min-de-trash-tree-nested",
        MINIMUM,
        nested_child_selected,
        &[],
    );
}
