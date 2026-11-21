//! Entries inside trashed groups (memo M218, Q562–Q569): finding them,
//! restoring and deleting one on its own. A restored entry whose group is in
//! the trash brings that group back at its former place with the same id,
//! holding only what was restored; whatever is restored later from the same
//! group joins it, so a group never comes back twice.

use super::{
    ChildOrder, Group, GroupId, NodeId, NotesDocument, Place, TrashEntry, TreeNode,
    traversal::find_group_mut,
};

/// Where a node sits in the trash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrashSpot {
    /// It is the trash entry at this index.
    Entry(usize),
    /// It is a child of the trashed group `parent`, at `index`.
    Child {
        /// The trashed group holding it.
        parent: GroupId,
        /// Its position among the group's children.
        index: usize,
    },
}

impl NotesDocument {
    /// The note or group `node` anywhere in the trash: an entry or inside a
    /// trashed group.
    #[must_use]
    pub fn find_in_trash(&self, node: NodeId) -> Option<&TreeNode> {
        self.trash
            .iter()
            .find_map(|entry| find_node(std::slice::from_ref(&entry.node), node))
    }

    /// The trash entry `node` belongs to: itself, or the trashed group it is
    /// inside. Its deadline applies to the node (Q565).
    #[must_use]
    pub fn trash_entry_holding(&self, node: NodeId) -> Option<&TrashEntry> {
        self.trash
            .iter()
            .find(|entry| find_node(std::slice::from_ref(&entry.node), node).is_some())
    }

    /// The trashed group that holds `node` directly, if `node` is inside a
    /// trashed group rather than an entry of its own.
    #[must_use]
    pub fn trashed_parent(&self, node: NodeId) -> Option<&Group> {
        match self.trash_spot(node)? {
            TrashSpot::Child { parent, .. } => {
                self.find_in_trash(NodeId::Group(parent))
                    .and_then(|found| match found {
                        TreeNode::Group(group) => Some(group),
                        TreeNode::Note(_) => None,
                    })
            }
            TrashSpot::Entry(_) => None,
        }
    }

    /// The title of the group `group`, in the tree or in the trash.
    #[must_use]
    pub fn group_title_anywhere(&self, group: GroupId) -> Option<&str> {
        self.group(group)
            .or_else(|| match self.find_in_trash(NodeId::Group(group)) {
                Some(TreeNode::Group(found)) => Some(found),
                _ => None,
            })
            .map(|found| found.title.as_str())
    }

    /// Puts `node` back from the trash (M140, M218): an entry to its former
    /// place, a child of a trashed group into that group. A parent group
    /// that is itself in the trash comes back first, empty, at its former
    /// place (Q567 a); a group already back takes the restored entries in,
    /// in the order they had when their group was deleted. A parent deleted
    /// for good means the end of the root level. Returns whether `node` was
    /// in the trash.
    pub fn restore_from_trash(&mut self, node: NodeId) -> bool {
        let Some(spot) = self.trash_spot(node) else {
            return false;
        };
        // The order of every trashed group, while their entries are still
        // there.
        let orders: Vec<ChildOrder> = self
            .trash
            .iter()
            .flat_map(|entry| entry.child_order.iter().cloned())
            .collect();
        let (restored, place) = match spot {
            TrashSpot::Entry(index) => {
                let entry = self.trash.remove(index);
                let place = Place {
                    parent: entry.former_parent,
                    index: entry.former_index,
                };
                (entry.node, place)
            }
            TrashSpot::Child { parent, index } => {
                let Some(children) = self.trashed_children_mut(parent) else {
                    return false;
                };
                let restored = children.remove(index);
                let place = Place {
                    parent: Some(parent),
                    index,
                };
                (restored, place)
            }
        };
        let place = self.live_place(place, &orders);
        self.put_back(place, restored, &orders);
        self.drop_restored_shells();
        true
    }

