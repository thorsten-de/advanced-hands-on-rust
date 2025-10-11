//! The `bevy_framework` module provides a framework for game state managing

mod game_menus;
use crate::add_phase;
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

impl<T: States + Copy + FromWorld + FreelyMutableState + Default> Plugin for GameStatePlugin<T> {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_state::<T>();
        app.add_plugins(bevy_egui::EguiPlugin {
            enable_multipass_for_primary_context: false,
        });
        let start = MenuResource {
            menu_state: self.menu_state,
            game_start_state: self.game_start_state,
            game_end_state: self.game_end_state,
        };
        app.insert_resource(start);

        add_phase!(app, T, self.menu_state,
            start => [ game_menus::setup::<T> ],
            run => [ game_menus::run::<T> ],
            exit => [ cleanup::<game_menus::MenuElement> ]);

        add_phase!(app, T, self.game_end_state,
            start => [ game_menus::setup::<T> ],
            run => [ game_menus::run::<T> ],
            exit => [ cleanup::<game_menus::MenuElement> ]);

        app.add_systems(OnEnter(T::default()), crate::bevy_assets::setup)
            .add_systems(
                Update,
                crate::bevy_assets::run::<T>.run_if(in_state(T::default())),
            )
            .add_systems(OnExit(T::default()), crate::bevy_assets::exit);
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

#[derive(Resource)]
pub(crate) struct MenuResource<T> {
    pub(crate) menu_state: T,
    pub(crate) game_start_state: T,
    pub(crate) game_end_state: T,
}

/// The `add_phase!`-macro lets you specify which systems are used for a
/// sepcific game phase.
#[macro_export]
macro_rules! add_phase {
    (
        $app:expr, $type:ty, $phase:expr,
        start => [ $($start:expr),*],
        run => [ $($run:expr),*],
        exit => [ $($exit:expr),*]
    ) => {
        $($app.add_systems(bevy::prelude::OnEnter::<$type>($phase), $start);)*
        $($app.add_systems(bevy::prelude::OnExit::<$type>($phase), $exit);)*
        $($app.add_systems(bevy::prelude::Update, $run.run_if(in_state($phase)));)*
    };
}
