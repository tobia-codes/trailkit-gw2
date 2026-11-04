//! Recursive lookups and edits over a list of tree nodes, used by
//! [`super::NotesDocument`].

use super::ids::{GroupId, NodeId, NoteId};
use super::node::{Group, Note, TreeNode};

/// A place in the tree: the parent (`None` = root level) and the index in
/// its children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Place {
    /// The parent group, or `None` for the root level.
    pub(super) parent: Option<GroupId>,
    /// The position in the parent's children.
    pub(super) index: usize,
}

/// Inserts `node` next to `sibling` (`offset` 0 = before, 1 = after) in
/// whichever list of `nodes` (or their descendants) holds it; hands `node`
/// back if `sibling` is missing.
pub(super) fn insert_next_to(
    nodes: &mut Vec<TreeNode>,
    sibling: NodeId,
    offset: usize,
    node: TreeNode,
) -> Result<(), TreeNode> {
    if let Some(index) = nodes.iter().position(|candidate| candidate.id() == sibling) {
        nodes.insert(index + offset, node);
        return Ok(());
    }
    let mut node = node;
    for candidate in nodes.iter_mut() {
        if let TreeNode::Group(group) = candidate {
            match insert_next_to(&mut group.children, sibling, offset, node) {
                Ok(()) => return Ok(()),
                Err(returned) => node = returned,
            }
        }
    }
    Err(node)
}

/// Calls `visit` for every note in `nodes`, nested ones included.
pub(super) fn for_each_note_mut(nodes: &mut [TreeNode], visit: &mut impl FnMut(&mut Note)) {
    for node in nodes {
        match node {
            TreeNode::Note(note) => visit(note),
            TreeNode::Group(group) => for_each_note_mut(&mut group.children, visit),
        }
    }
}

/// Searches `nodes` recursively for the note `id`.
pub(super) fn find_note(nodes: &[TreeNode], id: NoteId) -> Option<&Note> {
    nodes.iter().find_map(|node| match node {
        TreeNode::Note(note) if note.id == id => Some(note),
        TreeNode::Note(_) => None,
        TreeNode::Group(group) => find_note(&group.children, id),
    })
}

/// Searches `nodes` recursively for the note `id`, mutably.
pub(super) fn find_note_mut(nodes: &mut [TreeNode], id: NoteId) -> Option<&mut Note> {
    nodes.iter_mut().find_map(|node| match node {
        TreeNode::Note(note) if note.id == id => Some(note),
        TreeNode::Note(_) => None,
        TreeNode::Group(group) => find_note_mut(&mut group.children, id),
    })
}

/// Searches `nodes` recursively for the group `id`.
pub(super) fn find_group(nodes: &[TreeNode], id: GroupId) -> Option<&Group> {
    nodes.iter().find_map(|node| match node {
        TreeNode::Note(_) => None,
        TreeNode::Group(group) if group.id == id => Some(group),
        TreeNode::Group(group) => find_group(&group.children, id),
    })
}

/// Finds the place of `id` in `nodes` (the children of `parent`) or their
/// descendants.
pub(super) fn place_in(nodes: &[TreeNode], parent: Option<GroupId>, id: NodeId) -> Option<Place> {
    if let Some(index) = nodes.iter().position(|node| node.id() == id) {
        return Some(Place { parent, index });
    }
    nodes.iter().find_map(|node| match node {
        TreeNode::Note(_) => None,
        TreeNode::Group(group) => place_in(&group.children, Some(group.id), id),
    })
}

/// Searches `nodes` recursively for the group `id`, mutably.
pub(super) fn find_group_mut(nodes: &mut [TreeNode], id: GroupId) -> Option<&mut Group> {
    for node in nodes {
        let TreeNode::Group(group) = node else {
            continue;
        };
        if group.id == id {
            return Some(group);
        }
        if let Some(found) = find_group_mut(&mut group.children, id) {
            return Some(found);
        }
    }
    None
}

/// Finds the parent of `id`: `Some(None)` = root level, `Some(Some(group))`
/// = inside `group`, `None` = not in the tree.
pub(super) fn parent_of(nodes: &[TreeNode], id: NodeId) -> Option<Option<GroupId>> {
    if nodes.iter().any(|node| node.id() == id) {
        return Some(None);
    }
    nodes.iter().find_map(|node| match node {
        TreeNode::Note(_) => None,
        TreeNode::Group(group) => match parent_of(&group.children, id)? {
            None => Some(Some(group.id)),
            inner => Some(inner),
        },
    })
}
