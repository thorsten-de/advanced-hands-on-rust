use std::os::unix::raw::off_t;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::asset::RenderAssetUsages;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::render::mesh::PrimitiveTopology;
use my_library::egui::egui::Color32;
use my_library::*;

/// Game Phases for Mars Base One
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
enum GamePhase {
    #[default]
    Loading,
    MainMenu,
    WorldBuilding,
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
        cap_velocity.after(apply_velocity),
        check_collisions::<Player, Ground>, bounce, show_performance,
        camera_follow.after(cap_velocity)],
       exit => [cleanup::<GameElement>]
    );

    add_phase!(app, GamePhase, GamePhase::WorldBuilding,
        start => [ spawn_builder],
        run => [ show_builder ],
        exit => []
    );

    app.add_event::<Impulse>()
        .add_event::<PhysicsTick>()
        .add_event::<OnCollision<Player, Ground>>()
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
            GamePhase::WorldBuilding,
            GamePhase::GameOver,
        ))
        .add_plugins(
            AssetManager::new()
                .add_image("ship", "ship.png")?
                .add_image("ground", "ground.png")?
                .add_image("backdrop", "backdrop.png")?
                .add_image("mothership", "mothership.png")?,
        )
        .add_plugins(FrameTimeDiagnosticsPlugin { ..default() })
        .insert_resource(Animations::new())
        .run();

    Ok(())
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetStore>,
    loaded_assets: Res<LoadedAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let camera = Camera2d::default();
    // This determines the transformation from world-coordinates to screen-coordinates.
    // A camera defines how the viewport is rendered to show the world. Technically, this
    // is done with a *projection matrix*.
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::WindowSize,
        scale: 1.0,
        ..OrthographicProjection::default_2d()
    });
    commands.spawn((camera, projection, GameElement, MyCamera));

    let top = WORLD_SIZE as f32 / 2.0 * TILE_SIZE;

    spawn_image!(
        assets,
        commands,
        "ship",
        0.0,
        200.0 + top,
        10.0,
        &loaded_assets,
        GameElement,
        Player,
        Velocity::default(),
        PhysicsPosition::new(Vec2::new(0.0, 200.0 + top)),
        ApplyGravity,
        AxisAlignedBoundingBox::new(24.0, 24.0)
    );

    spawn_image!(
        assets,
        commands,
        "mothership",
        0.0,
        400.0 + top,
        10.0,
        &loaded_assets,
        GameElement
    );

    let x_scale = WORLD_SIZE as f32 * TILE_SIZE / 1792.0;
    let y_scale = (WORLD_SIZE as f32 + TOP_MARGIN) * TILE_SIZE / 1024.0;

    let center_x = 0.0; // as f32 * TILE_SIZE - WORLD_SIZE as f32 / 2.0 * TILE_SIZE;
    let center_y = TOP_MARGIN / 2.0 * TILE_SIZE; // as f32 * TILE_SIZE - WORLD_SIZE as f32 / 2.0 * TILE_SIZE;

    let mut transform = Transform::from_xyz(center_x, center_y, -10.0);
    transform.scale = Vec3::new(x_scale, y_scale, 1.0);
    commands
        .spawn(Sprite::from_image(
            assets.get_handle("backdrop", &loaded_assets).unwrap(),
        ))
        .insert(transform)
        .insert(GameElement);

    let mut lock = NEW_WORD.lock().unwrap();
    let world = lock.take().unwrap();
    world.spawn(
        &assets,
        &mut commands,
        &loaded_assets,
        &mut meshes,
        &mut materials,
    );
    commands.insert_resource(StaticQuadTree::new(Vec2::new(10240.0, 7680.0), 6));
}

