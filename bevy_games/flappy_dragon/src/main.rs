use bevy::prelude::*;
use my_library::*;

#[derive(Component)]
struct Flappy;

#[derive(Component)]
struct Obstacle; //(3)

/// Marker component denoting all entities spawned inside GamePhase::Flapping
#[derive(Component)]
struct FlappyElement;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
enum GamePhase {
    #[default]
    Loading,
    MainMenu,
    Flapping,
    GameOver,
}

fn main() -> anyhow::Result<()> {
    let mut app = App::new();

    add_phase!(app, GamePhase, GamePhase::Flapping,
        start => [ setup ],
        run => [ flap, clamp, move_walls, hit_wall, cycle_animations, continual_parallax,
                 physics_clock, sum_impulses, apply_gravity, apply_velocity],
        exit => [ cleanup::<FlappyElement> ]
    );

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            //(5)
            title: "Flappy Dragon - Bevy Edition".to_string(),
            resolution: bevy::window::WindowResolution::new(1024.0, 768.0),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(RandomPlugin) //(6)
    .add_plugins(GameStatePlugin::<GamePhase>::new(
        GamePhase::MainMenu,
        GamePhase::Flapping,
        GamePhase::GameOver,
    ))
    .add_plugins(
        AssetManager::new()
            .add_image("dragon", "flappy_dragon.png")?
            .add_image("wall", "wall.png")?
            .add_sound("flap", "dragonflap.ogg")?
            .add_sound("crash", "crash.ogg")?
            .add_sprite_sheet("flappy", "flappy_sprite_sheet.png", 62.0, 65.0, 4, 1)?
            .add_image("bg_static", "rocky-far-mountains.png")?
            .add_image("bg_far", "rocky-nowater-far.png")?
            .add_image("bg_mid", "rocky-nowater-mid.png")?
            .add_image("bg_close", "rocky-nowater-close.png")?,
    )
    .insert_resource(
        Animations::new()
            .with_animation(
                "Straight and Level",
                PerFrameAnimation::new(vec![
                    AnimationFrame::new(2, 500, vec![AnimationOption::NextFrame]),
                    AnimationFrame::new(3, 500, vec![AnimationOption::GoToFrame(0)]),
                ]),
            )
            .with_animation(
                "Flapping",
                PerFrameAnimation::new(vec![
                    AnimationFrame::new(
                        0,
                        66,
                        vec![
                            AnimationOption::NextFrame,
                            AnimationOption::PlaySound("flap".to_string()),
                        ],
                    ),
                    AnimationFrame::new(1, 66, vec![AnimationOption::NextFrame]),
                    AnimationFrame::new(2, 66, vec![AnimationOption::NextFrame]),
                    AnimationFrame::new(3, 66, vec![AnimationOption::NextFrame]),
                    AnimationFrame::new(2, 66, vec![AnimationOption::NextFrame]),
                    AnimationFrame::new(
                        1,
                        66,
                        vec![AnimationOption::SwitchToAnimation(
                            "Straight and Level".to_string(),
                        )],
                    ),
                ]),
            ),
    )
    .run();

    Ok(())
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetStore>,
    loaded_assets: AssetResource,
    mut rng: ResMut<RandomNumberGenerator>, //(7)
) {
    commands.spawn(Camera2d::default()).insert(FlappyElement); //(9)

    spawn_animated_sprite!(
        assets,
        commands,
        "flappy",
        -490.0,
        0.0,
        10.0, // higher z-level for placing flappy above the parallax backgrounds
        "Straight and Level",
        Flappy,
        FlappyElement,
        Velocity::default(),
        ApplyGravity
    );

    let width = 1280.0;
    spawn_image!(
        assets,
        commands,
        "bg_static",
        0.0,
        0.0,
        1.0, // static background layer
        &loaded_assets,
        FlappyElement
    );

    spawn_image!(
        assets,
        commands,
        "bg_far",
        0.0,
        0.0,
        2.0, // second parallax layer
        &loaded_assets,
        FlappyElement,
        ContinualParallax::new(width, 66, Vec2::new(1.0, 0.0))
    );
    spawn_image!(
        assets,
        commands,
        "bg_far",
        width,
        0.0,
        2.0, // second parallax layer
        &loaded_assets,
        FlappyElement,
        ContinualParallax::new(width, 66, Vec2::new(1.0, 0.0))
    );

    spawn_image!(
        assets,
        commands,
        "bg_mid",
        0.0,
        0.0,
        3.0, // third parallax layer
        &loaded_assets,
        FlappyElement,
        ContinualParallax::new(width, 33, Vec2::new(1.0, 0.0))
    );
    spawn_image!(
        assets,
        commands,
        "bg_mid",
        width,
        0.0,
        3.0, // third parallax layer
        &loaded_assets,
        FlappyElement,
        ContinualParallax::new(width, 33, Vec2::new(1.0, 0.0))
    );

    spawn_image!(
        assets,
        commands,
        "bg_close",
        0.0,
        0.0,
        4.0, // fourth parallax layer
        &loaded_assets,
        FlappyElement,
        ContinualParallax::new(width, 16, Vec2::new(2.0, 0.0))
    );
    spawn_image!(
        assets,
        commands,
        "bg_close",
        width,
        0.0,
        4.0, // fourth parallax layer
        &loaded_assets,
        FlappyElement,
        ContinualParallax::new(width, 16, Vec2::new(2.0, 0.0))
    );

    /*
    let Some((img, atlas)) = assets.get_atlas_handle("flappy") else {
        panic!()
    };
    commands.spawn((
        Sprite::from_atlas_image(
            img.clone(),
            TextureAtlas {
                layout: atlas.clone(),
                index: 0,
            },
        ),
        Transform::from_xyz(-490.0, 0.0, 1.0),
        AnimationCycle::new("Straight and Level"),
        Flappy { gravity: 0.0 },
        FlappyElement,
    ));
    */

    build_wall(&mut commands, &assets, &loaded_assets, rng.range(-5..5)); //(12)
}

