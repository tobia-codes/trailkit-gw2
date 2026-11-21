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
mod topic_search;

use crate::controls::hover::is_item_hovered;
use crate::drawing::draw_list::window_draw_list;
use nexus::imgui::{Condition, MouseButton, Ui, Window, WindowFocusedFlags, WindowHoveredFlags};

use crate::{
    chat_link::chat_link_chip::ChipContext,
    controls::button::ButtonState,
    feedback::status_line::Status,
    help::help_search::IndexedTopic,
    help::help_topics::{HelpTopic, parse_help_topics},
    layout::stack::Stack,
    layout::title_bar::{TitleBarState, TitleBarTexts, draw_window},
    markdown::{MarkdownDocument, parse},
    text::font_registry::FontScope,
    text::fonts::FontRole,
    text::text_layout::body_characters_width,
    theme::Theme,
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
    /// The search of the Help tab and find in a topic (BL30).
    pub search: HelpSearchTexts<'text>,
}

/// The texts of the help's search and of find in a topic (BL30, mock v9).
#[derive(Debug, Clone, Copy)]
pub struct HelpSearchTexts<'text> {
    /// Placeholder of the search above the topic list, "Search help …".
    pub search_hint: &'text str,
    /// Placeholder of find in a topic, "Find in topic …".
    pub find_hint: &'text str,
    /// Tooltip of the `x` of a field with text, "Clear search".
    pub clear: &'text str,
    /// Tooltip of the `x` of the empty find field, "Close (Escape)".
    pub close: &'text str,
    /// The position among the hits with `{current}` and `{total}`, e.g.
    /// "{current} of {total}".
    pub count: &'text str,
    /// Shown when nothing matches, "No results".
    pub no_results: &'text str,
    /// Tooltip of the previous button.
    pub previous: &'text str,
    /// Tooltip of the next button.
    pub next: &'text str,
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
    /// The help markdown in the current language (built from `help/<language>/`);
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
    /// The `###` headings, listed below the title while it is selected.
    sections: Vec<String>,
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
    /// The searchable text of each topic, in the same order (BL30).
    index: Vec<IndexedTopic>,
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
                    sections: topic.section_titles(),
                    lead,
                    document: parse(rest),
                }
            })
            .collect::<Vec<_>>();
        let index = topics
            .iter()
            .map(|topic| IndexedTopic::new(&topic.title, topic.lead.as_deref(), &topic.document))
            .collect();
        Self {
            source,
            topics,
            index,
        }
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
    /// The marked section, the pending scroll and the text selection of the
    /// Help tab.
    topic_view: help_tab::TopicView,
    /// Whether the window had the keyboard focus and no text field took the
    /// keys in the last frame: Ctrl+C and Ctrl+A belong to it then.
    takes_keys: bool,
    /// Whether the window had the keyboard focus in the last frame: Ctrl+F,
    /// F3 and Shift twice belong to it then (BL30, Q582).
    focused: bool,
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

    /// Whether Ctrl+A belongs to the window: it is open with the focus on
    /// the Help tab and no text field takes the keys (memo M217). An addon's
    /// window message callback catches it only then.
    #[must_use]
    pub fn takes_select_all_key(&self) -> bool {
        self.open && self.takes_keys && self.state.tab == HelpTab::Help
    }

    /// Whether Ctrl+C belongs to the window: as
    /// [`Self::takes_select_all_key`] and text of the topic is selected.
    #[must_use]
    pub fn takes_copy_key(&self) -> bool {
        self.takes_select_all_key() && !self.topic_view.selection.is_empty()
    }

    /// Whether the window had the keyboard focus in the last frame, e.g. so
    /// the addon sends Shift twice to its own note search (Q582).
    #[must_use]
    pub const fn is_focused(&self) -> bool {
        self.open && self.focused
    }

    /// Whether Ctrl+F belongs to the window: it is focused on the Help tab
    /// (BL30). An addon's window message callback catches it only then.
    #[must_use]
    pub fn takes_find_key(&self) -> bool {
        self.is_focused() && self.state.tab == HelpTab::Help
    }

    /// Whether F3 belongs to the window: as [`Self::takes_find_key`] while
    /// find in the topic is open.
    #[must_use]
    pub fn takes_find_step_key(&self) -> bool {
        self.takes_find_key() && self.topic_view.search.find_open()
    }

    /// Ctrl+F: opens find in the shown topic, or closes it while open.
    pub fn toggle_find_by_key(&mut self) {
        if self.takes_find_key() {
            self.topic_view.search.toggle_find();
        }
    }

    /// F3 / Shift+F3: the next or previous hit of find in the topic.
    pub fn step_find_by_key(&mut self, forward: bool) {
        if self.takes_find_step_key() {
            self.topic_view.search.step_find(forward);
        }
    }

    /// Puts the selected text of the topic on the clipboard after Ctrl+C.
    pub fn copy_selection_by_key(&self, ui: &Ui) {
        if self.takes_copy_key()
            && let Some(document) = self.shown_document()
            && let Some(text) = self.topic_view.selection.text(document)
        {
            ui.set_clipboard_text(text);
        }
    }

    /// Selects the whole topic after Ctrl+A.
    pub fn select_all_by_key(&mut self) {
        let shown = self
            .help
            .as_ref()
            .and_then(|help| help.topics.get(self.state.topic));
        if self.takes_select_all_key()
            && let Some(topic) = shown
        {
            self.topic_view.selection.select_all(&topic.document);
        }
    }

    /// The document of the shown topic, once the help is parsed.
    fn shown_document(&self) -> Option<&MarkdownDocument> {
        let help = self.help.as_ref()?;
        help.topics
            .get(self.state.topic)
            .map(|topic| &topic.document)
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

    /// Types `text` into the search above the topic list, e.g. for a
    /// snapshot.
    pub fn search_for(&mut self, text: &str) {
        self.topic_view.search.set_search_text(text);
    }

    /// Opens find in the shown topic with `text`, e.g. for a snapshot.
    pub fn find_in_topic(&mut self, text: &str) {
        self.topic_view.search.open_find_with(text);
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
        // Ctrl+C and Ctrl+A as ImGui sees them; in game the addon's window
        // message callback gets them first while they belong to the window.
        if ui.io().key_ctrl {
            if ui.is_key_index_pressed(VIRTUAL_KEY_C) {
                self.copy_selection_by_key(ui);
            } else if ui.is_key_index_pressed(VIRTUAL_KEY_A) {
                self.select_all_by_key();
            } else if ui.is_key_index_pressed(VIRTUAL_KEY_F) {
                self.toggle_find_by_key();
            }
        }
        let Self {
            title_bar,
            state,
            state_changed,
            help,
            topic_view,
            takes_keys,
            focused,
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
                *focused =
                    ui.is_window_focused_with_flags(WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS);
                *takes_keys = *focused && !ui.io().want_text_input;
                // Any left click in the window ends the selection; Shift+click
                // extends it (memo M217, user wish 2026-11-21).
                if ui.is_mouse_clicked(MouseButton::Left)
                    && !ui.io().key_shift
                    && ui.is_window_hovered_with_flags(WindowHoveredFlags::ROOT_AND_CHILD_WINDOWS)
                {
                    topic_view.selection.clear();
                }
                // The panes start `space.sm` below the tabs, as panes keep
                // it between each other (R148).
                Stack::vertical(theme).build(ui, |stack| {
                    if let Some(chosen) =
                        stack.add(|| draw_tabs(ui, theme, &content.texts, state.tab))
                    {
                        state.tab = chosen;
                        *state_changed = true;
                    }
                    stack.add(|| match state.tab {
                        HelpTab::Help => {
                            if let Some(help) = help.as_ref() {
                                *state_changed |= help_tab::draw_help_tab(
                                    ui,
                                    theme,
                                    help,
                                    state,
                                    topic_view,
                                    (content.chips, content.texts.search),
                                );
                            }
                        }
                        HelpTab::ReportProblem => {
                            action = problem_tab::draw_problem_tab(ui, theme, content);
                        }
                    });
                });
            },
        );
        self.open = open;
        action
    }
}

/// Virtual-key code of A (Ctrl+A selects the whole topic).
const VIRTUAL_KEY_A: i32 = 0x41;
/// Virtual-key code of C (Ctrl+C copies the selection).
const VIRTUAL_KEY_C: i32 = 0x43;
/// Virtual-key code of F (Ctrl+F finds in the topic, BL30).
const VIRTUAL_KEY_F: i32 = 0x46;

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
    } else if is_item_hovered(ui) {
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
