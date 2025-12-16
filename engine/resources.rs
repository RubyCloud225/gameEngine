// Resources : Global mutable container accessed safely by systems.
#[derive(Resource)]
pub struct FixedTime {
    // The target duration for one physics tick (1.0 / 60.0 seconds)
    pub const TIMESTEP: f32 = 0.01666667;
    // Time accumulated from variable delta_time
    pub accumulator: f32,
}

#[derive(Resource)]
pub struct RealTime {
    pub delta_seconds: f32,
}

#[derive(Resource)]
pub struct InputState {
    // A vector normalised to 1.0 in the direction of intended movement
    pub movement_direction: (f32, f32),
    pub is_sprinting: bool,
}

//Gravitational Acceleration (negative since y is up in games)
#[derive(Resource)]
pub struct PhysicsSettings {
    //gravitational acceleration
    pub gravity_acceleration: f32,
    // TODO damping, air density, motion
}

#[derive(Resource)]
pub struct CollisionPairs {
    // TODO
}