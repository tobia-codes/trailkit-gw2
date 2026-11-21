//! The Report a problem tab (R95, R100 Q299–Q302, R101): one pane with the
//! shared page head (the title in Heading1, the sentence that nothing is
//! sent automatically in Secondary, the separator), an optional warning card
//! (R104), then step 1 "Save the log" with
//! the export button, its hint and status line, step 2 "Report on GitHub"
//! with the primary button that opens the browser, and the footer naming
//! the log folders. The steps are started one by one; the GitHub button is
//! never disabled.

use crate::drawing::draw_list::window_draw_list;
use nexus::imgui::Ui;

use super::{HelpAction, HelpWindowContent};
use crate::{
    controls::button::{Button, ButtonKind},
    feedback::status_card::{CardDetail, CardStatus, StatusCard, draw_status_card},
    feedback::status_line::draw_status_line,
    layout::page::{PageHead, draw_page},
    layout::pane::{WindowDrag, draw_pane},
    layout::stack::{Stack, StackLayout},
    markdown::LinkKind,
    text::icons::Icon,
    text::text_layout::format_count,
    text::{TextStyle, draw_text, draw_text_wrapped},
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
    };
    draw_pane(
        ui,
        theme,
        "problem",
        [0.0, 0.0],
        WindowDrag::MovesWindow,
        || {
            draw_page(
                ui,
                theme,
                "problem_body",
                head,
                WindowDrag::MovesWindow,
                || {
                    // Card, steps and footer are sections, `space.xl` apart like
                    // the headings in the help text (Q530).
                    Stack::vertical(theme)
                        .gap(theme.spacing.extra_large)
                        .build(ui, |page| {
                            if let Some(notice) = content.problem.notice {
                                page.add(|| {
                                    draw_status_card(
                                        ui,
                                        theme,
                                        &StatusCard {
                                            status: CardStatus::Warning,
                                            text: notice.text,
                                            detail: Some(CardDetail {
                                                text: notice.detail,
                                                style: TextStyle::Secondary,
                                            }),
                                            buttons: &[],
                                            close_tooltip: None,
                                        },
                                    )
                                });
                            }
                            page.add(|| {
                                draw_step(ui, theme, 1, texts.save_log_step, |step| {
                                    let export = Button::new(texts.export_log)
                                        .icon(Icon::DOWNLOAD)
                                        .state(content.problem.export_button);
                                    if step.add(|| export.draw(ui, theme)) {
                                        action = Some(HelpAction::ExportLog);
                                    }
                                    step.add(|| {
                                        draw_text_wrapped(
                                            ui,
                                            theme,
                                            TextStyle::Small,
                                            texts.export_hint,
                                        );
                                    });
                                    if let Some((status, text)) = content.problem.status {
                                        step.add(|| {
                                            draw_status_line(
                                                ui,
                                                theme,
                                                status,
                                                text,
                                                TextStyle::Small,
                                            );
                                        });
                                    }
                                });
                            });
                            page.add(|| {
                                draw_step(ui, theme, 2, texts.report_step, |step| {
                                    let report = Button::new(texts.report_button)
                                        .kind(ButtonKind::Primary)
                                        .opens_link(LinkKind::Web);
                                    if step.add(|| report.draw(ui, theme)) {
                                        action = Some(HelpAction::ReportProblem);
                                    }
                                    step.add(|| {
                                        draw_text_wrapped(
                                            ui,
                                            theme,
                                            TextStyle::Small,
                                            texts.report_hint,
                                        );
                                    });
                                });
                            });
                            page.add_separator(theme);
                            page.add(|| {
                                draw_text_wrapped(ui, theme, TextStyle::Small, texts.footer)
                            });
                        });
                },
            );
        },
    );
    action
}

/// Draws one numbered step: the number in a `bg.active` circle on the left,
/// and beside it a stack (`space.sm` apart, R148) of the Heading3 `title`
/// and what `body` adds below it.
fn draw_step(
    ui: &Ui,
    theme: &Theme,
    number: u8,
    title: &str,
    body: impl FnOnce(&mut StackLayout<'_>),
) {
    let [left, top] = ui.cursor_screen_pos();
    let diameter = ui.text_line_height() + theme.spacing.extra_small;
    let indent = diameter + theme.spacing.small;
    let title_height = {
        let _style = TextStyle::Heading3.push(ui, theme);
        ui.text_line_height()
    };
    draw_step_number(
        ui,
        theme,
        number,
        [left + diameter / 2.0, top + title_height / 2.0],
        diameter,
    );
    ui.set_cursor_screen_pos([left + indent, top]);
    Stack::vertical(theme).build(ui, |step| {
        step.add(|| draw_text(ui, theme, TextStyle::Heading3, title));
        body(step);
    });
}

/// Draws `number` in BodyStrong centered in a `bg.active` circle of
/// `diameter` around `center` (screen).
fn draw_step_number(ui: &Ui, theme: &Theme, number: u8, center: [f32; 2], diameter: f32) {
    let mut buffer = [0; 20];
    let label = format_count(usize::from(number), &mut buffer);
    let _style = TextStyle::BodyStrong.push(ui, theme);
    let label_size = ui.calc_text_size(label);
    let draw_list = window_draw_list(ui);
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
