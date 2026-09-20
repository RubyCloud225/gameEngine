use bevy_ecs::{prelude::*, system::SystemState};
use game_engine::components::Trigger;
use game_engine::{components::*, resources::*, systems::*, *};

fn close(a: f32, b: f32) {
    assert!((a - b).abs() < 0.0001, "{a} != {b}");
}
fn cube() -> Hitbox {
    Hitbox {
        width: 1.0,
        height: 1.0,
        depth: 1.0,
    }
}
fn body(world: &mut World, pos: Position, vel: Velocity) -> Entity {
    world
        .spawn((
            FixedPhysics,
            pos,
            vel,
            Acceleration::default(),
            cube(),
            Grounded::default(),
        ))
        .id()
}
fn run<M>(world: &mut World, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) {
    let mut schedule = Schedule::default();
    schedule.add_systems(systems);
    schedule.run(world);
}
use bevy_ecs::system::ScheduleSystem;

#[test]
fn movement_uses_seconds_and_filters_fixed_bodies() {
    let mut w = new_world();
    let e = body(
        &mut w,
        Position::default(),
        Velocity {
            x: 60.0,
            y: 120.0,
            z: -60.0,
        },
    );
    let untouched = w
        .spawn((
            Position::default(),
            Velocity {
                x: 60.0,
                ..Default::default()
            },
        ))
        .id();
    run(&mut w, movement);
    assert_eq!(
        *w.get::<Position>(e).unwrap(),
        Position {
            x: 1.0,
            y: 2.0,
            z: -1.0
        }
    );
    assert_eq!(*w.get::<Position>(untouched).unwrap(), Position::default());
}

#[test]
fn input_sprints_and_normalizes_diagonal_without_changing_vertical_speed() {
    let mut w = new_world();
    let e = w
        .spawn((
            Player,
            MovementSpeed {
                base: 3.0,
                sprint_multiplier: 2.0,
            },
            Velocity {
                y: 5.0,
                ..Default::default()
            },
        ))
        .id();
    *w.resource_mut::<InputState>() = InputState {
        movement_direction: (1.0, 1.0),
        is_sprinting: true,
    };
    run(&mut w, input_to_velocity);
    let v = w.get::<Velocity>(e).unwrap();
    close(v.x.hypot(v.z), 6.0);
    close(v.y, 5.0);
}

#[test]
fn gravity_only_affects_marked_entities_and_respects_pause() {
    let mut w = new_world();
    let e = w.spawn((Velocity::default(), AffectedByGravity)).id();
    let other = w.spawn(Velocity::default()).id();
    run(&mut w, gravity_system);
    close(w.get::<Velocity>(e).unwrap().y, -9.81 / 60.0);
    close(w.get::<Velocity>(other).unwrap().y, 0.0);
    w.resource_mut::<TimeManager>().time_scale = 0.0;
    run(&mut w, gravity_system);
    close(w.get::<Velocity>(e).unwrap().y, -9.81 / 60.0);
}

#[test]
fn simple_floor_stops_downward_motion() {
    let mut w = new_world();
    let e = w
        .spawn((
            Player,
            Position {
                y: -1.0,
                ..Default::default()
            },
            Velocity {
                y: -5.0,
                ..Default::default()
            },
            BoundingBox {
                min: (-0.5, -0.5, -0.5),
                max: (0.5, 0.5, 0.5),
            },
            Grounded::default(),
        ))
        .id();
    run(&mut w, collision_resolution_system);
    close(w.get::<Position>(e).unwrap().y, 0.5);
    close(w.get::<Velocity>(e).unwrap().y, 0.0);
    assert!(w.get::<Grounded>(e).unwrap().is_grounded);
}

#[test]
fn broad_phase_uses_world_positions_and_clears_stale_pairs() {
    let mut w = new_world();
    let mut entities = vec![];
    for x in [0.0, 0.5, 10.0] {
        entities.push(
            w.spawn((
                Collidable,
                Position {
                    x,
                    ..Default::default()
                },
                BoundingBox {
                    min: (-0.5, -0.5, -0.5),
                    max: (0.5, 0.5, 0.5),
                },
            ))
            .id(),
        );
    }
    run(&mut w, broad_phase_system);
    assert_eq!(w.resource::<CollisionPairs>().pairs.len(), 1);
    w.get_mut::<Position>(entities[1]).unwrap().x = 5.0;
    run(&mut w, broad_phase_system);
    assert!(w.resource::<CollisionPairs>().pairs.is_empty());
}

