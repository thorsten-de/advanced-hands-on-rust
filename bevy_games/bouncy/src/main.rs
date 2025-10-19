use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

use my_library::{
    egui::egui::{Color32, emath::inverse_lerp},
    *,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
pub enum GamePhase {
    #[default]
    Loading,
    MainMenu,
    Bouncing,
    GameOver,
}

#[derive(Component)]
struct BouncyElement;

#[derive(Component)]
struct Ball;

#[derive(Resource, Default)]
struct CollisionTime {
    time: u128,
    checks: u32,
    fps: f64,
}

fn main() -> anyhow::Result<()> {
    let mut app = App::new();
    add_phase!(app, GamePhase, GamePhase::Bouncing,
      start => [ setup ],
      run => [ warp_at_edge, collisions, show_performance,
        continual_parallax, physics_clock, sum_impulses, apply_velocity ],
      exit => [ cleanup::<BouncyElement> ]
    );

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Naieve Collision".to_string(),
            resolution: bevy::window::WindowResolution::new(1024.0, 768.0),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(FrameTimeDiagnosticsPlugin { ..default() })
    .add_event::<Impulse>()
    .add_event::<PhysicsTick>()
    .add_plugins(GameStatePlugin::new(
        GamePhase::MainMenu,
        GamePhase::Bouncing,
        GamePhase::GameOver,
    ))
    .add_plugins(RandomPlugin)
    .add_plugins(AssetManager::new().add_image("green_ball", "green_ball.png")?)
    .run();

    Ok(())
}

fn spawn_bouncies(
    to_spawn: usize,
    commands: &mut Commands,
    rng: &mut ResMut<RandomNumberGenerator>,
    assets: &AssetStore,
    loaded_assets: &LoadedAssets,
) {
    for _ in 0..to_spawn {
        let position = Vec3::new(rng.range(-512.0..512.0), rng.range(-384.0..384.0), 0.0);
        let velocity = Vec3::new(rng.range(-1.0..1.0), rng.range(-1.0..1.0), 0.0);
        spawn_image!(
            assets,
            commands,
            "green_ball",
            position.x,
            position.y,
            position.z,
            &loaded_assets,
            BouncyElement,
            Velocity::new(velocity.x, velocity.y, velocity.z),
            AxisAlignedBoundingBox::new(8.0, 8.0),
            Ball
        );
    }
}

fn setup(
    mut commands: Commands,
    mut rng: ResMut<RandomNumberGenerator>,
    assets: Res<AssetStore>,
    loaded_assets: Res<LoadedAssets>,
) {
    commands.spawn(Camera2d::default()).insert(BouncyElement);
    commands.insert_resource(CollisionTime::default());
    commands.insert_resource(StaticQuadTree::new(
        Vec2::new(1024.0, 768.0),
        QUAD_TREE_DEPTH,
    ));
    spawn_bouncies(1, &mut commands, &mut rng, &assets, &loaded_assets);
}

fn warp_at_edge(mut query: Query<&mut Transform, With<Ball>>) {
    for mut transform in query.iter_mut() {
        let pos = &mut transform.translation;
        if pos.x < -512.0 {
            pos.x = 512.0;
        } else if pos.x > 512.0 {
            pos.x = -512.0;
        }

        if pos.y < -384.0 {
            pos.y = 384.0;
        } else if pos.y > 384.0 {
            pos.y = -384.0;
        }
    }
}

fn show_performance(
    mut egui_context: egui::EguiContexts,
    diagnostics: Res<DiagnosticsStore>, // get bevys diagnostic informations as a resource from DI
    mut collision_time: ResMut<CollisionTime>,
    mut commands: Commands,
    mut rng: ResMut<RandomNumberGenerator>,
    assets: Res<AssetStore>,
    query: Query<&Transform, With<Ball>>,
    loaded_assets: Res<LoadedAssets>,
) {
    let n_balls = query.iter().count(); // count the number of balls currently simulated
    let fps = diagnostics // get diagnostical information about the average fps of recent frames
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.average())
        .unwrap();
    collision_time.fps = fps;
    egui::egui::Window::new("Performance").show(egui_context.ctx_mut(), |ui| {
        let fps_text = format!("FPS: {fps:.1}"); // format fps with one decimal place
        let color = match fps as u32 {
            // color scale for fps ranges
            0..=29 => Color32::RED,
            30..=59 => Color32::GOLD,
            _ => Color32::GREEN,
        };
        ui.colored_label(color, &fps_text);
        ui.colored_label(
            color,
            &format!("Collision Time: {} ms", collision_time.time),
        );
        ui.label(&format!("Collision Checks: {}", collision_time.checks));
        ui.label(&format!("# Balls: {n_balls}"));
        if ui.button("Add Ball").clicked() {
            // add buttons to user interface for adding more balls
            println!(
                "{n_balls}, {}, {}, {:.0}",
                collision_time.time, collision_time.checks, collision_time.fps
            );
            spawn_bouncies(1, &mut commands, &mut rng, &assets, &loaded_assets);
        }
        if ui.button("Add 100 Balls").clicked() {
            println!(
                "{n_balls}, {}, {}, {:.0}",
                collision_time.time, collision_time.checks, collision_time.fps
            );
            spawn_bouncies(100, &mut commands, &mut rng, &assets, &loaded_assets);
        }
        if ui.button("Add 1000 Balls").clicked() {
            println!(
                "{n_balls}, {}, {}, {:.0}",
                collision_time.time, collision_time.checks, collision_time.fps
            );
            spawn_bouncies(1000, &mut commands, &mut rng, &assets, &loaded_assets);
        }
    });
}

