//! Draws the right-click menu of links and chat-link chips (memo plan M210)
//! and of selected text (M217, P4): "Copy" for the selection comes first,
//! then, after a divider, the entries of the link or chip under the mouse.

use std::cell::{Cell, RefCell};

use nexus::imgui::Ui;

use super::MarkdownAction;
use crate::{
    chat_link::{ChipText, chat_link_chip::ChipContext},
    controls::menu::{MenuItem, draw_menu_item, draw_popup},
    layout::separator::draw_separator,
    markdown::link_menu::{LinkMenuCommand, LinkMenuTarget, wiki_search_url},
    text::icons::Icon,
    theme::Theme,
};

/// ImGui id of the menu.
const MENU_ID: &str = "##markdown_link_menu";

thread_local! {
    /// What the open menu belongs to. Each addon DLL has its own copy; the
    /// view draws on the render thread and keeps no state of its own.
    static TARGET: RefCell<Option<LinkMenuTarget>> = const { RefCell::new(None) };
    /// The selected text the open menu copies, if it has "Copy".
    static SELECTION: RefCell<Option<String>> = const { RefCell::new(None) };
    /// Whether the menu opens when the current view ends.
    static OPEN_REQUESTED: Cell<bool> = const { Cell::new(false) };
}

/// Opens the menu for `target` at the end of the current markdown view.
pub(super) fn request_link_menu(target: LinkMenuTarget) {
    TARGET.set(Some(target));
    SELECTION.set(None);
    OPEN_REQUESTED.set(true);
}

/// Adds "Copy" for the selected `text` to the menu the current view opens,
/// or opens a menu with only that entry when no link or chip asked for one.
pub(super) fn request_selection_menu(text: String) {
    if !OPEN_REQUESTED.get() {
        TARGET.set(None);
    }
    SELECTION.set(Some(text));
    OPEN_REQUESTED.set(true);
}

/// Draws the menu if it is open and returns what its clicked entry asks for.
/// Copy commands have already filled the clipboard.
pub(super) fn draw_link_menu(
    ui: &Ui,
    theme: &Theme,
    chips: ChipContext<'_>,
) -> Option<MarkdownAction> {
    if OPEN_REQUESTED.take() {
        ui.open_popup(MENU_ID);
    }
    let target = TARGET.with_borrow(Clone::clone);
    let selection = SELECTION.with_borrow(Clone::clone);
    if target.is_none() && selection.is_none() {
        return None;
    }
    let mut action = None;
    draw_popup(ui, theme, MENU_ID, || {
        if let Some(text) = &selection {
            let item = MenuItem::new((chips.text)(ChipText::Copy)).icon(Icon::COPY);
            // Copying is confirmed like a chip click (user wish 2026-11-21,
            // replaces Q556).
            if draw_menu_item(ui, theme, &item) {
                ui.set_clipboard_text(text);
                action = Some(MarkdownAction::Copied(text.clone()));
            }
            if target.is_some() {
                draw_separator(ui, theme);
            }
        }
        let Some(target) = &target else {
            return;
        };
        for entry in target.entries() {
            let item = MenuItem::new((chips.text)(entry.label)).icon(entry.icon);
            let is_clicked = draw_menu_item(ui, theme, &item);
            if is_clicked {
                action = Some(perform(ui, target, entry.command, chips));
            }
        }
    });
    action
}

/// Carries out `command` for `target`.
fn perform(
    ui: &Ui,
    target: &LinkMenuTarget,
    command: LinkMenuCommand,
    chips: ChipContext<'_>,
) -> MarkdownAction {
    match (command, target) {
        (LinkMenuCommand::Open, LinkMenuTarget::Link(url)) => MarkdownAction::OpenLink(url.clone()),
        (LinkMenuCommand::OpenInWiki, LinkMenuTarget::Chip { code, .. }) => {
            let base = (chips.text)(ChipText::WikiSearchUrl);
            MarkdownAction::OpenLink(wiki_search_url(base, code))
        }
        _ => {
            let text = target.copied_text();
            ui.set_clipboard_text(text);
            MarkdownAction::Copied(text.to_owned())
        }
    }
}
