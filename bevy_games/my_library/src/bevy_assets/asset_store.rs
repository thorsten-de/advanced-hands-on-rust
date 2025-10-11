use bevy::{
    asset::{Asset, LoadedUntypedAsset},
    platform::collections::HashMap,
    prelude::*,
};

// Create aliases for some frequently used types, and protect against
// changes in bevy
pub type LoadedAssets = Assets<LoadedUntypedAsset>;
pub type AssetResource<'w> = Res<'w, LoadedAssets>;

/// Stores the handles for resources defined by the `AssetManager`
#[derive(Resource, Clone)]
pub struct AssetStore {
    pub(crate) asset_index: HashMap<String, Handle<LoadedUntypedAsset>>,
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

        commands.spawn((AudioPlayer::new(sound_handle.clone())));
    }
}
