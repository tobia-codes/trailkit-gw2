//! Stepping through the hits of a search over many pages (BL30, mock
//! https://claude.ai/artifact/DQRhxQtb4zyUxKB1hBtEVy v9): memo's notes and
//! trash, a help window's topics. The arrows go through the hits of the shown
//! page, then on into the next or previous page with hits, wrapping at either
//! end (memo Q505); "x of y" counts over all of them.
//!
//! ImGui-free, so it is tested on any platform. The app lists the pages with
//! hits as [`HitStop`]s in their list order and draws the shown page with the
//! current hit filled.

/// One page with hits that the arrows stop at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitStop<Page> {
    /// The page.
    pub page: Page,
    /// Its hits, at least one: a page that matches only by its title is one
    /// stop, so the arrows reach every match.
    pub hits: usize,
}

/// Where the shown page's current hit lies among all hits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitPosition {
    /// The 1-based place of the current hit, `None` while the shown page is
    /// no stop (e.g. a note shown because its group's title matches).
    pub current: Option<usize>,
    /// All hits of all stops.
    pub total: usize,
}

/// Where a step with the arrows leads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitStep<Page> {
    /// Another hit of the shown page: scroll it into view.
    InsidePage,
    /// The hit `current` of another `page`: select and show it.
    ToPage {
        /// The page to show.
        page: Page,
        /// Its current hit.
        current: usize,
    },
}

/// The current hit inside the shown page, kept between frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchSteps<Page> {
    /// The page the position belongs to; another shown page starts at its
    /// first hit.
    page: Option<Page>,
    /// The current hit in the page, 0-based in drawing order.
    current: usize,
    /// How many hits the page drew in the last frame; it replaces the
    /// stop's own count, which may miss what only drawing sees (chips).
    drawn: Option<usize>,
}

impl<Page> Default for SearchSteps<Page> {
    /// No page yet.
    fn default() -> Self {
        Self {
            page: None,
            current: 0,
            drawn: None,
        }
    }
}

impl<Page: Copy + PartialEq> SearchSteps<Page> {
    /// Starts over at the hit `current` of `page` (a new query, a click).
    pub fn start_at(&mut self, page: Option<Page>, current: usize) {
        self.page = page;
        self.current = current;
        self.drawn = None;
    }

    /// The current hit inside the shown page, 0-based.
    #[must_use]
    pub const fn current(&self) -> usize {
        self.current
    }

    /// The hits of `stop`: the drawn count once the page was drawn, else
    /// the stop's own.
    fn hits_of(&self, stop: &HitStop<Page>) -> usize {
        match self.drawn {
            Some(drawn) if self.page == Some(stop.page) => drawn.max(1),
            _ => stop.hits.max(1),
        }
    }

    /// Where the current hit of `shown` lies among the hits of `stops`;
    /// `None` without stops.
    #[must_use]
    pub fn position(&self, stops: &[HitStop<Page>], shown: Option<Page>) -> Option<HitPosition> {
        let mut total = 0;
        let mut current = None;
        for stop in stops {
            let hits = self.hits_of(stop);
            if Some(stop.page) == shown {
                current = Some(total + self.current.min(hits - 1) + 1);
            }
            total += hits;
        }
        (total > 0).then_some(HitPosition { current, total })
    }

    /// Steps to the next hit (`forward`) or the previous one from `shown`:
    /// inside the page first, then into the next or previous stop, wrapping
    /// at either end. `None` without stops.
    pub fn step(
        &mut self,
        stops: &[HitStop<Page>],
        shown: Option<Page>,
        forward: bool,
    ) -> Option<HitStep<Page>> {
        if stops.is_empty() {
            return None;
        }
        let place = stops.iter().position(|stop| Some(stop.page) == shown);
        if let Some(place) = place {
            let hits = self.hits_of(&stops[place]);
            let inside = self.current.min(hits - 1);
            let next_inside = if forward {
                (inside + 1 < hits).then_some(inside + 1)
            } else {
                inside.checked_sub(1)
            };
            if let Some(next_inside) = next_inside {
                self.page = shown;
                self.current = next_inside;
                return Some(HitStep::InsidePage);
            }
        }
        let count = stops.len();
        let target = match (place, forward) {
            (Some(place), true) => (place + 1) % count,
            (Some(place), false) => (place + count - 1) % count,
            (None, true) => 0,
            (None, false) => count - 1,
        };
        let stop = stops[target];
        let current = if forward { 0 } else { stop.hits.max(1) - 1 };
        self.start_at(Some(stop.page), current);
        Some(HitStep::ToPage {
            page: stop.page,
            current,
        })
    }

