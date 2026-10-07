use crate::component::Component;
use crate::entity::Entity;

use std::collections::HashMap;

/// Stores one component type contiguously.
///
/// EG:
/// - Storage<Position>
/// - Storage<Velocity>
/// - Storage<Health>
///
/// This can only have one component
#[derive(Debug)]
pub struct Storage<T: Component> {
    // Packed component data.
    components: Vec<T>,
    // Entity associated with each component slot.
    entities: Vec<Entity>,
    // Maps entity ID -> component index,
    // This gives fast looki[ without scanning the component array.
    indices: HashMap<u32, usize>,
}

impl<T: Component> Storage<T> {
    // Create an empty component storage
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            entities: Vec::new(),
            indices: HashMap::new(),
        }
    }

    // Insert or replace
}

impl<T: Component> Default for Storage<T> {
    fn default() -> Self {
        Self::new()
    }
}
