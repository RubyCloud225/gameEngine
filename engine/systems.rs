use bevy_ecs::prelude::*;
use crate::resources::{FixedTime, RealTime, InputState, PhysicsSettings, CollisionPairs };
use crate::components::{Position, Velocity, FixedPhysics, Time, Player, AffectedByGravity, BoundingBox, Collidable, Grounded};
use std::collections::HashMap;
use rand::Rng; // Add 'rand' crate to your Cargo.toml

type AssetId = usize;

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

// Integrating Acceleration, Velocity, and grounded state into 
// Narrow Phase Collision Resolution
// Calculate Target Position then movement from current to target as a potential collision sweep
// update the postion component once and saves CPU cycles
// Snap to surface when grounded by checking the axes in sequence
/*
follows the order:
    - Apply Forces: Sum all accelerations (Gravity + player inpur)
    - Integrate Velocity: v = v + a.dt
    - Broad Phase: Update the Spatial Grid so we know who is near whom
    - predict & Resolve: Use the velocity to find the Next Position and prune it against collisions
    - Update State: Set Grounded or Impact flags
To prevent stocking into the surface, epilson has been added to the position correction
The collision check next frame might say overlap true again, the epsilon ensures there is always
a minimal gap to prevent sticking.

The epilson value may need to be tuned based on the scale of the game world and act as
a buffer to prevent floating-point precision issues.

Trigger event: 
- OnTriggerEnter - when an entity first overlaps with a trigger volume
- OnTriggerExit - when an entity stops overlapping with a trigger volume
- OnTriggerStay - when an entity continues to overlap with a trigger volume

event driven architecture this integrates with other separate systems which perform animation
*/