    /// Removes `node` from the trash for good (M146, Q569), an entry or a
    /// child of a trashed group, and returns it.
    pub fn delete_from_trash(&mut self, node: NodeId) -> Option<TreeNode> {
        match self.trash_spot(node)? {
            TrashSpot::Entry(index) => Some(self.trash.remove(index).node),
            TrashSpot::Child { parent, index } => {
                Some(self.trashed_children_mut(parent)?.remove(index))
            }
        }
    }

    /// Where `node` is in the trash.
    fn trash_spot(&self, node: NodeId) -> Option<TrashSpot> {
        if let Some(index) = self.trash.iter().position(|entry| entry.id() == node) {
            return Some(TrashSpot::Entry(index));
        }
        self.trash.iter().find_map(|entry| match &entry.node {
            TreeNode::Group(group) => child_spot(group, node),
            TreeNode::Note(_) => None,
        })
    }

    /// The children of the trashed group `group`.
    fn trashed_children_mut(&mut self, group: GroupId) -> Option<&mut Vec<TreeNode>> {
        self.trash.iter_mut().find_map(|entry| {
            find_group_mut(std::slice::from_mut(&mut entry.node), group)
                .map(|found| &mut found.children)
        })
    }

    /// `place` with its parent in the tree: a parent in the trash comes
    /// back first (Q567 a); a parent deleted for good means the end of the
    /// root level.
    fn live_place(&mut self, place: Place, orders: &[ChildOrder]) -> Place {
        match place.parent {
            None => place,
            Some(parent) if self.bring_back_group(parent, orders) => place,
            Some(_) => Place {
                parent: None,
                index: usize::MAX,
            },
        }
    }

    /// Makes sure the group `group` is in the tree; a group in the trash
    /// comes back empty at its former place, its own trashed parents first
    /// (Q567 a). Returns `false` if it was deleted for good.
    fn bring_back_group(&mut self, group: GroupId, orders: &[ChildOrder]) -> bool {
        if self.group(group).is_some() {
            return true;
        }
        let (shell, place) = match self.trash_spot(NodeId::Group(group)) {
            Some(TrashSpot::Entry(index)) => {
                let entry = &self.trash[index];
                let TreeNode::Group(trashed) = &entry.node else {
                    return false;
                };
                let place = Place {
                    parent: entry.former_parent,
                    index: entry.former_index,
                };
                (empty_copy(trashed), place)
            }
            Some(TrashSpot::Child { parent, index }) => {
                let Some(TreeNode::Group(trashed)) = self.find_in_trash(NodeId::Group(group))
                else {
                    return false;
                };
                let place = Place {
                    parent: Some(parent),
                    index,
                };
                (empty_copy(trashed), place)
            }
            None => return false,
        };
        let place = self.live_place(place, orders);
        self.put_back(place, TreeNode::Group(shell), orders);
        true
    }

    /// Inserts `node` at `place` (index clamped, a missing parent = the end
    /// of the root level). Into a group that has a recorded order, it goes
    /// after the last child that came before it then. A group already in
    /// the tree with the same id takes the node's children instead, so it
    /// never comes back twice.
    fn put_back(&mut self, place: Place, node: TreeNode, orders: &[ChildOrder]) {
        if let TreeNode::Group(group) = node {
            if self.group(group.id).is_some() {
                for (index, child) in group.children.into_iter().enumerate() {
                    let place = Place {
                        parent: Some(group.id),
                        index,
                    };
                    self.put_back(place, child, orders);
                }
                return;
            }
            return self.insert_ordered(place, TreeNode::Group(group), orders);
        }
        self.insert_ordered(place, node, orders);
    }

    /// Inserts `node` into `place.parent` at the position its recorded order
    /// gives, else at `place.index` (clamped).
    fn insert_ordered(&mut self, place: Place, node: TreeNode, orders: &[ChildOrder]) {
        let order = place.parent.and_then(|parent| {
            orders
                .iter()
                .find(|order| order.group == parent)
                .map(|order| order.children.as_slice())
        });
        let Some(children) = self.children_mut(place.parent) else {
            self.root.push(node);
            return;
        };
        let index = order
            .and_then(|order| ordered_index(children, node.id(), order))
            .unwrap_or(place.index)
            .min(children.len());
        children.insert(index, node);
    }

