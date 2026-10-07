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
    /// Registers a component type and returns its unique ComponentId.
    pub fn register<T: 'static>(&mut self) -> ComponentId {
        let type_id = TypeId::of::<T>();
        if let Some(&id) = self.types.get(&type_id) {
            return id;
        }
        let id = ComponentId(self.next_id);
        self.next_id += 1;
        self.types.insert(type_id, id);
        id
    }
    /// Retrieves the ComponentId for a registered component type, if it exists.
    pub fn get<T: 'static>(&self) -> Option<ComponentId> {
        let type_id = TypeId::of::<T>();
        self.types.get(&type_id).copied()
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

    struct Velocity {
        x: f32,
        y: f32,
    }

    struct Health {
        current: i32,
        max: i32,
    }
    #[test]
    fn test_component_registry() {
        let mut registry = ComponentRegistry::new();
        let position_id = registry.register::<Position>();
        let velocity_id = registry.register::<Velocity>();
        let health_id = registry.register::<Health>();

        assert_eq!(registry.get::<Position>(), Some(position_id));
        assert_eq!(registry.get::<Velocity>(), Some(velocity_id));
        assert_eq!(registry.get::<Health>(), Some(health_id));
        assert_eq!(registry.get::<u32>(), None);
    }

    #[test]
    fn test_register_same_type_returns_same_id() {
        let mut registry = ComponentRegistry::new();
        let position_id1 = registry.register::<Position>();
        let position_id2 = registry.register::<Position>();
        assert_eq!(position_id1, position_id2);
    }
    #[test]
    fn test_register_different_types_returns_different_ids() {
        let mut registry = ComponentRegistry::new();
        let position_id = registry.register::<Position>();
        let velocity_id = registry.register::<Velocity>();
        assert_ne!(position_id, velocity_id);
    }
    #[test]
    fn test_get_unregistered_type_returns_none() {
        let registry = ComponentRegistry::new();
        assert_eq!(registry.get::<Position>(), None);
    }
    #[test]
    fn test_get_registered_type_returns_some() {
        let mut registry = ComponentRegistry::new();
        let position_id = registry.register::<Position>();
        assert_eq!(registry.get::<Position>(), Some(position_id));
    }
    #[test]
    fn test_register_and_get_multiple_types() {
        let mut registry = ComponentRegistry::new();
        let position_id = registry.register::<Position>();
        let velocity_id = registry.register::<Velocity>();
        let health_id = registry.register::<Health>();

        assert_eq!(registry.get::<Position>(), Some(position_id));
        assert_eq!(registry.get::<Velocity>(), Some(velocity_id));
        assert_eq!(registry.get::<Health>(), Some(health_id));
    }
    #[test]
    fn test_health_component_registration() {
        let mut registry = ComponentRegistry::new();
        let health_id = registry.register::<Health>();
        assert_eq!(registry.get::<Health>(), Some(health_id));
    }
}
