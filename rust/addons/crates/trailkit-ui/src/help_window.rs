//! The help window of every addon (ROADMAP R95, R100): an own window with
//! two neutral tabs, "Help" (left, the start tab) with the topic list and the
//! selected topic, and "Report a problem" with the two steps to save the log
//! and report on GitHub.
//!
//! The addon supplies the texts in its language (trailkit-ui has no
//! strings), its help markdown and the state of the log export; the window
//! returns what the user asked for ([`HelpAction`]). The shown tab, the
//! selected topic and the width of the topic list form a
//! [`HelpWindowState`] that the addon saves and restores (R101 Q314, Q318).
//!
//! Layout (R101) as in the addons' main windows: the tabs on the window
//! background, below them the topic list and the topic as two panes with a
//! splitter, or the problem page as one pane; every page has the shared
//! page head.

mod help_tab;
mod problem_tab;

use crate::draw_list::window_draw_list;
use nexus::imgui::{Condition, Ui, Window};

use crate::{
    button::ButtonState,
    chat_link_chip::ChipContext,
    font_registry::FontScope,
    fonts::FontRole,
    help_topics::{HelpTopic, parse_help_topics},
    markdown::{MarkdownDocument, parse},
    status_line::Status,
    text_layout::body_characters_width,
    theme::Theme,
    title_bar::{TitleBarState, TitleBarTexts, draw_window},
};

/// The narrowest the window may get, in body characters (Q297).
pub const MINIMUM_WIDTH_CHARACTERS: f32 = 60.0;

/// The width the window starts with, in body characters (Q297).
const START_WIDTH_CHARACTERS: f32 = 62.0;

/// The height the window starts with, in frame heights.
const START_HEIGHT_FRAMES: f32 = 20.0;

/// The width the topic list starts with, in body characters: 112 px at the
/// 16 px body font of the minimum window (R95, R101 Q315).
pub const DEFAULT_TOPIC_LIST_CHARACTERS: f32 = 14.0;

/// The tabs of the help window, left to right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HelpTab {
    /// The help topics; the window opens on it the first time (Q295).
    #[default]
    Help,
    /// Saving the log and reporting a problem.
    ReportProblem,
}

/// What the help window keeps across restarts (R101 Q314, Q318); the addon
/// stores it, e.g. in its UI state file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HelpWindowState {
    /// The shown tab.
    pub tab: HelpTab,
    /// The position of the selected topic; one past the end selects the
    /// first topic.
    pub topic: usize,
    /// The content width of the topic list in body characters.
    pub topic_list_characters: f32,
}

impl Default for HelpWindowState {
    /// The Help tab with the first topic and the list at its start width.
    fn default() -> Self {
        Self {
            tab: HelpTab::Help,
            topic: 0,
            topic_list_characters: DEFAULT_TOPIC_LIST_CHARACTERS,
        }
    }
}

/// What the user asked for in this frame; the addon carries it out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpAction {
    /// "Export log" was clicked: write the log export on a worker thread
    /// and pass its outcome as [`ProblemPage::status`].
    ExportLog,
    /// "Report a problem" was clicked: open the GitHub issue form.
    ReportProblem,
}

/// The texts of the help window in the addon's language (R100, mockup).
#[derive(Debug, Clone, Copy)]
pub struct HelpWindowTexts<'text> {
    /// Title bar; its title is `<window title> · <Help>` (R95).
    pub title_bar: TitleBarTexts<'text>,
    /// The left tab, "Help".
    pub help_tab: &'text str,
    /// The right tab and the page title, "Report a problem".
    pub problem_tab: &'text str,
    /// The sentence below the page title: nothing is sent automatically.
    pub problem_intro: &'text str,
    /// Step 1 heading, "Save the log".
    pub save_log_step: &'text str,
    /// The button of step 1, "Export log".
    pub export_log: &'text str,
    /// What the export holds, below the button.
    pub export_hint: &'text str,
    /// Step 2 heading, "Report on GitHub".
    pub report_step: &'text str,
    /// The primary button of step 2, "Report a problem".
    pub report_button: &'text str,
    /// That reports are public and need a GitHub account.
    pub report_hint: &'text str,
    /// Where the logs and `Nexus.log` are (Q259).
    pub footer: &'text str,
}

/// The state of the problem page that the addon owns.
#[derive(Debug, Clone, Copy)]
pub struct ProblemPage<'text> {
    /// The "Export log" button: ready, busy while the export runs, or
    /// disabled with the reason while there is no log yet.
    pub export_button: ButtonState<'text>,
    /// The status line below the export hint, if any (Q302): it stays until
    /// the window closes or a new export starts.
    pub status: Option<(Status, &'text str)>,
    /// A warning card above the steps, e.g. that the addon stopped after an
    /// error (ROADMAP R104, Q366); `None` shows no card.
    pub notice: Option<ProblemNotice<'text>>,
}