#[test]
fn full_schedule_lands_on_floor_and_keeps_contact() {
    let mut w = new_world();
    let e = body(
        &mut w,
        Position {
            y: 3.0,
            ..Default::default()
        },
        Velocity::default(),
    );
    w.entity_mut(e).insert(AffectedByGravity);
    let floor = w
        .spawn((
            Collidable,
            Position {
                y: -0.5,
                ..Default::default()
            },
            Hitbox {
                width: 20.0,
                height: 1.0,
                depth: 20.0,
            },
        ))
        .id();
    let mut schedule = physics_schedule();
    for _ in 0..120 {
        advance_frame(&mut w, &mut schedule, FixedTime::TIMESTEP);
    }
    close(w.get::<Position>(e).unwrap().y, 0.5 + EPSILON);
    assert!(w.get::<Grounded>(e).unwrap().is_grounded);
    assert_eq!(w.get::<Grounded>(e).unwrap().platform, Some(floor));
    close(w.get::<Velocity>(e).unwrap().y, 0.0);
}

#[test]
fn prediction_applies_acceleration_friction_and_freeze() {
    let mut w = new_world();
    let e = body(
        &mut w,
        Position::default(),
        Velocity {
            x: 60.0,
            ..Default::default()
        },
    );
    w.entity_mut(e).insert((
        Friction { coefficient: 0.0 },
        Acceleration {
            z: 60.0,
            ..Default::default()
        },
    ));
    w.get_mut::<Grounded>(e).unwrap().is_grounded = true;
    run(&mut w, physics_prediction_systems);
    close(w.get::<Position>(e).unwrap().x, 0.0);
    close(w.get::<Velocity>(e).unwrap().z, 0.0);
    w.resource_mut::<TimeManager>().time_scale = 0.0;
    run(&mut w, physics_prediction_systems);
    assert_eq!(*w.get::<Position>(e).unwrap(), Position::default());
}

#[test]
fn wall_collision_and_layer_filtering() {
    for enabled in [false, true] {
        let mut w = new_world();
        let e = body(
            &mut w,
            Position::default(),
            Velocity {
                x: 30.0,
                ..Default::default()
            },
        );
        w.entity_mut(e).insert(CollisionGroups {
            memberships: 1,
            filters: 2,
        });
        w.spawn((
            Collidable,
            Position {
                x: 1.2,
                ..Default::default()
            },
            cube(),
            CollisionGroups {
                memberships: 2,
                filters: if enabled { 1 } else { 0 },
            },
        ));
        run(&mut w, physics_prediction_systems);
        close(
            w.get::<Position>(e).unwrap().x,
            if enabled { 0.2 - EPSILON } else { 0.5 },
        );
    }
}

#[test]
fn step_up_requires_grounding_and_clear_headroom() {
    for ceiling in [false, true] {
        let mut w = new_world();
        let e = body(
            &mut w,
            Position {
                y: 0.5,
                ..Default::default()
            },
            Velocity {
                x: 30.0,
                ..Default::default()
            },
        );
        w.get_mut::<Grounded>(e).unwrap().is_grounded = true;
        w.spawn((
            Collidable,
            Position {
                x: 1.0,
                y: 0.1,
                z: 0.0,
            },
            Hitbox {
                width: 1.0,
                height: 0.2,
                depth: 1.0,
            },
        ));
        if ceiling {
            w.spawn((
                Collidable,
                Position {
                    x: 0.5,
                    y: 1.3,
                    z: 0.0,
                },
                Hitbox {
                    width: 2.0,
                    height: 0.2,
                    depth: 2.0,
                },
            ));
        }
        run(&mut w, physics_prediction_systems);
        if ceiling {
            assert!(w.get::<Position>(e).unwrap().y < 0.6);
        } else {
            close(w.get::<Position>(e).unwrap().y, 0.7 + EPSILON);
        }
    }
}

#[test]
fn grid_includes_negative_half_extents_and_removes_stale_cells() {
    let mut w = new_world();
    let e = w.spawn((Collidable, Position::default(), cube())).id();
    run(&mut w, rebuild_spatial_grid_system);
    assert!(w.resource::<SpatialGrid>().cells[&(-1, -1, -1)].contains(&e));
    w.despawn(e);
    run(&mut w, rebuild_spatial_grid_system);
    assert!(w.resource::<SpatialGrid>().cells.is_empty());
}

