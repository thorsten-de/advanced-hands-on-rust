//! The `bevy_framework` module provides a framework for game state managing

mod game_menus;
use bevy::{prelude::*, state::state::FreelyMutableState};

/// This plugin provides game state handling. It requires an enumeration of
/// known game states.
///
/// - Type `T` is the enumeration of the states of your game, implementing the
/// `States` trait
pub struct GameStatePlugin<T> {
    /// This state shows the menu screen
    menu_state: T,

    /// This state starts the game
    game_start_state: T,

    /// This state shows the game-over screen
    game_end_state: T,
}

impl<T> GameStatePlugin<T> {
    /// Construct a new `GameStatePlugin` for a given game state enumeraion of type `T`
    pub fn new(menu_state: T, game_start_state: T, game_end_state: T) -> Self {
        Self {
            menu_state,
            game_start_state,
            game_end_state,
        }
    }
}

impl<T: States + Copy + FromWorld + FreelyMutableState> Plugin for GameStatePlugin<T> {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_state::<T>();
        app.add_systems(Startup, setup_menus);
        let start = MenuResource {
            menu_state: self.menu_state,
            game_start_state: self.game_start_state,
            game_end_state: self.game_end_state,
        };
        app.insert_resource(start);

        app.add_systems(OnEnter(self.menu_state), game_menus::setup::<T>);
        app.add_systems(
            Update,
            game_menus::run::<T>.run_if(in_state(self.menu_state)),
        );
        app.add_systems(OnExit(self.menu_state), cleanup::<game_menus::MenuElement>);

        app.add_systems(OnEnter(self.game_end_state), game_menus::setup::<T>);
        app.add_systems(
            Update,
            game_menus::run::<T>.run_if(in_state(self.game_end_state)),
        );
        app.add_systems(
            OnExit(self.game_end_state),
            cleanup::<game_menus::MenuElement>,
        );
    }
}

/// Cleans up all entities spawned with a given component. If all entities of a given
/// game state `x` are tagged with a common component `XElement`, the state can be
/// cleaned up by `cleanup::<XElement>`  
pub fn cleanup<T>(query: Query<Entity, With<T>>, mut commands: Commands)
where
    T: Component,
{
    query
        .iter()
        .for_each(|entity| commands.entity(entity).despawn())
}

#[derive(Resource)]
pub(crate) struct MenuAssets {
    pub(crate) main_menu: Handle<Image>,
    pub(crate) game_over: Handle<Image>,
}

fn setup_menus(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = MenuAssets {
        main_menu: asset_server.load("main_menu.png"),
        game_over: asset_server.load("game_over.png"),
    };
    commands.insert_resource(assets);
}

#[derive(Resource)]
pub(crate) struct MenuResource<T> {
    pub(crate) menu_state: T,
    pub(crate) game_start_state: T,
    pub(crate) game_end_state: T,
}