fn bounce_on_collision(
    entity: Entity,
    ball_a: Vec3,
    ball_b: Vec3,
    impulse: &mut EventWriter<Impulse>,
) {
    let a_to_b = (ball_a - ball_b).normalize(); // get the direction, normalized to length 1
    impulse.write(Impulse {
        target: entity,
        amount: a_to_b / 8.0, // apply a force based on the direction
        absolute: false,
        source: 0,
    });
}

fn collisions_AABB(
    mut collision_time: ResMut<CollisionTime>,
    query: Query<(Entity, &Transform, &AxisAlignedBoundingBox)>,
    mut impulse: EventWriter<Impulse>,
    quad_tree: Res<StaticQuadTree>,
) {
    // Start the clock
    let now = std::time::Instant::now();

    // AABB Collision
    let mut n = 0;

    for (entity_a, ball_a, box_a) in query.iter() {
        let box_a = box_a.as_rect(ball_a.translation.truncate());

        for (entity_b, ball_b, box_b) in query.iter() {
            if entity_a != entity_b {
                let box_b = box_b.as_rect(ball_b.translation.truncate());
                if box_a.intersect(&box_b) {
                    bounce_on_collision(
                        entity_a,
                        ball_a.translation,
                        ball_b.translation,
                        &mut impulse,
                    );
                }
                n += 1;
            }
        }
    }

    // Store the time result
    collision_time.time = now.elapsed().as_millis();
    collision_time.checks = n;
}

fn collisions(
    mut collision_time: ResMut<CollisionTime>,
    query: Query<(Entity, &Transform, &AxisAlignedBoundingBox)>,
    mut impulse: EventWriter<Impulse>,
    quad_tree: Res<StaticQuadTree>,
) {
    // Start the clock
    let now = std::time::Instant::now();

    let mut spatial_index: HashMap<usize, Vec<(Entity, Rect2D)>> = HashMap::new();

    let tree_positions: Vec<(Entity, usize, Rect2D)> = query
        .iter()
        .map(|(entity, transform, bbox)| {
            let bbox = bbox.as_rect(transform.translation.truncate());
            let node = quad_tree.smallest_node(&bbox);
            for in_node in quad_tree.intersecting_nodes(&bbox) {
                if let Some(contents) = spatial_index.get_mut(&in_node) {
                    contents.push((entity, bbox));
                } else {
                    spatial_index.insert(in_node, vec![(entity, bbox)]);
                }
            }

            (entity, node, bbox)
        })
        .collect();

    // AABB with spatial quadrants
    let mut n = 0;

    for (entity, node, box_a) in tree_positions {
        if let Some(entities_here) = spatial_index.get(&node) {
            if let Some((entity_b, _)) = entities_here
                .iter()
                .filter(|(entity_b, _)| *entity_b != entity)
                .find(|(_, box_b)| {
                    n += 1;
                    box_a.intersect(box_b)
                })
            {
                let (_, ball_a, _) = query.get(entity).unwrap();
                let (_, ball_b, _) = query.get(*entity_b).unwrap();
                bounce_on_collision(entity, ball_a.translation, ball_b.translation, &mut impulse);
            }
        }
    }

    // Store the time result
    collision_time.time = now.elapsed().as_millis();
    collision_time.checks = n;
}

