//! The editor of one note (M51–M53, M58): title input, word-wrapping body,
//! formatting toolbar (M53, M94, M100, M102) and the preview beside the
//! body (M60, M72).
//!
//! The app draws the toolbar's end (Edit / Cancel / Save) and the
//! unsaved-changes dialog; it passes the preview layout in and saves what
//! the editor reports.

use std::ops::Range;

use nexus::imgui::{Key, Ui};
use trailkit_memo_shared::i18n::{Language, LocalizedText};
use trailkit_ui::{
    chat_link::chat_link_chip::ChipContext,
    controls::button::{
        ButtonState, draw_icon_button_with_state, draw_icon_toggle, icon_button_width,
    },
    editor::view::TextEditor,
    layout::pane::draw_pane,
    layout::splitter::draw_vertical_splitter,
    layout::toolbar::begin_toolbar_group,
    markdown::markdown_view::{MarkdownAction, SearchHighlight, draw_markdown},
    markdown::{self, MarkdownDocument},
    text::TextStyle,
    text::icons::Icon,
    text::search_hits::HitMark,
    text::text_search::SearchQuery,
    theme::Theme,
};

use crate::{
    i18n::EditorText,
    link_dialog::{LinkDialog, LinkDialogOutcome},
    link_edit::{link_draft, link_edit},
    note_editing::{ActiveFormats, EditSession, FormatCommand, active_formats, format_edit},
};

/// Width in characters (of `n`) that editor and preview each keep, so the
/// splitter between them stays usable (M72).
const MINIMUM_PREVIEW_SPLIT_CHARACTERS: f32 = 3.0;

/// Windows virtual-key code of `B` (Nexus feeds ImGui with virtual-key codes).
const VIRTUAL_KEY_B: i32 = 0x42;

/// Windows virtual-key code of `I`.
const VIRTUAL_KEY_I: i32 = 0x49;

/// Windows virtual-key code of `K` (Ctrl+K opens the link dialog, M208).
const VIRTUAL_KEY_K: i32 = 0x4B;

/// Windows virtual-key code of `S` (Ctrl+S saves, M93).
const VIRTUAL_KEY_S: i32 = 0x53;

/// One formatting button: icon, tooltip and command (`None` = disabled).
type FormatButton = (Icon, EditorText, Option<FormatCommand>);

/// The formatting buttons in groups, like Word's ribbon (M53, M100, M102):
/// headings, emphasis, lists, inserts. The event insert is disabled until
/// trailkit-core exists.
const FORMAT_GROUPS: [&[FormatButton]; 4] = [
    &[
        (
            Icon::HEADING_1,
            EditorText::FormatHeading1,
            Some(FormatCommand::Heading1),
        ),
        (
            Icon::HEADING_2,
            EditorText::FormatHeading2,
            Some(FormatCommand::Heading2),
        ),
    ],
    &[
        (
            Icon::BOLD,
            EditorText::FormatBold,
            Some(FormatCommand::Bold),
        ),
        (
            Icon::ITALIC,
            EditorText::FormatItalic,
            Some(FormatCommand::Italic),
        ),
    ],
    &[
        (
            Icon::LIST,
            EditorText::FormatBulletList,
            Some(FormatCommand::BulletList),
        ),
        (
            Icon::LIST_TODO,
            EditorText::FormatCheckbox,
            Some(FormatCommand::Checkbox),
        ),
    ],
    &[
        (
            Icon::LINK,
            EditorText::FormatLink,
            Some(FormatCommand::Link),
        ),
        (
            Icon::CALENDAR_CLOCK,
            EditorText::InsertEventRequiresCore,
            None,
        ),
    ],
];

/// A click in the formatting toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    /// Apply a formatting command to the body (M53).
    Format(FormatCommand),
    /// Show or hide the preview (M60).
    TogglePreview,
}

