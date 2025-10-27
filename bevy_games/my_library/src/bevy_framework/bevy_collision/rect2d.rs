//! Two-dimensional rectangle for collision detection

use bevy::prelude::*;

/// Two-dimensional rectangle for collision detection
#[derive(Debug, Clone, Copy)]
pub struct Rect2D {
    /// Top-left corner of the rectangle
    min: Vec2,
    /// Bottom-right corner of the rectangle
    max: Vec2,
}

impl Rect2D {
    /// Creates a new Rect2D
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Checks if this rect intersects with other
    pub fn intersect(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// Calculates the center coordinates of this rect
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    /// Divides this rect2D into its quadrants.
    pub fn quadrants(&self) -> Vec<Self> {
        let center = self.center();
        vec![
            // top-left
            Self::new(self.min, center),
            // top-right
            Self::new(
                Vec2::new(center.x, self.min.y),
                Vec2::new(self.max.x, center.y),
            ),
            // bottom-left
            Self::new(
                Vec2::new(self.min.x, center.y),
                Vec2::new(center.x, self.max.y),
            ),
            // bottom-right
            Self::new(center, self.max),
        ]
    }
}
