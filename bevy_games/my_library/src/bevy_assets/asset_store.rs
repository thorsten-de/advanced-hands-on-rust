use bevy::{
    asset::{Asset, LoadedUntypedAsset},
    platform::collections::HashMap,
    prelude::*,
};

// Create aliases for some frequently used types, and protect against
// changes in bevy
/// Untyped loaded assets
pub type LoadedAssets = Assets<LoadedUntypedAsset>;

/// Resource of loaded assets
pub type AssetResource<'w> = Res<'w, LoadedAssets>;

/// Stores the handles for resources defined by the `AssetManager`
#[derive(Resource, Clone)]
pub struct AssetStore {
    pub(crate) asset_index: HashMap<String, Handle<LoadedUntypedAsset>>,
    pub(crate) atlases_to_build: Vec<FutureAtlas>,
    pub(crate) atlases: HashMap<String, (Handle<Image>, Handle<TextureAtlasLayout>)>,
}

impl AssetStore {
    /// Returns a handle to a stored resource
    pub fn get_handle<T>(&self, index: &str, assets: &LoadedAssets) -> Option<Handle<T>>
    where
        T: Asset,
    {
        if let Some(handle_untyped) = self.asset_index.get(index) {
            if let Some(handle) = assets.get(handle_untyped) {
                return Some(handle.handle.clone().typed::<T>());
            }
            None
        } else {
            None
        }
    }

    /// Plays a sound
    pub fn play(&self, sound_name: &str, commands: &mut Commands, assets: &LoadedAssets) {
        let sound_handle: Handle<AudioSource> = self.get_handle(sound_name, assets).unwrap();

        commands.spawn((
            AudioPlayer::new(sound_handle.clone()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                ..default()
            },
        ));
    }

    /// Returns a handle to both the sprite image and the atlas layout
    pub fn get_atlas_handle(
        &self,
        index: &str,
    ) -> Option<(Handle<Image>, Handle<TextureAtlasLayout>)> {
        if let Some(handle) = self.atlases.get(index) {
            return Some(handle.clone());
        }
        None
    }
}

#[derive(Clone)]
pub(crate) struct FutureAtlas {
    pub(crate) tag: String,
    pub(crate) texture_tag: String,
    pub(crate) tile_size: Vec2,
    pub(crate) sprites_x: usize,
    pub(crate) sprites_y: usize,
}
