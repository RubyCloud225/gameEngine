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

        // check if the component type is already registered
        if let Some(&id) = self.types.get(&type_id) {
            return id;
        }
        // assign a new ComponentId to this component type
        let id = ComponentId(self.next_id);
        self.next_id += 1;

        // store the new ComponentId in the registry
        self.types.insert(type_id, id);
        id
    }

    /// Retrieves the ComponentId for a given component type, if it is registered.
    pub fn get<T: 'static>(&self) -> Option<ComponentId> {
        self.types.get(&TypeId::of::<T>()).copied()
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
    #[allow(dead_code)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[allow(dead_code)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    #[allow(dead_code)]
    struct Health {
        current: u32,
        max: u32,
    }

    #[allow(dead_code)]
    struct MovementIntent {
        value: f32,
    }

    #[allow(dead_code)]
    struct Perception {
        range: f32,
        fov: f32,
    }

    #[allow(dead_code)]
    struct Agent {
        id: u32,
        name: String,
    }

    #[test]
    fn test_component_registry() {
        let mut registry = ComponentRegistry::new();
        let position_id = registry.register::<Position>();
        let velocity_id = registry.register::<Velocity>();
        let health_id = registry.register::<Health>();
        let movement_intent_id = registry.register::<MovementIntent>();
        let perception_id = registry.register::<Perception>();
        let agent_id = registry.register::<Agent>();

        assert_eq!(registry.get::<Position>(), Some(position_id));
        assert_eq!(registry.get::<Velocity>(), Some(velocity_id));
        assert_eq!(registry.get::<Health>(), Some(health_id));
        assert_eq!(registry.get::<MovementIntent>(), Some(movement_intent_id));
        assert_eq!(registry.get::<Perception>(), Some(perception_id));
        assert_eq!(registry.get::<Agent>(), Some(agent_id));
    }

    #[test]
    fn test_register_same_component_twice() {
        let mut registry = ComponentRegistry::new();
        let position_id1 = registry.register::<Position>();
        let position_id2 = registry.register::<Position>();
        let velocity_id1 = registry.register::<Velocity>();
        let velocity_id2 = registry.register::<Velocity>();
        let health_id1 = registry.register::<Health>();
        let health_id2 = registry.register::<Health>();
        let movement_intent_id1 = registry.register::<MovementIntent>();
        let movement_intent_id2 = registry.register::<MovementIntent>();
        let perception_id1 = registry.register::<Perception>();
        let perception_id2 = registry.register::<Perception>();
        let agent_id1 = registry.register::<Agent>();
        let agent_id2 = registry.register::<Agent>();
        assert_eq!(health_id1, health_id2);
        assert_eq!(velocity_id1, velocity_id2);
        assert_eq!(position_id1, position_id2);
        assert_eq!(movement_intent_id1, movement_intent_id2);
        assert_eq!(perception_id1, perception_id2);
        assert_eq!(agent_id1, agent_id2);
    }

    #[test]
    fn test_get_unregistered_component() {
        let mut registry = ComponentRegistry::new();
        assert_eq!(registry.get::<Position>(), None);
        assert_eq!(registry.get::<Velocity>(), None);
        assert_eq!(registry.get::<Health>(), None);
        assert_eq!(registry.get::<MovementIntent>(), None);
        assert_eq!(registry.get::<Perception>(), None);
        assert_eq!(registry.get::<Agent>(), None);
    }

    #[test]
    fn component_types_recieve_unique_ids() {
        let mut registry = ComponentRegistry::new();
        let position_id = registry.register::<Position>();
        let velocity_id = registry.register::<Velocity>();
        let health_id = registry.register::<Health>();
        let movement_intent_id = registry.register::<MovementIntent>();
        let perception_id = registry.register::<Perception>();
        let agent_id = registry.register::<Agent>();

        assert_ne!(position_id, velocity_id);
        assert_ne!(position_id, health_id);
        assert_ne!(position_id, movement_intent_id);
        assert_ne!(position_id, perception_id);
        assert_ne!(position_id, agent_id);
        assert_ne!(velocity_id, health_id);
        assert_ne!(velocity_id, movement_intent_id);
        assert_ne!(velocity_id, perception_id);
        assert_ne!(velocity_id, agent_id);
        assert_ne!(health_id, movement_intent_id);
        assert_ne!(health_id, perception_id);
        assert_ne!(health_id, agent_id);
        assert_ne!(movement_intent_id, perception_id);
        assert_ne!(movement_intent_id, agent_id);
        assert_ne!(perception_id, agent_id);
    }

    #[test]
    fn component_ids_remain_stable() {
        let mut registry = ComponentRegistry::new();
        let position_id = registry.register::<Position>();
        let velocity_id = registry.register::<Velocity>();
        let health_id = registry.register::<Health>();
        let movement_intent_id = registry.register::<MovementIntent>();
        let perception_id = registry.register::<Perception>();
        let agent_id = registry.register::<Agent>();

        assert_eq!(registry.get::<Position>(), Some(position_id));
        assert_eq!(registry.get::<Velocity>(), Some(velocity_id));
        assert_eq!(registry.get::<Health>(), Some(health_id));
        assert_eq!(registry.get::<MovementIntent>(), Some(movement_intent_id));
        assert_eq!(registry.get::<Perception>(), Some(perception_id));
        assert_eq!(registry.get::<Agent>(), Some(agent_id));
    }
}
