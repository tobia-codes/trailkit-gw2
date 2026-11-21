//! The search of the Help tab and find in the shown topic (BL30, Q575–Q583,
//! mock https://claude.ai/artifact/DQRhxQtb4zyUxKB1hBtEVy v9): the always
//! shown field above the topic list with "x of y" and the arrows over all
//! topics, and the Ctrl+F bar below the topic's head. Both use the shared
//! search of `controls::search`.

use nexus::imgui::Ui;

use super::HelpSearchTexts;
use crate::{
    controls::search::{
        bar::{CountPlacement, SearchBarTexts, SearchStatus, draw_search_bar},
        field::{SearchField, SearchFieldTexts},
    },
    help::help_search::{HelpSearchResults, IndexedTopic},
    markdown::markdown_view::SearchHighlight,
    text::search_steps::{HitStep, SearchSteps},
    text::text_search::SearchQuery,
    theme::Theme,
};

/// The search of the Help tab and find in the topic, kept between frames.
#[derive(Debug, Default)]
pub(super) struct TopicSearch {
    /// The field above the topic list; never closed (Q575 A).
    field: SearchField,
    /// The query and its results while the field holds one.
    current: Option<CurrentQuery>,
    /// The current hit of the list search in the shown topic.
    steps: SearchSteps<usize>,
    /// Find in the shown topic (Ctrl+F).
    find: Find,
    /// Scroll the current hit into view once it is drawn.
    reveal: bool,
}

/// One query and what it found.
#[derive(Debug)]
pub(super) struct CurrentQuery {
    /// The field text the results belong to.
    text: String,
    /// The parsed query.
    pub(super) query: SearchQuery,
    /// What the topic list shows.
    pub(super) results: HelpSearchResults,
}

/// Find in the shown topic (Ctrl+F, like memo M198).
#[derive(Debug, Default)]
struct Find {
    /// Whether the bar is open.
    open: bool,
    /// The find field; its text survives closing.
    field: SearchField,
    /// The phrase of the field, `None` while it is blank.
    query: Option<SearchQuery>,
    /// The current hit, 0-based in drawing order.
    current: usize,
    /// How many hits the topic drew in the last frame.
    hits: usize,
}

/// What the topic draws marked in this frame.
pub(super) struct TopicMarks<'a> {
    /// What to mark, the current hit filled.
    pub(super) highlight: SearchHighlight<'a>,
    /// Whether the marks are find's (else the list search's).
    pub(super) finding: bool,
}

/// A step the list search asks of the Help tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SearchMove {
    /// Show this topic at its current hit.
    ToTopic(usize),
}

impl TopicSearch {
    /// The query and its results while the field holds one.
    pub(super) const fn current(&self) -> Option<&CurrentQuery> {
        self.current.as_ref()
    }

    /// Whether find in the topic is open.
    pub(super) const fn find_open(&self) -> bool {
        self.find.open
    }

    /// Opens find in the topic with the list search's text, if any, or
    /// closes it while open (Ctrl+F).
    pub(super) fn toggle_find(&mut self) {
        if self.find.open {
            self.find.open = false;
            return;
        }
        let offered = self.field.text.trim();
        if !offered.is_empty() && offered != self.find.field.text {
            let offered = offered.to_owned();
            self.find.field.set_text(&offered);
            self.find_text_changed();
        }
        if self.find.query.is_none() {
            self.find_text_changed();
        }
        self.find.open = true;
        self.find.field.request_focus();
        self.reveal = true;
    }

    /// Puts `text` into the search field as if typed.
    pub(super) fn set_search_text(&mut self, text: &str) {
        self.field.set_text(text);
    }

    /// Opens find with `text` in its field as if typed.
    pub(super) fn open_find_with(&mut self, text: &str) {
        self.find.field.set_text(text);
        self.find_text_changed();
        self.find.open = true;
    }

    /// The find text changed: parse it and start at the first hit.
    fn find_text_changed(&mut self) {
        self.find.query = SearchQuery::phrase(&self.find.field.text);
        self.find.current = 0;
        self.reveal = true;
    }

    /// The next (`forward`) or previous find hit, wrapping (memo Q505).
    pub(super) fn step_find(&mut self, forward: bool) {
        let count = self.find.hits;
        if count == 0 {
            return;
        }
        let current = self.find.current.min(count - 1);
        self.find.current = if forward {
            (current + 1) % count
        } else {
            (current + count - 1) % count
        };
        self.reveal = true;
    }

    /// Brings the results up to date with the field; a new query starts at
    /// the first hit of the shown topic `shown`.
    fn update(&mut self, index: &[IndexedTopic], shown: usize) {
        if self
            .current
            .as_ref()
            .is_some_and(|current| current.text == self.field.text)
        {
            return;
        }
        let Some(query) = SearchQuery::parse(&self.field.text) else {
            self.current = None;
            return;
        };
        if self
            .current
            .as_ref()
            .is_none_or(|current| current.query != query)
        {
            self.steps.start_at(Some(shown), 0);
            self.reveal = true;
        }
        let results = HelpSearchResults::compute(index, &query);
        self.current = Some(CurrentQuery {
            text: self.field.text.clone(),
            query,
            results,
        });
    }

