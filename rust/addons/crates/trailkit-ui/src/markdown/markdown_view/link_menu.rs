//! Draws the right-click menu of links and chat-link chips (memo plan M210).

use std::cell::{Cell, RefCell};

use nexus::imgui::Ui;

use super::MarkdownAction;
use crate::{
    chat_link::{ChipText, chat_link_chip::ChipContext},
    controls::menu::{MenuItemTone, draw_menu_item_with_icon, draw_popup},
    markdown::link_menu::{LinkMenuCommand, LinkMenuTarget, wiki_search_url},
    theme::Theme,
};

/// ImGui id of the menu.
const MENU_ID: &str = "##markdown_link_menu";

thread_local! {
    /// What the open menu belongs to. Each addon DLL has its own copy; the
    /// view draws on the render thread and keeps no state of its own.
    static TARGET: RefCell<Option<LinkMenuTarget>> = const { RefCell::new(None) };
    /// Whether the menu opens when the current view ends.
    static OPEN_REQUESTED: Cell<bool> = const { Cell::new(false) };
}

/// Opens the menu for `target` at the end of the current markdown view.
pub(super) fn request_link_menu(target: LinkMenuTarget) {
    TARGET.set(Some(target));
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
    let target = TARGET.with_borrow(Clone::clone)?;
    let mut action = None;
    draw_popup(ui, theme, MENU_ID, || {
        for entry in target.entries() {
            let is_clicked = draw_menu_item_with_icon(
                ui,
                theme,
                entry.icon,
                (chips.text)(entry.label),
                MenuItemTone::Regular,
                None,
            );
            if is_clicked {
                action = Some(perform(ui, &target, entry.command, chips));
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
