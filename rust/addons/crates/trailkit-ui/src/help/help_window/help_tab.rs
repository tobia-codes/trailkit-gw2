//! The Help tab (R95, R100 Q298 variant A, R101): the topic list and the
//! selected topic as two panes with a splitter between them, like an addon's
//! main window. The selected topic lists its sections below it; a click on
//! one scrolls to its heading and marks it (R155 c, Q546 A). The topic has the
//! shared page head: its title in Heading1, the lead sentence in Secondary,
//! the separator, and the scrolling content.
//!
//! Above the list sits the help's search (BL30, mock v9): while it holds a
//! query, the list shows only matching topics, opened to their sections with
//! hits, with the hits counted at the row ends and marked in the titles; a
//! click jumps to the first hit. Ctrl+F finds in the shown topic below its
//! head.

use nexus::imgui::{ChildWindow, StyleVar, Ui};

use super::topic_search::{SearchMove, TopicSearch};
use super::{HelpSearchTexts, HelpWindowState, ParsedHelp, ParsedTopic};
use crate::{
    chat_link::chat_link_chip::ChipContext,
    help::help_search::{HelpSearchResults, SECTION_HEADING_LEVEL},
    layout::page::{PageHead, PagePart, draw_page_with_bar},
    layout::pane_columns::{FillColumn, FixedColumn, PaneColumns, draw_resizable_pane_columns},
    layout::stack::Stack,
    lists::list_row::{ListRow, RowChevron, draw_list_row},
    markdown::markdown_view::{MarkdownAction, draw_markdown_with_heading_tops, open_link},
    markdown::text_selection::TextSelection,
    text::text_search::SearchQuery,
    text::{TextStyle, draw_text_wrapped},
    theme::Theme,
};

/// The narrowest the topic list may be dragged, in body characters (Q315).
const MINIMUM_TOPIC_LIST_CHARACTERS: f32 = 8.0;

/// The room the topic keeps beside the list, in body characters (Q315).
const MINIMUM_TOPIC_CHARACTERS: f32 = 30.0;

/// What the Help tab shows besides the saved state; it is not saved, so the
/// help opens at a topic's start.
#[derive(Debug, Default)]
pub(super) struct TopicView {
    /// The section of the selected topic that was clicked last and is
    /// marked in the list; `None` marks the topic itself.
    section: Option<usize>,
    /// Where the topic should scroll in the next frame.
    scroll: Option<ScrollTarget>,
    /// The selected text of the shown topic (memo M217: everything that is
    /// read can be copied, user wish 2026-11-21).
    pub(super) selection: TextSelection,
    /// The topic the selection belongs to; another topic selects nothing.
    selection_topic: Option<usize>,
    /// The search above the list and find in the topic (BL30).
    pub(super) search: TopicSearch,
}

/// Where a topic scrolls after a click in the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScrollTarget {
    /// The topic's start.
    Start,
    /// The heading of the section at this position.
    Section(usize),
}

/// A click in the topic list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListClick {
    /// A topic's row or chevron.
    Topic(usize),
    /// A section row: the topic and the section's position in it.
    Section(usize, usize),
}

/// Draws the topic list with its search and the selected topic (clamped to
/// the list), and applies a click on a topic, a step of the search and a
/// drag of the splitter to `state`. Returns whether `state` should be
/// saved: a topic was chosen or the splitter let go.
pub(super) fn draw_help_tab(
    ui: &Ui,
    theme: &Theme,
    help: &ParsedHelp,
    state: &mut HelpWindowState,
    view: &mut TopicView,
    (chips, texts): (ChipContext<'_>, HelpSearchTexts<'_>),
) -> bool {
    if help.topics.is_empty() {
        return false;
    }
    if state.topic >= help.topics.len() {
        state.topic = 0;
    }
    let topic_list = [FixedColumn {
        id: "topics",
        characters: state.topic_list_characters,
        minimum_characters: MINIMUM_TOPIC_LIST_CHARACTERS,
    }];
    let columns = PaneColumns {
        fixed: &topic_list,
        fill: FillColumn {
            id: "topic",
            minimum_characters: MINIMUM_TOPIC_CHARACTERS,
        },
    };
    let mut click = None;
    let mut search_move = None;
    let selected = state.topic;
    let response = draw_resizable_pane_columns(ui, theme, &columns, |column| {
        if column == 0 {
            (click, search_move) = draw_list_column(ui, theme, help, selected, view, texts);
        } else {
            if view.selection_topic != Some(selected) {
                view.selection.clear();
                view.selection_topic = Some(selected);
            }
            draw_topic(
                ui,
                theme,
                (selected, &help.topics[selected]),
                view,
                (chips, texts),
            );
        }
    });
    if let Some((_, characters)) = response.resized {
        state.topic_list_characters = characters;
    }
    let chosen = match (click, search_move) {
        (Some(ListClick::Topic(index) | ListClick::Section(index, _)), _)
        | (None, Some(SearchMove::ToTopic(index))) => Some(index),
        (None, None) => None,
    };
    let topic_changed = chosen.is_some_and(|index| index != state.topic);
    if let Some(index) = chosen {
        state.topic = index;
    }
    apply_click(click, view);
    if search_move.is_some() {
        view.section = None;
    }
    topic_changed || response.released
}

