//! The `bevy_assets` module implements an *Asset Manager* for Bevy.
//!
//! - lists game assets upfront
//! - tags assets for easy access
//! - provides a single resource to get asset handles
//!
//! This allows to declare all assets in one place to manage them more
//! easily.
mod asset_manager;
mod asset_store;
mod loading_menu;

pub use asset_manager::AssetManager;
pub use asset_store::*;
pub(crate) use loading_menu::*;

/// Spawns an image stored by the asset manager
#[macro_export]
macro_rules! spawn_image {
    ($assets:expr, $commands:expr, $index:expr, $x:expr, $y:expr, $z:expr, $resource:expr, $($component:expr),*) => {
        $commands.spawn((
            Sprite::from_image($assets.get_handle($index, $resource).unwrap()),
            Transform::from_xyz($x, $y, $z)))
            $(
                .insert($component)
            )*
    };
}
