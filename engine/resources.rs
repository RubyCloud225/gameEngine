// Resources : Global mutable container accessed safely by systems.
use std::collections::HashMap;
use bevy_ecs::prelude::*;




#[derive(Resource)]
pub struct FixedTime {
    // The target duration for one physics tick (1.0 / 60.0 seconds)
    pub const TIMESTEP: f32 = 0.01666667;
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

#[derive(Resource)]
pub struct CollisionPairs {
    // TODO
}

// Spatial partitioning grid for optimizing collision detection
#[derive(Resource)]
pub struct SpatialGrid {
    pub cell_size: f32,
    pub cells: HashMap<(i32, i32, i32), Vec<Entity>>,
}

impl SpatialGrid {
    pub fn clear(&mut self) { self.cells.clear(); }
    pub fn insert_entity(&mut self, Entity, pos: &Position, hit: &Hitbox) {
        let min = (
            (pos.x / self.cell_size). floor() as i32,
            (pos.y / self.cell_size). floor() as i32,
            (pos.z / self.cell_size). floor() as i32,
        );
        let max = (
            ((pos.x + hit.width) / self.cell_size). floor() as i32,
            ((pos.y + hit.height) / self.cell_size). floor() as i32,
            ((pos.z + hit.depth) / self.cell_size). floor() as i32,
        );

        for x in min.0..=max.0 {
            for y in min.1..=max.1 {
                for z in min.2..=max.2 {
                    self.cells.entry((x, y, z)).or_insert_with(Vec::new).push(entity);
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
    pub time_scale: f32,      // 1.0 = normal, 0.0 = frozen
    pub remaining_stop: f32,  // How much longer to stay frozen (in real seconds)
}

impl Default for TimeManager {
    fn default() -> Self {
        Self { time_scale: 1.0, remaining_stop: 0.0 }
    }
}