/// The warning card at the top of the problem page (ROADMAP R104, Q367): no
/// buttons and no close button, because the steps below are the actions and
/// the card stays while its state lasts.
#[derive(Debug, Clone, Copy)]
pub struct ProblemNotice<'text> {
    /// The status sentence in `warning`.
    pub text: &'text str,
    /// The line under it in Secondary.
    pub detail: &'text str,
}

/// Everything one frame of the help window shows.
#[derive(Clone, Copy)]
pub struct HelpWindowContent<'text> {
    /// The ImGui id of the window, stable across languages, e.g.
    /// `trailkit.memo.help`.
    pub id: &'text str,
    /// The help markdown in the current language (`help/<language>.md`);
    /// it is parsed again only when another text is passed.
    pub help_text: &'static str,
    /// The texts.
    pub texts: HelpWindowTexts<'text>,
    /// The problem page.
    pub problem: ProblemPage<'text>,
    /// The texts of chat-link chips in the help text.
    pub chips: ChipContext<'text>,
}

/// One help topic ready to draw.
struct ParsedTopic {
    /// The `##` heading: the list entry and the topic's Heading1 title.
    title: String,
    /// The opening sentence, drawn Secondary below the title.
    lead: Option<String>,
    /// The rest of the topic.
    document: MarkdownDocument,
}

/// The help topics parsed from one help text.
struct ParsedHelp {
    /// The text they were parsed from, to notice a language change.
    source: &'static str,
    /// The topics in order.
    topics: Vec<ParsedTopic>,
}

impl ParsedHelp {
    /// Parses `source` into its topics, parsing each topic's markdown once
    /// here, not every frame.
    fn parse(source: &'static str) -> Self {
        let topics = parse_help_topics(source)
            .iter()
            .map(|topic: &HelpTopic| {
                let (lead, rest) = topic.split_lead();
                ParsedTopic {
                    title: topic.title.clone(),
                    lead,
                    document: parse(rest),
                }
            })
            .collect();
        Self { source, topics }
    }
}

/// The help window of one addon: whether it is open, the shown tab and
/// topic, and the parsed help text.
#[derive(Default)]
pub struct HelpWindow {
    /// Whether the window is shown.
    open: bool,
    /// Bring the window to the front in the next frame.
    focus: bool,
    /// The title bar's collapse state.
    title_bar: TitleBarState,
    /// Tab, topic and topic list width.
    state: HelpWindowState,
    /// Whether `state` changed in a way the addon should save.
    state_changed: bool,
    /// The parsed help text, once the window was drawn.
    help: Option<ParsedHelp>,
}

impl HelpWindow {
    /// A closed help window on the Help tab with the first topic.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Shows the saved `state`, e.g. once the addon has read its UI state.
    pub fn restore(&mut self, state: HelpWindowState) {
        self.state = state;
    }

    /// Tab, topic and topic list width.
    #[must_use]
    pub const fn state(&self) -> HelpWindowState {
        self.state
    }

    /// The state to save if the user changed the tab or the topic or let go
    /// of the splitter since the last call; `None` otherwise.
    pub fn take_state_to_save(&mut self) -> Option<HelpWindowState> {
        std::mem::take(&mut self.state_changed).then_some(self.state)
    }

    /// Whether the window is shown.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    /// The shown tab.
    #[must_use]
    pub const fn tab(&self) -> HelpTab {
        self.state.tab
    }