pub fn physics_prediction_systems(
    grid: Res<SpatialGrid>, 
    time: Res<Time>, 
    mut query: Query<(Entity, &mut Position, &mut Velocity, &Acceleration, &Hitbox, &mut Grounded)>,
    collidables: Query<(&Position, &Hitbox), With<Collidable>>,
    triggers: Query<(&Position, &Hitbox), With<Trigger>>,
    step_cfg: Res<StepConfig>,
    mut commands: Commands,
    mut trackers: Query<&mut TriggerTracker>
) {
    let dt = time.delta_seconds * time_manager.time_scale;

    for (entity, mut pos, mut vel, acc, hit, mut grounded) int query.iter_mut() {
        // update velocity and calculate intended movement
        vel.x += acc.x * dt;
        vel.y += acc.y * dt;
        vel.z += acc.z * dt;
        // Predict
        let movement_dir = (vel.x * dit, vel.y * dt, vel.z * dt);
        // Reset grounded state
        grounded.0 = false;
        // Apply Friction
        // Only if horizontal friction if the player is on the ground
        if grounded.0 {
            vel.x *= friction.0.powf(dt);
            vel.z *= friction.0.powf(dt);
        }
        // Move current to last, and clear current for the new check
        tracker.last_frame = tracker.current_frame.clone();
        tracker.current_frame.clear();
        // Broad Phase - get nearby entities from spatial grid
        let (gx, gy, gz) = grid.world_to_grid(pos.x, pos.y, pos.z);
        let neighbors = grid.get_nearby(pos.x, pos.y, pos.z);

        // trigger Detection
        for &other_entity in &neighbors {
            if entity == other_entity {
                continue; // skip self
            }
            if let Ok((t_pos, t_hit, t_group)) = triggers.get(other_entity) {
                if test_overlap(
                    pos.x, pos.y, pos.z, hit,
                    t_pos.x, t_pos.y, t_pos.z, t_hit,
                ) {
                    // Trigger Enter Event
                    commands.spawn(OnTriggerEnter { target_entity: other });
                    tracker.current_frame.insert(other);
                } {
                    // Trigger Exit Event
                    commands.spawn(OnTriggerExit { target_entity: other });
                    tracker.last_frame.insert(other);
                }{
                    // Trigger Stay Event
                    commands.spawn(OnTriggerStay { target_entity: other });
                    tracker.current_frame.insert(other);
                    tracker.last_frame.insert(other);
                }
                
                // only do maths if the mask matches
                if my_group.can_collide(&t_group.memberships) {
                    // handle collision resolution here if needed
                    if test_overlap(
                        pos.x, pos.y, pos.z, hit,
                        t_pos.x, t_pos.y, t_pos.z, t_hit,
                    ) {
                        // Collision detected with trigger
                        // Handle collision resolution here if needed
                        tracker.current_frame.insert(other_entity);
                    }
            }
        }
        // Compare States
        // Enter: In current but not in last
        for &entered in tracker.current_frame.difference(&tracker.last_frame) {
            commends.spawn(OnTriggerEnter { actor: entity, target: entered_entity });
        }
        // Exit: In last but not in current
        for &exited in tracker.last_frame.difference(&tracker.current_frame) {
            commands.spawn(OnTriggerExit { actor: entity, target: exited_entity });
        }
        // Stay: In both current and last
        for &stayed in tracker.current_frame.intersection(&tracker.last_frame) {
            commands.spawn(OnTriggerStay { actor: entity, target: stayed_entity });
        }
        // X/Z Axis Resolution (with step up)
        let mut target_x = pos.x + movement_dir.0;
        let mut target_z = pos.z + movement_dir.2;
        for &other in &neighbors {
            if entity == other {
                continue; // skip self
            }
            if let Ok((w_pos, w_hit)) = collidables.get(other) {
                if test_overlap(
                    target_x, pos.y, target_z, hit,
                    w_pos.x, w_pos.y, w_pos.z, w_hit,
                ) {
                    // Simple resolution: stop horizontal movement
                    target_x = pos.x;
                    target_z = pos.z;

                    // Health reduction on collision with damage sources could be handled here
                    commands.spawn(DamageRequest {
                        target: entity,
                        amount: 10.0, // Example damage value
                        damage_type: DamageType::Physical,
                    });

                    // Stumble effect: reduce Velocity and add camera shake
                    vel.x *= 0.5; // Reduce horizontal speed by 50%
                    vel.z *= 0.5;
                    trauma.value = (trauma.value + 0.3).min(1.0); // Add trauma for camera shake
                }

                // Y-axis step up logic (only if we collided horizontally)
                if test_overlap(
                    pos.x, target_y, pos.z, hit,
                    w_pos.x, w_pos.y, w_pos.z, w_hit,
                ) {
                    // Collision detected on y axis, we are grounded
                    if vel.y < 0.0 {
                        target_y = w_pos.y + w_hit.max.1 - hit.min.1 + EPSILON;
                        grounded.is_grounded = true;
                        grounded.platform = Some(other_entity);
                    } else {
                        target_y = w_pos.y + w_hit.min.1 - hit.max.1 - EPSILON;
                    }
                    vel.y = 0.0;

                    commands.spawn(DamageRequest {
                        target: entity,
                        amount: 5.0, // Example damage value for hitting the ceiling
                        damage_type: DamageType::Physical,
                    });

                    vel.x *= 0.5; // Reduce horizontal speed by 50% on vertical collision as well
                    vel.z *= 0.5;
                    trauma.value = (trauma.value + 0.2).min(1.0); // Add trauma for camera shake
                }
                // attempt step up - check if there is space aboce
                let step_y = pos.y + step_cfg.max_height;
                if test_overlap(
                    target_x, step_y, target_z, hit,
                    w_pos.x, w_pos.y, w_pos.z, w_hit,
                ) {
                    // successful step up
                    pos.y = step_y;
                } else {
                    // if failure snap to edge with epilson
                    if movement_dir.2.abs() > 0.0 {
                        target_z = if vel.z > 0.0 {
                            w_pos.z - w_hit.min.2 - hit.max.2 - EPSILON
                        } else {
                            w_pos.z + w_hit.max.2 - hit.min.2 + EPSILON
                        }
                        vel.z = 0.0;
                    }
                }  
            }
        }
    }
    pos.x = target_x;
    pos.z = target_z;

    // Y Axis Resolution (Gravity and Grounding)
    let mut target_y = pos.y + movement_dir.1;
    for &other_entity in &neighbors {
        if entity == other {
            continue; // skip self
        }
        if let Ok((w_pos, w_hit)) = collidables.get(other_entity) {
            if test_overlap(
                pos.x, target_y, pos.z, hit,
                w_pos.x, w_pos.y, w_pos.z, w_hit,
            ) {
                // Collision detected on Y axis
                if vel.y < 0.0 {
                    // Landing on surface
                    target_y = w_pos.y + w_hit.max.1 - hit.min.1 + EPSILON;
                    grounded.is_grounded = true;
                    grounded.platform = Some(other_entity);
                } else{
                    // Hitting ceiling
                    target_y = w_pos.y + w_hit.min.1 - hit.max.1 - EPSILON;
                }
                vel.y = 0.0;
            }
        }
    }
} 
    pos.y = target_y;
}

// Helper function to test overlap between two hitboxes given positions
fn test_overlap(
    x1: f32, y1: f32, z1: f32, hit1: &Hitbox,
    x2: f32, y2: f32, z2: f32, hit2: &Hitbox,
) -> bool {
    !(x1 + hit1.max.0 < x2 + hit2.min.0 ||
      x1 + hit1.min.0 > x2 + hit2.max.0 ||
      y1 + hit1.max.1 < y2 + hit2.min.1 ||
      y1 + hit1.min.1 > y2 + hit2.max.1 ||
      z1 + hit1.max.2 < z2 + hit2.min.2 ||
      z1 + hit1.min.2 > z2 + hit2.max.2)
}

