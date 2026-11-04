//! Searching the note list (memo plan M118–M126): which rows a search shows,
//! how many notes match per group, and how often a note's content matches.
//!
//! ImGui-free so it is unit-tested on any platform. The index is rebuilt
//! only when the notes change; results only when the query, the notes or the
//! selection change — never per frame.

use std::collections::{HashMap, HashSet};

use trailkit_ui::{markdown, text_search::SearchQuery};

use trailkit_memo_shared::list_filter::ListFilter;
use trailkit_memo_shared::note_tree::{
    Group, GroupId, NodeId, Note, NoteId, NotesDocument, TreeNode,
};

/// Lower-cased title and visible text of every note and group title, built
/// once per change of the notes.
#[derive(Debug, Clone, Default)]
pub struct NoteSearchIndex {
    /// Lower-cased note titles and visible content, by note.
    notes: HashMap<NoteId, IndexedNote>,
    /// Lower-cased group titles, by group.
    groups: HashMap<GroupId, String>,
}

/// The searchable text of one note.
#[derive(Debug, Clone)]
struct IndexedNote {
    /// The title in lower case.
    title: String,
    /// The visible content (markdown markers and link targets removed,
    /// M120) in lower case.
    content: String,
}

impl NoteSearchIndex {
    /// Indexes every note and group of `document` (the trash excluded).
    #[must_use]
    pub fn build(document: &NotesDocument) -> Self {
        let mut index = Self::default();
        index.add_nodes(&document.root);
        index
    }

    /// Adds `nodes` and their descendants.
    fn add_nodes(&mut self, nodes: &[TreeNode]) {
        for node in nodes {
            match node {
                TreeNode::Note(note) => {
                    let content = markdown::plain_text(&markdown::parse(&note.body));
                    self.notes.insert(
                        note.id,
                        IndexedNote {
                            title: note.title.to_lowercase(),
                            content: content.to_lowercase(),
                        },
                    );
                }
                TreeNode::Group(group) => {
                    self.groups.insert(group.id, group.title.to_lowercase());
                    self.add_nodes(&group.children);
                }
            }
        }
    }
}

/// What a search shows in the note list (M118, M121, M122).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchResults {
    /// Rows shown: matching notes, groups whose title matches (with all
    /// their content), groups above a shown row, and the kept-visible node.
    visible: HashSet<NodeId>,
    /// Matching notes below each shown group, nested groups included (M121).
    group_counts: HashMap<GroupId, usize>,
    /// Content occurrences of notes that match, but not by their title alone
    /// (shown at the row end, M122).
    content_hits: HashMap<NoteId, usize>,
    /// Whether any note or group matches (the kept-visible node does not
    /// count); without, the list shows the empty state (M126).
    has_hits: bool,
}

impl SearchResults {
    /// Searches `document` with `index` (built from the same document).
    /// `keep_visible` — the open note — stays visible even when it does not
    /// match (M125).
    #[must_use]
    pub fn compute(
        document: &NotesDocument,
        index: &NoteSearchIndex,
        query: &SearchQuery,
        keep_visible: Option<NodeId>,
    ) -> Self {
        let mut results = Self::default();
        let search = Search {
            index,
            query,
            keep_visible,
        };
        search.visit_nodes(&document.root, false, &mut results);
        results
    }

    /// Whether anything matches.
    #[must_use]
    pub const fn has_hits(&self) -> bool {
        self.has_hits
    }
}

impl ListFilter for SearchResults {
    fn is_visible(&self, node: NodeId) -> bool {
        self.visible.contains(&node)
    }

    fn group_count(&self, group: GroupId) -> usize {
        self.group_counts.get(&group).copied().unwrap_or(0)
    }

    fn content_hits(&self, note: NoteId) -> Option<usize> {
        self.content_hits.get(&note).copied()
    }
}

/// One search run over the tree.
struct Search<'search> {
    /// The searchable texts.
    index: &'search NoteSearchIndex,
    /// What to look for.
    query: &'search SearchQuery,
    /// A node shown even without a match (the open note).
    keep_visible: Option<NodeId>,
}

impl Search<'_> {
    /// Visits `nodes`; `inside_match` means an enclosing group's title
    /// matched, so everything is shown. Returns the matching notes found
    /// and whether any row was made visible.
    fn visit_nodes(
        &self,
        nodes: &[TreeNode],
        inside_match: bool,
        results: &mut SearchResults,
    ) -> (usize, bool) {
        let mut matches = 0;
        let mut any_visible = false;
        for node in nodes {
            let (node_matches, visible) = match node {
                TreeNode::Note(note) => self.visit_note(note, inside_match, results),
                TreeNode::Group(group) => self.visit_group(group, inside_match, results),
            };
            matches += node_matches;
            any_visible |= visible;
        }
        (matches, any_visible)
    }

    /// Visits one group: shown with everything inside if its title matches,
    /// otherwise shown when a descendant is.
    fn visit_group(
        &self,
        group: &Group,
        inside_match: bool,
        results: &mut SearchResults,
    ) -> (usize, bool) {
        let title_matches = self
            .index
            .groups
            .get(&group.id)
            .is_some_and(|title| self.query.matches_lowered(title));
        results.has_hits |= title_matches;
        let shows_everything = inside_match || title_matches;
        let (matches, child_visible) = self.visit_nodes(&group.children, shows_everything, results);
        let node = NodeId::Group(group.id);
        let visible = shows_everything || child_visible || self.keep_visible == Some(node);
        if visible {
            results.visible.insert(node);
            results.group_counts.insert(group.id, matches);
        }
        (matches, visible)
    }

    /// Visits one note; it matches when every term occurs in its title or
    /// its content (M119).
    fn visit_note(
        &self,
        note: &Note,
        inside_match: bool,
        results: &mut SearchResults,
    ) -> (usize, bool) {
        let node = NodeId::Note(note.id);
        let Some(indexed) = self.index.notes.get(&note.id) else {
            return (0, false);
        };
        let matches = self.query.terms().iter().all(|term| {
            indexed.title.contains(term.as_str()) || indexed.content.contains(term.as_str())
        });
        if matches {
            results.has_hits = true;
            if !self.query.matches_lowered(&indexed.title) {
                results
                    .content_hits
                    .insert(note.id, self.query.count_in_lowered(&indexed.content));
            }
        }
        let visible = matches || inside_match || self.keep_visible == Some(node);
        if visible {
            results.visible.insert(node);
        }
        (usize::from(matches), visible)
    }
}

