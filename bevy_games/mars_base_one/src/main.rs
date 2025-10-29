use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use my_library::*;

/// Game Phases for Mars Base One
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
enum GamePhase {
    #[default]
    Loading,
    MainMenu,
    Playing,
    GameOver,
}

///  Component for identifying game element entities
#[derive(Component)]
struct GameElement;

/// Component that identifies the player entity
#[derive(Component)]
struct Player;

/// Component representing the camera tag
#[derive(Component)]
struct MyCamera;

/// Component to tag ground entities
#[derive(Component)]
struct Ground;

fn main() -> anyhow::Result<()> {
    let mut app = App::new();

    add_phase!(app, GamePhase, GamePhase::Playing,
       start => [ setup ],
       run => [movement, end_game, physics_clock, sum_impulses, apply_gravity, apply_velocity,
        cap_velocity.after(apply_velocity), camera_follow.after(cap_velocity)],
       exit => [cleanup::<GameElement>]
    );

    app.add_event::<Impulse>()
        .add_event::<PhysicsTick>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Mars Base One".to_string(),
                resolution: bevy::window::WindowResolution::new(1024.0, 768.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RandomPlugin)
        .add_plugins(GameStatePlugin::new(
            GamePhase::MainMenu,
            GamePhase::Playing,
            GamePhase::GameOver,
        ))
        .add_plugins(
            AssetManager::new()
                .add_image("ship", "ship.png")?
                .add_image("ground", "ground.png")?,
        )
        .insert_resource(Animations::new())
        .run();

    Ok(())
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetStore>,
    loaded_assets: Res<LoadedAssets>,
    mut rng: ResMut<RandomNumberGenerator>,
) {
    let camera = Camera2d::default();
    // This determines the transformation from world-coordinates to screen-coordinates.
    // A camera defines how the viewport is rendered to show the world. Technically, this
    // is done with a *projection matrix*.
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::WindowSize,
        scale: 0.5,
        ..OrthographicProjection::default_2d()
    });
    commands.spawn((camera, projection, GameElement, MyCamera));

    spawn_image!(
        assets,
        commands,
        "ship",
        0.0,
        0.0,
        1.0,
        &loaded_assets,
        GameElement,
        Player,
        Velocity::default(),
        PhysicsPosition::new(Vec2::new(0.0, 0.0)) // ApplyGravity
    );

    let world = World::new(200, 200, &mut rng);
    world.spawn(&assets, &mut commands, &loaded_assets);
    commands.insert_resource(StaticQuadTree::new(Vec2::new(10240.0, 7680.0), 6));
}

fn end_game(
    mut state: ResMut<NextState<GamePhase>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(transform) = player_query.single() else {
        return;
    };
    if transform.translation.y < -384.0
        || transform.translation.y > 384.0
        || transform.translation.x < -512.0
        || transform.translation.x > 512.0
    {
        state.set(GamePhase::GameOver);
    }
}

fn movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(Entity, &mut Transform), With<Player>>,
    mut impulses: EventWriter<Impulse>,
) {
    let Ok((entity, mut transform)) = player_query.single_mut() else {
        return;
    };
    if keyboard.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        transform.rotate(Quat::from_rotation_z(f32::to_radians(2.0)));
    }
    if keyboard.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        transform.rotate(Quat::from_rotation_z(f32::to_radians(-2.0)));
    }
    if keyboard.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        impulses.write(Impulse {
            target: entity,
            amount: transform.local_y().as_vec3(),
            absolute: false,
            source: 1,
        });
    }
}

fn cap_velocity(mut player_query: Query<&mut Velocity, With<Player>>) {
    let Ok(mut velocity) = player_query.single_mut() else {
        return;
    };
    let v2 = velocity.0.truncate();
    if v2.length() > 5.0 {
        let v2 = v2.normalize() * 5.0;
        velocity.0.x = v2.x;
        velocity.0.y = v2.y;
    }
}

fn camera_follow(
    player_query: Query<&Transform, (With<Player>, Without<MyCamera>)>,
    mut camera_query: Query<&mut Transform, (With<MyCamera>, Without<Player>)>,
) {
    let Ok(player) = player_query.single() else {
        return;
    };
    let Ok(mut camera) = camera_query.single_mut() else {
        return;
    };
    camera.translation = Vec3::new(player.translation.x, player.translation.y, 10.0);
}

/// Defines the world by a 2d-matrix of cells.
struct World {
    /// If a cell is a solid wall for each given index
    solid: Vec<bool>,
    /// Horizontal map size
    width: usize,
    /// Vertical map size
    height: usize,
}

const CELL_SIZE: f32 = 24.0;

impl World {
    /// Calculates the 1d index for a given cell in the 2d matrix
    fn map_idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Creates a new world
    fn new(width: usize, height: usize, rng: &mut RandomNumberGenerator) -> Self {
        let mut result = Self {
            width,
            height,
            solid: vec![true; width * height],
        };

        result
    }
    /// Spawns the world into the game
    fn spawn(&self, assets: &AssetStore, commands: &mut Commands, loaded_assets: &LoadedAssets) {
        let x_offset = self.width as f32 * CELL_SIZE / 2.0;
        let y_offset = self.height as f32 * CELL_SIZE / 2.0;

        for y in 0..self.height {
            for x in 0..self.width {
                if self.solid[self.map_idx(x, y)] {
                    let position = Vec2::new(
                        x as f32 * CELL_SIZE - x_offset,
                        y as f32 * CELL_SIZE - y_offset,
                    );

                    spawn_image!(
                        assets,
                        commands,
                        "ground",
                        position.x,
                        position.y,
                        -1.0,
                        &loaded_assets,
                        GameElement,
                        Ground,
                        PhysicsPosition::new(position),
                        AxisAlignedBoundingBox::new(CELL_SIZE, CELL_SIZE)
                    );
                }
            }
        }
    }
}