/*
To clear and rebuild system for Spatial Grid each physics step
For every action the system needs to be cleared to prevent the stale data
rebuilding is faster then tracking movement across cells

Other of this:
- Gravity/Input systems
- Grid Rebuild System - clear and rebuild the spatial grid
- physics_prediction_systems - integrate movement and resolve collisions
*/

pub fn rebuild_spatial_grid_system(
    mut grid: ResMut<SpatialGrid>,
    query: Query<(Entity, &Position, &Hitbox), With<Collidable>>,
) {
    // Wipe the old data
    grid.cells.clear();
    for (ent, pos, hit) in query.iter() {
        grid.insert_entity(ent, pos, hit);
    }
}

// Helper function: Logic for Center-based overlap
fn test_overlap(
    p1: &Position, h1: &Hitbox,
    p2: &Position, h2: &Hitbox,
) -> bool {
    // Calculate half-extents
    let h1_x = h1.width / 2.0;
    let h1_y = h1.height / 2.0;
    let h1_z = h1.depth / 2.0;

    let h2_x = h2.width / 2.0;
    let h2_y = h2.height / 2.0;
    let h2_z = h2.depth / 2.0;

    (p1.x - h1_x < p2.x + h2_x && p1.x + h1_x > p2.x - h2_x) &&
    (p1.y - h1_y < p2.y + h2_y && p1.y + h1_y > p2.y - h2_y) &&
    (p1.z - h1_z < p2.z + h2_z && p1.z + h1_z > p2.z - h2_z)
}
pub fn cleanup_trigger_events_system(
    mut commands: Commands,
    query: Query<Entity, Or<(With<OnTriggerEnter>, With<OnTriggerExit>, With<OnTriggerStay>)>>,
) {
    for event_entity in query.iter() {
        commands.entity(event_entity).despawn();
    }
}

// Health and damage system
// Animation and effects system
fn animation_task_graph_system(
    mut query: Query<(&Velocity, &Grounded, &mut AnimationController)>,
    task_graph: Res<AnimationTaskGraph>,
) {
    for (velocity, grounded, mut anim) in &mut query {
        anim.blend_time = if grounded.0 {
            0.1 // Faster blend when grounded
        } else {
            0.3 // Slower blend when in air for more dramatic effect
        };
        anim.state = if let Some(platform_entity) = grounded.platform {
            // If grounded on a platform, we might want a different animation
            AnimationState::IdleOnPlatform
        } else if velocity.x.abs() > 0.1 || velocity.z.abs() > 0.1 {
            // If moving horizontally, play running animation
            AnimationState::Running
        } else {
            // Default to idle animation
            AnimationState::Idle
        };
        // Schedule animation blend task
        task_graph.schedule(AnimationBlendTask {
            entity: query.entity(),
            target_state: anim.state,
            blend_time: anim.blend_time,
        });
    }
}

// Moving Platform System
pub fn moving_platform_system(
    time: Res<Time>,
    mut player_query: Query<(&mut Position, &Grounded, &mut Velocity)>,
    platform_query: Query<(&Position, &Velocity), With<Collidable>>,
) {
    let dt = time.delta_seconds:
    for (mut player_pos, grounded, mut player_vel) in player_query.iter_mut() {
        if grounded.0 {
            if let Some(platform_entity) = grounded.1 {
                if let Ok((platform_pos, platform_vel)) = platform_query.get(platform_entity) {
                    // Move the player along with the platform
                    player_pos.x += platform_vel.x * dt;
                    player_pos.y += platform_vel.y * dt;
                    player_pos.z += platform_vel.z * dt;

                    // Optionally adjust player's velocity to match platform's velocity
                    player_vel.x += platform_vel.x;
                    player_vel.y += platform_vel.y;
                    player_vel.z += platform_vel.z;
                    // TODO:: Momentum transfer could be added here for more realism
                    if !grounded.is_grounded {
                        if let Some(last_platform) = grounded.platform {
                            if let Ok(plat_vel) = platforms.get(last_platform) {
                                // INJECT MOMENTUM
                                // The player's new world velocity = their jump + platform speed
                                vel.x += plat_vel.x;
                                vel.y += plat_vel.y;
                                vel.z += plat_vel.z;
                            }
                            // Clear the platform so we don't apply momentum twice
                            grounded.platform = None;
                        }
                    }
                    
                }
            }
        }
    }
}

pub struct Ray {
    pub origin: Position,
    pub direction: (f32, f32, f32),
    pub length: f32,
}

