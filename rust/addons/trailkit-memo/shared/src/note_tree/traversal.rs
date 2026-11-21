//! Recursive lookups and edits over a list of tree nodes, used by
//! [`super::NotesDocument`].

use super::ids::{GroupId, NodeId, NoteId};
use super::node::{Group, Note, TreeNode};

/// A place in the tree: the parent (`None` = root level) and the index in
/// its children. Undo remembers where a moved or deleted entry was (M135).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Place {
    /// The parent group, or `None` for the root level.
    pub parent: Option<GroupId>,
    /// The position in the parent's children.
    pub index: usize,
}

/// A row of a tree list: a node with its depth and, for a group, whether
/// it is open (memo M218, Q570).
#[derive(Debug, Clone, Copy)]
pub struct ShownRow<'tree> {
    /// The note or group.
    pub node: &'tree TreeNode,
    /// Its depth; 0 is the top level of the visited nodes.
    pub level: usize,
    /// For a group, whether its children follow; `false` for a note.
    pub open: bool,
}

/// Visits the rows a tree list shows, in display order: every node of
/// `nodes` that `is_shown` accepts and, below an open group (`is_open`),
/// its children one level deeper. The note list and the trash share it
/// (Q570); each draws its rows itself. Allocates nothing.
pub fn visit_shown<'tree>(
    nodes: &'tree [TreeNode],
    is_shown: &impl Fn(&TreeNode) -> bool,
    is_open: &impl Fn(&Group) -> bool,
    visit: &mut impl FnMut(ShownRow<'tree>),
) {
    visit_level(nodes, 0, is_shown, is_open, visit);
}

/// [`visit_shown`] for `nodes` at depth `level`.
fn visit_level<'tree>(
    nodes: &'tree [TreeNode],
    level: usize,
    is_shown: &impl Fn(&TreeNode) -> bool,
    is_open: &impl Fn(&Group) -> bool,
    visit: &mut impl FnMut(ShownRow<'tree>),
) {
    for node in nodes.iter().filter(|node| is_shown(node)) {
        let open = matches!(node, TreeNode::Group(group) if is_open(group));
        visit(ShownRow { node, level, open });
        if let (TreeNode::Group(group), true) = (node, open) {
            visit_level(&group.children, level + 1, is_shown, is_open, visit);
        }
    }
}

/// Inserts `node` next to `sibling` (`offset` 0 = before, 1 = after) in
/// whichever list of `nodes` (or their descendants) holds it; hands `node`
/// back if `sibling` is missing.
#[expect(
    clippy::result_large_err,
    reason = "the error hands the moved node back by value, like the Ok path"
)]
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

/// Calls `visit` for every note in `nodes` and nested groups, in list order.
pub(super) fn for_each_note(nodes: &[TreeNode], visit: &mut impl FnMut(&Note)) {
    for node in nodes {
        match node {
            TreeNode::Note(note) => visit(note),
            TreeNode::Group(group) => for_each_note(&group.children, visit),
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