fn end_game(
    mut state: ResMut<NextState<GamePhase>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(transform) = player_query.single() else {
        return;
    };
    if false
        && (transform.translation.y < -384.0
            || transform.translation.y > 384.0
            || transform.translation.x < -512.0
            || transform.translation.x > 512.0)
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

fn bounce(
    mut collisions: EventReader<OnCollision<Player, Ground>>,
    mut player_query: Query<&PhysicsPosition, With<Player>>,
    ground_query: Query<&PhysicsPosition, With<Ground>>,
    mut impulses: EventWriter<Impulse>,
) {
    let mut bounce = Vec2::default();
    let mut entity = None;
    let mut bounces = 0;
    for collision in collisions.read() {
        if let Ok(player) = player_query.single_mut() {
            if let Ok(ground) = ground_query.get(collision.entity_b) {
                entity = Some(collision.entity_a);
                let difference = player.start_frame - ground.start_frame;
                bounces += 1;
                bounce += difference;
            }
        }
    }
    if bounces > 0 {
        bounce = bounce.normalize();
        impulses.write(Impulse {
            target: entity.unwrap(),
            amount: Vec3::new(bounce.x, bounce.y, 0.0),
            absolute: true,
            source: 2,
        });
    }
}

const WORLD_SIZE: usize = 200;
const TOP_MARGIN: f32 = 60.0;

fn spawn_builder() {
    use std::sync::atomic::Ordering;

    WORLD_READY.store(false, Ordering::Relaxed);

    // Start a new world-building thread. This thread runs outside of
    // Bevy's systems, and has no access to Bevy's DI container
    std::thread::spawn(|| {
        // Give the thread its own rng. So no unsafe reference must be hold
        // between frames
        let mut rng = my_library::RandomNumberGenerator::new();
        // Spawn the world
        info!("Start building the world.");

        let world = World::new(WORLD_SIZE, WORLD_SIZE, &mut rng);

        // Swap the world getting exclusive access to its mutex
        let mut lock = NEW_WORD.lock().unwrap();
        *lock = Some(world);

        info!("Finished building the world.");

        // Notify of successful finished generation
        WORLD_READY.store(true, Ordering::Relaxed);
    });
}

fn show_builder(mut state: ResMut<NextState<GamePhase>>, mut egui_context: egui::EguiContexts) {
    egui::egui::Window::new("Performance").show(egui_context.ctx_mut(), |ui| {
        ui.label("Building World");
    });
    if WORLD_READY.load(Ordering::Relaxed) {
        state.set(GamePhase::Playing);
    }
}

fn show_performance(
    mut egui_context: egui::EguiContexts,
    diagnostics: Res<DiagnosticsStore>, // get bevys diagnostic informations as a resource from DI
    mut commands: Commands,
) {
    let fps = diagnostics // get diagnostical information about the average fps of recent frames
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.average())
        .unwrap_or(0.0);

    egui::egui::Window::new("Performance").show(egui_context.ctx_mut(), |ui| {
        let fps_text = format!("FPS: {fps:.1}"); // format fps with one decimal place
        let color = match fps as u32 {
            // color scale for fps ranges
            0..=29 => Color32::RED,
            30..=50 => Color32::GOLD,
            _ => Color32::GREEN,
        };
        ui.colored_label(color, &fps_text);
    });
}

/// Defines the world by a 2d-matrix of tiles.
struct World {
    /// If a tile is a solid wall for each given index
    solid: Vec<bool>,
    /// Horizontal map size
    width: usize,
    /// Vertical map size
    height: usize,
    /// The mesh representing each tile
    mesh: Option<Mesh>,
    /// The position of each tile
    tile_positions: Vec<(f32, f32)>,
}

const TILE_SIZE: f32 = 24.0;
const SOLID_PERCENT: f32 = 0.6;

