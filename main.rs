#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct PhysicsSet;


// Configure the Physics step order
schedule.add_systems(
    (
        systems::gravity_system, // 1. apply gravity
        systems::collision_detection_system, // check for collisions and resolve forces (Velocity zeroing)
        systems::movement,
    )
    // ensure all three run as part of the fixed physics schedule in this exact order
    .chain()
    .in_set(PhysicsSet)   
)



