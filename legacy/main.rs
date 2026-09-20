use game_engine::{
    advance_frame, components::*, new_world, physics_schedule, resources::FixedTime,
};

fn main() {
    let mut world = new_world();
    let body = world
        .spawn((
            Player,
            FixedPhysics,
            AffectedByGravity,
            Position {
                x: 0.0,
                y: 3.0,
                z: 0.0,
            },
            Velocity::default(),
            Acceleration::default(),
            Hitbox {
                width: 1.0,
                height: 1.0,
                depth: 1.0,
            },
            Grounded::default(),
        ))
        .id();
    world.spawn((
        Collidable,
        Position {
            x: 0.0,
            y: -0.5,
            z: 0.0,
        },
        Hitbox {
            width: 20.0,
            height: 1.0,
            depth: 20.0,
        },
    ));
    let mut physics = physics_schedule();
    for _ in 0..120 {
        advance_frame(&mut world, &mut physics, FixedTime::TIMESTEP);
    }
    println!(
        "After 120 ticks: {:?}; grounded={}",
        world.get::<Position>(body).unwrap(),
        world.get::<Grounded>(body).unwrap().is_grounded
    );
}