/// What the editor needs from the app this frame.
pub struct EditorInput<'a> {
    /// Whether the preview is shown beside the body (M60).
    pub preview_visible: bool,
    /// The body's share of the room for body and preview, already clamped
    /// to a usable range.
    pub preview_split_fraction: f32,
    /// Whether a dialog covers the editor: typing must not change what the
    /// dialog asks about.
    pub is_dialog_open: bool,
    /// The active search, whose hits the preview marks (M122).
    pub search: Option<&'a SearchQuery>,
    /// Texts and copy state of chat-link chips.
    pub chips: ChipContext<'a>,
}

/// What the user did in the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorAction {
    /// Save the edit ("Save", Ctrl+S; M52, M93, M194).
    Save,
    /// A click on a link or chip in the preview.
    Markdown(MarkdownAction),
}

/// The result of drawing the editor for one frame.
#[derive(Debug, Default)]
pub struct EditorOutput {
    /// The user's actions, in order.
    pub actions: Vec<EditorAction>,
    /// The new split while the splitter is dragged.
    pub preview_split_fraction: Option<f32>,
    /// Whether the splitter was released, so the split should be saved.
    pub split_released: bool,
}

/// The editor of one note (M51–M53, M58).
#[derive(Debug)]
pub struct Editor {
    /// Uncommitted title and body.
    session: EditSession,
    /// The word-wrapping body editor; its text is mirrored into
    /// `session.body` on every change.
    body: TextEditor,
    /// The parsed preview of the body (M60); `None` when it must be parsed
    /// again because the text changed.
    preview: Option<MarkdownDocument>,
    /// Whether the title input was active in the last frame.
    title_was_active: bool,
    /// Whether the title input should take the focus with its text selected
    /// (M161); stays set until the input is active.
    title_focus_pending: bool,
    /// Formatting active at the cursor, cached per text revision and
    /// selection (M69).
    active: Option<(u64, Range<usize>, ActiveFormats)>,
    /// Find hits in the body while the find bar is open (M198).
    find: Option<BodyFind>,
    /// The open link dialog (M208).
    link_dialog: Option<LinkDialog>,
    /// Whether the link dialog opens in the next drawn frame.
    link_dialog_requested: bool,
}

/// Find hits in the body, cached per text revision and phrase (M198).
#[derive(Debug)]
struct BodyFind {
    /// Text revision the hits belong to.
    revision: u64,
    /// The lower-cased phrase they were found for.
    phrase: Vec<String>,
    /// The current hit the marks were set for.
    current: usize,
    /// Byte ranges of the hits in the body, in text order.
    ranges: Vec<Range<usize>>,
}

impl Editor {
    /// Opens an editor for `session`, focused on the body.
    #[must_use]
    pub fn new(session: EditSession) -> Self {
        let body = TextEditor::new(session.body.clone());
        Self {
            session,
            body,
            preview: None,
            title_was_active: false,
            title_focus_pending: false,
            active: None,
            find: None,
            link_dialog: None,
            link_dialog_requested: false,
        }
    }

    /// The edit in progress.
    #[must_use]
    pub const fn session(&self) -> &EditSession {
        &self.session
    }

    /// Ends editing and hands out the edited title and body.
    #[must_use]
    pub fn into_session(self) -> EditSession {
        self.session
    }

    /// The edited title, for changes from outside the title input.
    pub const fn title_mut(&mut self) -> &mut String {
        &mut self.session.title
    }

    /// The body text as edited so far.
    #[must_use]
    pub fn body_text(&self) -> &str {
        self.body.text()
    }

    /// Selects `range` (byte offsets) in the body and scrolls to it.
    pub fn select_in_body(&mut self, range: Range<usize>) {
        self.body.select(range);
    }

    /// The text selected in the body, e.g. to fill the find field (M198).
    #[must_use]
    pub fn selected_body_text(&self) -> &str {
        self.body
            .text()
            .get(self.body.selection())
            .unwrap_or_default()
    }

