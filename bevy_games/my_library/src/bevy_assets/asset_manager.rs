use bevy::{platform::collections::HashMap, prelude::*};

use crate::AssetStore;

/// Supported asset types
#[derive(Clone, PartialEq, Debug)]
pub enum AssetType {
    Image,
    Sound,
    /// Defines a set of frames (sub-images) on an image
    SpriteSheet {
        /// The frame size (x, y)
        tile_size: Vec2,
        /// number of columns
        sprites_x: usize,
        /// number of rows
        sprites_y: usize,
    },
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
            asset_list: vec![
                (
                    "main_menu".to_string(),
                    "main_menu.png".to_string(),
                    AssetType::Image,
                ),
                (
                    "game_over".to_string(),
                    "game_over.png".to_string(),
                    AssetType::Image,
                ),
            ],
        }
    }

    /// Adds an image to the asset manager
    pub fn add_image<S: ToString>(mut self, tag: S, filename: S) -> anyhow::Result<Self> {
        let filename = filename.to_string();
        Self::asset_exists(&filename)?;

        self.asset_list
            .push((tag.to_string(), filename, AssetType::Image));
        Ok(self)
    }

    /// Adds a sound to the asset manager
    pub fn add_sound<S: ToString>(mut self, tag: S, filename: S) -> anyhow::Result<Self> {
        let filename = filename.to_string();
        Self::asset_exists(&filename)?;

        self.asset_list
            .push((tag.to_string(), filename, AssetType::Sound));
        Ok(self)
    }

    /// Adds a sprite sheet to the asset manager
    pub fn add_sprite_sheet<S: ToString>(
        mut self,
        tag: S,
        filename: S,
        sprite_width: f32,
        sprite_height: f32,
        sprites_x: usize,
        sprites_y: usize,
    ) -> anyhow::Result<Self> {
        let filename = filename.to_string();
        Self::asset_exists(&filename)?;

        self.asset_list.push((
            tag.to_string(),
            filename,
            AssetType::SpriteSheet {
                tile_size: Vec2::new(sprite_width, sprite_height),
                sprites_x,
                sprites_y,
            },
        ));

        Ok(self)
    }

    fn asset_exists(filename: &String) -> Result<(), anyhow::Error> {
        let current_directory = std::env::current_dir()?;
        let assets = current_directory.join("assets");
        let new_image = assets.join(filename);
        if !new_image.exists() {
            return Err(anyhow::Error::msg(format!(
                "{} not found in assets directory",
                &filename
            )));
        }
        Ok(())
    }
}

impl Plugin for AssetManager {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone());
    }
}

pub(crate) fn setup_asset_store(
    asset_resource: &AssetManager,
    commands: &mut Commands,
    asset_server: &AssetServer,
) -> AssetStore {
    let mut assets = AssetStore {
        asset_index: HashMap::new(),
        atlases: HashMap::new(),
        atlases_to_build: Vec::new(),
    };

    asset_resource
        .asset_list
        .iter()
        .for_each(|(tag, filename, asset_type)| match asset_type {
            AssetType::SpriteSheet {
                tile_size,
                sprites_x,
                sprites_y,
            } => {
                // Load the underlying image and place it under a special tag
                let image_handle = asset_server.load_untyped(filename);
                let base_tag = format!("{tag}_base");
                assets.asset_index.insert(base_tag.clone(), image_handle);

                // Add atlas details with the original tag
                assets.atlases_to_build.push(crate::FutureAtlas {
                    tag: tag.clone(),
                    texture_tag: base_tag,
                    tile_size: *tile_size,
                    sprites_x: *sprites_x,
                    sprites_y: *sprites_y,
                });
            }
            _ => {
                assets
                    .asset_index
                    .insert(tag.clone(), asset_server.load_untyped(filename));
            }
        });
    commands.remove_resource::<AssetManager>();
    commands.insert_resource(assets.clone());
    assets
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
