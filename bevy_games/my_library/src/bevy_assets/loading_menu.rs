use bevy::asset::LoadState;
use bevy::state::state::FreelyMutableState;
use bevy::{asset::LoadedUntypedAsset, prelude::*};
use bevy_egui::{EguiContexts, egui::Window};

use crate::bevy_assets::asset_manager::setup_asset_store;
use crate::{AssetManager, AssetStore, MenuResource};

/// Store handles to be loaded
#[derive(Resource)]
pub(crate) struct AsstesToLoad(Vec<Handle<LoadedUntypedAsset>>);

// Setup resources for loading stage
pub(crate) fn setup(
    assets: Option<Res<AssetStore>>,
    asset_manager: Option<Res<AssetManager>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let assets = match assets {
        Some(assets) => assets.into_inner(),
        None => &setup_asset_store(
            asset_manager.as_ref().unwrap(),
            &mut commands,
            &asset_server,
        ),
    };

    // Handles are cloned to get ownership of the handles
    let assets_to_load: Vec<Handle<LoadedUntypedAsset>> =
        assets.asset_index.values().cloned().collect();
    commands.insert_resource(AsstesToLoad(assets_to_load));
}

// Processing in loading stage
pub(crate) fn run<T>(
    asset_server: Res<AssetServer>,
    mut to_load: ResMut<AsstesToLoad>,
    mut state: ResMut<NextState<T>>,
    mut egui_context: EguiContexts,
    menu_info: Res<MenuResource<T>>,
) where
    T: States + FromWorld + FreelyMutableState,
{
    to_load
        .0
        .retain(|handle| match asset_server.get_load_state(handle.id()) {
            Some(LoadState::Loaded) => false,
            _ => true,
        });

    if to_load.0.is_empty() {
        state.set(menu_info.menu_state.clone());
    }
    info!("{} assets remaining", to_load.0.len());

    Window::new("Loading, Please Wait").show(egui_context.ctx_mut(), |ui| {
        ui.label(&format!("{} assets remaining", to_load.0.len()));
    });
}

/// Cleanup resources after loading stage
pub(crate) fn exit(mut commands: Commands) {
    commands.remove_resource::<AsstesToLoad>();
}