/// Applies a click in the list to `view`: during a search it jumps to the
/// first hit of the topic or the section (Q578), else to the topic's start
/// or the section's heading.
fn apply_click(click: Option<ListClick>, view: &mut TopicView) {
    let Some(click) = click else {
        return;
    };
    let results = view.search.current().map(|current| &current.results);
    match click {
        ListClick::Topic(index) => {
            view.section = None;
            let has_hits = results.is_some_and(|results| results.topics[index].matches);
            if has_hits {
                view.search.start_at(index, 0);
            } else {
                view.scroll = Some(ScrollTarget::Start);
            }
        }
        ListClick::Section(index, section) => {
            view.section = Some(section);
            let first_hit = results
                .and_then(|results| results.topics.get(index))
                .and_then(|topic| topic.sections.get(section))
                .filter(|section| section.hits > 0)
                .map(|section| section.first_hit);
            match first_hit {
                Some(first_hit) => view.search.start_at(index, first_hit),
                None => view.scroll = Some(ScrollTarget::Section(section)),
            }
        }
    }
}

/// Draws the list column: the search, a divider (mock v9) and the rows in
/// their own scroll region, so the search stays in place. Returns a click
/// and a step of the search's arrows.
fn draw_list_column(
    ui: &Ui,
    theme: &Theme,
    help: &ParsedHelp,
    selected: usize,
    view: &mut TopicView,
    texts: HelpSearchTexts<'_>,
) -> (Option<ListClick>, Option<SearchMove>) {
    let mut click = None;
    let mut search_move = None;
    Stack::vertical(theme).build(ui, |stack| {
        search_move = stack.add(|| {
            view.search
                .draw_list_search(ui, theme, &help.index, selected, texts)
        });
        stack.add_separator(theme);
        stack.add(|| {
            ChildWindow::new("topic_rows")
                .size([0.0, 0.0])
                .draw_background(false)
                .build(ui, || {
                    let search = view
                        .search
                        .current()
                        .map(|current| (&current.query, &current.results));
                    click = match search {
                        Some((_, results)) if !results.has_hits() => {
                            draw_text_wrapped(ui, theme, TextStyle::Secondary, texts.no_results);
                            None
                        }
                        Some((query, results)) => draw_found_topics(
                            ui,
                            theme,
                            help,
                            selected,
                            view.section,
                            (query, results),
                        ),
                        None => draw_topic_list(ui, theme, help, selected, view.section),
                    };
                });
        });
    });
    (click, search_move)
}

/// Draws one row per topic and, below the selected one, a row per section
/// (R155 c), `space.xs` apart like memo's note list (R148), and returns the
/// click. The marked row is `section` of the selected topic, or the topic
/// itself. The gap is ImGui's item spacing, as in memo's list, so the indent
/// guides reach up through it (R156).
fn draw_topic_list(
    ui: &Ui,
    theme: &Theme,
    help: &ParsedHelp,
    selected: usize,
    section: Option<usize>,
) -> Option<ListClick> {
    let mut click = None;
    let _rows = ui.push_style_var(StyleVar::ItemSpacing([0.0, theme.spacing.extra_small]));
    for (index, topic) in help.topics.iter().enumerate() {
        let _id = ui.push_id(i32::try_from(index).unwrap_or(i32::MAX));
        let is_selected = index == selected;
        // The selected topic always shows its sections (R155 c).
        let chevron = match (topic.sections.is_empty(), is_selected) {
            (true, _) => RowChevron::Blank,
            (false, true) => RowChevron::Open,
            (false, false) => RowChevron::Closed,
        };
        let row = ListRow {
            selected: is_selected && section.is_none(),
            chevron,
            // Topics look like memo's groups, sections like its notes (Q531).
            title_style: TextStyle::BodyStrong,
            ..ListRow::new("##topic", &topic.title)
        };
        let response = draw_list_row(ui, theme, &row);
        if response.clicked || response.chevron_clicked {
            click = Some(ListClick::Topic(index));
        }
        if !is_selected {
            continue;
        }
        for (section_index, title) in topic.sections.iter().enumerate() {
            let _section_id = ui.push_id(i32::try_from(section_index).unwrap_or(i32::MAX));
            let row = ListRow {
                selected: section == Some(section_index),
                level: 1,
                ..ListRow::new("##section", title)
            };
            if draw_list_row(ui, theme, &row).clicked {
                click = Some(ListClick::Section(index, section_index));
            }
        }
    }
    click
}

