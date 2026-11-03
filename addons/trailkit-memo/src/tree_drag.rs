//! Drop zones of the note list's drag & drop (memo plan M106–M111).
//!
//! The window records one [`RowRect`] per drawn row; [`resolve_drop`] turns
//! the mouse position into the [`InsertionPoint`] the drop goes to and the
//! indicator that shows it. Platform-independent so it can be unit-tested
//! on any host (ROADMAP R23).

use trailkit_memo_shared::note_tree::{GroupId, InsertionPoint, NodeId};

/// Mouse movement in pixels after which a press on a row starts a drag
/// (M106), so a click never moves anything by accident.
pub const DRAG_THRESHOLD: f32 = 6.0;

/// Seconds a drag must rest over the middle of a collapsed group before the
/// group expands (M108).
pub const EXPAND_DELAY_SECONDS: f64 = 0.7;

/// What kind of row a [`RowRect`] belongs to; decides its drop zones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    /// A note: upper half = before, lower half = after (M107).
    Note,
    /// An expanded group: upper half = before, lower half = first position
    /// inside the group (M107).
    ExpandedGroup(GroupId),
    /// A collapsed group: top quarter = before, bottom quarter = after,
    /// middle = into, at the end (M108).
    CollapsedGroup(GroupId),
}

/// The screen rectangle of one drawn row of the note list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RowRect {
    /// The note or group of the row.
    pub node: NodeId,
    /// Which drop zones the row has.
    pub kind: RowKind,
    /// Left edge (indented by the row's depth).
    pub left: f32,
    /// Right edge.
    pub right: f32,
    /// Top edge.
    pub top: f32,
    /// Bottom edge.
    pub bottom: f32,
}

/// How the list shows where a drop goes (trailkit-ui skill §4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DropIndicator {
    /// A horizontal line between rows, from `left` to `right` at `y`.
    Line {
        /// Screen y of the line.
        y: f32,
        /// Left end (indented to the level the drop goes to).
        left: f32,
        /// Right end.
        right: f32,
    },
    /// A highlighted collapsed group the drop goes into.
    Highlight {
        /// The row's top-left corner.
        min: [f32; 2],
        /// The row's bottom-right corner.
        max: [f32; 2],
    },
}

/// Where a drop at the mouse position goes and how it is shown.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropTarget {
    /// Where the dragged node is inserted.
    pub point: InsertionPoint,
    /// The indicator for that place.
    pub indicator: DropIndicator,
    /// The collapsed group whose middle is hovered; it expands after
    /// [`EXPAND_DELAY_SECONDS`] (M108).
    pub collapsed_group: Option<GroupId>,
}

/// Finds the drop target for the mouse at height `mouse_y` over `rows`
/// (in display order). `indent` is the indentation of one tree level, used
/// for the line of a drop into an expanded group. Each row's zones reach to
/// the middle of the gap to its neighbors, where the lines are drawn. Below
/// the last row the drop goes to the end of the root level (M111); above
/// the first row there is no target.
#[must_use]
pub fn resolve_drop(rows: &[RowRect], mouse_y: f32, indent: f32) -> Option<DropTarget> {
    let first = rows.first()?;
    let last = rows.last()?;
    if mouse_y < first.top {
        return None;
    }
    if mouse_y >= last.bottom {
        let root_left = rows.iter().map(|row| row.left).fold(first.left, f32::min);
        return Some(DropTarget {
            point: InsertionPoint::End(None),
            indicator: DropIndicator::Line {
                y: last.bottom,
                left: root_left,
                right: last.right,
            },
            collapsed_group: None,
        });
    }
    let index = (0..rows.len())
        .find(|&index| mouse_y < zone_bottom(rows, index))
        .unwrap_or(rows.len() - 1);
    let row = &rows[index];
    let top = index
        .checked_sub(1)
        .map_or(row.top, |previous| zone_bottom(rows, previous));
    let bottom = zone_bottom(rows, index);
    let height = bottom - top;
    let offset = mouse_y - top;
    let line = |y: f32, left: f32| DropIndicator::Line {
        y,
        left,
        right: row.right,
    };
    let before = DropTarget {
        point: InsertionPoint::Before(row.node),
        indicator: line(top, row.left),
        collapsed_group: None,
    };
    let after = DropTarget {
        point: InsertionPoint::After(row.node),
        indicator: line(bottom, row.left),
        collapsed_group: None,
    };
    Some(match row.kind {
        RowKind::Note if offset < height / 2.0 => before,
        RowKind::Note => after,
        RowKind::ExpandedGroup(_) if offset < height / 2.0 => before,
        RowKind::ExpandedGroup(group) => DropTarget {
            point: InsertionPoint::Start(group),
            indicator: line(bottom, row.left + indent),
            collapsed_group: None,
        },
        RowKind::CollapsedGroup(_) if offset < height / 4.0 => before,
        RowKind::CollapsedGroup(_) if offset >= height * 3.0 / 4.0 => after,
        RowKind::CollapsedGroup(group) => DropTarget {
            point: InsertionPoint::End(Some(group)),
            indicator: DropIndicator::Highlight {
                min: [row.left, row.top],
                max: [row.right, row.bottom],
            },
            collapsed_group: Some(group),
        },
    })
}