    /// Starts the list search's hits in `topic` at its hit `first_hit`, after
    /// a click on the topic or one of its sections (Q578).
    pub(super) fn start_at(&mut self, topic: usize, first_hit: usize) {
        self.steps.start_at(Some(topic), first_hit);
        self.reveal = true;
    }

    /// Draws the search above the topic list (not closable, Q575 A) with
    /// the count and arrows below it while topics match; returns a step to
    /// another topic.
    pub(super) fn draw_list_search(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        index: &[IndexedTopic],
        shown: usize,
        texts: HelpSearchTexts<'_>,
    ) -> Option<SearchMove> {
        self.update(index, shown);
        let stops = self
            .current
            .as_ref()
            .map_or(&[][..], |current| current.results.hit_stops());
        let position = self.steps.position(stops, Some(shown));
        let count = position.map_or_else(String::new, |position| {
            let current = position
                .current
                .map_or_else(|| "–".to_owned(), |current| current.to_string());
            texts
                .count
                .replace("{current}", &current)
                .replace("{total}", &position.total.to_string())
        });
        let status = if position.is_some() {
            SearchStatus::Hits
        } else {
            SearchStatus::Empty
        };
        let bar_texts = SearchBarTexts {
            field: SearchFieldTexts {
                hint: texts.search_hint,
                clear: texts.clear,
                close: None,
            },
            count: &count,
            no_results: texts.no_results,
            previous: texts.previous,
            next: texts.next,
        };
        let response = draw_search_bar(
            ui,
            theme,
            &mut self.field,
            status,
            CountPlacement::Below,
            bar_texts,
        );
        if response.text_changed {
            self.update(index, shown);
        }
        let forward = if response.next {
            true
        } else if response.previous {
            false
        } else {
            return None;
        };
        let stops = self.current.as_ref()?.results.hit_stops();
        let step = self.steps.step(stops, Some(shown), forward)?;
        self.reveal = true;
        match step {
            HitStep::InsidePage => None,
            HitStep::ToPage { page, .. } => Some(SearchMove::ToTopic(page)),
        }
    }

    /// Draws the find bar of the topic while open (closable, count beside
    /// the field, wrapping when narrow).
    pub(super) fn draw_find_bar(&mut self, ui: &Ui, theme: &Theme, texts: HelpSearchTexts<'_>) {
        let count = texts
            .count
            .replace("{current}", &(self.find.current + 1).to_string())
            .replace("{total}", &self.find.hits.to_string());
        let status = match (&self.find.query, self.find.hits) {
            (None, _) => SearchStatus::Empty,
            (Some(_), 0) => SearchStatus::NoResults,
            (Some(_), _) => SearchStatus::Hits,
        };
        let bar_texts = SearchBarTexts {
            field: SearchFieldTexts {
                hint: texts.find_hint,
                clear: texts.clear,
                close: Some(texts.close),
            },
            count: &count,
            no_results: texts.no_results,
            previous: texts.previous,
            next: texts.next,
        };
        let response = draw_search_bar(
            ui,
            theme,
            &mut self.find.field,
            status,
            CountPlacement::Beside,
            bar_texts,
        );
        if response.text_changed {
            self.find_text_changed();
        }
        if response.previous {
            self.step_find(false);
        }
        if response.next {
            self.step_find(true);
        }
        if response.close_clicked {
            self.find.open = false;
        }
    }

    /// What `topic` marks in this frame: find's hits while it has a
    /// phrase, else the list search's, each with the current hit filled.
    pub(super) fn marks(&self, topic: usize) -> Option<TopicMarks<'_>> {
        if self.find.open
            && let Some(query) = &self.find.query
        {
            return Some(TopicMarks {
                highlight: SearchHighlight::with_current(query, self.find.current),
                finding: true,
            });
        }
        let current = self.current.as_ref()?;
        let shows_topic = current
            .results
            .topics
            .get(topic)
            .is_some_and(|result| result.matches);
        // A topic shown without a match marks nothing filled.
        let hit = if shows_topic {
            self.steps.current()
        } else {
            usize::MAX
        };
        Some(TopicMarks {
            highlight: SearchHighlight::with_current(&current.query, hit),
            finding: false,
        })
    }

    /// Takes what `topic` drew: `drawn` hits, marked by find if `finding`,
    /// with the current hit at content y `current_hit_y`. Returns the y to
    /// scroll into view, once after a step or a new query.
    pub(super) fn record_drawn(
        &mut self,
        topic: usize,
        finding: bool,
        drawn: usize,
        current_hit_y: Option<f32>,
    ) -> Option<f32> {
        if finding {
            self.find.hits = drawn;
            if self.find.current >= drawn {
                self.find.current = 0;
            }
        } else {
            self.steps.record_drawn(Some(topic), drawn);
        }
        current_hit_y.filter(|_| std::mem::take(&mut self.reveal))
    }
}