/// Defines an axis-aligned bounding box (AABB) for
/// collision detection of an entity
#[derive(Component)]
struct AxisAlignedBoundingBox {
    /// Stores the size of the parent entity,
    /// wherever position it is rendered in space
    half_size: Vec2,
}

/// Represents the TopLeft and BottomRight corner of a rectangle
#[derive(Debug, Clone, Copy)]
struct Rect2D {
    min: Vec2,
    max: Vec2,
}

impl AxisAlignedBoundingBox {
    /// Creates a new AABB
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            half_size: Vec2::new(width / 2.0, height / 2.0),
        }
    }

    /// Converts an AABB with a position into a Rect2D
    fn as_rect(&self, translate: Vec2) -> Rect2D {
        Rect2D::new(translate - &self.half_size, translate + &self.half_size)
    }
}

impl Rect2D {
    /// Creates a new Rect2D
    fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    fn intersect(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    fn quadrants(&self) -> Vec<Self> {
        let center = self.center();
        vec![
            // top-left
            Self::new(self.min, center),
            // top-right
            Self::new(
                Vec2::new(center.x, self.min.y),
                Vec2::new(self.max.x, center.y),
            ),
            // bottom-left
            Self::new(
                Vec2::new(self.min.x, center.y),
                Vec2::new(center.x, self.max.y),
            ),
            // bottom-right
            Self::new(center, self.max),
        ]
    }
}

const QUAD_TREE_DEPTH: usize = 4;

#[derive(Debug)]
pub struct StaticQuadTreeNode {
    /// Size of this node
    bounds: Rect2D,
    /// Children quadrants of this node. None if this is
    /// on the maximal depth level
    children: Option<[usize; 4]>,
}

/// Resource for nodes in a quad-tree collision detection algorithm
#[derive(Debug, Resource)]
pub struct StaticQuadTree {
    nodes: Vec<StaticQuadTreeNode>,
}

impl StaticQuadTree {
    fn new(screen_size: Vec2, max_depth: usize) -> Self {
        let mut nodes = Vec::new();

        let half = screen_size / 2.0;
        let top = StaticQuadTreeNode {
            bounds: Rect2D::new(Vec2::ZERO - half, half),
            children: None,
        };
        nodes.push(top);
        Self::subdivide(&mut nodes, 0, 1, max_depth);
        Self { nodes }
    }

    fn subdivide(
        nodes: &mut Vec<StaticQuadTreeNode>,
        index: usize,
        depth: usize,
        max_depth: usize,
    ) {
        let mut children = nodes[index].bounds.quadrants();
        let n = nodes.len();
        let child_index = [n, n + 1, n + 2, n + 3];
        nodes[index].children = Some(child_index);
        children.drain(0..4).for_each(|quad| {
            nodes.push(StaticQuadTreeNode {
                bounds: quad,
                children: None,
            })
        });

        if depth < max_depth {
            for index in child_index {
                Self::subdivide(nodes, index, depth + 1, max_depth);
            }
        }
    }

    /// Finds the smallest quadrant that completely contains an entity
    fn smallest_node(&self, target: &Rect2D) -> usize {
        let mut current_index = 0;

        #[allow(clippy::while_let_loop)]
        loop {
            if let Some(children) = self.nodes[current_index].children {
                let matches: Vec<usize> = children
                    .iter()
                    .filter_map(|child| {
                        if self.nodes[*child].bounds.intersect(target) {
                            Some(*child)
                        } else {
                            None
                        }
                    })
                    .collect();

                if matches.len() == 1 {
                    // The target is contained in only one quadrant, so dive deeper
                    // within that quadrant
                    current_index = matches[0];
                } else {
                    // If there is more than one match, the target is contained in
                    // more than one quadrants, thus overlapping a boundary. We
                    // cannot get deeper than we are.
                    break;
                }
            } else {
                // There are no children quadrants defined at this level, so
                // don't dive deeper.
                break;
            }
        }
        current_index
    }

    fn intersecting_nodes(&self, target: &Rect2D) -> HashSet<usize> {
        let mut result = HashSet::new();
        self.intersect(0, &mut result, target);
        result
    }

    fn intersect(&self, index: usize, result: &mut HashSet<usize>, target: &Rect2D) {
        if self.nodes[index].bounds.intersect(target) {
            result.insert(index);
            if let Some(children) = &self.nodes[index].children {
                for child in children {
                    self.intersect(*child, result, target);
                }
            }
        }
    }
}
