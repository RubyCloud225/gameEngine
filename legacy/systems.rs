#![allow(clippy::type_complexity)] // ECS queries describe component access explicitly.
use crate::components::Trigger;
use crate::components::*;
use crate::resources::*;
use bevy_ecs::prelude::*;
use rand::Rng;
use std::collections::HashSet;

pub fn accumulate_time_system(mut fixed: ResMut<FixedTime>, real: Res<RealTime>) {
    fixed.accumulator += real.delta_seconds.max(0.0);
}

pub fn input_to_velocity(
    input: Res<InputState>,
    mut players: Query<(&MovementSpeed, &mut Velocity), With<Player>>,
) {
    for (speed, mut velocity) in &mut players {
        let speed = speed.base
            * if input.is_sprinting {
                speed.sprint_multiplier
            } else {
                1.0
            };
        let (x, z) = input.movement_direction;
        let length = x.hypot(z).max(1.0);
        velocity.x = x / length * speed;
        velocity.z = z / length * speed;
    }
}

/// Simple integration; use this OR physics_prediction_systems, not both on a body.
pub fn movement(
    mut query: Query<(&mut Position, &Velocity), With<FixedPhysics>>,
    manager: Res<TimeManager>,
) {
    let dt = FixedTime::TIMESTEP * manager.time_scale;
    for (mut p, v) in &mut query {
        p.x += v.x * dt;
        p.y += v.y * dt;
        p.z += v.z * dt;
    }
}

pub fn gravity_system(
    mut query: Query<&mut Velocity, With<AffectedByGravity>>,
    settings: Res<PhysicsSettings>,
    manager: Res<TimeManager>,
) {
    for mut velocity in &mut query {
        velocity.y += settings.gravity_acceleration * FixedTime::TIMESTEP * manager.time_scale;
    }
}

/// BoundingBox is a local offset from Position. Hitbox uses centre and dimensions.
pub fn broad_phase_system(
    query: Query<(Entity, &Position, &BoundingBox), With<Collidable>>,
    mut pairs: ResMut<CollisionPairs>,
) {
    pairs.pairs.clear();
    for [(a, pa, ba), (b, pb, bb)] in query.iter_combinations() {
        if pa.x + ba.min.0 < pb.x + bb.max.0
            && pa.x + ba.max.0 > pb.x + bb.min.0
            && pa.y + ba.min.1 < pb.y + bb.max.1
            && pa.y + ba.max.1 > pb.y + bb.min.1
            && pa.z + ba.min.2 < pb.z + bb.max.2
            && pa.z + ba.max.2 > pb.z + bb.min.2
        {
            pairs.pairs.push((a, b));
        }
    }
}

/// Optional simple floor at y=0, for use with movement instead of the full collision system.
pub fn collision_resolution_system(
    mut players: Query<(&mut Position, &mut Velocity, &BoundingBox, &mut Grounded), With<Player>>,
) {
    for (mut pos, mut vel, bbox, mut grounded) in &mut players {
        grounded.is_grounded = pos.y + bbox.min.1 <= 0.0;
        grounded.platform = None;
        if grounded.is_grounded {
            pos.y = -bbox.min.1;
            vel.y = vel.y.max(0.0);
        }
    }
}

pub fn test_overlap(p: &Position, h: &Hitbox, q: &Position, k: &Hitbox) -> bool {
    (p.x - q.x).abs() < (h.width + k.width) / 2.0
        && (p.y - q.y).abs() < (h.height + k.height) / 2.0
        && (p.z - q.z).abs() < (h.depth + k.depth) / 2.0
}

fn groups_match(a: Option<&CollisionGroups>, b: Option<&CollisionGroups>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a.can_collide(b),
        _ => true,
    }
}

type Obstacles<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Position,
        &'static Hitbox,
        Option<&'static CollisionGroups>,
    ),
    (With<Collidable>, Without<Trigger>),
>;
type Bodies<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Position,
        &'static mut Velocity,
        &'static Acceleration,
        &'static Hitbox,
        &'static mut Grounded,
        Option<&'static Friction>,
        Option<&'static CollisionGroups>,
    ),
    With<FixedPhysics>,
