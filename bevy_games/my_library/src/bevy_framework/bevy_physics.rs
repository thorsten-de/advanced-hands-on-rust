use bevy::prelude::*;
use bevy_egui::egui::frame;

// How frequently should the physics tick fire (ms)
const PHYSICS_TICK_TIME: u128 = 33;

/// Stores the time between frames
#[derive(Default)]
pub struct PhysicsTimer(u128);

/// Event fired for each tick
#[derive(Event)]
pub struct PhysicsTick;

/// System that keeps track of the time and emits PhysicsTick events
pub fn physics_clock(
    mut clock: Local<PhysicsTimer>,
    time: Res<Time>,
    mut on_tick: EventWriter<PhysicsTick>,
    mut physics_position: Query<(&mut PhysicsPosition, &mut Transform)>,
) {
    let ms_since_last_call = time.delta().as_millis();
    clock.0 += ms_since_last_call;
    if clock.0 >= PHYSICS_TICK_TIME {
        clock.0 = 0;
        physics_position
            .iter_mut()
            .for_each(|(mut pos, mut transform)| {
                transform.translation.x = pos.end_frame.x;
                transform.translation.y = pos.end_frame.y;
                pos.start_frame = pos.end_frame;
            });
        on_tick.write(PhysicsTick);
    } else {
        let frame_progress = clock.0 as f32 / PHYSICS_TICK_TIME as f32;
        physics_position
            .iter_mut()
            .for_each(|(pos, mut transform)| {
                let interpolated_pos = pos.interpolate(frame_progress);
                transform.translation.x = interpolated_pos.x;
                transform.translation.y = interpolated_pos.y;
            });
    }
}

/// Component to track movement over time as Velocity
#[derive(Component)]
pub struct Velocity(pub Vec3);

impl Default for Velocity {
    fn default() -> Self {
        Self(Vec3::ZERO)
    }
}

impl Velocity {
    /// Creates a new 3-dimensional velocity
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3 { x, y, z })
    }

    /// Creates a new 2-dimensional velocity (third dimension set to 0)
    pub fn new_2d(x: f32, y: f32) -> Self {
        Self::new(x, y, 0.0)
    }
}

/// Apply an impulse to a physics-based component, affecting its velocity
#[derive(Event)]
pub struct Impulse {
    /// The entity the impulse should effect
    pub target: Entity,
    /// the velocity adjustment to be applied
    pub amount: Vec3,
    /// override the velocity instead of applying an impulse. E.G. bouncing of a wall
    pub absolute: bool,
    /// Indicates the event source, used for deduplication of events within a physics tick
    pub source: i32,
}

/// System for calculating total forces applyed to an enitity within a physics tick
pub fn sum_impulses(mut impulses: EventReader<Impulse>, mut velocities: Query<&mut Velocity>) {
    let mut dedupe_by_source = std::collections::HashMap::new();
    for impulse in impulses.read() {
        dedupe_by_source.insert(impulse.source, impulse);
    }
    let mut absolute = std::collections::HashSet::new();
    for (_, impulse) in dedupe_by_source {
        if let Ok(mut velocity) = velocities.get_mut(impulse.target) {
            if absolute.contains(&impulse.target) {
                continue;
            }
            if impulse.absolute {
                velocity.0 = impulse.amount;
                absolute.insert(impulse.target);
            } else {
                velocity.0 += impulse.amount;
            }
        }
    }
}

/// System that applies the calculated velocities to the transforms on
/// each tick of the physics clock
pub fn apply_velocity(
    mut tick: EventReader<PhysicsTick>,
    mut movement: Query<(&Velocity, &mut PhysicsPosition)>,
) {
    for _tick in tick.read() {
        movement
            .iter_mut()
            .for_each(|(velocity, mut position)| position.end_frame += velocity.0.truncate());
    }
}

/// This component is defined on entities that are subject to gravity.
/// In the example flappy the dragon is, but obstacles are not.
#[derive(Component)]
pub struct ApplyGravity;

/// System to apply gravity on marked entities for every tick
/// of the physics clock.
pub fn apply_gravity(
    mut tick: EventReader<PhysicsTick>,
    mut gravity: Query<&mut Velocity, With<ApplyGravity>>,
) {
    for _tick in tick.read() {
        gravity.iter_mut().for_each(|mut velocity| {
            velocity.0.y -= 0.75;
        });
    }
}

/// Collate the start and end frame positions of a physics entity
#[derive(Component)]
pub struct PhysicsPosition {
    /// The position at the start of the fixed time frame
    pub start_frame: Vec2,
    /// The position at the end of the fixed time frame
    pub end_frame: Vec2,
}

impl PhysicsPosition {
    /// Create a new physics position without any forces applied
    /// in this frame yet
    pub fn new(start: Vec2) -> Self {
        Self {
            start_frame: start,
            end_frame: start,
        }
    }

    fn interpolate(&self, t: f32) -> Vec2 {
        self.start_frame + (self.end_frame - self.start_frame) * t
    }
}