pub fn shoot_ray_system(
    grid: Res<SpatialGrid>,
    ray: Ray,
    my_groups: &CollisionGroups,
    targets: Query<(Entity, &Position, &Hitbox, &CollisionGroups)>,
) -> Option<(Entity, f32)> { // Returns hit entity and distance
    let mut closest_hit: Option<(Entity, f32)> = None;

    // 1. Get cells along the ray path (Simplified Broad Phase)
    let cells = grid.get_cells_along_ray(&ray); 

    for cell in cells {
        if let Some(entities) = grid.cells.get(&cell) {
            for &entity in entities {
                if let Ok((ent, pos, hit, groups)) = targets.get(entity) {
                    // 2. LAYER FILTERING: Can our bullet hit this?
                    if my_groups.can_interact_with(groups.memberships) {
                        // 3. MATH: Ray-AABB Intersection
                        if let Some(dist) = ray_aabb_intersection(&ray, pos, hit) {
                            if dist < closest_hit.map(|(_, d)| d).unwrap_or(ray.length) {
                                closest_hit = Some((ent, dist));
                            }
                        }
                    }
                }
            }
        }
        // If we hit something in this cell, we can stop (Early Exit)
        if closest_hit.is_some() { break; }
    }
    closest_hit
}

pub fn health_system(
    mut commands: Commands, 
    // We add Position to the query so we can spawn effects instantly
    mut health_query: Query<(Entity, &mut Health, &Position, Option<&DamageResistance>)>,
    // Assuming DamageRequests is a Resource containing a Vec of events
    mut damage_requests: ResMut<DamageRequests>, 
) {
    // We use drain(..) to empty the requests as we process them
    for request in damage_requests.events.drain(..) {
        let target = request.target;

        if let Ok((entity, mut health, pos, resistance)) = health_query.get_mut(target) {
            if health.invulnerable { continue; }

            // 1. Calculate Mitigation
            // We use .unwrap_or(0.0) to handle cases where there is no resistance component
            let resist_perc = resistance.map(|r| {
                match request.damage_type {
                    DamageType::Physical => r.physical_percentage,
                    DamageType::Fire => r.fire_percentage,
                    _ => 0.0,
                }
            }).unwrap_or(0.0);

            let final_damage = (request.amount as f32 * (1.0 - resist_perc)).ceil();
            if final_damage > 0.0 { trauma.value = (trauma.value + 0.3).min(1.0);} // No damage to apply

            // 2. Apply Damage & Death Logic
            if final_damage >= health.current {
                health.current = 0.0;
                // Don't despawn immediately! Insert a marker so a DeathSystem 
                // can play an animation first.
                commands.entity(entity).insert(Dead); 
            } else {
                health.current -= final_damage;
            }

            if final_damage > 10.0 {
                time_manager.time_scale = 0.0;
                time_manager.remaining_stop = 0.05; // Freeze for 0.1 real seconds
            }

            // 3. Spawn Effects (Using the Position we queried)
            commands.spawn(DamageEffect { 
                position: pos.clone(), 
                amount: final_damage 
            });
        }
    }
}

pub fn camera_shake_system(
    time: Res<Time>,
    mut trauma: ResMut<CameraTrauma>,
    mut camera_query: Query<(&mut Position, &mut CameraOffset), With<MainCamera>>,
) {
    let dt = time.delta_seconds;
    let mut rng = rand::thread_rng();

    // 1. Decay trauma over time (linear decay)
    let decay_speed = 0.8; 
    trauma.value = (trauma.value - decay_speed * dt).max(0.0);

    if let Ok((pos, mut offset)) = camera_query.get_single_mut() {
        if trauma.value > 0.0 {
            // 2. Square the trauma for a better "feel"
            let shake_intensity = trauma.value * trauma.value;
            
            // 3. Generate random offsets
            let max_offset = 0.5; // Max units to shake
            offset.x = max_offset * shake_intensity * rng.gen_range(-1.0..1.0);
            offset.y = max_offset * shake_intensity * rng.gen_range(-1.0..1.0);
            // We usually don't shake Z (depth) for 2D/3D top-down, 
            // but for FPS you might add subtle Z shake.
        } else {
            offset.x = 0.0;
            offset.y = 0.0;
        }
    }
}

/*
Time Dilation System
movement calculated as Position = Velocity * delta_time * time_dilation_factor
*/
pub fn time_management_system(
    real_time: Res<Time<Real>>, // Bevy's Real time doesn't get scaled
    mut manager: ResMut<TimeManager>,
) {
    if manager.remaining_stop > 0.0 {
        manager.remaining_stop -= real_time.delta_seconds();
        
        if manager.remaining_stop <= 0.0 {
            manager.time_scale = 1.0; // Resume normal time
        }
    }
}