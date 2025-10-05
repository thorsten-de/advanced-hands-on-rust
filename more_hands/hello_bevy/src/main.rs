use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, movement)
        .run();
}

#[derive(Component)]
struct Dragon;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d::default());
    let dragon_image = asset_server.load("dragon.png");

    commands
        .spawn(Sprite::from_image(dragon_image))
        .insert(Dragon);
}

fn movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut dragon_query: Query<&mut Transform, With<Dragon>>,
) {
    let delta = if keyboard.pressed(KeyCode::ArrowLeft) {
        Vec2::new(-1.0, 0.0)
    } else if keyboard.pressed(KeyCode::ArrowRight) {
        Vec2::new(1.0, 0.0)
    } else if keyboard.pressed(KeyCode::ArrowDown) {
        Vec2::new(0.0, -1.0)
    } else if keyboard.pressed(KeyCode::ArrowUp) {
        Vec2::new(0.0, 1.0)
    } else {
        Vec2::ZERO
    };

    dragon_query.iter_mut().for_each(|mut transform| {
        transform.translation += delta.extend(0.0);
    });
}
