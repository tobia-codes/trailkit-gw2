//! A rectangle in screen coordinates, e.g. a window or a dialog on it.

/// A rectangle in screen coordinates (pixels from the top-left of the game
/// window).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenRect {
    /// Top-left corner.
    pub min: [f32; 2],
    /// Bottom-right corner.
    pub max: [f32; 2],
}

impl ScreenRect {
    /// The rectangle at `position` (top-left corner) with `size`.
    #[must_use]
    pub const fn from_position_size(position: [f32; 2], size: [f32; 2]) -> Self {
        Self {
            min: position,
            max: [position[0] + size[0], position[1] + size[1]],
        }
    }

    /// Width and height.
    #[must_use]
    pub const fn size(&self) -> [f32; 2] {
        [self.max[0] - self.min[0], self.max[1] - self.min[1]]
    }

    /// Whether `point` lies on the rectangle, edges included.
    #[must_use]
    pub fn contains(&self, point: [f32; 2]) -> bool {
        (self.min[0]..=self.max[0]).contains(&point[0])
            && (self.min[1]..=self.max[1]).contains(&point[1])
    }
}

#[cfg(test)]
mod tests {
    use super::ScreenRect;

    /// The rectangle a test checks: 100 × 50 at (10, 20).
    const RECT: ScreenRect = ScreenRect::from_position_size([10.0, 20.0], [100.0, 50.0]);

    #[test]
    fn from_position_size_keeps_the_size() {
        assert_eq!(RECT.max, [110.0, 70.0]);
        assert_eq!(RECT.size(), [100.0, 50.0]);
    }

    #[test]
    fn contains_points_on_the_rectangle_and_its_edges() {
        assert!(RECT.contains([50.0, 40.0]));
        assert!(RECT.contains([10.0, 20.0]));
        assert!(RECT.contains([110.0, 70.0]));
    }

    #[test]
    fn does_not_contain_points_beside_it() {
        assert!(!RECT.contains([9.0, 40.0]));
        assert!(!RECT.contains([50.0, 71.0]));
    }
}