#[test]
fn triggers_enter_stay_exit_and_cleanup() {
    let mut w = new_world();
    let actor = w
        .spawn((Position::default(), cube(), TriggerTracker::default()))
        .id();
    let trigger = w.spawn((Trigger, Position::default(), cube())).id();
    run(&mut w, trigger_detection_system);
    let events: Vec<_> = w.query::<&OnTriggerEnter>().iter(&w).collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].actor, actor);
    assert_eq!(events[0].target_entity, trigger);
    run(
        &mut w,
        (cleanup_trigger_events_system, trigger_detection_system).chain(),
    );
    assert_eq!(w.query::<&OnTriggerStay>().iter(&w).count(), 1);
    assert_eq!(w.query::<&OnTriggerEnter>().iter(&w).count(), 0);
    w.get_mut::<Position>(actor).unwrap().x = 5.0;
    run(
        &mut w,
        (cleanup_trigger_events_system, trigger_detection_system).chain(),
    );
    assert_eq!(w.query::<&OnTriggerExit>().iter(&w).count(), 1);
    run(&mut w, cleanup_trigger_events_system);
    assert_eq!(w.query::<&OnTriggerExit>().iter(&w).count(), 0);
}

#[test]
fn moving_platform_carries_without_accumulating_velocity_and_transfers_once() {
    let mut w = new_world();
    let platform = w
        .spawn((
            Collidable,
            Velocity {
                x: 6.0,
                ..Default::default()
            },
        ))
        .id();
    let e = w
        .spawn((
            Player,
            Position::default(),
            Velocity::default(),
            Grounded {
                is_grounded: true,
                platform: Some(platform),
            },
        ))
        .id();
    for _ in 0..10 {
        run(&mut w, moving_platform_system);
    }
    close(w.get::<Position>(e).unwrap().x, 1.0);
    close(w.get::<Velocity>(e).unwrap().x, 0.0);
    w.get_mut::<Grounded>(e).unwrap().is_grounded = false;
    run(&mut w, moving_platform_system);
    run(&mut w, moving_platform_system);
    close(w.get::<Velocity>(e).unwrap().x, 6.0);
    assert_eq!(w.get::<Grounded>(e).unwrap().platform, None);
}

#[test]
fn ray_handles_parallel_inside_range_and_non_unit_direction() {
    let mut ray = Ray {
        origin: Position::default(),
        direction: (2.0, 0.0, 0.0),
        length: 10.0,
    };
    let target = Position {
        x: 3.0,
        ..Default::default()
    };
    close(ray_aabb_intersection(&ray, &target, &cube()).unwrap(), 2.5);
    ray.length = 2.0;
    assert!(ray_aabb_intersection(&ray, &target, &cube()).is_none());
    ray.length = 10.0;
    ray.origin.y = 5.0;
    assert!(ray_aabb_intersection(&ray, &target, &cube()).is_none());
    ray.origin = target;
    close(ray_aabb_intersection(&ray, &target, &cube()).unwrap(), 0.0);
    ray.direction = (0.0, 0.0, 0.0);
    assert!(ray_aabb_intersection(&ray, &target, &cube()).is_none());
}

#[test]
fn raycast_returns_nearest_allowed_entity() {
    let mut w = new_world();
    let mut near = None;
    for (x, filter) in [(8.0, 1), (3.0, 1), (1.0, 0)] {
        let e = w
            .spawn((
                Collidable,
                Position {
                    x,
                    ..Default::default()
                },
                cube(),
                CollisionGroups {
                    memberships: 1,
                    filters: filter,
                },
            ))
            .id();
        if x == 3.0 {
            near = Some(e);
        }
    }
    run(&mut w, rebuild_spatial_grid_system);
    let mut state: SystemState<Query<(Entity, &Position, &Hitbox, &CollisionGroups)>> =
        SystemState::new(&mut w);
    let query = state.get(&w);
    let hit = shoot_ray_system(
        w.resource::<SpatialGrid>(),
        &Ray {
            origin: Position::default(),
            direction: (1.0, 0.0, 0.0),
            length: 10.0,
        },
        &CollisionGroups {
            memberships: 1,
            filters: 1,
        },
        &query,
    )
    .unwrap();
    assert_eq!(Some(hit.0), near);
    close(hit.1, 2.5);
}

