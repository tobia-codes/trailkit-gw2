//! Searching the note list (memo plan M118–M126): which rows a search shows,
//! how many notes match per group, and how often a note's content matches.
//!
//! ImGui-free so it is unit-tested on any platform. The index is rebuilt
//! only when the notes change; results only when the query, the notes or the
//! selection change — never per frame.

use std::collections::{HashMap, HashSet};

use trailkit_ui::{markdown, text::text_search::SearchQuery};

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

    /// Indexes every trash entry of `document` with what trashed groups
    /// hold (BL31, M218).
    #[must_use]
    pub fn build_trash(document: &NotesDocument) -> Self {
        let mut index = Self::default();
        for entry in &document.trash {
            index.add_nodes(std::slice::from_ref(&entry.node));
        }
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
    /// Whether any note or group matches; without, the list shows the empty
    /// state (M126).
    has_hits: bool,
    /// The matching notes in list order with how many hits each holds for
    /// the arrows of the search (BL30, mock v9).
    stops: Vec<HitStop>,
}

/// One matching note the search arrows step through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitStop {
    /// The note.
    pub note: NoteId,
    /// Its hits: the marked places in its text, or one for a note that
    /// matches by its title only, so every matching note is a stop.
    pub hits: usize,
}

impl SearchResults {
    /// Searches `document` with `index` (built from the same document).
    /// Only matches are visible; the open note too only when it matches
    /// (M196).
    #[must_use]
    pub fn compute(document: &NotesDocument, index: &NoteSearchIndex, query: &SearchQuery) -> Self {
        let mut results = Self::default();
        let search = Search { index, query };
        search.visit_nodes(&document.root, false, &mut results);
        results
    }

    /// Searches the trash of `document` with an index from
    /// [`NoteSearchIndex::build_trash`], entries newest first as the trash
    /// lists them (BL31); notes inside trashed groups are searched too.
    #[must_use]
    pub fn compute_trash(
        document: &NotesDocument,
        index: &NoteSearchIndex,
        query: &SearchQuery,
    ) -> Self {
        let mut results = Self::default();
        let search = Search { index, query };
        for entry in document.trash.iter().rev() {
            search.visit_nodes(std::slice::from_ref(&entry.node), false, &mut results);
        }
        results
    }

    /// Whether anything matches.
    #[must_use]
    pub const fn has_hits(&self) -> bool {
        self.has_hits
    }

    /// The matching notes in list order, for stepping through the hits.
    #[must_use]
    pub fn hit_stops(&self) -> &[HitStop] {
        &self.stops
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
        let visible = shows_everything || child_visible;
        if visible {
            results.visible.insert(NodeId::Group(group.id));
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
            // Counted like the reading view marks them: merged ranges of
            // every term in the visible text (M120, M122).
            let text_hits = self.query.hit_ranges(&indexed.content).len();
            results.stops.push(HitStop {
                note: note.id,
                hits: text_hits.max(1),
            });
            if !self.query.matches_lowered(&indexed.title) {
                results
                    .content_hits
                    .insert(note.id, self.query.count_in_lowered(&indexed.content));
            }
        }
        let visible = matches || inside_match;
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
    use trailkit_memo_shared::note_tree::ResetSchedule;

    /// A note with `title` and `body`.
    fn note(title: &str, body: &str) -> Note {
        let mut note = Note::new(
            title.to_owned(),
            ResetSchedule::Never,
            Timestamp::UNIX_EPOCH,
        );
        note.body = body.to_owned();
        note
    }

    /// A group with `title` and `children` (search ignores whether it is
    /// collapsed, M124).
    fn group(title: &str, children: Vec<TreeNode>) -> Group {
        let mut group = Group::new(title.to_owned());
        group.children = children;
        group
    }

    /// Runs `query` over `document`.
    fn search(document: &NotesDocument, query: &str) -> SearchResults {
        let query = SearchQuery::parse(query).expect("query has terms");
        SearchResults::compute(document, &NoteSearchIndex::build(document), &query)
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
            unknown_fields: Default::default(),
        };
        (document, ids)
    }