/// Draws the topics that match the search (Q577 A): each opened to its
/// sections with hits (all of them when its title matches), the topic's hit
/// count at the row end, hits marked in the titles (Q580 A). Returns the
/// click.
fn draw_found_topics(
    ui: &Ui,
    theme: &Theme,
    help: &ParsedHelp,
    selected: usize,
    section: Option<usize>,
    (query, results): (&SearchQuery, &HelpSearchResults),
) -> Option<ListClick> {
    let mut click = None;
    let _rows = ui.push_style_var(StyleVar::ItemSpacing([0.0, theme.spacing.extra_small]));
    for (index, (topic, result)) in help.topics.iter().zip(&results.topics).enumerate() {
        if !result.matches {
            continue;
        }
        let _id = ui.push_id(i32::try_from(index).unwrap_or(i32::MAX));
        let shows_sections = result.sections.iter().any(|section| section.visible);
        let count = (result.hits > 0).then(|| result.hits.to_string());
        let row = ListRow {
            selected: index == selected && section.is_none(),
            chevron: if shows_sections {
                RowChevron::Open
            } else {
                RowChevron::Blank
            },
            title_style: TextStyle::BodyStrong,
            title_hits: Some(query),
            trailing: count.as_deref(),
            ..ListRow::new("##topic", &topic.title)
        };
        let response = draw_list_row(ui, theme, &row);
        if response.clicked || response.chevron_clicked {
            click = Some(ListClick::Topic(index));
        }
        for (section_index, (title, found)) in
            topic.sections.iter().zip(&result.sections).enumerate()
        {
            if !found.visible {
                continue;
            }
            let _section_id = ui.push_id(i32::try_from(section_index).unwrap_or(i32::MAX));
            let row = ListRow {
                selected: index == selected && section == Some(section_index),
                level: 1,
                title_hits: Some(query),
                ..ListRow::new("##section", title)
            };
            if draw_list_row(ui, theme, &row).clicked {
                click = Some(ListClick::Section(index, section_index));
            }
        }
    }
    click
}

/// Draws `topic` (at position `index`) with the page head, the find bar
/// while open, and its markdown below with the search's or find's hits
/// marked, scrolled to `view`'s target or the current hit once drawn. A
/// clicked web link opens in the browser.
fn draw_topic(
    ui: &Ui,
    theme: &Theme,
    (index, topic): (usize, &ParsedTopic),
    view: &mut TopicView,
    (chips, texts): (ChipContext<'_>, HelpSearchTexts<'_>),
) {
    let head = PageHead {
        title: &topic.title,
        trailing: None,
        subtitle: topic.lead.as_deref(),
    };
    let TopicView {
        scroll,
        selection,
        search,
        ..
    } = view;
    let has_bar = search.find_open();
    let action = draw_page_with_bar(ui, theme, "topic_body", head, has_bar, |part| match part {
        PagePart::Bar => {
            search.draw_find_bar(ui, theme, texts);
            None
        }
        PagePart::Content => {
            let mut marks = search.marks(index);
            let (action, heading_tops) = draw_markdown_with_heading_tops(
                ui,
                &topic.document,
                theme,
                chips,
                SECTION_HEADING_LEVEL,
                marks.as_mut().map(|marks| &mut marks.highlight),
                selection,
            );
            let drawn = marks.map(|marks| {
                (
                    marks.finding,
                    marks.highlight.hit_count(),
                    marks.highlight.current_hit_y(),
                )
            });
            let reveal = drawn.and_then(|(finding, count, current_hit_y)| {
                search.record_drawn(index, finding, count, current_hit_y)
            });
            match (reveal, scroll.take()) {
                // A hit wins over the click's own target (Q578).
                (Some(hit), _) => reveal_line(ui, hit),
                (None, Some(ScrollTarget::Start)) => ui.set_scroll_y(0.0),
                (None, Some(ScrollTarget::Section(section))) => {
                    ui.set_scroll_y(heading_tops.get(section).copied().unwrap_or_default());
                }
                (None, None) => {}
            }
            Some(action)
        }
    });
    if let Some(Some(MarkdownAction::OpenLink(url))) = action {
        open_link(&url);
    }
}

/// Scrolls the topic so the line at content `y` shows, a third down the
/// view, unless it is visible already (like memo M198).
fn reveal_line(ui: &Ui, y: f32) {
    let line = ui.text_line_height();
    let top = ui.scroll_y();
    let height = ui.window_size()[1];
    if y < top || y + line > top + height {
        ui.set_scroll_y((y - height / 3.0).max(0.0));
    }
}