#[test]
fn damage_mitigation_death_effects_and_hit_stop() {
    let mut w = new_world();
    let e = w
        .spawn((
            Position::default(),
            Health {
                current: 20.0,
                max: 20.0,
                invulnerable: false,
            },
            DamageResistance {
                physical: 0.5,
                fire: 0.0,
                ice: 0.0,
                electric: 0.0,
            },
        ))
        .id();
    w.resource_mut::<DamageRequests>()
        .events
        .push(DamageRequest {
            target: e,
            amount: 20.0,
            damage_type: DamageType::Physical,
        });
    run(&mut w, health_system);
    close(w.get::<Health>(e).unwrap().current, 10.0);
    close(w.resource::<CameraTrauma>().value, 0.3);
    assert!(w.resource::<DamageRequests>().events.is_empty());
    w.resource_mut::<DamageRequests>()
        .events
        .push(DamageRequest {
            target: e,
            amount: 30.0,
            damage_type: DamageType::Fire,
        });
    run(&mut w, health_system);
    assert!(w.get::<Dead>(e).is_some());
    close(w.get::<Health>(e).unwrap().current, 0.0);
    assert_eq!(w.query::<&DamageEffect>().iter(&w).count(), 2);
    close(w.resource::<TimeManager>().time_scale, 0.0);
    w.resource_mut::<RealTime>().delta_seconds = 0.1;
    run(&mut w, time_management_system);
    close(w.resource::<TimeManager>().time_scale, 1.0);
}

#[test]
fn invulnerability_and_all_resistance_types() {
    for (kind, invulnerable) in [
        (DamageType::Ice, false),
        (DamageType::Electric, false),
        (DamageType::Fire, true),
    ] {
        let mut w = new_world();
        let e = w
            .spawn((
                Position::default(),
                Health {
                    current: 100.0,
                    max: 100.0,
                    invulnerable,
                },
                DamageResistance {
                    physical: 0.0,
                    fire: 0.0,
                    ice: 1.0,
                    electric: 1.0,
                },
            ))
            .id();
        w.resource_mut::<DamageRequests>()
            .events
            .push(DamageRequest {
                target: e,
                amount: 50.0,
                damage_type: kind,
            });
        run(&mut w, health_system);
        close(w.get::<Health>(e).unwrap().current, 100.0);
        assert_eq!(w.query::<&DamageEffect>().iter(&w).count(), 0);
    }
}

#[test]
fn animation_queues_current_state_and_replaces_old_tasks() {
    let mut w = new_world();
    let e = w
        .spawn((
            Velocity::default(),
            Grounded {
                is_grounded: true,
                platform: None,
            },
            AnimationController {
                name: "player".into(),
                state: AnimationState::Idle,
                blend_time: 0.0,
            },
        ))
        .id();
    run(&mut w, animation_task_graph_system);
    assert_eq!(
        w.resource::<AnimationTaskGraph>().tasks[0].target_state,
        AnimationState::Idle
    );
    w.get_mut::<Velocity>(e).unwrap().x = 2.0;
    run(&mut w, animation_task_graph_system);
    assert_eq!(w.resource::<AnimationTaskGraph>().tasks.len(), 1);
    assert_eq!(
        w.get::<AnimationController>(e).unwrap().state,
        AnimationState::Run
    );
    w.get_mut::<Grounded>(e).unwrap().is_grounded = false;
    run(&mut w, animation_task_graph_system);
    assert_eq!(
        w.get::<AnimationController>(e).unwrap().state,
        AnimationState::Fall
    );
    w.entity_mut(e).insert(Dead);
    run(&mut w, animation_task_graph_system);
    assert_eq!(
        w.get::<AnimationController>(e).unwrap().state,
        AnimationState::Die
    );
}

#[test]
fn camera_shake_is_bounded_and_resets_when_trauma_expires() {
    let mut w = new_world();
    let e = w
        .spawn((
            MainCamera,
            CameraOffset {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        ))
        .id();
    w.resource_mut::<CameraTrauma>().value = 1.0;
    run(&mut w, camera_shake_system);
    let offset = w.get::<CameraOffset>(e).unwrap();
    assert!(offset.x.abs() <= 0.5 && offset.y.abs() <= 0.5);
    w.resource_mut::<RealTime>().delta_seconds = 2.0;
    run(&mut w, camera_shake_system);
    let offset = w.get::<CameraOffset>(e).unwrap();
    close(offset.x, 0.0);
    close(offset.y, 0.0);
}

#[test]
fn frame_accumulator_waits_for_complete_ticks() {
    let mut w = new_world();
    let e = body(
        &mut w,
        Position::default(),
        Velocity {
            x: 60.0,
            ..Default::default()
        },
    );
    let mut schedule = physics_schedule();
    advance_frame(&mut w, &mut schedule, FixedTime::TIMESTEP / 2.0);
    close(w.get::<Position>(e).unwrap().x, 0.0);
    advance_frame(&mut w, &mut schedule, FixedTime::TIMESTEP / 2.0);
    close(w.get::<Position>(e).unwrap().x, 1.0);
}