/// The lower end of the drop zones of `rows[index]`: the middle of the gap
/// to the next row, or the row's own bottom for the last row.
fn zone_bottom(rows: &[RowRect], index: usize) -> f32 {
    let row = &rows[index];
    rows.get(index + 1)
        .map_or(row.bottom, |next| (row.bottom + next.top) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use trailkit_memo_shared::note_tree::NoteId;

    /// A row of `kind` for `node` between `top` and `top + 20`, indented by
    /// `left`.
    fn row(node: NodeId, kind: RowKind, left: f32, top: f32) -> RowRect {
        RowRect {
            node,
            kind,
            left,
            right: 200.0,
            top,
            bottom: top + 20.0,
        }
    }

    /// The point a drop at `mouse_y` goes to.
    fn point_at(rows: &[RowRect], mouse_y: f32) -> Option<InsertionPoint> {
        resolve_drop(rows, mouse_y, 16.0).map(|target| target.point)
    }

    #[test]
    fn zones_of_notes_and_groups() {
        let note = NodeId::Note(NoteId::new_unique());
        let (expanded, collapsed) = (GroupId::new_unique(), GroupId::new_unique());
        let rows = [
            row(note, RowKind::Note, 0.0, 0.0),
            row(
                NodeId::Group(expanded),
                RowKind::ExpandedGroup(expanded),
                0.0,
                20.0,
            ),
            row(
                NodeId::Group(collapsed),
                RowKind::CollapsedGroup(collapsed),
                16.0,
                40.0,
            ),
        ];
        assert_eq!(point_at(&rows, -1.0), None);
        assert_eq!(point_at(&rows, 9.0), Some(InsertionPoint::Before(note)));
        assert_eq!(point_at(&rows, 11.0), Some(InsertionPoint::After(note)));
        assert_eq!(
            point_at(&rows, 29.0),
            Some(InsertionPoint::Before(NodeId::Group(expanded)))
        );
        assert_eq!(point_at(&rows, 31.0), Some(InsertionPoint::Start(expanded)));
        assert_eq!(
            point_at(&rows, 44.0),
            Some(InsertionPoint::Before(NodeId::Group(collapsed)))
        );
        assert_eq!(
            point_at(&rows, 50.0),
            Some(InsertionPoint::End(Some(collapsed)))
        );
        assert_eq!(
            point_at(&rows, 56.0),
            Some(InsertionPoint::After(NodeId::Group(collapsed)))
        );
        assert_eq!(point_at(&rows, 90.0), Some(InsertionPoint::End(None)));
    }

    #[test]
    fn indicators_show_the_target_level() {
        let note = NodeId::Note(NoteId::new_unique());
        let group = GroupId::new_unique();
        let rows = [
            row(
                NodeId::Group(group),
                RowKind::ExpandedGroup(group),
                0.0,
                0.0,
            ),
            row(note, RowKind::Note, 16.0, 20.0),
        ];
        let line = |mouse_y| resolve_drop(&rows, mouse_y, 16.0).map(|target| target.indicator);
        // Into the expanded group: indented line below its title.
        assert_eq!(
            line(15.0),
            Some(DropIndicator::Line {
                y: 20.0,
                left: 16.0,
                right: 200.0
            })
        );
        // Below the list: root level line.
        assert_eq!(
            line(50.0),
            Some(DropIndicator::Line {
                y: 40.0,
                left: 0.0,
                right: 200.0
            })
        );
    }

    #[test]
    fn gaps_between_rows_belong_to_the_nearer_row() {
        let (first, second) = (
            NodeId::Note(NoteId::new_unique()),
            NodeId::Note(NoteId::new_unique()),
        );
        let rows = [
            row(first, RowKind::Note, 0.0, 0.0),
            row(second, RowKind::Note, 0.0, 24.0),
        ];
        assert_eq!(point_at(&rows, 21.0), Some(InsertionPoint::After(first)));
        assert_eq!(point_at(&rows, 23.0), Some(InsertionPoint::Before(second)));
        let line = resolve_drop(&rows, 21.0, 16.0).map(|target| target.indicator);
        assert_eq!(
            line,
            Some(DropIndicator::Line {
                y: 22.0,
                left: 0.0,
                right: 200.0
            })
        );
    }

    #[test]
    fn collapsed_group_middle_is_highlighted() {
        let group = GroupId::new_unique();
        let rows = [row(
            NodeId::Group(group),
            RowKind::CollapsedGroup(group),
            0.0,
            0.0,
        )];
        let target = resolve_drop(&rows, 10.0, 16.0);
        assert_eq!(
            target.and_then(|target| target.collapsed_group),
            Some(group)
        );
        assert!(matches!(
            target.map(|target| target.indicator),
            Some(DropIndicator::Highlight { .. })
        ));
    }
}
