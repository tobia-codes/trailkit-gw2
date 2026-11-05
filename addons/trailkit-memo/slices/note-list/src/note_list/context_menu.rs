//! Context menus of the note list: a row's menu with "Rename", the extras
//! of other slices, "Move to…" and, on groups and empty spots, "New note" /
//! "New group" (M86, M96–M99, M112).

use nexus::imgui::Ui;
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText, Text},
    note_tree::{GroupId, InsertionPoint, NodeId, Note, TreeNode as Node},
};
use trailkit_ui::{
    icons::Icon,
    menu::{MenuItemTone, begin_submenu, draw_menu_item, draw_menu_item_with_icon, draw_popup},
    theme::Theme,
};

use super::{NoteListAction, RenameStep, RowContext, delete::blocks_delete};
use crate::i18n::NoteListText;

/// Draws the context menu of a row if it is open (must run every frame).
/// `note` is the row's note; groups pass `None`.
pub(super) fn draw_context_menu(
    ui: &Ui,
    node: NodeId,
    note: Option<&Note>,
    context: &mut RowContext<'_, '_>,
) {
    let (language, theme) = (context.language, context.theme);
    draw_popup(ui, "context", || {
        if draw_menu_item(ui, theme, NoteListText::Rename.get(language), false) {
            context.rename_steps.push(RenameStep::Start(node));
        }
        // Other slices add their items to notes, e.g. the reset type
        // submenu (M86); groups have none.
        if let Some(note) = note {
            (context.extras.context_menu)(ui, note);
        }
        if let Some(_submenu) = begin_submenu(ui, theme, NoteListText::MoveTo.get(language)) {
            draw_move_targets(ui, node, context);
        }
        // Only groups offer creating, into themselves (M96, M97).
        if let NodeId::Group(group) = node {
            ui.separator();
            draw_create_menu_items(
                ui,
                language,
                theme,
                InsertionPoint::End(Some(group)),
                context.actions,
            );
        }
        // Deleting comes last, set off by a separator (M133); the edited
        // note and its groups wait until the editor is left (M141).
        ui.separator();
        let blocked = blocks_delete(context.document, node, context.edited);
        if draw_menu_item_with_icon(
            ui,
            theme,
            Icon::TRASH_2,
            NoteListText::Delete.get(language),
            MenuItemTone::Regular,
            blocked.then(|| NoteListText::SaveOrCancelFirst.get(language)),
        ) {
            context.actions.push(NoteListAction::Delete(node));
        }
    });
}

/// Draws the "Move to" submenu items (M112): "Top level" and every group as
/// an indented flat list; the current place is checked, the moved group
/// and its descendants are left out. A choice moves to the end.
fn draw_move_targets(ui: &Ui, node: NodeId, context: &mut RowContext<'_, '_>) {
    let (theme, language) = (context.theme, context.language);
    let current = context.document.parent(node).flatten();
    if draw_menu_item(
        ui,
        theme,
        NoteListText::TopLevel.get(language),
        current.is_none(),
    ) && current.is_some()
    {
        context
            .actions
            .push(NoteListAction::Move(node, InsertionPoint::End(None)));
    }
    draw_move_target_groups(ui, &context.document.root, node, current, context);
}

/// Draws one "Move to" item per group in `nodes`, nested groups indented
/// below their parent (M112); skips the moved group `node` with its
/// descendants. `current` is the moved node's parent.
fn draw_move_target_groups(
    ui: &Ui,
    nodes: &[Node],
    node: NodeId,
    current: Option<GroupId>,
    context: &mut RowContext<'_, '_>,
) {
    let indent = context.theme.list.indent;
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
        if draw_menu_item(ui, context.theme, &group.title, is_current) && !is_current {
            context.actions.push(NoteListAction::Move(
                node,
                InsertionPoint::End(Some(group.id)),
            ));
        }
        draw_move_target_groups(ui, &group.children, node, current, context);
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
    if draw_menu_item(ui, theme, Text::NewNote.get(language), false) {
        actions.push(NoteListAction::NewNote(Some(point)));
    }
    if draw_menu_item(ui, theme, Text::NewGroup.get(language), false) {
        actions.push(NoteListAction::NewGroup(Some(point)));
    }
}
