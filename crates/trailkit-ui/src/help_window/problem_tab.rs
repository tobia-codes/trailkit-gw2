//! The Report a problem tab (R95, R100 Q299–Q302): the page title, the
//! sentence that nothing is sent automatically, step 1 "Save the log" with
//! the export button, its hint and status line, step 2 "Report on GitHub"
//! with the primary button that opens the browser, and the footer naming
//! the log folders. The steps are started one by one; the GitHub button is
//! never disabled.

use nexus::imgui::{ChildWindow, Ui};

use super::{HelpAction, HelpWindowContent};
use crate::{
    button::{Button, ButtonKind},
    icons::Icon,
    separator::draw_separator,
    status_line::draw_status_line,
    text::{TextStyle, draw_text, draw_text_wrapped},
    text_layout::format_count,
    theme::Theme,
};

/// Draws the page in its own scrolling region and returns the clicked
/// action.
pub(super) fn draw_problem_tab(
    ui: &Ui,
    theme: &Theme,
    content: &HelpWindowContent<'_>,
) -> Option<HelpAction> {
    let texts = &content.texts;
    let mut action = None;
    ChildWindow::new("problem").size([0.0, 0.0]).build(ui, || {
        draw_text_wrapped(ui, theme, TextStyle::Heading2, texts.problem_tab);
        draw_text_wrapped(ui, theme, TextStyle::Secondary, texts.problem_intro);
        draw_step(ui, theme, 1, texts.save_log_step, || {
            let export = Button::new(texts.export_log)
                .icon(Icon::DOWNLOAD)
                .state(content.problem.export_button);
            if export.draw(ui, theme) {
                action = Some(HelpAction::ExportLog);
            }
            draw_text_wrapped(ui, theme, TextStyle::Small, texts.export_hint);
            if let Some((status, text)) = content.problem.status {
                draw_status_line(ui, theme, status, text, TextStyle::Small);
            }
        });
        draw_step(ui, theme, 2, texts.report_step, || {
            let report = Button::new(texts.report_button)
                .kind(ButtonKind::Primary)
                .opens_link();
            if report.draw(ui, theme) {
                action = Some(HelpAction::ReportProblem);
            }
            draw_text_wrapped(ui, theme, TextStyle::Small, texts.report_hint);
        });
        draw_separator(ui, theme);
        draw_text_wrapped(ui, theme, TextStyle::Small, texts.footer);
    });
    action
}

/// Draws one numbered step: the number in a `bg.active` circle on the left,
/// the Heading3 `title` beside it and `body` indented below the title.
fn draw_step(ui: &Ui, theme: &Theme, number: u8, title: &str, body: impl FnOnce()) {
    ui.dummy([0.0, theme.spacing.small]);
    let [left, top] = ui.cursor_screen_pos();
    let diameter = ui.text_line_height() + theme.spacing.extra_small;
    let indent = diameter + theme.spacing.small;
    ui.indent_by(indent);
    ui.set_cursor_screen_pos([left + indent, top]);
    let title_height = {
        let _style = TextStyle::Heading3.push(ui, theme);
        ui.text_line_height()
    };
    draw_text(ui, theme, TextStyle::Heading3, title);
    draw_step_number(
        ui,
        theme,
        number,
        [left + diameter / 2.0, top + title_height / 2.0],
        diameter,
    );
    body();
    ui.unindent_by(indent);
}

/// Draws `number` in BodyStrong centered in a `bg.active` circle of
/// `diameter` around `center` (screen).
fn draw_step_number(ui: &Ui, theme: &Theme, number: u8, center: [f32; 2], diameter: f32) {
    let mut buffer = [0; 20];
    let label = format_count(usize::from(number), &mut buffer);
    let _style = TextStyle::BodyStrong.push(ui, theme);
    let label_size = ui.calc_text_size(label);
    let draw_list = ui.get_window_draw_list();
    draw_list
        .add_circle(center, diameter / 2.0, theme.surface.active.to_array())
        .filled(true)
        .num_segments(24)
        .build();
    draw_list.add_text(
        [
            center[0] - label_size[0] / 2.0,
            center[1] - label_size[1] / 2.0,
        ],
        theme.text.primary.to_array(),
        label,
    );
}