static WORLD_READY: AtomicBool = AtomicBool::new(false);
static NEW_WORD: Mutex<Option<World>> = Mutex::new(None);

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
            mesh: None,
            tile_positions: Vec::new(),
        };

        result.clear_tiles(width / 2, height / 2);

        let mut holes = vec![(width / 2, height / 2)];

        for _ in 0..10 {
            let x = rng.range(5..width - 5);
            let y = rng.range(5..height - 5);
            holes.push((x, y));
            result.clear_tiles(x, y);
            result.clear_tiles(x + 2, y);
            result.clear_tiles(x - 2, y);
            result.clear_tiles(x, y + 2);
            result.clear_tiles(x, y - 2);
        }
        for i in 0..holes.len() {
            let start = holes[i];
            let end = holes[(i + 1) % holes.len()];
            result.clear_line(start, end);
        }

        for y in height / 2..height {
            result.clear_tiles(width / 2, y);
        }

        result.outward_diffusion(&holes, rng);

        let (mesh, tile_positions) = result.build_mesh();
        result.mesh = Some(mesh);
        result.tile_positions = tile_positions;

        result
    }

    fn find_random_closed_tile(&self, rng: &mut RandomNumberGenerator) -> (usize, usize) {
        loop {
            let x = rng.range(0..self.width);
            let y = rng.range(0..self.height);
            let idx = self.map_idx(x, y);
            if self.solid[idx] {
                return (x, y);
            }
        }
    }

    fn outward_diffusion(&mut self, holes: &Vec<(usize, usize)>, rng: &mut RandomNumberGenerator) {
        let mut done = false;
        while !done {
            let start_tile = holes[rng.range(0..10)];
            let target = self.find_random_closed_tile(rng);

            let (mut x, mut y) = (start_tile.0 as f32, start_tile.1 as f32);
            let (slope_x, slope_y) = (
                (target.0 as f32 - x) / self.width as f32,
                (target.1 as f32 - y) / self.height as f32,
            );
            loop {
                if x < 1.0 || x >= self.width as f32 || y < 1.0 || y >= self.height as f32 {
                    break;
                }
                let tile_id = self.map_idx(x as usize, y as usize);
                if self.solid[tile_id] {
                    self.clear_tiles(x as usize, y as usize);
                    break;
                }
                x += slope_x;
                y += slope_y;
            }

            let solid_count = self.solid.iter().filter(|s| **s).count();
            let solid_percent = solid_count as f32 / (self.width * self.height) as f32;
            if solid_percent < SOLID_PERCENT {
                done = true;
            }
        }
    }

    /// Spawns the world into the game
    fn spawn(
        &self,
        assets: &AssetStore,
        commands: &mut Commands,
        loaded_assets: &LoadedAssets,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) {
        let mesh = self.mesh.as_ref().unwrap().clone();
        let mesh_handle = meshes.add(mesh);
        let material_handle = materials.add(ColorMaterial {
            texture: Some(assets.get_handle("ground", loaded_assets).unwrap()),
            ..default()
        });
        commands
            .spawn(Mesh2d(mesh_handle))
            .insert(MeshMaterial2d(material_handle))
            .insert(Transform::from_xyz(0.0, 0.0, 0.0));

        for (x, y) in self.tile_positions.iter() {
            commands
                .spawn_empty()
                .insert(GameElement)
                .insert(Ground)
                .insert(PhysicsPosition::new(Vec2::new(*x, *y)))
                .insert(AxisAlignedBoundingBox::new(TILE_SIZE, TILE_SIZE));
        }
    }

    fn clear_tiles(&mut self, x: usize, y: usize) {
        for offset_x in -1..=1 {
            for offset_y in -1..=1 {
                let x = x as isize + offset_x;
                let y = y as isize + offset_y;

                // The checks ensure that there will always be a solid one-cell border around the map
                if 0 < x && x < self.width as isize - 1 && 0 < y && y < self.height as isize {
                    let idx = self.map_idx(x as usize, y as usize);
                    self.solid[idx] = false;
                }
            }
        }
    }

    fn clear_line(&mut self, start: (usize, usize), end: (usize, usize)) {
        let (mut x, mut y) = (start.0 as f32, start.1 as f32);
        let (slope_x, slope_y) = (
            (end.0 as f32 - x) / self.width as f32,
            (end.1 as f32 - y) / self.height as f32,
        );
        loop {
            let (tx, ty) = (x as usize, y as usize);
            if tx < 1 || tx >= self.width || ty < 1 || ty >= self.height {
                break;
            }
            if tx == end.0 && ty == end.1 {
                break;
            }
            self.clear_tiles(x as usize, y as usize);
            x += slope_x;
            y += slope_y;
        }
    }

    fn build_mesh(&self) -> (Mesh, Vec<(f32, f32)>) {
        let mut position = Vec::new();
        let mut uv = Vec::new();
        let mut tile_positions = Vec::new();

        let x_offset = self.width as f32 / 2.0 * TILE_SIZE;
        let y_offset = self.height as f32 / 2.0 * TILE_SIZE;

        for y in 0..self.height {
            for x in 0..self.width {
                if self.solid[self.map_idx(x, y)] {
                    let left = x as f32 * TILE_SIZE - x_offset;
                    let right = (x as f32 + 1.0) * TILE_SIZE - x_offset;
                    let top = y as f32 * TILE_SIZE - y_offset;
                    let bottom = (y as f32 + 1.0) * TILE_SIZE - y_offset;

                    position.push([left, bottom, 1.0]);
                    position.push([right, bottom, 1.0]);
                    position.push([right, top, 1.0]);
                    position.push([right, top, 1.0]);
                    position.push([left, bottom, 1.0]);
                    position.push([left, top, 1.0]);

                    uv.push([0.0, 1.0]);
                    uv.push([1.0, 1.0]);
                    uv.push([1.0, 0.0]);
                    uv.push([1.0, 0.0]);
                    uv.push([0.0, 1.0]);
                    uv.push([0.0, 0.0]);

                    let mut needs_physics = false;

                    // Only enable physics on tiles that are on the edge or not
                    // completely surronded by solid tiles

                    if x == 0 || x > self.width - 3 || y == 0 || y > self.height - 3 {
                        needs_physics = true;
                    } else {
                        let solid_count = self.solid[self.map_idx(x - 1, y)] as u8
                            + self.solid[self.map_idx(x + 1, y)] as u8
                            + self.solid[self.map_idx(x, y - 1)] as u8
                            + self.solid[self.map_idx(x, y + 1)] as u8;

                        needs_physics = solid_count < 4;
                    }

                    if needs_physics {
                        tile_positions.push((left + TILE_SIZE / 2.0, top + TILE_SIZE / 2.0));
                    }
                }
            }
        }

        info!("{} tiles need physics", tile_positions.len());

        (
            Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, position)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv),
            tile_positions,
        )
    }
}