    #[test]
    fn matching_notes_show_with_their_groups() {
        let (document, [mithril, dailies, vault, raids, wing]) = sample();
        let results = search(&document, "ORE");
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
        let results = search(&document, "ore");
        let (NodeId::Note(mithril), NodeId::Note(vault)) = (mithril, vault) else {
            unreachable!()
        };
        // "ore", "more" contains "ore" too: three occurrences.
        assert_eq!(results.content_hits(vault), Some(3));
        assert_eq!(results.content_hits(mithril), Some(1));
        let results = search(&document, "mithril");
        assert_eq!(results.content_hits(mithril), None, "title match");
    }

    #[test]
    fn terms_may_split_between_title_and_content() {
        let (document, [mithril, ..]) = sample();
        assert!(search(&document, "mithril farm").is_visible(mithril));
        assert!(!search(&document, "mithril raid").is_visible(mithril));
    }

    #[test]
    fn a_matching_group_title_shows_everything_inside() {
        let (document, [_, dailies, vault, raids, wing]) = sample();
        let results = search(&document, "dailies");
        for node in [dailies, vault, raids, wing] {
            assert!(results.is_visible(node), "{node:?}");
        }
    }

    #[test]
    fn link_targets_and_markers_are_not_searched() {
        let (document, [.., wing]) = sample();
        assert!(!search(&document, "example").is_visible(wing));
        assert!(!search(&document, "**").has_hits());
        assert!(search(&document, "guide").is_visible(wing));
    }

    #[test]
    fn matching_notes_are_stops_in_list_order() {
        let (document, [mithril, _, vault, _, wing]) = sample();
        let (NodeId::Note(mithril), NodeId::Note(vault), NodeId::Note(wing)) =
            (mithril, vault, wing)
        else {
            unreachable!()
        };
        let stops = |query: &str| -> Vec<(NoteId, usize)> {
            search(&document, query)
                .hit_stops()
                .iter()
                .map(|stop| (stop.note, stop.hits))
                .collect()
        };
        // "more" holds "ore" too: three marked places in the vault note.
        assert_eq!(stops("ore"), vec![(mithril, 1), (vault, 3)]);
        // A title-only match is one stop; a group-title match adds none.
        assert_eq!(stops("wing"), vec![(wing, 1)]);
        assert!(stops("dailies").is_empty());
        // Two terms: their places merge where they touch.
        assert_eq!(stops("mithril ore"), vec![(mithril, 1), (vault, 4)]);
    }

    #[test]
    fn the_trash_is_searched_newest_first_with_group_contents() {
        let (mut document, [mithril, dailies, vault, _, _]) = sample();
        for (node, seconds) in [(mithril, 1), (dailies, 2)] {
            let now = Timestamp::from_second(seconds).expect("valid time");
            assert!(document.move_to_trash(node, now));
        }
        let query = SearchQuery::parse("ore").expect("query has terms");
        let index = NoteSearchIndex::build_trash(&document);
        let results = SearchResults::compute_trash(&document, &index, &query);
        let (NodeId::Note(mithril_note), NodeId::Note(vault_note)) = (mithril, vault) else {
            unreachable!()
        };
        let order: Vec<_> = results.hit_stops().iter().map(|stop| stop.note).collect();
        assert_eq!(order, vec![vault_note, mithril_note], "newest entry first");
        assert!(results.is_visible(dailies) && results.is_visible(vault));
        assert!(!SearchResults::compute(&document, &index, &query).has_hits());
    }

    #[test]
    fn without_a_match_nothing_is_visible() {
        let (document, [mithril, dailies, vault, raids, wing]) = sample();
        let results = search(&document, "zzz");
        assert!(!results.has_hits());
        for node in [mithril, dailies, vault, raids, wing] {
            assert!(!results.is_visible(node), "{node:?} (M196)");
        }
    }
}