    /// Finds `query` in the body and marks its hits, `current` filled and
    /// the others framed; `None` clears the marks (M198). Cheap while text,
    /// phrase and current hit stay the same. Returns the number of hits.
    pub fn update_find(&mut self, query: Option<&SearchQuery>, current: usize) -> usize {
        let Some(query) = query else {
            if self.find.take().is_some() {
                self.body.set_find_hits([]);
            }
            return 0;
        };
        let revision = self.body.revision();
        let is_stale = self
            .find
            .as_ref()
            .is_none_or(|find| find.revision != revision || find.phrase != query.terms());
        if is_stale {
            self.find = Some(BodyFind {
                revision,
                phrase: query.terms().to_vec(),
                current: usize::MAX,
                ranges: query.hit_ranges(self.body.text()),
            });
        }
        let Some(find) = self.find.as_mut() else {
            return 0;
        };
        if find.current != current {
            find.current = current;
            let marks = find.ranges.iter().enumerate().map(|(index, range)| {
                let mark = if index == current {
                    HitMark::Filled
                } else {
                    HitMark::Framed
                };
                (range.clone(), mark)
            });
            self.body.set_find_hits(marks);
        }
        find.ranges.len()
    }

    /// Selects the find hit `index` (see [`Self::update_find`]) with the
    /// cursor after it and scrolls to it (M198).
    pub fn select_find_hit(&mut self, index: usize) {
        if let Some(range) = self
            .find
            .as_ref()
            .and_then(|find| find.ranges.get(index))
            .cloned()
        {
            self.body.select(range);
        }
    }

    /// Moves the keyboard focus into the title input with the whole title
    /// selected, so typing replaces it ("Rename", M161).
    pub fn focus_title(&mut self) {
        self.body.unfocus();
        self.title_focus_pending = true;
    }

    /// Gives the body the keyboard focus back (e.g. after the find bar
    /// closed, M198).
    pub fn focus_body(&mut self) {
        self.body.focus();
    }

    /// Takes the keyboard focus from the body (e.g. for the search field).
    pub fn unfocus_body(&mut self) {
        self.body.unfocus();
    }

    /// Applies a formatting command to the body at its selection (M53); the
    /// link command opens the link dialog instead (M208).
    pub fn format(&mut self, command: FormatCommand) {
        if command == FormatCommand::Link {
            self.link_dialog_requested = true;
            return;
        }
        let edit = format_edit(self.body.text(), self.body.selection(), command);
        self.body
            .apply_edit(edit.range, &edit.replacement, edit.selection);
        self.body.focus();
        self.sync_body();
    }

    /// The formatting active at the cursor (recomputed only when the text or
    /// the selection changed).
    fn active_formats(&mut self) -> ActiveFormats {
        let revision = self.body.revision();
        let selection = self.body.selection();
        match &self.active {
            Some((cached_revision, cached_selection, formats))
                if *cached_revision == revision && *cached_selection == selection =>
            {
                *formats
            }
            _ => {
                let formats = active_formats(self.body.text(), selection.clone());
                self.active = Some((revision, selection, formats));
                formats
            }
        }
    }

    /// Copies the editor text into the session and marks the preview stale.
    fn sync_body(&mut self) {
        self.session.body.clear();
        self.session.body.push_str(self.body.text());
        self.preview = None;
    }