>;

/// Discrete axis-separated AABB motion against a snapshot of colliders.
/// Large displacements can tunnel through thin walls; this is not swept rigid-body physics.
pub fn physics_prediction_systems(
    mut queries: ParamSet<(Obstacles, Bodies)>,
    step: Res<StepConfig>,
    manager: Res<TimeManager>,
) {
    let dt = FixedTime::TIMESTEP * manager.time_scale;
    if dt <= 0.0 {
        return;
    }
    // Snapshot prevents mutable/immutable Position query conflicts while bodies are integrated.
    let obstacles: Vec<_> = queries
        .p0()
        .iter()
        .map(|(e, p, h, g)| (e, *p, *h, g.copied()))
        .collect();
    for (entity, mut pos, mut vel, acc, hit, mut grounded, friction, groups) in &mut queries.p1() {
        vel.x += acc.x * dt;
        vel.y += acc.y * dt;
        vel.z += acc.z * dt;
        let was_grounded = grounded.is_grounded;
        if was_grounded && let Some(f) = friction {
            let retention = f.coefficient.clamp(0.0, 1.0).powf(dt);
            vel.x *= retention;
            vel.z *= retention;
        }
        grounded.is_grounded = false;
        grounded.platform = None;
        let candidates: Vec<_> = obstacles
            .iter()
            .filter(|(e, _, _, g)| *e != entity && groups_match(groups, g.as_ref()))
            .collect();
        for axis in [0, 2, 1] {
            let speed = match axis {
                0 => vel.x,
                1 => vel.y,
                _ => vel.z,
            };
            let mut target = *pos;
            match axis {
                0 => target.x += speed * dt,
                1 => target.y += speed * dt,
                _ => target.z += speed * dt,
            }
            for (other, p, h, _) in &candidates {
                if !test_overlap(&target, hit, p, h) {
                    continue;
                }
                if axis != 1 && was_grounded && speed != 0.0 {
                    let step_y = p.y + h.height / 2.0 + hit.height / 2.0 + EPSILON;
                    let rise = step_y - pos.y;
                    let stepped = Position {
                        y: step_y,
                        ..target
                    };
                    if rise > 0.0
                        && rise <= step.max_height + EPSILON
                        && !candidates
                            .iter()
                            .any(|(_, p, h, _)| test_overlap(&stepped, hit, p, h))
                    {
                        target = stepped;
                        continue;
                    }
                }
                match axis {
                    0 if speed != 0.0 => {
                        target.x = p.x - speed.signum() * ((h.width + hit.width) / 2.0 + EPSILON);
                        vel.x = 0.0;
                    }
                    2 if speed != 0.0 => {
                        target.z = p.z - speed.signum() * ((h.depth + hit.depth) / 2.0 + EPSILON);
                        vel.z = 0.0;
                    }
                    1 if speed != 0.0 => {
                        target.y = p.y - speed.signum() * ((h.height + hit.height) / 2.0 + EPSILON);
                        vel.y = 0.0;
                        if speed < 0.0 {
                            grounded.is_grounded = true;
                            grounded.platform = Some(*other);
                        }
                    }
                    _ => {}
                }
            }
            *pos = target;
        }
        // Preserve contact while stationary or during the tiny epsilon separation.
        if vel.y <= 0.0 && !grounded.is_grounded {
            let probe = Position {
                y: pos.y - 2.0 * EPSILON,
                ..*pos
            };
            if let Some((entity, _, _, _)) = candidates
                .iter()
                .find(|(_, p, h, _)| test_overlap(&probe, hit, p, h))
            {
                grounded.is_grounded = true;
                grounded.platform = Some(*entity);
            }
        }
    }
}

pub fn rebuild_spatial_grid_system(
    mut grid: ResMut<SpatialGrid>,
    query: Query<(Entity, &Position, &Hitbox), Or<(With<Collidable>, With<Trigger>)>>,
) {
    grid.clear();
    for (entity, pos, hit) in &query {
        grid.insert_entity(entity, pos, hit);
    }
}

