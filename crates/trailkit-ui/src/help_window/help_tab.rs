//! The Help tab (R95, R100 Q298 variant A, R101): the topic list and the
//! selected topic as two panes with a splitter between them, like an addon's
//! main window. The topic has the shared page head: its title in Heading1,
//! the lead sentence in Secondary, the separator, and the scrolling content.

use nexus::imgui::Ui;

use super::{HelpWindowState, ParsedHelp, ParsedTopic};
use crate::{
    chat_link_chip::ChipContext,
    list_row::{ListRow, draw_list_row},
    markdown_view::{MarkdownAction, draw_markdown, first_line_font, open_web_link},
    page::{PageHead, draw_page},
    text::TextStyle,
    theme::Theme,
    two_panes::{PaneSide, TwoPanes, draw_two_panes},
};

/// The narrowest the topic list may be dragged, in body characters (Q315).
const MINIMUM_TOPIC_LIST_CHARACTERS: f32 = 8.0;

/// The room the topic keeps beside the list, in body characters (Q315).
const MINIMUM_TOPIC_CHARACTERS: f32 = 30.0;

/// Draws the topic list and the selected topic (clamped to the list), and
/// applies a click on a topic and a drag of the splitter to `state`.
/// Returns whether `state` should be saved: a topic was chosen or the
/// splitter let go.
pub(super) fn draw_help_tab(
    ui: &Ui,
    theme: &Theme,
    help: &ParsedHelp,
    state: &mut HelpWindowState,
    chips: ChipContext<'_>,
) -> bool {
    if help.topics.is_empty() {
        return false;
    }
    if state.topic >= help.topics.len() {
        state.topic = 0;
    }
    let panes = TwoPanes {
        left_id: "topics",
        right_id: "topic",
        left_characters: state.topic_list_characters,
        left_minimum_characters: MINIMUM_TOPIC_LIST_CHARACTERS,
        right_minimum_characters: MINIMUM_TOPIC_CHARACTERS,
    };
    let mut chosen = None;
    let selected = state.topic;
    let response = draw_two_panes(ui, theme, &panes, |side| match side {
        PaneSide::Left => chosen = draw_topic_list(ui, theme, help, selected),
        PaneSide::Right => draw_topic(ui, theme, &help.topics[selected], chips),
    });
    if let Some(characters) = response.left_characters {
        state.topic_list_characters = characters;
    }
    let topic_changed = chosen.is_some_and(|index| index != state.topic);
    if let Some(index) = chosen {
        state.topic = index;
    }
    topic_changed || response.released
}

/// Draws one row per topic and returns the clicked one.
fn draw_topic_list(ui: &Ui, theme: &Theme, help: &ParsedHelp, selected: usize) -> Option<usize> {
    let mut clicked = None;
    for (index, topic) in help.topics.iter().enumerate() {
        let _id = ui.push_id(i32::try_from(index).unwrap_or(i32::MAX));
        let row = ListRow {
            id: "##topic",
            selected: index == selected,
            title: &topic.title,
            title_style: TextStyle::Body,
            trailing: None,
            detail: None,
        };
        if draw_list_row(ui, theme, &row).clicked {
            clicked = Some(index);
        }
    }
    clicked
}

/// Draws `topic` with the page head and its markdown below. A clicked web
/// link opens in the browser.
fn draw_topic(ui: &Ui, theme: &Theme, topic: &ParsedTopic, chips: ChipContext<'_>) {
    let head = PageHead {
        title: &topic.title,
        trailing: None,
        subtitle: topic.lead.as_deref(),
        first_line: first_line_font(&topic.document),
    };
    let action = draw_page(ui, theme, "topic_body", head, || {
        draw_markdown(ui, &topic.document, theme, chips, None)
    });
    if let Some(Some(MarkdownAction::OpenLink(url))) = action {
        open_web_link(&url);
    }
}
