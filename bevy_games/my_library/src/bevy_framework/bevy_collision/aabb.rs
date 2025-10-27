//! Detect collisions with Axis-aligned bounding boxes (AABB)

use super::rect2d::Rect2D;
use bevy::prelude::*;

/// Defines an axis-aligned bounding box (AABB) for
/// collision detection of an entity
#[derive(Component)]
pub struct AxisAlignedBoundingBox {
    /// Stores the size of the parent entity,
    /// wherever position it is rendered in space
    half_size: Vec2,
}

impl AxisAlignedBoundingBox {
    /// Creates a new AABB
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            half_size: Vec2::new(width / 2.0, height / 2.0),
        }
    }

    /// Converts an AABB with a position into a Rect2D
    pub fn as_rect(&self, translate: Vec2) -> Rect2D {
        Rect2D::new(translate - &self.half_size, translate + &self.half_size)
    }
}