    /// Removes trashed groups left empty whose group is back in the tree:
    /// nothing of them is left to restore.
    fn drop_restored_shells(&mut self) {
        let live: Vec<GroupId> = self
            .trash
            .iter()
            .filter_map(|entry| match &entry.node {
                TreeNode::Group(group) => Some(group),
                TreeNode::Note(_) => None,
            })
            .flat_map(group_ids)
            .filter(|group| self.group(*group).is_some())
            .collect();
        let is_restored_shell = |node: &TreeNode| {
            matches!(node, TreeNode::Group(group)
                if group.children.is_empty() && live.contains(&group.id))
        };
        self.trash.retain(|entry| !is_restored_shell(&entry.node));
        for entry in &mut self.trash {
            if let TreeNode::Group(group) = &mut entry.node {
                retain_deep(group, &is_restored_shell);
            }
        }
    }
}

/// Where `node` goes among `children` by the recorded `order`: right after
/// the last child that came before it; `None` if `order` does not know it.
fn ordered_index(children: &[TreeNode], node: NodeId, order: &[NodeId]) -> Option<usize> {
    let rank = |id: NodeId| order.iter().position(|candidate| *candidate == id);
    let own = rank(node)?;
    Some(
        children
            .iter()
            .rposition(|child| rank(child.id()).is_some_and(|other| other < own))
            .map_or(0, |before| before + 1),
    )
}

/// `node` among `nodes` and their descendants.
fn find_node(nodes: &[TreeNode], node: NodeId) -> Option<&TreeNode> {
    nodes.iter().find_map(|candidate| {
        if candidate.id() == node {
            return Some(candidate);
        }
        match candidate {
            TreeNode::Group(group) => find_node(&group.children, node),
            TreeNode::Note(_) => None,
        }
    })
}

/// Where `node` is below `group`: its direct parent and index.
fn child_spot(group: &Group, node: NodeId) -> Option<TrashSpot> {
    group
        .children
        .iter()
        .enumerate()
        .find_map(|(index, child)| {
            if child.id() == node {
                return Some(TrashSpot::Child {
                    parent: group.id,
                    index,
                });
            }
            match child {
                TreeNode::Group(inner) => child_spot(inner, node),
                TreeNode::Note(_) => None,
            }
        })
}

/// The ids of `group` and every group inside it.
fn group_ids(group: &Group) -> Vec<GroupId> {
    let mut ids = vec![group.id];
    for child in &group.children {
        if let TreeNode::Group(inner) = child {
            ids.extend(group_ids(inner));
        }
    }
    ids
}

/// Removes the children of `group`, at any depth, that `remove` matches.
fn retain_deep(group: &mut Group, remove: &impl Fn(&TreeNode) -> bool) {
    group.children.retain(|child| !remove(child));
    for child in &mut group.children {
        if let TreeNode::Group(inner) = child {
            retain_deep(inner, remove);
        }
    }
}