fn build_wall(
    commands: &mut Commands,
    assets: &AssetStore,
    loaded_assets: &LoadedAssets,
    gap_y: i32,
) {
    for y in -12..=12 {
        //(14)
        if y < gap_y - 4 || y > gap_y + 4 {
            spawn_image!(
                assets,
                commands,
                "wall",
                512.0,
                y as f32 * 32.0,
                10.0, // draw above the parallax backgrounds
                &loaded_assets,
                Obstacle,
                FlappyElement,
                Velocity::new_2d(-4.0, 0.0)
            );
            //(15)
        }
    }
}

fn flap(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut AnimationCycle)>,
    mut impulse: EventWriter<Impulse>,
) {
    if keyboard.pressed(KeyCode::Space) {
        if let Ok((flappy, mut animation)) = query.single_mut() {
            impulse.write(Impulse {
                target: flappy,
                amount: Vec3::Y, // Vec3::new(0.0, 1.0, 0.0),
                absolute: false,
                source: 0,
            });
            animation.switch("Flapping");
        }
    }
}

fn clamp(mut query: Query<&mut Transform, With<Flappy>>, mut state: ResMut<NextState<GamePhase>>) {
    if let Ok(mut transform) = query.single_mut() {
        if transform.translation.y > 384.0 {
            transform.translation.y = 384.0; //(21)
        } else if transform.translation.y < -384.0 {
            state.set(GamePhase::GameOver);
        }
    }
}

fn move_walls(
    mut commands: Commands,
    query: Query<&mut Transform, With<Obstacle>>,
    assets: Res<AssetStore>,
    loaded_assets: AssetResource,
    delete: Query<Entity, With<Obstacle>>,
    mut rng: ResMut<RandomNumberGenerator>,
) {
    let mut rebuild = false;
    for transform in query.iter() {
        if transform.translation.x < -530.0 {
            rebuild = true; //(23)
        }
    }
    if rebuild {
        for entity in delete.iter() {
            commands.entity(entity).despawn();
        }
        build_wall(&mut commands, &assets, &loaded_assets, rng.range(-5..5));
    }
}

fn hit_wall(
    player: Query<&Transform, With<Flappy>>,  //(24)
    walls: Query<&Transform, With<Obstacle>>, //(25)
    mut state: ResMut<NextState<GamePhase>>,
    assets: Res<AssetStore>,
    loaded_assets: Res<LoadedAssets>,
    mut commands: Commands,
) {
    if let Ok(player) = player.single() {
        //(26)
        for wall in walls.iter() {
            //(27)
            let distance = player.translation.distance(wall.translation); //(28)
            if distance < 32.0 {
                state.set(GamePhase::GameOver);
                assets.play("crash", &mut commands, &loaded_assets);
            }
        }
    }
}
