//! The Report a problem tab (R95, R100 Q299–Q302, R101): one pane with the
//! shared page head (the title in Heading1, the sentence that nothing is
//! sent automatically in Secondary, the separator), then step 1 "Save the log" with
//! the export button, its hint and status line, step 2 "Report on GitHub"
//! with the primary button that opens the browser, and the footer naming
//! the log folders. The steps are started one by one; the GitHub button is
//! never disabled.

use nexus::imgui::Ui;

use super::{HelpAction, HelpWindowContent};
use crate::{
    button::{Button, ButtonKind},
    fonts::FontRole,
    icons::Icon,
    page::{PageHead, draw_page},
    pane::draw_pane,
    separator::draw_separator,
    status_line::draw_status_line,
    text::{TextStyle, draw_text, draw_text_wrapped},
    text_layout::format_count,
    theme::Theme,
};

/// Draws the page in one pane over the full width, its content scrolling
/// below the page head, and returns the clicked action.
pub(super) fn draw_problem_tab(
    ui: &Ui,
    theme: &Theme,
    content: &HelpWindowContent<'_>,
) -> Option<HelpAction> {
    let texts = &content.texts;
    let mut action = None;
    let head = PageHead {
        title: texts.problem_tab,
        trailing: None,
        subtitle: Some(texts.problem_intro),
        first_line: FontRole::Heading3,
    };
    draw_pane(ui, theme, "problem", [0.0, 0.0], || {
        draw_page(ui, theme, "problem_body", head, || {
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
    });
    action
}

/// Draws one numbered step: the number in a `bg.active` circle on the left,
/// the Heading3 `title` beside it and `body` indented below the title. The
/// first step follows the separator directly, later ones keep
/// `spacing.small` from the step above.
fn draw_step(ui: &Ui, theme: &Theme, number: u8, title: &str, body: impl FnOnce()) {
    if number > 1 {
        ui.dummy([0.0, theme.spacing.small]);
    }
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