    /// Takes the hits `shown` drew in this frame: another page starts at its
    /// first hit, and the current hit stays in range.
    pub fn record_drawn(&mut self, shown: Option<Page>, drawn: usize) {
        if shown != self.page {
            self.start_at(shown, 0);
        }
        self.drawn = Some(drawn);
        if drawn > 0 && self.current >= drawn {
            self.current = drawn - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stops `(page, hits)`.
    fn stops(pages: &[(u8, usize)]) -> Vec<HitStop<u8>> {
        pages
            .iter()
            .map(|&(page, hits)| HitStop { page, hits })
            .collect()
    }

    #[test]
    fn steps_go_through_the_page_then_wrap_over_all_stops() {
        let stops = stops(&[(1, 2), (3, 1)]);
        let mut steps = SearchSteps::default();
        steps.start_at(Some(1), 0);
        let place = |steps: &SearchSteps<u8>, shown| {
            steps.position(&stops, Some(shown)).and_then(|p| p.current)
        };
        assert_eq!(place(&steps, 1), Some(1));
        assert_eq!(steps.step(&stops, Some(1), true), Some(HitStep::InsidePage));
        assert_eq!(place(&steps, 1), Some(2));
        assert_eq!(
            steps.step(&stops, Some(1), true),
            Some(HitStep::ToPage {
                page: 3,
                current: 0
            })
        );
        assert_eq!(place(&steps, 3), Some(3));
        assert_eq!(
            steps.step(&stops, Some(3), true),
            Some(HitStep::ToPage {
                page: 1,
                current: 0
            }),
            "wraps after the last hit"
        );
        assert_eq!(
            steps.step(&stops, Some(1), false),
            Some(HitStep::ToPage {
                page: 3,
                current: 0
            }),
            "wraps before the first hit"
        );
    }

    #[test]
    fn backwards_lands_on_the_last_hit_of_the_previous_page() {
        let stops = stops(&[(1, 3), (2, 1)]);
        let mut steps = SearchSteps::default();
        steps.start_at(Some(2), 0);
        assert_eq!(
            steps.step(&stops, Some(2), false),
            Some(HitStep::ToPage {
                page: 1,
                current: 2
            })
        );
        assert_eq!(
            steps.position(&stops, Some(1)),
            Some(HitPosition {
                current: Some(3),
                total: 4
            })
        );
    }

    #[test]
    fn a_page_without_hits_has_no_place_and_steps_to_the_ends() {
        let stops = stops(&[(1, 1), (2, 1)]);
        let mut steps = SearchSteps::default();
        steps.start_at(Some(9), 0);
        assert_eq!(
            steps.position(&stops, Some(9)),
            Some(HitPosition {
                current: None,
                total: 2
            })
        );
        assert_eq!(
            steps.step(&stops, Some(9), false),
            Some(HitStep::ToPage {
                page: 2,
                current: 0
            })
        );
        assert_eq!(steps.step(&[], Some(9), true), None);
        assert_eq!(steps.position(&[], Some(9)), None);
    }

    #[test]
    fn drawn_hits_replace_the_stop_count_and_clamp_the_cursor() {
        let stops = stops(&[(1, 3)]);
        let mut steps = SearchSteps::default();
        steps.start_at(Some(1), 2);
        steps.record_drawn(Some(1), 2);
        assert_eq!(steps.current(), 1);
        assert_eq!(
            steps.position(&stops, Some(1)),
            Some(HitPosition {
                current: Some(2),
                total: 2
            })
        );
        // Another shown page starts at its first hit.
        steps.record_drawn(Some(4), 5);
        assert_eq!(steps.current(), 0);
    }
}
