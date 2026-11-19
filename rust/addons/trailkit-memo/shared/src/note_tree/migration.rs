//! Migrations of `notes.json` between schema versions (memo plan M29).
//!
//! Version 2 renamed the note field `kind` to `reset_schedule` with the
//! values `never`, `daily` and `weekly` (was `normal`, `daily_reset`,
//! `weekly_reset`, M203) and removed the group field `expanded`, which is
//! UI state in `ui_state.json` now (M201; collapsed groups open once).

use serde_json::Value;
use trailkit_storage::{DocumentFields, MigrationError, SchemaVersion};

/// Migrates `fields` from version `from` to `from.next()`.
///
/// # Errors
///
/// Returns [`MigrationError::NoMigrationPath`] for a version without a
/// step, and [`MigrationError::InvalidDocument`] if a note has an unknown
/// `kind` or the tree does not have the expected shape.
pub(super) fn migrate_one_step(
    from: SchemaVersion,
    mut fields: DocumentFields,
) -> Result<DocumentFields, MigrationError> {
    if from != SchemaVersion::FIRST {
        return Err(MigrationError::NoMigrationPath { from });
    }
    let invalid = |reason: String| MigrationError::InvalidDocument { from, reason };
    if let Some(root) = fields.get_mut("root") {
        rename_kinds_in_nodes(root).map_err(invalid)?;
    }
    if let Some(trash) = fields.get_mut("trash") {
        let entries = trash
            .as_array_mut()
            .ok_or_else(|| invalid("`trash` is not an array".to_owned()))?;
        for entry in entries {
            if let Some(node) = entry.get_mut("node") {
                rename_kinds_in_node(node).map_err(invalid)?;
            }
        }
    }
    Ok(fields)
}

/// Applies [`rename_kinds_in_node`] to every node of the array `nodes`.
///
/// # Errors
///
/// Returns a reason for the log if `nodes` is not an array or a node fails.
fn rename_kinds_in_nodes(nodes: &mut Value) -> Result<(), String> {
    let nodes = nodes
        .as_array_mut()
        .ok_or_else(|| "a node list is not an array".to_owned())?;
    nodes.iter_mut().try_for_each(rename_kinds_in_node)
}

/// Renames `kind` to `reset_schedule` in a note; in a group, removes
/// `expanded` and migrates every node below it.
///
/// # Errors
///
/// Returns a reason for the log if the node is not an object or a note has
/// an unknown `kind`.
fn rename_kinds_in_node(node: &mut Value) -> Result<(), String> {
    let node = node
        .as_object_mut()
        .ok_or_else(|| "a node is not an object".to_owned())?;
    if let Some(children) = node.get_mut("children") {
        rename_kinds_in_nodes(children)?;
        node.remove("expanded");
        return Ok(());
    }
    let Some(kind) = node.remove("kind") else {
        return Ok(());
    };
    let reset_schedule = match kind.as_str() {
        Some("normal") => "never",
        Some("daily_reset") => "daily",
        Some("weekly_reset") => "weekly",
        _ => return Err(format!("unknown note kind {kind}")),
    };
    node.insert("reset_schedule".to_owned(), reset_schedule.into());
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// Runs the version 1 → 2 step on `document`.
    fn migrate(document: Value) -> Result<Value, MigrationError> {
        let Value::Object(fields) = document else {
            panic!("test document must be an object");
        };
        migrate_one_step(SchemaVersion::FIRST, fields).map(Value::Object)
    }

    #[test]
    fn renames_kind_and_drops_expanded_in_tree_and_trash() {
        let migrated = migrate(json!({
            "root": [
                {"type": "note", "kind": "normal", "title": "A"},
                {"type": "group", "expanded": false, "children": [
                    {"type": "note", "kind": "daily_reset", "last_reset_applied_at": "2026-11-01T00:00:00Z"},
                    {"type": "group", "expanded": true, "children": [{"type": "note", "kind": "weekly_reset"}]}
                ]}
            ],
            "trash": [{"node": {"type": "note", "kind": "normal"}, "former_index": 0}]
        }))
        .expect("valid version 1 document");
        assert_eq!(
            migrated,
            json!({
                "root": [
                    {"type": "note", "reset_schedule": "never", "title": "A"},
                    {"type": "group", "children": [
                        {"type": "note", "reset_schedule": "daily", "last_reset_applied_at": "2026-11-01T00:00:00Z"},
                        {"type": "group", "children": [{"type": "note", "reset_schedule": "weekly"}]}
                    ]}
                ],
                "trash": [{"node": {"type": "note", "reset_schedule": "never"}, "former_index": 0}]
            })
        );
    }

    #[test]
    fn rejects_unknown_kind_and_bad_shape() {
        assert!(migrate(json!({"root": [{"type": "note", "kind": "monthly"}]})).is_err());
        assert!(migrate(json!({"root": {"type": "note"}})).is_err());
        assert!(migrate(json!({"trash": "none"})).is_err());
    }

    #[test]
    fn has_no_step_from_version_two() {
        let result = migrate_one_step(SchemaVersion(2), DocumentFields::new());
        assert!(matches!(
            result,
            Err(MigrationError::NoMigrationPath { .. })
        ));
    }
}
