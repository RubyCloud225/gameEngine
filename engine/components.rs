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

#[derive(Component)]
struct Acceleration {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Component)]
struct Hitbox {
    width: f32,
    height: f32,
    depth: f32,
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
pub struct Grounded(pub is_grounded: bool, pub platform: Option<Entity>); // True if the entity is currently resting on a surface

pub const MAX_STEP_HEIGHT: f32 = 0.3; // Maximum height the player can step over
pub const EPSILON: f32 = 0.001: // Small value to prevent floating-point precision issues

#[derive(Component)]
pub struct StepConfig {
    pub max_height: f32,
}

impl Default for StepConfig {
    fn default() -> Self {
        StepConfig {
            max_height: MAX_STEP_HEIGHT,
        }
    }
}
#[derive(Component)]
pub struct Friction {
    pub coefficient: f32,
}
#[derive(Component)]
pub struct Trigger;
#[derive(Component, Default)]
pub struct TriggerTracker {
    pub current_frame: HashSet<Entity>,
    pub last_frame: HashSet<Entity>,
}
#[derive(Component)]
pub struct OnTriggerEnter {
    pub target_entity: Entity,
}

#[derive(Component)]
pub struct OnTriggerExit {
    pub target_entity: Entity,
}
#[derive(Component)]
pub struct OnTriggerStay {
    pub target_entity: Entity,
}

#[derive(Component)]
pub struct CollisionGroups {
    pub memberships: u32, // current layers this entity belongs to
    pub filters: u32,     // layers this entity collides with
}

impl CollisionGroups {
    pub fn can_collide(&self, other: &CollisionGroups) -> bool {
        (self.memberships & other.filters) != 0 && (other.memberships & self.filters) != 0
    }
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub invulnerable: bool,
}

#[derive(Component)]
pub struct DamageResistance {
    pub physical: f32,
    pub fire: f32,
    pub ice: f32,
    pub electric: f32,
}

pub enum DamageType {
    Physical,
    Fire,
    Ice,
    Electric,
}

#[derive(Component)]
pub struct DamageRequest {
    pub target: Entity,
    pub amount: f32,
    pub damage_type: DamageType,
}



#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct CameraOffset {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}