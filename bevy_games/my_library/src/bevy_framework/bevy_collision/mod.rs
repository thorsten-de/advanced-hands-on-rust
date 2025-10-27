//! This module implements a collision detection framework for bevy

mod aabb;
mod rect2d;
mod static_quadtree;

pub use aabb::AxisAlignedBoundingBox;
pub use rect2d::Rect2D;
pub use static_quadtree::*;

use bevy::{platform::collections::HashMap, prelude::*};
use std::marker::PhantomData;