/// `group` without its children: what comes back first of a trashed group.
fn empty_copy(group: &Group) -> Group {
    Group {
        id: group.id,
        title: group.title.clone(),
        children: Vec::new(),
        unknown_fields: group.unknown_fields.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note_tree::{Note, ResetSchedule, node::tests::noon};

    /// A tree `[First, Raids[Weekly, Daily, Archive[Plan]], Last]`.
    struct Sample {
        /// The document.
        document: NotesDocument,
        /// The group "Raids".
        raids: GroupId,
        /// The group "Archive" inside "Raids".
        archive: GroupId,
        /// The note "Weekly" in "Raids".
        weekly: NodeId,
        /// The note "Daily" in "Raids".
        daily: NodeId,
        /// The note "Plan" in "Archive".
        plan: NodeId,
    }

    /// Builds the sample tree.
    fn sample() -> Sample {
        let note = |title: &str| Note::new(title.to_owned(), ResetSchedule::Never, noon());
        let (weekly, daily, plan) = (note("Weekly"), note("Daily"), note("Plan"));
        let ids = (
            NodeId::Note(weekly.id),
            NodeId::Note(daily.id),
            NodeId::Note(plan.id),
        );
        let mut archive = Group::new("Archive".to_owned());
        archive.children = vec![TreeNode::Note(plan)];
        let mut raids = Group::new("Raids".to_owned());
        let (raids_id, archive_id) = (raids.id, archive.id);
        raids.children = vec![
            TreeNode::Note(weekly),
            TreeNode::Note(daily),
            TreeNode::Group(archive),
        ];
        Sample {
            document: NotesDocument {
                root: vec![
                    TreeNode::Note(note("First")),
                    TreeNode::Group(raids),
                    TreeNode::Note(note("Last")),
                ],
                trash: Vec::new(),
                unknown_fields: Default::default(),
            },
            raids: raids_id,
            archive: archive_id,
            weekly: ids.0,
            daily: ids.1,
            plan: ids.2,
        }
    }

    /// The titles of `nodes`, groups with their children in brackets.
    fn outline(nodes: &[TreeNode]) -> String {
        nodes
            .iter()
            .map(|node| match node {
                TreeNode::Note(note) => note.title.clone(),
                TreeNode::Group(group) => format!("{}[{}]", group.title, outline(&group.children)),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// The outline of the trash entries.
    fn trash_outline(document: &NotesDocument) -> String {
        let nodes: Vec<TreeNode> = document
            .trash
            .iter()
            .map(|entry| entry.node.clone())
            .collect();
        outline(&nodes)
    }

    #[test]
    fn a_child_comes_back_in_its_group_and_the_rest_stays() {
        let Sample {
            mut document,
            raids,
            daily,
            ..
        } = sample();
        assert!(document.move_to_trash(NodeId::Group(raids), noon()));
        assert!(document.restore_from_trash(daily));
        assert_eq!(outline(&document.root), "First, Raids[Daily], Last");
        assert_eq!(trash_outline(&document), "Raids[Weekly, Archive[Plan]]");
        // The rest joins the same group, in its former order.
        assert!(document.restore_from_trash(NodeId::Group(raids)));
        assert_eq!(
            outline(&document.root),
            "First, Raids[Weekly, Daily, Archive[Plan]], Last"
        );
        assert!(document.trash.is_empty());
    }

    #[test]
    fn a_nested_child_brings_back_every_group_above_it() {
        let Sample {
            mut document,
            raids,
            plan,
            archive,
            ..
        } = sample();
        assert!(document.move_to_trash(NodeId::Group(raids), noon()));
        assert!(document.restore_from_trash(plan));
        assert_eq!(outline(&document.root), "First, Raids[Archive[Plan]], Last");
        assert_eq!(
            trash_outline(&document),
            "Raids[Weekly, Daily]",
            "the empty Archive is gone"
        );
        assert!(document.find_in_trash(NodeId::Group(archive)).is_none());
    }

    #[test]
    fn a_note_deleted_before_its_group_brings_the_group_back() {
        let Sample {
            mut document,
            raids,
            weekly,
            ..
        } = sample();
        assert!(document.move_to_trash(weekly, noon()));
        assert!(document.move_to_trash(NodeId::Group(raids), noon()));
        assert!(document.restore_from_trash(weekly));
        assert_eq!(outline(&document.root), "First, Raids[Weekly], Last");
        assert_eq!(trash_outline(&document), "Raids[Daily, Archive[Plan]]");
        assert_eq!(document.group_title_anywhere(raids), Some("Raids"));
    }

    #[test]
    fn restoring_the_last_child_empties_the_trashed_group_away() {
        let Sample {
            mut document,
            raids,
            weekly,
            daily,
            archive,
            ..
        } = sample();
        assert!(document.move_to_trash(NodeId::Group(raids), noon()));
        for node in [weekly, daily, NodeId::Group(archive)] {
            assert!(document.restore_from_trash(node));
        }
        assert_eq!(
            outline(&document.root),
            "First, Raids[Weekly, Daily, Archive[Plan]], Last"
        );
        assert!(document.trash.is_empty());
    }

    #[test]
    fn a_child_can_be_deleted_for_good_and_found_with_its_deadline() {
        let Sample {
            mut document,
            raids,
            daily,
            weekly,
            ..
        } = sample();
        assert!(document.move_to_trash(NodeId::Group(raids), noon()));
        assert_eq!(
            document.trash_entry_holding(daily).map(TrashEntry::id),
            Some(NodeId::Group(raids))
        );
        assert_eq!(
            document.trashed_parent(daily).map(|group| group.id),
            Some(raids)
        );
        assert!(document.trashed_parent(NodeId::Group(raids)).is_none());
        assert!(document.delete_from_trash(daily).is_some());
        assert_eq!(trash_outline(&document), "Raids[Weekly, Archive[Plan]]");
        assert!(document.find_in_trash(weekly).is_some());
        assert!(!document.restore_from_trash(daily), "gone for good");
    }

    #[test]
    fn the_child_order_is_saved_only_for_groups_and_optional_on_load() {
        let Sample {
            mut document,
            raids,
            weekly,
            ..
        } = sample();
        assert!(document.move_to_trash(weekly, noon()));
        assert!(document.move_to_trash(NodeId::Group(raids), noon()));
        let note_entry = serde_json::to_value(&document.trash[0]).expect("serializes");
        assert!(note_entry.get("child_order").is_none(), "a note has none");
        let group_entry = serde_json::to_value(&document.trash[1]).expect("serializes");
        assert_eq!(group_entry["child_order"].as_array().map(Vec::len), Some(2));
        // A file of an older version has no order and still loads.
        let mut older = group_entry;
        older
            .as_object_mut()
            .map(|fields| fields.remove("child_order"));
        let loaded: TrashEntry = serde_json::from_value(older).expect("loads");
        assert!(loaded.child_order.is_empty());
    }

    #[test]
    fn entries_added_meanwhile_keep_their_place_among_restored_ones() {
        let Sample {
            mut document,
            raids,
            weekly,
            daily,
            archive,
            ..
        } = sample();
        assert!(document.move_to_trash(NodeId::Group(raids), noon()));
        assert!(document.restore_from_trash(daily));
        // A note the user adds to the group that came back.
        let added = Note::new("Added".to_owned(), ResetSchedule::Never, noon());
        if let Some(group) = document.group_mut(raids) {
            group.children.push(TreeNode::Note(added));
        }
        assert!(document.restore_from_trash(weekly));
        assert!(document.restore_from_trash(NodeId::Group(archive)));
        assert_eq!(
            outline(&document.root),
            "First, Raids[Weekly, Daily, Archive[Plan], Added], Last",
            "nothing is overwritten; restored entries slot in by their old order"
        );
    }

    #[test]
    fn an_entry_gone_from_the_recorded_order_is_skipped() {
        let Sample {
            mut document,
            raids,
            weekly,
            daily,
            archive,
            ..
        } = sample();
        assert!(document.move_to_trash(NodeId::Group(raids), noon()));
        // "Daily" sat between "Weekly" and "Archive"; it is deleted for good.
        assert!(document.delete_from_trash(daily).is_some());
        assert!(document.restore_from_trash(NodeId::Group(archive)));
        assert!(document.restore_from_trash(weekly));
        assert_eq!(
            outline(&document.root),
            "First, Raids[Weekly, Archive[Plan]], Last",
            "the missing entry is skipped, the others keep their order"
        );
    }
}