#[cfg(test)]
mod tests {
    use jiff::Timestamp;

    use super::*;
    use trailkit_memo_shared::note_tree::NoteKind;

    /// A note with `title` and `body`.
    fn note(title: &str, body: &str) -> Note {
        let mut note = Note::new(title.to_owned(), NoteKind::Normal, Timestamp::UNIX_EPOCH);
        note.body = body.to_owned();
        note
    }

    /// A collapsed group with `title` and `children`.
    fn group(title: &str, children: Vec<TreeNode>) -> Group {
        let mut group = Group::new(title.to_owned());
        group.children = children;
        group.expanded = false;
        group
    }

    /// Runs `query` over `document`.
    fn search(document: &NotesDocument, query: &str, keep: Option<NodeId>) -> SearchResults {
        let query = SearchQuery::parse(query).expect("query has terms");
        SearchResults::compute(document, &NoteSearchIndex::build(document), &query, keep)
    }

    /// Root: note "Mithril" (body "ore farm"), group "Dailies" with note
    /// "Wizard vault" ("**Mithril** ore x2") and group "Raids" with note
    /// "Wing 1" ("[guide](https://ore.example)").
    fn sample() -> (NotesDocument, [NodeId; 5]) {
        let mithril = note("Mithril", "ore farm");
        let vault = note("Wizard vault", "**Mithril** ore x2, more ore");
        let wing = note("Wing 1", "[guide](https://ore.example)");
        let raids = group("Raids", vec![TreeNode::Note(wing.clone())]);
        let dailies = group(
            "Dailies",
            vec![
                TreeNode::Note(vault.clone()),
                TreeNode::Group(raids.clone()),
            ],
        );
        let ids = [
            NodeId::Note(mithril.id),
            NodeId::Group(dailies.id),
            NodeId::Note(vault.id),
            NodeId::Group(raids.id),
            NodeId::Note(wing.id),
        ];
        let document = NotesDocument {
            root: vec![TreeNode::Note(mithril), TreeNode::Group(dailies)],
            trash: Vec::new(),
        };
        (document, ids)
    }

    #[test]
    fn matching_notes_show_with_their_groups() {
        let (document, [mithril, dailies, vault, raids, wing]) = sample();
        let results = search(&document, "ORE", None);
        assert!(results.has_hits());
        for (node, visible) in [
            (mithril, true),
            (dailies, true),
            (vault, true),
            (raids, false),
            (wing, false),
        ] {
            assert_eq!(results.is_visible(node), visible, "{node:?}");
        }
        let NodeId::Group(dailies) = dailies else {
            unreachable!()
        };
        assert_eq!(results.group_count(dailies), 1);
    }

    #[test]
    fn content_only_matches_report_their_hit_count() {
        let (document, [mithril, _, vault, _, _]) = sample();
        let results = search(&document, "ore", None);
        let (NodeId::Note(mithril), NodeId::Note(vault)) = (mithril, vault) else {
            unreachable!()
        };
        // "ore", "more" contains "ore" too: three occurrences.
        assert_eq!(results.content_hits(vault), Some(3));
        assert_eq!(results.content_hits(mithril), Some(1));
        let results = search(&document, "mithril", None);
        assert_eq!(results.content_hits(mithril), None, "title match");
    }

    #[test]
    fn terms_may_split_between_title_and_content() {
        let (document, [mithril, ..]) = sample();
        assert!(search(&document, "mithril farm", None).is_visible(mithril));
        assert!(!search(&document, "mithril raid", None).is_visible(mithril));
    }

    #[test]
    fn a_matching_group_title_shows_everything_inside() {
        let (document, [_, dailies, vault, raids, wing]) = sample();
        let results = search(&document, "dailies", None);
        for node in [dailies, vault, raids, wing] {
            assert!(results.is_visible(node), "{node:?}");
        }
    }

    #[test]
    fn link_targets_and_markers_are_not_searched() {
        let (document, [.., wing]) = sample();
        assert!(!search(&document, "example", None).is_visible(wing));
        assert!(!search(&document, "**", None).has_hits());
        assert!(search(&document, "guide", None).is_visible(wing));
    }

    #[test]
    fn the_open_note_stays_visible_without_a_match() {
        let (document, [_, dailies, _, raids, wing]) = sample();
        let results = search(&document, "zzz", Some(wing));
        assert!(!results.has_hits());
        for node in [dailies, raids, wing] {
            assert!(results.is_visible(node), "{node:?}");
        }
    }
}