pub fn trigger_detection_system(
    mut actors: Query<(
        Entity,
        &Position,
        &Hitbox,
        &mut TriggerTracker,
        Option<&CollisionGroups>,
    )>,
    triggers: Query<(Entity, &Position, &Hitbox, Option<&CollisionGroups>), With<Trigger>>,
    mut commands: Commands,
) {
    for (actor, p, h, mut tracker, groups) in &mut actors {
        tracker.last_frame = std::mem::take(&mut tracker.current_frame);
        for (target, tp, th, tg) in &triggers {
            if actor != target && groups_match(groups, tg) && test_overlap(p, h, tp, th) {
                tracker.current_frame.insert(target);
            }
        }
        for &target_entity in tracker.current_frame.difference(&tracker.last_frame) {
            commands.spawn(OnTriggerEnter {
                actor,
                target_entity,
            });
        }
        for &target_entity in tracker.last_frame.difference(&tracker.current_frame) {
            commands.spawn(OnTriggerExit {
                actor,
                target_entity,
            });
        }
        for &target_entity in tracker.current_frame.intersection(&tracker.last_frame) {
            commands.spawn(OnTriggerStay {
                actor,
                target_entity,
            });
        }
    }
}

pub fn cleanup_trigger_events_system(
    mut commands: Commands,
    query: Query<
        Entity,
        Or<(
            With<OnTriggerEnter>,
            With<OnTriggerExit>,
            With<OnTriggerStay>,
        )>,
    >,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn animation_task_graph_system(
    mut query: Query<(
        Entity,
        &Velocity,
        &Grounded,
        &mut AnimationController,
        Option<&Dead>,
    )>,
    mut graph: ResMut<AnimationTaskGraph>,
) {
    graph.tasks.clear();
    for (entity, velocity, grounded, mut anim, dead) in &mut query {
        anim.blend_time = if grounded.is_grounded { 0.1 } else { 0.3 };
        anim.state = if dead.is_some() {
            AnimationState::Die
        } else if !grounded.is_grounded {
            if velocity.y > 0.0 {
                AnimationState::Jump
            } else {
                AnimationState::Fall
            }
        } else if velocity.x.abs() > 0.1 || velocity.z.abs() > 0.1 {
            AnimationState::Run
        } else if grounded.platform.is_some() {
            AnimationState::IdleOnPlatform
        } else {
            AnimationState::Idle
        };
        graph.tasks.push(AnimationBlendTask {
            entity,
            target_state: anim.state,
            blend_time: anim.blend_time,
        });
    }
}

type Riders<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Position,
        &'static mut Grounded,
        &'static mut Velocity,
    ),
    With<Player>,
