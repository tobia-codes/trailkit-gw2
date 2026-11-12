//! The Help tab (R95, R100 Q298 variant A): the topic list on the left at a
//! fixed width, a faint line, and the selected topic on the right, each
//! scrolling on its own.

use nexus::imgui::{ChildWindow, Ui};

use super::{ParsedHelp, ParsedTopic};
use crate::{
    chat_link_chip::ChipContext,
    list_row::{ListRow, draw_list_row},
    markdown_view::{MarkdownAction, draw_markdown, open_web_link},
    text::{TextStyle, draw_text_wrapped},
    text_layout::body_characters_width,
    theme::Theme,
};

/// The width of the topic list in body characters: 112 px at the 16 px
/// body font of the minimum window (R95).
const TOPIC_LIST_CHARACTERS: f32 = 14.0;

/// Draws the topic list and the topic with index `selected` (clamped to the
/// list), and selects a clicked topic.
pub(super) fn draw_help_tab(
    ui: &Ui,
    theme: &Theme,
    help: &ParsedHelp,
    selected: &mut usize,
    chips: ChipContext<'_>,
) {
    if help.topics.is_empty() {
        return;
    }
    *selected = (*selected).min(help.topics.len() - 1);
    let list_width = body_characters_width(ui, TOPIC_LIST_CHARACTERS);
    let top = ui.cursor_screen_pos()[1];
    ChildWindow::new("topics")
        .size([list_width, 0.0])
        .build(ui, || {
            for (index, topic) in help.topics.iter().enumerate() {
                let _id = ui.push_id(i32::try_from(index).unwrap_or(i32::MAX));
                let row = ListRow {
                    id: "##topic",
                    selected: index == *selected,
                    title: &topic.title,
                    title_style: TextStyle::Body,
                    trailing: None,
                    detail: None,
                };
                if draw_list_row(ui, theme, &row).clicked {
                    *selected = index;
                }
            }
        });
    let gap = theme.spacing.small;
    ui.same_line_with_spacing(0.0, gap);
    let line_x = ui.cursor_screen_pos()[0];
    let bottom = top + ui.item_rect_size()[1];
    ui.get_window_draw_list()
        .add_line(
            [line_x, top],
            [line_x, bottom],
            theme.border.subtle.to_array(),
        )
        .thickness(theme.border.width)
        .build();
    ui.same_line_with_spacing(0.0, gap + theme.border.width);
    ChildWindow::new("topic").size([0.0, 0.0]).build(ui, || {
        draw_topic(ui, theme, &help.topics[*selected], chips);
    });
}

/// Draws `topic`: Heading2 title, the Secondary lead and the markdown body.
/// A clicked web link opens in the browser.
fn draw_topic(ui: &Ui, theme: &Theme, topic: &ParsedTopic, chips: ChipContext<'_>) {
    draw_text_wrapped(ui, theme, TextStyle::Heading2, &topic.title);
    if let Some(lead) = &topic.lead {
        draw_text_wrapped(ui, theme, TextStyle::Secondary, lead);
    }
    if let Some(MarkdownAction::OpenLink(url)) =
        draw_markdown(ui, &topic.document, theme, chips, None)
    {
        open_web_link(&url);
    }
}
