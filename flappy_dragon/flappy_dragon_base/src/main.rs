use bevy::{app::AppExit, prelude::*};
use my_library::*;

#[derive(Component)]
struct Flappy {
    //(1)
    gravity: f32, //(2)
}

#[derive(Component)]
struct Obstacle; //(3)

#[derive(Resource)]
struct Assets {
    //(4)
    dragon: Handle<Image>,
    wall: Handle<Image>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                //(5)
                title: "Flappy Dragon - Bevy Edition".to_string(),
                resolution: bevy::window::WindowResolution::new(1024.0, 768.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RandomPlugin) //(6)
        .add_systems(Startup, setup)
        .add_systems(Update, gravity)
        .add_systems(Update, flap)
        .add_systems(Update, clamp)
        .add_systems(Update, move_walls)
        .add_systems(Update, hit_wall)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<RandomNumberGenerator>, //(7)
) {
    let assets = Assets {
        //(8)
        dragon: asset_server.load("flappy_dragon.png"),
        wall: asset_server.load("wall.png"),
    };

    commands.spawn(Camera2d::default()); //(9)
    commands.spawn((
        Sprite::from_image(assets.dragon.clone()), //(10)
        Transform::from_xyz(-490.0, 0.0, 1.0),     //(11)
        Flappy { gravity: 0.0 },
    ));

    build_wall(&mut commands, assets.wall.clone(), rng.range(-5..5)); //(12)
    commands.insert_resource(assets); //(13)
}

fn build_wall(commands: &mut Commands, wall_sprite: Handle<Image>, gap_y: i32) {
    for y in -12..=12 {
        //(14)
        if y < gap_y - 4 || y > gap_y + 4 {
            //(15)
            commands.spawn((
                Sprite::from_image(wall_sprite.clone()),
                Transform::from_xyz(512.0, y as f32 * 32.0, 1.0),
                Obstacle,
            ));
        }
    }
}

fn gravity(mut query: Query<(&mut Flappy, &mut Transform)>) {
    if let Ok((mut flappy, mut transform)) = query.single_mut() {
        //(16)
        flappy.gravity += 0.1; //(17)
        transform.translation.y -= flappy.gravity; //(18)
    }
}

fn flap(keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Flappy>) {
    if keyboard.pressed(KeyCode::Space) {
        if let Ok(mut flappy) = query.single_mut() {
            flappy.gravity = -5.0; //(19)
        }
    }
}

fn clamp(
    mut query: Query<&mut Transform, With<Flappy>>,
    mut exit: EventWriter<AppExit>, //(20)
) {
    if let Ok(mut transform) = query.single_mut() {
        if transform.translation.y > 384.0 {
            transform.translation.y = 384.0; //(21)
        } else if transform.translation.y < -384.0 {
            exit.write(AppExit::Success); //(22)
        }
    }
}

fn move_walls(
    mut commands: Commands,
    mut query: Query<&mut Transform, With<Obstacle>>,
    delete: Query<Entity, With<Obstacle>>,
    assets: Res<Assets>,
    mut rng: ResMut<RandomNumberGenerator>,
) {
    let mut rebuild = false;
    for mut transform in query.iter_mut() {
        transform.translation.x -= 4.0;
        if transform.translation.x < -530.0 {
            rebuild = true; //(23)
        }
    }
    if rebuild {
        for entity in delete.iter() {
            commands.entity(entity).despawn();
        }
        build_wall(&mut commands, assets.wall.clone(), rng.range(-5..5));
    }
}

fn hit_wall(
    player: Query<&Transform, With<Flappy>>,  //(24)
    walls: Query<&Transform, With<Obstacle>>, //(25)
    mut exit: EventWriter<AppExit>,
) {
    if let Ok(player) = player.single() {
        //(26)
        for wall in walls.iter() {
            //(27)
            let distance = player.translation.distance(wall.translation); //(28)
            if distance < 32.0 {
                exit.write(AppExit::Success); //(29)
            }
        }
    }
}