>;
type Platforms<'w, 's> =
    Query<'w, 's, (Entity, &'static Velocity), (With<Collidable>, Without<Player>)>;
pub fn moving_platform_system(
    mut queries: ParamSet<(Platforms, Riders)>,
    manager: Res<TimeManager>,
) {
    let dt = FixedTime::TIMESTEP * manager.time_scale;
    if dt <= 0.0 {
        return;
    }
    let platforms: std::collections::HashMap<_, _> =
        queries.p0().iter().map(|(e, v)| (e, *v)).collect();
    for (mut pos, mut grounded, mut vel) in &mut queries.p1() {
        if let Some(platform) = grounded.platform {
            if let Some(pv) = platforms.get(&platform) {
                if grounded.is_grounded && vel.y <= 0.0 {
                    pos.x += pv.x * dt;
                    pos.y += pv.y * dt;
                    pos.z += pv.z * dt;
                } else {
                    vel.x += pv.x;
                    vel.y += pv.y;
                    vel.z += pv.z;
                    grounded.is_grounded = false;
                    grounded.platform = None;
                }
            } else {
                grounded.is_grounded = false;
                grounded.platform = None;
            }
        }
    }
}

pub struct Ray {
    pub origin: Position,
    pub direction: (f32, f32, f32),
    pub length: f32,
}

/// Slab intersection. Direction is normalized so the result is a world-space distance.
pub fn ray_aabb_intersection(ray: &Ray, p: &Position, h: &Hitbox) -> Option<f32> {
    let (x, y, z) = ray.direction;
    let length = (x * x + y * y + z * z).sqrt();
    if !length.is_finite() || length == 0.0 || !ray.length.is_finite() || ray.length < 0.0 {
        return None;
    }
    let mut near: f32 = 0.0;
    let mut far = ray.length;
    for (o, d, center, extent) in [
        (ray.origin.x, x / length, p.x, h.width / 2.0),
        (ray.origin.y, y / length, p.y, h.height / 2.0),
        (ray.origin.z, z / length, p.z, h.depth / 2.0),
    ] {
        if d.abs() < f32::EPSILON {
            if o < center - extent || o > center + extent {
                return None;
            }
        } else {
            let a = (center - extent - o) / d;
            let b = (center + extent - o) / d;
            near = near.max(a.min(b));
            far = far.min(a.max(b));
            if near > far {
                return None;
            }
        }
    }
    Some(near)
}

/// Query helper called from a system, not added directly to a Schedule.
/// Examines all indexed candidates to guarantee the nearest hit regardless of cell order.
pub fn shoot_ray_system(
    grid: &SpatialGrid,
    ray: &Ray,
    groups: &CollisionGroups,
    targets: &Query<(Entity, &Position, &Hitbox, &CollisionGroups)>,
) -> Option<(Entity, f32)> {
    let mut seen = HashSet::new();
    let mut closest = None;
    for &entity in grid.cells.values().flatten() {
        if !seen.insert(entity) {
            continue;
        }
        if let Ok((entity, pos, hit, target_groups)) = targets.get(entity)
            && groups.can_collide(target_groups)
            && let Some(distance) = ray_aabb_intersection(ray, pos, hit)
            && closest.is_none_or(|(_, d)| distance < d)
        {
            closest = Some((entity, distance));
        }
    }
    closest
}

pub fn health_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Health, &Position, Option<&DamageResistance>), Without<Dead>>,
    mut requests: ResMut<DamageRequests>,
    mut trauma: ResMut<CameraTrauma>,
    mut manager: ResMut<TimeManager>,
) {
    for request in requests.events.drain(..) {
        if let Ok((entity, mut health, pos, resistance)) = query.get_mut(request.target) {
            if health.invulnerable || health.current <= 0.0 || !request.amount.is_finite() {
                continue;
            }
            let resistance = resistance
                .map(|r| match request.damage_type {
                    DamageType::Physical => r.physical,
                    DamageType::Fire => r.fire,
                    DamageType::Ice => r.ice,
                    DamageType::Electric => r.electric,
                })
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let damage = (request.amount.max(0.0) * (1.0 - resistance)).ceil();
            if damage <= 0.0 {
                continue;
            }
            health.current = (health.current - damage).max(0.0);
            if health.current == 0.0 {
                commands.entity(entity).insert(Dead);
            }
            trauma.value = (trauma.value + 0.3).min(1.0);
            if damage > 10.0 {
                manager.time_scale = 0.0;
                manager.remaining_stop = 0.05;
            }
            commands.spawn(DamageEffect {
                position: *pos,
                amount: damage,
            });
        }
    }
}

pub fn camera_shake_system(
    time: Res<RealTime>,
    mut trauma: ResMut<CameraTrauma>,
    mut cameras: Query<&mut CameraOffset, With<MainCamera>>,
) {
    trauma.value = (trauma.value - 0.8 * time.delta_seconds).clamp(0.0, 1.0);
    let mut rng = rand::thread_rng();
    for mut offset in &mut cameras {
        let intensity = 0.5 * trauma.value * trauma.value;
        offset.x = intensity * rng.gen_range(-1.0..1.0);
        offset.y = intensity * rng.gen_range(-1.0..1.0);
        offset.z = 0.0;
    }
}

pub fn time_management_system(real: Res<RealTime>, mut manager: ResMut<TimeManager>) {
    if manager.remaining_stop > 0.0 {
        manager.remaining_stop = (manager.remaining_stop - real.delta_seconds).max(0.0);
        if manager.remaining_stop == 0.0 {
            manager.time_scale = 1.0;
        }
    }
}
