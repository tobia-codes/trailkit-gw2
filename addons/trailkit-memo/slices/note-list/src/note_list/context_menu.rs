//! Menus of an entry: a row's context menu and the note pane's `⋯` menu share
//! "Rename", "Move to…", the extras of other slices and "Delete"; groups and
//! empty spots also offer "New note" / "New group" (M86, M96–M99, M112, M160).

use nexus::imgui::Ui;
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText, Text},
    note_tree::{GroupId, InsertionPoint, NodeId, Note, NoteId, NotesDocument, TreeNode as Node},
};
use trailkit_ui::{
    icons::Icon,
    menu::{MenuItemTone, begin_submenu, draw_menu_item, draw_menu_item_with_icon, draw_popup},
    separator::draw_compact_separator,
    theme::Theme,
};

use super::{NoteListAction, RowContext, delete::blocks_delete, rename::RenameStep};
use crate::i18n::NoteListText;

/// What the items of an entry's menu need.
pub struct EntryMenu<'menu> {
    /// The whole tree ("Move to" lists its groups, M112).
    pub document: &'menu NotesDocument,
    /// The note open in the editor (M141).
    pub edited: Option<NoteId>,
    /// UI language.
    pub language: Language,
    /// Active theme.
    pub theme: &'menu Theme,
    /// Draws the items other slices add to a note, e.g. the reset type
    /// submenu (M86).
    pub note_items: &'menu mut dyn FnMut(&Ui, &Note),
    /// Collects the actions the items trigger.
    pub actions: &'menu mut Vec<NoteListAction>,
}

/// Draws the context menu of a row if it is open (must run every frame).
/// `note` is the row's note; groups pass `None`.
pub(super) fn draw_context_menu(
    ui: &Ui,
    node: NodeId,
    note: Option<&Note>,
    context: &mut RowContext<'_, '_>,
) {
    let mut menu = EntryMenu {
        document: context.document,
        edited: context.edited,
        language: context.language,
        theme: context.theme,
        note_items: &mut *context.extras.context_menu,
        actions: &mut *context.actions,
    };
    let rename_steps = &mut *context.rename_steps;
    let theme = context.theme;
    draw_popup(ui, theme, "context", || {
        if draw_entry_menu_items(ui, node, note, &mut menu) {
            rename_steps.push(RenameStep::Start(node));
        }
    });
}

/// Draws the `⋯` menu of `note` if it is open (must run every frame, right
/// after [`trailkit_ui::menu::place_next_popup`]): the same items as the row's
/// context menu (M160). Returns whether "Rename" was chosen, which the caller
/// answers with the editor's title input (M161).
pub fn draw_note_menu(ui: &Ui, popup_id: &str, note: &Note, mut menu: EntryMenu<'_>) -> bool {
    let mut rename = false;
    let theme = menu.theme;
    draw_popup(ui, theme, popup_id, || {
        rename = draw_entry_menu_items(ui, NodeId::Note(note.id), Some(note), &mut menu);
    });
    rename
}

/// Draws the items of an entry's menu; one order everywhere (M160). Returns
/// whether "Rename" was chosen; everything else lands in `menu.actions`.
fn draw_entry_menu_items(
    ui: &Ui,
    node: NodeId,
    note: Option<&Note>,
    menu: &mut EntryMenu<'_>,
) -> bool {
    let (language, theme) = (menu.language, menu.theme);
    let rename = draw_menu_item_with_icon(
        ui,
        theme,
        Icon::PENCIL,
        NoteListText::Rename.get(language),
        MenuItemTone::Regular,
        None,
    );
    if let Some(_submenu) = begin_submenu(
        ui,
        theme,
        Icon::FOLDER_INPUT,
        NoteListText::MoveTo.get(language),
    ) {
        draw_move_targets(ui, node, menu);
    }
    if let Some(note) = note {
        (menu.note_items)(ui, note);
    }
    // Only groups offer creating, into themselves (M96, M97).
    if let NodeId::Group(group) = node {
        draw_compact_separator(ui, theme);
        draw_create_menu_items(
            ui,
            language,
            theme,
            InsertionPoint::End(Some(group)),
            menu.actions,
        );
    }
    // Deleting comes last, set off by a separator (M133); the edited note and
    // its groups wait until the editor is left (M141).
    draw_compact_separator(ui, theme);
    let blocked = blocks_delete(menu.document, node, menu.edited);
    if draw_menu_item_with_icon(
        ui,
        theme,
        Icon::TRASH_2,
        NoteListText::Delete.get(language),
        MenuItemTone::Danger,
        blocked.then(|| NoteListText::SaveOrCancelFirst.get(language)),
    ) {
        menu.actions.push(NoteListAction::Delete(node));
    }
    rename
}

/// Draws the "Move to" submenu items (M112): "Top level" and every group as
/// an indented flat list; the current place is checked, the moved group
/// and its descendants are left out. A choice moves to the end.
fn draw_move_targets(ui: &Ui, node: NodeId, menu: &mut EntryMenu<'_>) {
    let (theme, language) = (menu.theme, menu.language);
    let current = menu.document.parent(node).flatten();
    if draw_menu_item(
        ui,
        theme,
        NoteListText::TopLevel.get(language),
        current.is_none(),
    ) && current.is_some()
    {
        menu.actions
            .push(NoteListAction::Move(node, InsertionPoint::End(None)));
    }
    let document = menu.document;
    draw_move_target_groups(ui, &document.root, node, current, menu);
}

/// Draws one "Move to" item per group in `nodes`, nested groups indented
/// below their parent (M112); skips the moved group `node` with its
/// descendants. `current` is the moved node's parent.
fn draw_move_target_groups(
    ui: &Ui,
    nodes: &[Node],
    node: NodeId,
    current: Option<GroupId>,
    menu: &mut EntryMenu<'_>,
) {
    let indent = menu.theme.list.indent;
    for child in nodes {
        let Node::Group(group) = child else {
            continue;
        };
        if NodeId::Group(group.id) == node {
            continue;
        }
        let _id = ui.push_id(group.id.imgui_id());
        ui.indent_by(indent);
        let is_current = current == Some(group.id);
        if draw_menu_item(ui, menu.theme, &group.title, is_current) && !is_current {
            menu.actions.push(NoteListAction::Move(
                node,
                InsertionPoint::End(Some(group.id)),
            ));
        }
        draw_move_target_groups(ui, &group.children, node, current, menu);
        ui.unindent_by(indent);
    }
}

/// Draws "New note" and "New group" menu items that create at `point`
/// (M96); the new entry is renamed right away (M43).
pub(super) fn draw_create_menu_items(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    point: InsertionPoint,
    actions: &mut Vec<NoteListAction>,
) {
    if draw_menu_item_with_icon(
        ui,
        theme,
        Icon::FILE_PLUS,
        Text::NewNote.get(language),
        MenuItemTone::Regular,
        None,
    ) {
        actions.push(NoteListAction::NewNote(Some(point)));
    }
    if draw_menu_item_with_icon(
        ui,
        theme,
        Icon::FOLDER_PLUS,
        Text::NewGroup.get(language),
        MenuItemTone::Regular,
        None,
    ) {
        actions.push(NoteListAction::NewGroup(Some(point)));
    }
}
