//! The `bevy_framework` module provides a framework for game state managing

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

impl<T: States + FromWorld + FreelyMutableState> Plugin for GameStatePlugin<T> {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_state::<T>();
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
