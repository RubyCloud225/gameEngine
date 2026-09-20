//! A headless Bevy ECS playground. Positions are centres; hitboxes are full dimensions.
pub mod components;
pub mod resources;
pub mod systems;

use bevy_ecs::prelude::*;
use components::*;
use resources::*;

/// Resources needed by the existing systems, with a 60 Hz simulation step.
pub fn new_world() -> World {
    let mut world = World::new();
    world.insert_resource(FixedTime::default());
    world.insert_resource(RealTime {
        delta_seconds: FixedTime::TIMESTEP,
    });
    world.insert_resource(Time {
        delta_seconds: FixedTime::TIMESTEP,
    });
    world.insert_resource(InputState {
        movement_direction: (0.0, 0.0),
        is_sprinting: false,
    });
    world.insert_resource(PhysicsSettings {
        gravity_acceleration: -9.81,
    });
    world.insert_resource(CollisionPairs::default());
    world.insert_resource(SpatialGrid::default());
    world.insert_resource(StepConfig::default());
    world.insert_resource(CameraTrauma::default());
    world.insert_resource(TimeManager::default());
    world.insert_resource(DamageRequests::default());
    world.insert_resource(AnimationTaskGraph::default());
    world
}

/// One fixed tick. Bodies need FixedPhysics, Position, Velocity, Acceleration,
/// Hitbox and Grounded. Do not add movement as well: that would integrate twice.
pub fn physics_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            systems::cleanup_trigger_events_system,
            systems::input_to_velocity,
            systems::moving_platform_system,
            systems::gravity_system,
            systems::physics_prediction_systems,
            systems::rebuild_spatial_grid_system,
            systems::broad_phase_system,
            systems::trigger_detection_system,
            systems::health_system,
            systems::animation_task_graph_system,
        )
            .chain(),
    );
    schedule
}

/// Advance using unscaled real time. Run camera and hit-stop once per rendered frame.
pub fn advance_frame(world: &mut World, physics: &mut Schedule, delta_seconds: f32) {
    assert!(delta_seconds.is_finite() && delta_seconds >= 0.0);
    world.resource_mut::<RealTime>().delta_seconds = delta_seconds;
    let mut frame = Schedule::default();
    frame.add_systems(
        (
            systems::time_management_system,
            systems::accumulate_time_system,
        )
            .chain(),
    );
    frame.run(world);
    while world.resource::<FixedTime>().accumulator >= FixedTime::TIMESTEP {
        world.resource_mut::<FixedTime>().accumulator -= FixedTime::TIMESTEP;
        physics.run(world);
    }
    let mut effects = Schedule::default();
    effects.add_systems(systems::camera_shake_system);
    effects.run(world);
}
