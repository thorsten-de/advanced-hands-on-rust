use bevy::prelude::*;

/// Supported asset types
#[derive(Clone)]
pub enum AssetType {
    Image,
}

/// The bevy resource to manages assets.
#[derive(Resource, Clone)]
pub struct AssetManager {
    asset_list: Vec<(String, String, AssetType)>,
}

impl AssetManager {
    /// Creates a new asset manager resource
    pub fn new() -> Self {
        Self {
            asset_list: Vec::new(),
        }
    }
}
