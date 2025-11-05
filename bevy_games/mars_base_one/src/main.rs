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
struct Player {
    /// Number of miners that ware rescued
    miners_saved: u32,
    /// Current shield level
    shields: i32,
    /// Current fuel level
    fuel: i32,
    /// Current score
    score: u32,
}

/// Component representing the camera tag
#[derive(Component)]
struct MyCamera;

/// Component to tag ground entities
#[derive(Component)]
struct Ground;

/// A component to tag miner entities
#[derive(Component)]
struct Miner;

/// A component to tag shield power-ups
#[derive(Component)]
struct Battery;

/// Component to tag fuel power-ups
#[derive(Component)]
struct Fuel;

/// Event that defines the spawning of new particles
#[derive(Event)]
pub struct SpawnParticle {
    // Pposition of the particle
    position: Vec2,
    /// Color of the Particle
    color: LinearRgba,
    /// Velocity (and therefore direction) of the moving particle
    velocity: Vec3,
}

/// A component that designates a particle entity
#[derive(Component)]
pub struct Particle {
    /// How log does it take for a particle to fade away?
    pub lifetime: f32,
}

/// At the end of the game, this event notifies about the final score
#[derive(Event)]
struct FinalScore(u32);

/// State for the players score after playing the game
#[derive(Default)]
struct ScoreState {
    /// Optional final score, may not be submitted yet
    score: Option<u32>,
    /// Players name
    player_name: String,
    /// Has the score been submitted to the highscore server?
    submitted: bool,
}

/// DTO to submit high-score entries to the server
#[derive(serde::Serialize, serde::Deserialize)]
struct HighScoreEntry {
    /// Players name
    name: String,
    /// Final score
    score: u32,
}

/// DTO holding a table of high-scores
#[derive(serde::Deserialize)]
struct HighScoreTable {
    entries: Vec<HighScoreEntry>,
}

/// Structure to receive a high-score table through a channel
#[derive(Default)]
struct HighScoreTableState {
    entries: Option<HighScoreTable>,
    receiver: Option<std::sync::mpsc::Receiver<HighScoreTable>>,
}

