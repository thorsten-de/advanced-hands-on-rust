use bevy::{platform::collections::HashMap, prelude::*};

use crate::AssetStore;

/// Supported asset types
#[derive(Clone, PartialEq, Debug)]
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

    pub fn add_image<S: ToString>(mut self, tag: S, filename: S) -> anyhow::Result<Self> {
        let filename = filename.to_string();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let current_directory = std::env::current_dir()?;
            let assets = current_directory.join("assets");
            let new_image = assets.join(&filename);
            if !new_image.exists() {
                return Err(anyhow::Error::msg(format!(
                    "{} not found in assets directory",
                    &filename
                )));
            }
        }

        self.asset_list
            .push((tag.to_string(), filename, AssetType::Image));

        Ok(self)
    }
}

impl Plugin for AssetManager {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone())
            .add_systems(Startup, setup);
    }
}

fn setup(
    asset_resource: Res<AssetManager>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut assets = AssetStore {
        asset_index: HashMap::new(),
    };

    asset_resource
        .asset_list
        .iter()
        .for_each(|(tag, filename, asset_type)| match asset_type {
            _ => {
                assets
                    .asset_index
                    .insert(tag.clone(), asset_server.load_untyped(filename));
            }
        });

    commands.remove_resource::<AssetManager>();
    commands.insert_resource(assets);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn add_image_when_existing_pushes_asset_to_list() {
        let asset_manager = AssetManager::new();

        let result = asset_manager.add_image("tag", "existing.png");
        assert!(result.is_ok_and(|am| {
            assert!(am.asset_list.len() > 0);
            assert_eq!("tag", am.asset_list[0].0);
            assert_eq!("existing.png", am.asset_list[0].1);
            assert_eq!(AssetType::Image, am.asset_list[0].2);
            true
        }));
    }

    #[test]
    pub fn add_image_when_not_existing_returns_error() {
        let asset_manager = AssetManager::new();

        let result = asset_manager.add_image("tag", "non-existing.png");

        assert!(result.is_err());
    }
}