    /// Opens the window, or closes it if it is open: the title bar `?`
    /// (Q296).
    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.focus = self.open;
    }

    /// Opens the window or brings it to the front; never closes it: the
    /// Nexus options button "Open help" (Q296).
    pub fn open(&mut self) {
        self.open = true;
        self.focus = true;
    }

    /// Shows `tab`, e.g. for a snapshot.
    pub fn show_tab(&mut self, tab: HelpTab) {
        self.state.tab = tab;
    }

    /// Draws the window if it is open and returns what the user asked for.
    /// It starts 62 characters wide, centered in the game window, and is
    /// at least 60 characters wide (Q297).
    pub fn draw(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        content: &HelpWindowContent<'_>,
    ) -> Option<HelpAction> {
        let display = ui.io().display_size;
        let line = {
            let _body = FontScope::push(ui, FontRole::Body);
            ui.frame_height()
        };
        let window = Window::new(content.id)
            .size(
                [
                    body_characters_width(ui, START_WIDTH_CHARACTERS),
                    line * START_HEIGHT_FRAMES,
                ],
                Condition::FirstUseEver,
            )
            .position(
                [display[0] / 2.0, display[1] / 2.0],
                Condition::FirstUseEver,
            )
            .position_pivot([0.5, 0.5]);
        self.draw_in(ui, theme, window, content)
    }

    /// Like [`Self::draw`] in `window` (its position and size), e.g. at a
    /// fixed size for a snapshot; the minimum width is added here.
    pub fn draw_in(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        window: Window<'_, &str>,
        content: &HelpWindowContent<'_>,
    ) -> Option<HelpAction> {
        if !self.open {
            return None;
        }
        if self
            .help
            .as_ref()
            .is_none_or(|help| !std::ptr::eq(help.source, content.help_text))
        {
            self.help = Some(ParsedHelp::parse(content.help_text));
        }
        let mut window = window.size_constraints(
            [body_characters_width(ui, MINIMUM_WIDTH_CHARACTERS), 0.0],
            [f32::MAX, f32::MAX],
        );
        if std::mem::take(&mut self.focus) {
            window = window.focused(true);
        }
        let mut open = true;
        let mut action = None;
        let Self {
            title_bar,
            state,
            state_changed,
            help,
            ..
        } = self;
        let _ = draw_window(
            ui,
            theme,
            window,
            content.texts.title_bar,
            &[],
            title_bar,
            &mut open,
            || {
                let _body = FontScope::push(ui, FontRole::Body);
                if let Some(chosen) = draw_tabs(ui, theme, &content.texts, state.tab) {
                    state.tab = chosen;
                    *state_changed = true;
                }
                match state.tab {
                    HelpTab::Help => {
                        if let Some(help) = help.as_ref() {
                            *state_changed |=
                                help_tab::draw_help_tab(ui, theme, help, state, content.chips);
                        }
                    }
                    HelpTab::ReportProblem => {
                        action = problem_tab::draw_problem_tab(ui, theme, content);
                    }
                }
            },
        );
        self.open = open;
        action
    }
}

/// Draws the two neutral tabs (R95: `bg.active` behind the shown one, no
/// accent) and returns the clicked one.
fn draw_tabs(
    ui: &Ui,
    theme: &Theme,
    texts: &HelpWindowTexts<'_>,
    shown: HelpTab,
) -> Option<HelpTab> {
    let mut clicked = None;
    for (index, (tab, label)) in [
        (HelpTab::Help, texts.help_tab),
        (HelpTab::ReportProblem, texts.problem_tab),
    ]
    .into_iter()
    .enumerate()
    {
        if index > 0 {
            ui.same_line_with_spacing(0.0, theme.spacing.extra_small);
        }
        if draw_tab(ui, theme, label, tab == shown) {
            clicked = Some(tab);
        }
    }
    // The panes start `pane.gap` below the tabs, as panes keep it between
    // each other.
    let item_spacing = ui.clone_style().item_spacing[1];
    ui.dummy([0.0, (theme.pane.gap - 2.0 * item_spacing).max(0.0)]);
    clicked
}

/// Draws one tab: the label in `text.primary` on `bg.active` while shown,
/// otherwise in `text.secondary` with `bg.hover` while hovered. Returns
/// whether it was clicked.
fn draw_tab(ui: &Ui, theme: &Theme, label: &str, shown: bool) -> bool {
    let padding = ui.clone_style().frame_padding;
    let text_size = ui.calc_text_size(label);
    let size = [text_size[0] + 2.0 * padding[0] * 1.5, ui.frame_height()];
    let clicked = ui.invisible_button(label, size);
    let [min, max] = [ui.item_rect_min(), ui.item_rect_max()];
    let fill = if shown || ui.is_item_active() {
        Some(theme.surface.active)
    } else if ui.is_item_hovered() {
        Some(theme.surface.hover)
    } else {
        None
    };
    let draw_list = window_draw_list(ui);
    if let Some(fill) = fill {
        draw_list
            .add_rect(min, max, fill.to_array())
            .filled(true)
            .rounding(theme.radius.medium)
            .build();
    }
    let color = if shown {
        theme.text.primary
    } else {
        theme.text.secondary
    };
    draw_list.add_text(
        [
            min[0] + (size[0] - text_size[0]) / 2.0,
            min[1] + (size[1] - text_size[1]) / 2.0,
        ],
        color.to_array(),
        label,
    );
    clicked && !shown
}
