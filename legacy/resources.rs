// Resources : Global mutable container accessed safely by systems.
use crate::components::{AnimationState, DamageRequest, Hitbox, Position};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct FixedTime {
    // The target duration for one physics tick (1.0 / 60.0 seconds)

    // Time accumulated from variable delta_time
    pub accumulator: f32,
}

#[derive(Resource)]
pub struct RealTime {
    pub delta_seconds: f32,
}

#[derive(Resource)]
pub struct InputState {
    // A vector normalised to 1.0 in the direction of intended movement
    pub movement_direction: (f32, f32),
    pub is_sprinting: bool,
}

//Gravitational Acceleration (negative since y is up in games)
#[derive(Resource)]
pub struct PhysicsSettings {
    //gravitational acceleration
    pub gravity_acceleration: f32,
    // TODO damping, air density, motion
}

#[derive(Resource, Default)]
pub struct CollisionPairs {
    // Stores pairs of entities that are currently colliding
    pub pairs: Vec<(Entity, Entity)>,
}

// Spatial partitioning grid for optimizing collision detection
#[derive(Resource)]
pub struct SpatialGrid {
    pub cell_size: f32,
    pub cells: HashMap<(i32, i32, i32), Vec<Entity>>,
}

impl SpatialGrid {
    pub fn clear(&mut self) {
        self.cells.clear();
    }
    pub fn insert_entity(&mut self, entity: Entity, pos: &Position, hit: &Hitbox) {
        assert!(self.cell_size.is_finite() && self.cell_size > 0.0);
        let min = (
            ((pos.x - hit.width / 2.0) / self.cell_size).floor() as i32,
            ((pos.y - hit.height / 2.0) / self.cell_size).floor() as i32,
            ((pos.z - hit.depth / 2.0) / self.cell_size).floor() as i32,
        );
        let max = (
            ((pos.x + hit.width / 2.0) / self.cell_size).floor() as i32,
            ((pos.y + hit.height / 2.0) / self.cell_size).floor() as i32,
            ((pos.z + hit.depth / 2.0) / self.cell_size).floor() as i32,
        );

        for x in min.0..=max.0 {
            for y in min.1..=max.1 {
                for z in min.2..=max.2 {
                    self.cells.entry((x, y, z)).or_default().push(entity);
                }
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct CameraTrauma {
    pub value: f32, // 0.0 to 1.0
}

#[derive(Resource)]
pub struct TimeManager {
    pub time_scale: f32,     // 1.0 = normal, 0.0 = frozen
    pub remaining_stop: f32, // How much longer to stay frozen (in real seconds)
}

impl Default for TimeManager {
    fn default() -> Self {
        Self {
            time_scale: 1.0,
            remaining_stop: 0.0,
        }
    }
}
impl FixedTime {
    pub const TIMESTEP: f32 = 1.0 / 60.0;
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self {
            cell_size: 2.0,
            cells: HashMap::new(),
        }
    }
}

#[derive(Resource, Default)]
pub struct DamageRequests {
    pub events: Vec<DamageRequest>,
}

pub struct AnimationBlendTask {
    pub entity: Entity,
    pub target_state: AnimationState,
    pub blend_time: f32,
}

/// Pending blend requests for an eventual animation backend. Rebuilt each tick.
#[derive(Resource, Default)]
pub struct AnimationTaskGraph {
    pub tasks: Vec<AnimationBlendTask>,
}