    /// Draws the formatting buttons in groups with separators (M53, M102),
    /// then the preview toggle; the app continues the same toolbar row.
    /// Buttons of the formatting at the cursor are drawn in the accent color
    /// (M69); the disabled event button in `text.disabled`.
    pub fn draw_toolbar(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        preview_visible: bool,
    ) -> Option<ToolbarAction> {
        let active = self.active_formats();
        let mut clicked = None;
        let spacing = ui.clone_style().item_spacing[0];
        let mut id = 0;
        for group in FORMAT_GROUPS {
            let width = group
                .iter()
                .map(|(icon, _, _)| icon_button_width(ui, *icon))
                .sum::<f32>()
                + spacing * (group.len().saturating_sub(1)) as f32;
            begin_toolbar_group(ui, theme, width);
            for (index, (icon, tooltip, command)) in group.iter().enumerate() {
                if index > 0 {
                    ui.same_line();
                }
                id += 1;
                let _id = ui.push_id(id);
                // Formatting at the cursor is a kept state, so neutral, not
                // accent (R79).
                let Some(command) = command else {
                    // Disabled: grey icon, no hover, the reason as tooltip (R93).
                    let reason = tooltip.get(language);
                    draw_icon_button_with_state(
                        ui,
                        theme,
                        *icon,
                        reason,
                        ButtonState::Disabled { reason },
                    );
                    continue;
                };
                let is_on = active.is_active(*command);
                if draw_icon_toggle(ui, theme, *icon, tooltip.get(language), is_on) {
                    clicked = Some(ToolbarAction::Format(*command));
                }
            }
        }

        // The preview toggle is its own group, neutral like every toggle (R79).
        begin_toolbar_group(ui, theme, icon_button_width(ui, Icon::EYE));
        let tooltip = if preview_visible {
            EditorText::HidePreview
        } else {
            EditorText::ShowPreview
        };
        if draw_icon_toggle(ui, theme, Icon::EYE, tooltip.get(language), preview_visible) {
            clicked = Some(ToolbarAction::TogglePreview);
        }
        clicked
    }

    /// Draws the title input as a level 1 heading. It takes the focus for
    /// "Rename" with the title selected (M161) and keeps the typed title
    /// when Escape leaves it. Read-only while a dialog is open.
    fn draw_title_input(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        is_dialog_open: bool,
    ) {
        // ImGui reverts an input to its content at activation when Escape is
        // pressed; keep the typed text instead (only copied on that frame).
        let escape_pressed = ui.is_key_pressed(Key::Escape);
        let title_backup =
            (escape_pressed && self.title_was_active).then(|| self.session.title.clone());
        {
            let _heading = TextStyle::Heading1.push(ui, theme);
            ui.set_next_item_width(-f32::MIN_POSITIVE);
            if self.title_focus_pending {
                ui.set_keyboard_focus_here();
            }
            ui.input_text("##title", &mut self.session.title)
                .hint(EditorText::TitleHint.get(language))
                .read_only(is_dialog_open)
                // Only for "Rename"; a click into the title places the cursor.
                .auto_select_all(self.title_focus_pending)
                .build();
        }
        self.title_was_active = ui.is_item_active();
        if self.title_was_active {
            self.title_focus_pending = false;
        }
        if ui.is_item_deactivated()
            && let Some(title) = title_backup
        {
            self.session.title = title;
        }
    }

