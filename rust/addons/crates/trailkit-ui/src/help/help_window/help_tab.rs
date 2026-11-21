//! The Help tab (R95, R100 Q298 variant A, R101): the topic list and the
//! selected topic as two panes with a splitter between them, like an addon's
//! main window. The selected topic lists its sections below it; a click on
//! one scrolls to its heading and marks it (R155 c, Q546 A). The topic has the shared page head: its title in Heading1,
//! the lead sentence in Secondary, the separator, and the scrolling content.

use nexus::imgui::{StyleVar, Ui};

use super::{HelpWindowState, ParsedHelp, ParsedTopic};
use crate::{
    chat_link::chat_link_chip::ChipContext,
    layout::page::{PageHead, draw_page},
    layout::pane_columns::{FillColumn, FixedColumn, PaneColumns, draw_resizable_pane_columns},
    lists::list_row::{ListRow, RowChevron, draw_list_row},
    markdown::markdown_view::{MarkdownAction, draw_markdown_with_heading_tops, open_link},
    markdown::text_selection::TextSelection,
    text::TextStyle,
    theme::Theme,
};

/// The narrowest the topic list may be dragged, in body characters (Q315).
const MINIMUM_TOPIC_LIST_CHARACTERS: f32 = 8.0;

/// The room the topic keeps beside the list, in body characters (Q315).
const MINIMUM_TOPIC_CHARACTERS: f32 = 30.0;

/// The heading level of a topic's sections (`###`).
const SECTION_HEADING_LEVEL: u8 = 3;

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
    /// A section row of the selected topic.
    Section(usize),
}

/// Draws the topic list and the selected topic (clamped to the list), and
/// applies a click on a topic and a drag of the splitter to `state`.
/// Returns whether `state` should be saved: a topic was chosen or the
/// splitter let go.
pub(super) fn draw_help_tab(
    ui: &Ui,
    theme: &Theme,
    help: &ParsedHelp,
    state: &mut HelpWindowState,
    view: &mut TopicView,
    chips: ChipContext<'_>,
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
    let selected = state.topic;
    let response = draw_resizable_pane_columns(ui, theme, &columns, |column| {
        if column == 0 {
            click = draw_topic_list(ui, theme, help, selected, view.section);
        } else {
            if view.selection_topic != Some(selected) {
                view.selection.clear();
                view.selection_topic = Some(selected);
            }
            draw_topic(ui, theme, &help.topics[selected], view, chips);
        }
    });
    if let Some((_, characters)) = response.resized {
        state.topic_list_characters = characters;
    }
    let topic_changed = matches!(click, Some(ListClick::Topic(index)) if index != state.topic);
    match click {
        Some(ListClick::Topic(index)) => {
            state.topic = index;
            view.section = None;
            view.scroll = Some(ScrollTarget::Start);
        }
        Some(ListClick::Section(index)) => {
            view.section = Some(index);
            view.scroll = Some(ScrollTarget::Section(index));
        }
        None => {}
    }
    topic_changed || response.released
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
                click = Some(ListClick::Section(section_index));
            }
        }
    }
    click
}

/// Draws `topic` with the page head and its markdown below, scrolled to
/// `view`'s target once it is drawn. A clicked web link opens in the
/// browser.
fn draw_topic(
    ui: &Ui,
    theme: &Theme,
    topic: &ParsedTopic,
    view: &mut TopicView,
    chips: ChipContext<'_>,
) {
    let head = PageHead {
        title: &topic.title,
        trailing: None,
        subtitle: topic.lead.as_deref(),
    };
    let action = draw_page(ui, theme, "topic_body", head, || {
        let (action, heading_tops) = draw_markdown_with_heading_tops(
            ui,
            &topic.document,
            theme,
            chips,
            SECTION_HEADING_LEVEL,
            &mut view.selection,
        );
        match view.scroll.take() {
            Some(ScrollTarget::Start) => ui.set_scroll_y(0.0),
            Some(ScrollTarget::Section(index)) => {
                ui.set_scroll_y(heading_tops.get(index).copied().unwrap_or_default());
            }
            None => {}
        }
        action
    });
    if let Some(Some(MarkdownAction::OpenLink(url))) = action {
        open_link(&url);
    }
}
