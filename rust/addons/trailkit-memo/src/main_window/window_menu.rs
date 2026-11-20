//! The window menu of the memo main window (M166): the title bar's menu button
//! opens a panel over the full content height with a dimmed backdrop. It holds
//! the ways to the notes, the search and the trash, and hides the note list.

use nexus::imgui::Ui;
use trailkit_ui::{
    controls::menu::{MenuEntry, MenuItem, draw_menu_panel},
    feedback::backdrop::{Backdrop, BackdropArea},
    text::icons::Icon,
    theme::Theme,
};

use trailkit_memo_shared::i18n::{Language, LocalizedText, Text};
use trailkit_memo_trash::i18n::TrashText;

use super::{MemoWindow, WindowAction};

/// Largest share of the window the menu may cover, so a strip of the backdrop
/// stays visible to click away (`lay-drawer`).
const WINDOW_MENU_MAXIMUM_FRACTION: f32 = 0.85;
/// Narrowest the menu gets in average characters, so it reads as a panel and
/// not as a list tailored to its longest label (M166).
const WINDOW_MENU_MINIMUM_CHARACTERS: f32 = 27.0;

/// One entry of the window menu and what choosing it does.
#[derive(Debug, Clone)]
struct WindowMenuEntry {
    /// What the entry shows.
    entry: MenuEntry<'static>,
    /// What choosing it does; `None` for a separator.
    action: Option<WindowAction>,
}

impl WindowMenuEntry {
    /// A row showing `item` that triggers `action`.
    const fn row(item: MenuItem<'static>, action: WindowAction) -> Self {
        Self {
            entry: MenuEntry::Item(item),
            action: Some(action),
        }
    }

    /// A separator that sets the rows below apart.
    const SEPARATOR: Self = Self {
        entry: MenuEntry::Separator,
        action: None,
    };
}

impl MemoWindow {
    /// The entries the menu offers now. Without notes there is nothing to
    /// search or hide; the editor hides the note list by itself (M167).
    fn window_menu_entries(&self, language: Language) -> Vec<WindowMenuEntry> {
        let has_notes = !self.document.root.is_empty();
        let mut entries = vec![WindowMenuEntry::row(
            MenuItem::new(Text::MenuNotes.get(language))
                .icon(Icon::FILE_TEXT)
                .selected(!self.trash.visible),
            WindowAction::ShowNotes,
        )];
        if has_notes {
            entries.push(WindowMenuEntry::row(
                MenuItem::new(Text::MenuSearch.get(language))
                    .icon(Icon::SEARCH)
                    .shortcut(Text::SearchShortcut.get(language)),
                WindowAction::ShowSearch,
            ));
        }
        entries.push(WindowMenuEntry::row(
            MenuItem::new(TrashText::Trash.get(language))
                .icon(Icon::TRASH_2)
                .selected(self.trash.visible),
            WindowAction::OpenTrash,
        ));
        if has_notes && self.editor.is_none() {
            let label = if self.ui_state.note_list_hidden {
                Text::ShowNoteList.get(language)
            } else {
                Text::HideNoteList.get(language)
            };
            entries.push(WindowMenuEntry::SEPARATOR);
            entries.push(WindowMenuEntry::row(
                MenuItem::new(label).icon(Icon::PANEL_LEFT),
                WindowAction::ToggleNoteList,
            ));
        }
        entries
    }

    /// Draws the open menu over the content area from `start` with `size`: it
    /// appears instantly (`overlay-motion`) and closes on a click on the
    /// backdrop or when a row is chosen.
    pub(super) fn draw_window_menu(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        start: [f32; 2],
        size: [f32; 2],
    ) {
        if !self.window_menu_open {
            return;
        }
        ui.set_cursor_screen_pos(start);
        let menu = self.window_menu_entries(language);
        let entries: Vec<_> = menu.iter().map(|entry| entry.entry).collect();
        let mut chosen = None;
        let is_backdrop_clicked =
            Backdrop::new("window_menu_backdrop", BackdropArea::Region { size })
                .clickable(true)
                .draw(ui, theme, || {
                    let minimum = ui.calc_text_size("n")[0] * WINDOW_MENU_MINIMUM_CHARACTERS;
                    let maximum = size[0] * WINDOW_MENU_MAXIMUM_FRACTION;
                    chosen =
                        draw_menu_panel(ui, theme, "window_menu", [minimum, maximum], &entries)
                            .and_then(|index| menu.get(index))
                            .and_then(|entry| entry.action.clone());
                });
        if let Some(action) = chosen {
            self.window_menu_open = false;
            self.actions.push(action);
        } else if is_backdrop_clicked {
            self.window_menu_open = false;
        }
    }
}