fn main() -> anyhow::Result<()> {
    let mut app = App::new();

    add_phase!(app, GamePhase, GamePhase::Playing,
       start => [ setup ],
       run => [movement, end_game, physics_clock, sum_impulses, apply_gravity, apply_velocity,
        cap_velocity.after(apply_velocity),
        check_collisions::<Player, Ground>, bounce, show_performance, score_display,
        camera_follow.after(cap_velocity),
        spawn_particle_system, particle_age_system, miner_beacon,
        check_collisions::<Player, Miner>,
        check_collisions::<Player, Fuel>,
        check_collisions::<Player, Battery>,
        collect_and_despawn_game_element::<Miner,  { BurstColor:: Green as u8 }>,
        collect_and_despawn_game_element::<Fuel,  { BurstColor:: Orange as u8 }>,
        collect_and_despawn_game_element::<Battery,  { BurstColor::Magenta as u8 }>
        ],
       exit => [submit_score, cleanup::<GameElement>.after(submit_score)]
    );

    add_phase!(app, GamePhase, GamePhase::WorldBuilding,
        start => [ spawn_builder],
        run => [ show_builder ],
        exit => []
    );

    add_phase!(app, GamePhase, GamePhase::GameOver,
        start => [],
        run => [ final_score ],
        exit => []
    );

    app.add_systems(
        Update,
        highscore_table.run_if(in_state(GamePhase::MainMenu)),
    );
    app.add_event::<Impulse>()
        .add_event::<PhysicsTick>()
        .add_event::<OnCollision<Player, Ground>>()
        .add_event::<OnCollision<Player, Miner>>()
        .add_event::<OnCollision<Player, Fuel>>()
        .add_event::<OnCollision<Player, Battery>>()
        .add_event::<SpawnParticle>()
        .add_event::<FinalScore>()
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
                .add_image("mothership", "mothership.png")?
                .add_image("particle", "particle.png")?
                .add_image("spaceman", "spaceman.png")?
                .add_image("fuel", "fuel.png")?
                .add_image("battery", "battery.png")?,
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
        scale: 0.5,
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
        Player {
            miners_saved: 0,
            shields: 500,
            fuel: 100_00,
            score: 0,
        },
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

fn end_game(mut state: ResMut<NextState<GamePhase>>, player_query: Query<&Player>) {
    let Ok(player) = player_query.single() else {
        return;
    };

    if player.miners_saved == 1 {
        state.set(GamePhase::GameOver);
    }
}

fn spawn_particle(
    particles: &mut EventWriter<SpawnParticle>,
    direction: &Dir3,
    transform: &Transform,
) {
    particles.write(SpawnParticle {
        position: direction.truncate()
            + Vec2::new(transform.translation.x, transform.translation.y),
        color: LinearRgba::new(0.0, 1.0, 1.0, 1.0),
        velocity: -direction.as_vec3(),
    });
}

fn movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(Entity, &mut Transform, &mut Player)>,
    mut impulses: EventWriter<Impulse>,
    mut particles: EventWriter<SpawnParticle>,
) {
    let Ok((entity, mut transform, mut player)) = player_query.single_mut() else {
        return;
    };

    if keyboard.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        transform.rotate(Quat::from_rotation_z(f32::to_radians(2.0)));
        spawn_particle(&mut particles, &-transform.local_x(), &transform);
    }
    if keyboard.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        transform.rotate(Quat::from_rotation_z(f32::to_radians(-2.0)));
        spawn_particle(&mut particles, &transform.local_x(), &transform);
    }
    if keyboard.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        if player.fuel > 0 {
            impulses.write(Impulse {
                target: entity,
                amount: transform.local_y().as_vec3(),
                absolute: false,
                source: 1,
            });
            spawn_particle(&mut particles, &transform.local_y(), &transform);
            player.fuel -= 1;
        }
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
    mut player_query: Query<(&PhysicsPosition, &mut Player)>,
    ground_query: Query<&PhysicsPosition, With<Ground>>,
    mut impulses: EventWriter<Impulse>,
    mut particles: EventWriter<SpawnParticle>,
    mut state: ResMut<NextState<GamePhase>>,
) {
    let mut bounce = Vec2::default();
    let mut entity = None;
    let mut bounces = 0;
    for collision in collisions.read() {
        if let Ok((player_pos, _)) = player_query.single_mut() {
            if let Ok(ground) = ground_query.get(collision.entity_b) {
                entity = Some(collision.entity_a);
                let difference = player_pos.start_frame - ground.start_frame;
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

        let Ok((player_pos, mut player)) = player_query.single_mut() else {
            return;
        };
        particle_burst(
            player_pos.end_frame,
            LinearRgba::new(0.0, 0.0, 1.0, 1.0),
            &mut particles,
            3.0,
        );
        player.shields -= 1;
        if player.shields <= 0 {
            state.set(GamePhase::GameOver);
        }
    }
}

fn spawn_particle_system(
    mut commands: Commands,
    mut reader: EventReader<SpawnParticle>,
    assets: Res<AssetStore>,
    loaded_assets: Res<LoadedAssets>,
) {
    for particle in reader.read() {
        let mut sprite = Sprite::from_image(assets.get_handle("particle", &loaded_assets).unwrap());
        sprite.color = particle.color.into();

        commands.spawn((
            sprite,
            Transform::from_xyz(particle.position.x, particle.position.y, 5.0),
            GameElement,
            Particle { lifetime: 2.0 },
            Velocity(particle.velocity),
            PhysicsPosition::new(particle.position),
        ));
    }
}

fn particle_age_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Particle, &mut Sprite)>,
) {
    for (entity, mut particle, mut sprite) in query.iter_mut() {
        particle.lifetime -= time.delta_secs();
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }

        sprite.color.set_alpha(particle.lifetime / 2.0);
    }
}

fn particle_burst(
    center: Vec2,
    color: LinearRgba,
    spawn: &mut EventWriter<SpawnParticle>,
    velocity: f32,
) {
    for angle in 0..360 {
        let angle = (angle as f32).to_radians();
        let velocity = Vec3::new(angle.cos() * velocity, angle.sin() * velocity, 0.0);
        spawn.write(SpawnParticle {
            position: center,
            color,
            velocity,
        });
    }
}

