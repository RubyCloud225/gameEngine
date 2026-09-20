// define the components - Data
use bevy_ecs::prelude::*;
use std::collections::HashSet;
// ECS data for movement must be stored by the structs
// ECS schedule can store all position data together and all velocity data together.

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct Acceleration {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct Hitbox {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

// to do Time-
#[derive(Resource)]
pub struct Time {
    pub delta_seconds: f32, // Time elapsed since the last frame
}
#[derive(Component)]
pub struct FixedPhysics;

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

#[derive(Component, Default, Debug)]
pub struct Grounded {
    pub is_grounded: bool,
    pub platform: Option<Entity>,
} // True if the entity is currently resting on a surface

pub const MAX_STEP_HEIGHT: f32 = 0.3; // Maximum height the player can step over
pub const EPSILON: f32 = 0.001; // Small value to prevent floating-point precision issues

#[derive(Component, Resource)]
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

// Triggers and events
#[derive(Component)]
pub struct Trigger;
#[derive(Component, Default)]
pub struct TriggerTracker {
    pub current_frame: HashSet<Entity>,
    pub last_frame: HashSet<Entity>,
}
#[derive(Component)]
pub struct OnTriggerEnter {
    pub actor: Entity,
    pub target_entity: Entity,
}

#[derive(Component)]
pub struct OnTriggerExit {
    pub actor: Entity,
    pub target_entity: Entity,
}
#[derive(Component)]
pub struct OnTriggerStay {
    pub actor: Entity,
    pub target_entity: Entity,
}

// Collision groups and layers

#[derive(Component, Clone, Copy)]
pub struct CollisionGroups {
    pub memberships: u32, // current layers this entity belongs to
    pub filters: u32,     // layers this entity collides with
}

impl CollisionGroups {
    pub fn can_collide(&self, other: &CollisionGroups) -> bool {
        (self.memberships & other.filters) != 0 && (other.memberships & self.filters) != 0
    }
}

// Health and lifeline

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub invulnerable: bool,
}
#[derive(Component)]
pub struct Lifetime {
    pub remaining_seconds: f32,
}

// Damage and knockback
#[derive(Component)]
pub struct DamageResistance {
    pub physical: f32,
    pub fire: f32,
    pub ice: f32,
    pub electric: f32,
}

#[derive(Clone, Copy, Debug)]
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
pub struct DamageEvent {
    pub target: Entity,
    pub amount: f32,
    pub damage_type: DamageType,
}
#[derive(Component)]
pub struct Knockback {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[derive(Component)]
pub struct KnockbackResistance {
    pub value: f32, // 0.0 to 1.0, where 1.0 means no knockback taken
}

// animation and effects
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationState {
    Idle,
    IdleOnPlatform,
    Walk,
    Run,
    Dash,
    Jump,
    Fall,
    Attack,
    Die,
}

#[derive(Component)]
pub struct AnimationController {
    pub name: String,
    pub state: AnimationState,
    pub blend_time: f32,
}
#[derive(Component)]
pub struct ParticleEffect {
    pub name: String,
    pub duration: f32,
    pub looped: bool,
}
// Sprinting
#[derive(Component)]
pub struct Sprinting {
    pub is_sprinting: bool,
}

#[derive(Component)]
pub struct SprintConfig {
    pub sprint_multiplier: f32,
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct CameraOffset {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct DamageEffect {
    pub position: Position,
    pub amount: f32,
}
