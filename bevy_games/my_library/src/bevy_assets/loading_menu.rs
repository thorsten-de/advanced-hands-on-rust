use crate::bevy_assets::asset_manager::setup_asset_store;
use crate::egui::{EguiContexts, egui::Window};
use crate::{AssetManager, AssetStore, LoadedAssets, MenuResource};
use bevy::asset::LoadState;
use bevy::state::state::FreelyMutableState;
use bevy::{asset::LoadedUntypedAsset, prelude::*};

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
    mut store: ResMut<AssetStore>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    loaded_assets: Res<LoadedAssets>,
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
        load_atlases(&mut store, &mut texture_atlases, &loaded_assets);
        state.set(menu_info.menu_state.clone());
    }
    info!("Loading, {} assets remaining", to_load.0.len());

    Window::new("Loading, Please Wait").show(egui_context.ctx_mut(), |ui| {
        ui.label(format!("{} assets remaining", to_load.0.len()))
    });
}

/// Cleanup resources after loading stage
pub(crate) fn exit(mut commands: Commands) {
    commands.remove_resource::<AsstesToLoad>();
}

/// Build the texture when the underlying image is loaded
fn load_atlases(
    store: &mut AssetStore,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    loaded_assets: &LoadedAssets,
) {
    for new_atlas in store.atlases_to_build.iter() {
        let atlas = TextureAtlasLayout::from_grid(
            new_atlas.tile_size.as_uvec2(),
            new_atlas.sprites_x as u32,
            new_atlas.sprites_y as u32,
            None,
            None,
        );

        let atlas_handle = texture_atlases.add(atlas);
        let img = store
            .get_handle(&new_atlas.texture_tag, loaded_assets)
            .unwrap();
        store
            .atlases
            .insert(new_atlas.tag.clone(), (img, atlas_handle));
    }
}
