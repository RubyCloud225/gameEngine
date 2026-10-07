use std::collections::HashMap;
use std::{
    any::{Any, TypeId},
    hash::Hash,
};

/// Marker trait for data that can be stored as an ECS component.
/// Components should primarily contain data rather than behaviour.
pub trait Component: Any + Send + Sync + 'static {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(u32);

#[derive(Debug, Default)]
pub struct ComponentRegistry {
    types: HashMap<TypeId, ComponentId>,
    next_id: u32,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}

// Automatically make any compatible type a Component
//
// This means you do not need to manually implement Component
// for every struct such as Position, Velocity or Health.
impl<T> Component for T where T: Any + Send + Sync + 'static {}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position {
        x: f32,
        y: f32,
    }
}