fn miner_beacon(
    mut rng: ResMut<RandomNumberGenerator>,
    miners: Query<&Transform, With<Miner>>,
    mut spawn: EventWriter<SpawnParticle>,
) {
    for miner in miners.iter() {
        if rng.range(0..100) == 0 {
            particle_burst(
                miner.translation.truncate(),
                LinearRgba::new(1.0, 1.0, 0.0, 1.0),
                &mut spawn,
                10.0,
            );
        }
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

        let mut world = World::new(WORLD_SIZE, WORLD_SIZE, &mut rng);

        // Shuffle possible miner positions and limit the size to 20
        use my_library::rand::seq::SliceRandom;
        world.spawn_positions.shuffle(&mut rng.rng);

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

fn score_display(player: Query<&Player>, mut egui_context: egui::EguiContexts) {
    let Ok(player) = player.single() else {
        return;
    };
    egui::egui::Window::new("Score").show(egui_context.ctx_mut(), |ui| {
        ui.label(format!("Score: {}", player.score));
        ui.label(format!("Miners Saved: {}", player.miners_saved));
        ui.label(format!("Shields: {}", player.shields));
        ui.label(format!("Fuel: {}", player.fuel));
    });
}

/// Emits a FinalScore event with the player's score to subsequent game phases
fn submit_score(player: Query<&Player>, mut final_score: EventWriter<FinalScore>) {
    for player in player.iter() {
        final_score.write(FinalScore(player.score));
    }
}

/// Receives FinalScore events and displays the score to the user
fn final_score(
    mut final_score: EventReader<FinalScore>,
    mut state: Local<ScoreState>,
    mut egui_context: egui::EguiContexts,
) {
    // Set the final score to the last received message
    for score in final_score.read() {
        state.score = Some(score.0);
    }
    if state.submitted {
        return;
    }

    // Display the score window with text input element
    if let Some(score) = state.score {
        egui::egui::Window::new("Final Score").show(egui_context.ctx_mut(), |ui| {
            ui.label(format!("Final score: {}", score));
            ui.label("Please enter your name:");
            ui.text_edit_singleline(&mut state.player_name);
            if ui.button("Submit Score").clicked() {
                state.submitted = true;
                let entry = HighScoreEntry {
                    name: state.player_name.clone(),
                    score,
                };
                std::thread::spawn(move || {
                    ureq::post("http://localhost:3030/submit-score")
                        .timeout(std::time::Duration::from_secs(5))
                        .send_json(entry)
                        .expect("Failed to submit score");
                });
            }
        });
    }
}

/// System for retrieving the highscore table from the server
fn highscore_table(mut state: Local<HighScoreTableState>, mut egui_context: egui::EguiContexts) {
    if state.receiver.is_none() {
        // Create the channel
        let (transmitter, receiver) = std::sync::mpsc::channel();
        state.receiver = Some(receiver);

        std::thread::spawn(move || {
            let table = ureq::get("http://localhost:3030/highscores")
                .timeout(std::time::Duration::from_secs(5))
                .call()
                .unwrap()
                .into_json::<HighScoreTable>()
                .unwrap();
            let _ = transmitter.send(table);
        });
    } else {
        // Receive the result
        if let Some(rx) = &state.receiver {
            if let Ok(table) = rx.try_recv() {
                state.entries = Some(table);
            }
        }
    }

    if let Some(table) = &state.entries {
        // Display the table, if received
        egui::egui::Window::new("High Scores").show(egui_context.ctx_mut(), |ui| {
            for entry in table.entries.iter() {
                ui.label(format!("{}: {}", entry.name, entry.score));
            }
        });
    }
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
    /// Positions on which entites can be spawned
    spawn_positions: Vec<(f32, f32)>,
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
            spawn_positions: Vec::new(),
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

        let (mesh, tile_positions, spawn_positions) = result.build_mesh();
        result.mesh = Some(mesh);
        result.tile_positions = tile_positions;
        result.spawn_positions = spawn_positions;

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
        for (x, y) in self.spawn_positions.iter().take(20) {
            spawn_image!(
                assets,
                commands,
                "spaceman",
                *x,
                *y,
                10.0,
                loaded_assets,
                GameElement,
                Miner,
                Velocity::default(),
                PhysicsPosition::new(Vec2::new(*x, *y)),
                AxisAlignedBoundingBox::new(48.0, 48.0)
            );
        }

        for (x, y) in self.spawn_positions.iter().skip(20).take(20) {
            spawn_image!(
                assets,
                commands,
                "fuel",
                *x,
                *y,
                10.0,
                loaded_assets,
                GameElement,
                Fuel,
                Velocity::default(),
                PhysicsPosition::new(Vec2::new(*x, *y)),
                AxisAlignedBoundingBox::new(48.0, 48.0)
            );
        }

        for (x, y) in self.spawn_positions.iter().skip(40).take(20) {
            spawn_image!(
                assets,
                commands,
                "battery",
                *x,
                *y,
                10.0,
                loaded_assets,
                GameElement,
                Battery,
                Velocity::default(),
                PhysicsPosition::new(Vec2::new(*x, *y)),
                AxisAlignedBoundingBox::new(48.0, 48.0)
            );
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

    fn build_mesh(&self) -> (Mesh, Vec<(f32, f32)>, Vec<(f32, f32)>) {
        let mut position = Vec::new();
        let mut uv = Vec::new();
        let mut tile_positions = Vec::new();
        let mut possible_miner_positions = Vec::new();

        let x_offset = self.width as f32 / 2.0 * TILE_SIZE;
        let y_offset = self.height as f32 / 2.0 * TILE_SIZE;

        for y in 0..self.height {
            for x in 0..self.width {
                let left = x as f32 * TILE_SIZE - x_offset;
                let right = (x as f32 + 1.0) * TILE_SIZE - x_offset;
                let top = y as f32 * TILE_SIZE - y_offset;
                let bottom = (y as f32 + 1.0) * TILE_SIZE - y_offset;
                if self.solid[self.map_idx(x, y)] {
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

                    let needs_physics;

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
                } else {
                    if x > 1
                        && x < self.width - 3
                        && y > 1
                        && y < self.height - 3
                        && self.solid[self.map_idx(x, y - 1)]
                    {
                        possible_miner_positions
                            .push((left + TILE_SIZE / 2.0, top + TILE_SIZE / 2.0));
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
            possible_miner_positions,
        )
    }
}

/// A trait for collecting things for the player
trait OnCollect {
    /// The effect of collecting a particular thing
    fn effect(player: &mut Player);
}

impl OnCollect for Miner {
    fn effect(player: &mut Player) {
        player.miners_saved += 1;

        player.score += 1000;
        if player.shields > 0 {
            player.score += player.shields as u32;
        }
        if player.fuel > 1000 {
            player.score += player.fuel as u32;
        }
    }
}

impl OnCollect for Fuel {
    fn effect(player: &mut Player) {
        player.fuel += 1000;
    }
}

impl OnCollect for Battery {
    fn effect(player: &mut Player) {
        player.shields += 100;
    }
}

#[repr(u8)]
enum BurstColor {
    Green,
    Orange,
    Magenta,
}

impl From<u8> for BurstColor {
    fn from(value: u8) -> Self {
        match value {
            0 => BurstColor::Green,
            1 => BurstColor::Orange,
            2 => BurstColor::Magenta,
            _ => panic!("Invalid BurstColor value"),
        }
    }
}

impl Into<LinearRgba> for BurstColor {
    fn into(self) -> LinearRgba {
        match self {
            BurstColor::Green => LinearRgba::new(0.0, 1.0, 0.0, 1.0),
            BurstColor::Orange => LinearRgba::new(1.0, 0.5, 0.0, 1.0),
            BurstColor::Magenta => LinearRgba::new(1.0, 0.0, 1.0, 1.0),
        }
    }
}

fn collect_and_despawn_game_element<T: Component + OnCollect, const COLOR: u8>(
    mut collisions: EventReader<OnCollision<Player, T>>,
    mut commands: Commands,
    mut player: Query<(&mut Player, &Transform)>,
    mut spawn: EventWriter<SpawnParticle>,
) {
    let mut collected = Vec::new();
    for collision in collisions.read() {
        collected.push(collision.entity_b);
    }

    let Ok((mut player, player_pos)) = player.single_mut() else {
        return;
    };
    for miner in collected.iter() {
        if commands.get_entity(*miner).is_ok() {
            commands.entity(*miner).despawn();
        }
        T::effect(&mut player);
    }

    if !collected.is_empty() {
        particle_burst(
            player_pos.translation.truncate(),
            BurstColor::from(COLOR).into(),
            &mut spawn,
            2.0,
        );
    }
}