    /// Draws the title input and the markdown body filling the rest of the
    /// pane, with the preview beside it when visible (M51–M53, M60, M72).
    pub fn draw(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        input: EditorInput<'_>,
    ) -> EditorOutput {
        let mut output = EditorOutput::default();
        self.draw_title_input(ui, language, theme, input.is_dialog_open);

        if input.is_dialog_open {
            self.body.unfocus();
        }
        // Ctrl+S works like "Save" from title and text (M93).
        if !input.is_dialog_open
            && (self.body.is_focused() || self.title_was_active)
            && ui.io().key_ctrl
            && ui.is_key_index_pressed(VIRTUAL_KEY_S)
        {
            output.actions.push(EditorAction::Save);
        }
        if self.body.is_focused() && ui.io().key_ctrl {
            if ui.is_key_index_pressed(VIRTUAL_KEY_B) {
                self.format(FormatCommand::Bold);
            } else if ui.is_key_index_pressed(VIRTUAL_KEY_I) {
                self.format(FormatCommand::Italic);
            } else if ui.is_key_index_pressed(VIRTUAL_KEY_K) {
                self.format(FormatCommand::Link);
            }
        }
        self.update_link_dialog(ui, input.is_dialog_open);
        // Preview (M60, M72): editor and preview always stay side by side,
        // however narrow; the minimum only keeps the splitter usable.
        let available = ui.content_region_avail()[0];
        let usable = (available - theme.pane.gap).max(0.0);
        let preview_minimum = (ui.calc_text_size("n")[0] * MINIMUM_PREVIEW_SPLIT_CHARACTERS
            + 2.0 * theme.pane.padding)
            .min(usable / 2.0);
        let editor_width = (usable * input.preview_split_fraction)
            .clamp(preview_minimum, usable - preview_minimum);
        let width = if input.preview_visible {
            editor_width
        } else {
            0.0
        };
        let response = self
            .body
            .draw(ui, theme, "##body", [width, 0.0], input.chips);
        if response.changed {
            self.sync_body();
        }
        if input.preview_visible {
            ui.same_line_with_spacing(0.0, 0.0);
            let splitter = draw_vertical_splitter(
                ui,
                theme,
                "preview_splitter",
                editor_width,
                (preview_minimum, usable - preview_minimum),
                ui.content_region_avail()[1],
            );
            output.preview_split_fraction = splitter.left_width.map(|width| width / usable);
            output.split_released = splitter.released;
            ui.same_line_with_spacing(0.0, 0.0);
            let preview = self
                .preview
                .get_or_insert_with(|| markdown::parse(&self.session.body));
            let actions = &mut output.actions;
            draw_pane(ui, theme, "preview", [0.0, 0.0], || {
                // The preview is read-only: links open and chips copy, but
                // checkboxes do not toggle. It marks search hits (M122).
                let mut highlight = input.search.map(SearchHighlight::new);
                match draw_markdown(ui, preview, theme, input.chips, highlight.as_mut()) {
                    Some(
                        action @ (MarkdownAction::OpenLink(_) | MarkdownAction::ChatLinkCopied(_)),
                    ) => {
                        actions.push(EditorAction::Markdown(action));
                    }
                    Some(MarkdownAction::ToggleTask(_)) | None => {}
                }
            });
        }
        output
    }

    /// Opens the link dialog if asked to and keeps the keyboard away from the
    /// body while it is open; a dialog of the app closes it (M208).
    fn update_link_dialog(&mut self, ui: &Ui, is_dialog_open: bool) {
        if std::mem::take(&mut self.link_dialog_requested) && !is_dialog_open {
            let clipboard = ui.clipboard_text();
            let draft = link_draft(
                self.body.text(),
                self.body.selection(),
                clipboard.as_deref(),
            );
            self.link_dialog = Some(LinkDialog::new(draft));
        }
        if is_dialog_open {
            self.link_dialog = None;
        } else if self.link_dialog.is_some() {
            self.body.unfocus();
        }
    }

    /// Whether the link dialog is open.
    #[must_use]
    pub const fn is_link_dialog_open(&self) -> bool {
        self.link_dialog.is_some()
    }

    /// Closes the link dialog without a change and returns to the body.
    pub fn cancel_link_dialog(&mut self) {
        if self.link_dialog.take().is_some() {
            self.body.focus();
        }
    }

    /// Draws the open link dialog into the current, dimmed window and applies
    /// its result.
    pub fn draw_link_dialog(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let Some(dialog) = self.link_dialog.as_mut() else {
            return;
        };
        let outcome = dialog.draw(ui, language, theme);
        let edit = match &outcome {
            LinkDialogOutcome::Open => return,
            LinkDialogOutcome::Cancelled => None,
            LinkDialogOutcome::Submitted { text, address } => {
                Some(link_edit(dialog.draft(), text, address))
            }
            LinkDialogOutcome::Removed => Some(format_edit(
                self.body.text(),
                dialog.draft().range.clone(),
                FormatCommand::Link,
            )),
        };
        self.link_dialog = None;
        if let Some(edit) = edit {
            self.body
                .apply_edit(edit.range, &edit.replacement, edit.selection);
            self.sync_body();
        }
        self.body.focus();
    }
}
