// define the components - Data
use bevy_ecs::prelude::Component;
// ECS data for movement must be stored by the structs
// ECS schedule can store all position data together and all velocity data together.

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

// to do Time- 
#[derive(Component)]
struct Time {
    delta_seconds: f32, // Time elapsed since the last frame
}

#[derive(Components)]
struct FixedPhysics;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct MovementSpeed {
    pub base: f32,
    pub sprint_multiplier: f32,
}

#[derive(Component)]
pub struct Mass {
    pub value: f32,
}

#[derive(Component)]
pub struct AffectedByGravity;

// Collision
#[derive(Component)]
pub struct BoundingBox {
    pub min: (f32, f32, f32), // minimum (x,y, z) corner of the box
    pub max: (f32, f32, f32),
}

#[derive(Component)]
pub struct Collidable;

#[derive(Component)]
pub struct Grounded(pub bool); // True if the entity is currently resting on a surface