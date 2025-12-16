use bevy_ecs::prelude::*;
use crate::resources::{FixedTime, RealTime, InputState, PhysicsSettings, CollisionPairs };
use crate::components::{Position, Velocity, FixedPhysics, Time, Player, AffectedByGravity, BoundingBox, Collidable, Grounded};

fn accumulate_time_system(
    mut fixed_time: ResMut<FixedTime>,
    real_time: Res<RealTime>,
) {
    fixed_time.accumulator += real_time.delta_seconds;
}

// Input system Converts user input into a Velocity
fn input_to_velocity(input: Res<InputState>, mut player_query: Query<(&MovementSpeed, &mut Velocity), With<Player>>,) {
    let (speed_attributes, mut velocity) = player_query.single_mut().expect("There should be exactly one player!");
    // calculate the intended speed based on sprinting
    let mut speed = speed_attributes.base;
    if input.is_sprinting {
        speed *= speed_attributes.sprint_multiplier;
    }
    // Calculate new velocity
    let (input_x, input_z) = input.movement_direction;

    // this is just for walking but to expand forces will need to be included
    velocity.x = input_x * speed;
    velocity.z = input_z * speed;
    // This system only updates
    // target velocity, The fixed physics system will use this value with the FixedTime delta in the next step
}
// Movement system - parallel logic
// takes a query as a parameter
// - &mut Position: Mutable access (WRITE access)
// - &Velocity: Immutable access (READ access)
// This filter all entities that possess BOTH components
fn movement(mut query: Query<(&mut Position, &Velocity). With<FixedPhysics>>, time: Res<FixedTime> ) {
    // The loop iterates over every single entity that moves
    // the Key is that this entire iteration is automatically
    let dt = time.TIMESTEP;
    for (mut position, velocity) in &mut query {
        position.x += velocity.x; // * time.delta_seconds
        position.y += velocity.y;
        position.z += velocity.z;
    }
}

fn gravity_system(
    mut query: Query<&mut Velocity, With<AffectedByGravity>>,
    settings: Res<PhysicsSettings>,
    // use a fixed timestep
    fixed_time: Res<FixedTime>,
) {
    let dt = fixed_time.TIMESTEP;
    let gravity_accel = settings.gravity_acceleration;
    // PARALLEL KERNEL 
    // scheduler runs this concurrently over all entities affected by gravity
    for mut velocity in &mut query {
        // v = v + a * t apply acceleration
        // only modify the Y-axis velocity
        velocity.y += gravity_accel * dt;
    }
    // end of Parallel kernel
}

fn broad_phase_system(
    query: Query<(Entity, &BoundingBox), WIth<Collidable>>,
    mut collision_pairs: ResMut<CollisionPairs>,
) {
    // Clear list of potential collisions
    collision_pairs.clear()
    // Spatial Query
    // Actual OcTree AABB Tree Logic - check billions of bounding box overlaps
    // basic Iteration: iterate over all possible pairs (N^2),
    // Spatial structure reduces this to N log N or better.
    // Conceptual loop:
    for [(entity_a, box_a), (entity_b, box_b)] in query.iter_combinations() {
        if is_overlapping(box_a, box_b) {
            // add pair to the list for Narrow Phase
            collision_pairs.pairs.push((entity_a, entity_b));
        }
    }
} // is_overlapping function is sample AABB - AABB check.

fn collision_resolution_system(
    mut player_query: Query<(
        &mut Position,
        &mut Velocity,
        &BoundingBox,
        &mut Grounded
    ), With<Player>>,
) {
    // Get player data(just movement for now)
    let (mut pos, mut vel, bbox, mut grounded) = player_query.single_mut().expect("One Player");
    // --- CONCEPTUAL PLAYER-VS-FLOOR LOGIC ---
    // In a real engine, this is a complex function. For simple walking, we simplify:
    let floor_height: f32 = 0.0; // Assume the world floor is at Y=0

    // Check if the bottom of the player's bounding box has passed the floor (Y-axis collision)
    if pos.y + bbox.min.1 <= floor_height {
        // 2. Resolve Position (push the player out of the floor)
        pos.y = floor_height - bbox.min.1; 
        
        // 3. Resolve Velocity (stop all downward motion)
        if vel.y < 0.0 {
            vel.y = 0.0;
        }
        
        // 4. Update Status (Stop gravity accumulation)
        grounded.0 = true;
    } else {
        grounded.0 = false;
    }
}