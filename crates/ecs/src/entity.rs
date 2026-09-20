/// Handle used to identify an entity inside the ECS.
///
/// `id` identifies the slot
/// `generation` identifies the current lifetime of that slot
///
/// The generation prevents stale handles from becoming valid again
/// when the old entity ID is recycled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

impl Entity {
    /// Creates a new entity handle from ID and generation
    pub const fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }
}

/// Internal state for one entity slot
///
/// This is not exposed outside the entity manager
///
/// eg:
///     slot 5;
///     generation = 2
///     alive = true
/// means: Entity {id: 5, generation: 2}
///
/// is currently valid
#[derive(Debug, Clone)]
struct EntitySlot {
    generation: u32,
    alive: bool,
}

/// Owns entity creation, destruction and ID reuse.
///
/// The manager keeps:
/// - a contiguous list of entity slot
/// - a free-list of IDs that may be reused
/// - the total number of live entities.
///
/// Entity IDs are intentionally recycled to avoid endlessly increasing IDs.
///
/// Generation counters make this safe.
#[derive(Debug, Default)]
pub struct EntityManager {
    /// One slot for every entity ID that has ever been allocated
    slots: Vec<EntitySlot>,

    /// IDs belonging to destroyed entities
    /// can be reused by future entities
    free_ids: Vec<u32>,
    alive_count: usize,
}

impl EntityManager {
    /// Create an empty manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether this handle identifies a live entity of the same generation.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.slots
            .get(entity.id as usize)
            .is_some_and(|slot| slot.alive && slot.generation == entity.generation)
    }

    /// Returns the number of currently live entities.
    pub fn alive_count(&self) -> usize {
        self.alive_count
    }

    /// Returns the number of allocated slots, including destroyed entities' slots.
    /// This is not the backing vector's reserved memory capacity.
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// Create a new entity
    ///
    /// The manager first attempts to reuse an ID belonging to a
    /// previously destroyed entity
    ///
    /// If no reusable ID exists, a new slot is appended.
    pub fn create(&mut self) -> Entity {
        // Reuse a previously destroyed entity slot where possible.
        match self.free_ids.pop() {
            Some(id) => {
                let slot = &mut self.slots[id as usize];
                debug_assert!(!slot.alive);
                slot.alive = true;
                self.alive_count += 1;
                Entity::new(id, slot.generation)
            }
            None => {
                // No reusable slot: allocate a new entity ID.
                let id = u32::try_from(self.slots.len()).expect("entity ID capacity exhausted");
                self.slots.push(EntitySlot {
                    generation: 0,
                    alive: true,
                });
                self.alive_count += 1;
                Entity::new(id, 0)
            }
        }
    }

    /// Destroy an Entity
    ///
    /// Returns `true` when the entity was valid and successfully destroyed
    ///
    /// Returns `false` when:
    ///
    /// - the ID does not exist;
    /// - the entity was already destroyed;
    /// - the generation is stale.
    ///
    /// Destruction increments the slot generation before making the
    /// ID available for reuse.
    pub fn destroy(&mut self, entity: Entity) -> bool {
        // reject some IDs that were never allocated
        let Some(slot) = self.slots.get_mut(entity.id as usize) else {
            return false;
        };
        // Reject stale handles and entities that are already dead
        if !slot.alive || slot.generation != entity.generation {
            return false;
        }

        slot.alive = false;

        // Incrementing the generation invalidates all old handles
        // referring to this slot
        slot.generation = slot.generation.wrapping_add(1);
        // The numeric ID may now be recycled
        self.free_ids.push(entity.id);
        self.alive_count -= 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destroyed_ids_are_reused_with_a_new_generation() {
        let mut manager = EntityManager::new();
        let old = manager.create();
        assert!(manager.destroy(old));
        assert_eq!(manager.alive_count, 0);
        assert!(!manager.destroy(old));
        let replacement = manager.create();
        assert_eq!(replacement.id, old.id);
        assert_eq!(replacement.generation, old.generation + 1);
        assert!(!manager.destroy(old));
        assert_eq!(manager.alive_count, 1);
        assert!(manager.destroy(replacement));
    }

    #[test]
    fn invalid_handles_do_not_change_live_entities() {
        let mut manager = EntityManager::new();
        let entity = manager.create();
        assert!(!manager.destroy(Entity::new(entity.id + 1, 0)));
        assert!(!manager.destroy(Entity::new(entity.id, entity.generation + 1)));
        assert_eq!(manager.alive_count, 1);
        assert!(manager.free_ids.is_empty());
        assert!(manager.destroy(entity));
    }

    #[test]
    fn creates_entity() {
        let mut manager = EntityManager::new();
        let entity = manager.create();
        assert_eq!(entity.id, 0);
        assert_eq!(entity.generation, 0);
        assert!(manager.is_alive(entity));
        assert_eq!(manager.alive_count(), 1);
    }

    #[test]
    fn creates_unique_live_entities() {
        let mut manager = EntityManager::new();
        let first = manager.create();
        let second = manager.create();
        assert_ne!(first.id, second.id);
        assert!(manager.is_alive(first));
        assert!(manager.is_alive(second));
        assert_eq!(manager.alive_count(), 2);
    }
    #[test]
    fn destroys_entity() {
        let mut manager = EntityManager::new();

        let entity = manager.create();

        let result = manager.destroy(entity);

        assert!(result);
        assert!(!manager.is_alive(entity));
        assert_eq!(manager.alive_count(), 0);
    }

    #[test]
    fn cannot_destroy_entity_twice() {
        let mut manager = EntityManager::new();

        let entity = manager.create();

        assert!(manager.destroy(entity));

        // Second attempt should fail rather than panic.
        assert!(!manager.destroy(entity));

        assert_eq!(manager.alive_count(), 0);
    }

    #[test]
    fn reuses_destroyed_entity_id() {
        let mut manager = EntityManager::new();

        let first = manager.create();

        assert!(manager.destroy(first));

        let second = manager.create();

        // Numeric ID is recycled.
        assert_eq!(first.id, second.id);

        // But the new entity has a different generation.
        assert_ne!(first.generation, second.generation);

        assert!(manager.is_alive(second));
    }

    #[test]
    fn stale_entity_handle_is_invalid() {
        let mut manager = EntityManager::new();

        let old_entity = manager.create();

        assert!(manager.destroy(old_entity));

        let new_entity = manager.create();

        // The slot has been reused...
        assert_eq!(old_entity.id, new_entity.id);

        // ...but the old handle must remain invalid.
        assert!(!manager.is_alive(old_entity));
        assert!(manager.is_alive(new_entity));
    }

    #[test]
    fn invalid_entity_id_is_not_alive() {
        let manager = EntityManager::new();

        let fake_entity = Entity::new(999, 0);

        assert!(!manager.is_alive(fake_entity));
    }

    #[test]
    fn cannot_destroy_invalid_entity_id() {
        let mut manager = EntityManager::new();

        let fake_entity = Entity::new(999, 0);

        assert!(!manager.destroy(fake_entity));
        assert_eq!(manager.alive_count(), 0);
    }

    #[test]
    fn alive_count_tracks_entity_lifecycle() {
        let mut manager = EntityManager::new();

        let first = manager.create();
        let second = manager.create();
        let third = manager.create();

        assert_eq!(manager.alive_count(), 3);

        assert!(manager.destroy(second));

        assert_eq!(manager.alive_count(), 2);

        assert!(manager.destroy(first));
        assert!(manager.destroy(third));

        assert_eq!(manager.alive_count(), 0);
    }

    #[test]
    fn capacity_tracks_allocated_slots() {
        let mut manager = EntityManager::new();

        let first = manager.create();
        let _second = manager.create();

        assert_eq!(manager.capacity(), 2);

        assert!(manager.destroy(first));

        // Destroying an entity does not remove its slot.
        assert_eq!(manager.capacity(), 2);

        let _third = manager.create();

        // The old slot should be reused rather than allocating another one.
        assert_eq!(manager.capacity(), 2);
    }

    #[test]
    fn repeatedly_recycles_entity_slot_safely() {
        let mut manager = EntityManager::new();

        let mut entity = manager.create();

        for _ in 0..1000 {
            let old_entity = entity;

            assert!(manager.is_alive(old_entity));
            assert!(manager.destroy(old_entity));
            assert!(!manager.is_alive(old_entity));

            entity = manager.create();

            assert_eq!(old_entity.id, entity.id);
            assert_ne!(old_entity.generation, entity.generation);
            assert!(manager.is_alive(entity));
        }

        assert_eq!(manager.alive_count(), 1);
        assert_eq!(manager.capacity(), 1);
    }
}
