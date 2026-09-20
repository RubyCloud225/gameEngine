# ECS Entity Management

## 1. Purpose

Entity management provides stable identifiers for objects stored in the ECS.

An entity is only an identifier.

It does not contain gameplay data and it does not contain behaviour.

The entity system is responsible for:

* creating entities;
* recycling entity IDs;
* tracking generations;
* destroying entities;
* rejecting stale handles;
* reporting whether an entity is alive.

Entity management must remain independent of:

* rendering;
* physics;
* AI;
* GPU execution;
* component storage implementation details.

---

# 2. Entity Representation

Use a compact entity handle containing:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}
```

The `id` identifies the slot.

The `generation` identifies which lifetime of that slot the handle belongs to.

Example:

```text
Entity {
    id: 42,
    generation: 3,
}
```

means:

> slot 42, third generation of that slot.

---

# 3. Why Generation Tracking Exists

Without generations, recycled IDs can create invalid references.

Example:

```text
Create entity
    ↓
Entity { id: 7 }
    ↓
Destroy entity 7
    ↓
Create new entity
    ↓
ID 7 reused
```

A system holding the old entity reference would now accidentally refer to the new entity.

Generation tracking prevents this.

Example:

```text
Old entity
Entity { id: 7, generation: 2 }

Destroy

New entity
Entity { id: 7, generation: 3 }
```

The old handle remains:

```text
Entity { id: 7, generation: 2 }
```

and is therefore invalid.

---

# 4. Entity Allocator

The allocator owns entity slots.

Initial design:

```rust
pub struct EntityAllocator {
    generations: Vec<u32>,
    free_ids: Vec<u32>,
    alive_count: usize,
}
```

Responsibilities:

```text
generations
    stores the current generation for every slot

free_ids
    stores reusable entity IDs

alive_count
    tracks the number of currently active entities
```

---

# 5. Allocation Behaviour

## New Slot

If there are no free IDs:

```text
generations.len() = 5
```

allocate:

```text
id = 5
generation = 0
```

and append:

```text
generations.push(0)
```

---

## Recycled Slot

If:

```text
free_ids = [3]
```

allocation should reuse:

```text
id = 3
```

with whatever generation is currently stored for slot 3.

Example:

```text
generations[3] = 2
```

returns:

```rust
Entity {
    id: 3,
    generation: 2,
}
```

---

# 6. Entity Creation

Conceptual interface:

```rust
impl EntityAllocator {
    pub fn create(&mut self) -> Entity;
}
```

Possible implementation:

```rust
pub fn create(&mut self) -> Entity {
    let id = match self.free_ids.pop() {
        Some(id) => id,
        None => {
            let id = self.generations.len() as u32;
            self.generations.push(0);
            id
        }
    };

    self.alive_count += 1;

    Entity {
        id,
        generation: self.generations[id as usize],
    }
}
```

---

# 7. Entity Destruction

Conceptual interface:

```rust
pub fn destroy(&mut self, entity: Entity) -> bool;
```

The allocator must first validate the handle.

If valid:

```text
1. Increment generation
2. Add ID to free list
3. Decrement alive count
4. Return true
```

If stale or invalid:

```text
Return false
```

Possible implementation:

```rust
pub fn destroy(&mut self, entity: Entity) -> bool {
    if !self.is_alive(entity) {
        return false;
    }

    let generation = &mut self.generations[entity.id as usize];

    *generation = generation.wrapping_add(1);

    self.free_ids.push(entity.id);
    self.alive_count -= 1;

    true
}
```

---

# 8. Alive Check

Conceptual interface:

```rust
pub fn is_alive(&self, entity: Entity) -> bool;
```

Validation requires:

```text
entity.id exists
AND
entity.generation == generations[entity.id]
AND
entity.id is not currently free
```

This last condition matters.

Generation equality alone is not sufficient immediately after creation/destruction edge cases unless active state is tracked explicitly.

---

# 9. Recommended Slot State

To make lifecycle state explicit, use:

```rust
struct EntitySlot {
    generation: u32,
    alive: bool,
}
```

Then:

```rust
pub struct EntityAllocator {
    slots: Vec<EntitySlot>,
    free_ids: Vec<u32>,
    alive_count: usize,
}
```

This is clearer than deriving alive state indirectly.

Recommended first implementation:

```rust
#[derive(Debug, Clone)]
struct EntitySlot {
    generation: u32,
    alive: bool,
}
```

---

# 10. Recommended Allocator Design

```rust
pub struct EntityAllocator {
    slots: Vec<EntitySlot>,
    free_ids: Vec<u32>,
    alive_count: usize,
}
```

Creation:

```rust
pub fn create(&mut self) -> Entity {
    if let Some(id) = self.free_ids.pop() {
        let slot = &mut self.slots[id as usize];
        slot.alive = true;

        self.alive_count += 1;

        return Entity {
            id,
            generation: slot.generation,
        };
    }

    let id = self.slots.len() as u32;

    self.slots.push(EntitySlot {
        generation: 0,
        alive: true,
    });

    self.alive_count += 1;

    Entity {
        id,
        generation: 0,
    }
}
```

Destruction:

```rust
pub fn destroy(&mut self, entity: Entity) -> bool {
    let Some(slot) = self.slots.get_mut(entity.id as usize) else {
        return false;
    };

    if !slot.alive || slot.generation != entity.generation {
        return false;
    }

    slot.alive = false;
    slot.generation = slot.generation.wrapping_add(1);

    self.free_ids.push(entity.id);
    self.alive_count -= 1;

    true
}
```

Alive check:

```rust
pub fn is_alive(&self, entity: Entity) -> bool {
    self.slots
        .get(entity.id as usize)
        .map(|slot| {
            slot.alive &&
            slot.generation == entity.generation
        })
        .unwrap_or(false)
}
```

---

# 11. Public Interface

The first entity allocator should expose only:

```rust
impl EntityAllocator {
    pub fn new() -> Self;

    pub fn create(&mut self) -> Entity;

    pub fn destroy(&mut self, entity: Entity) -> bool;

    pub fn is_alive(&self, entity: Entity) -> bool;

    pub fn alive_count(&self) -> usize;

    pub fn capacity(&self) -> usize;
}
```

Avoid expanding this API until component storage requires it.

---

# 12. Error Behaviour

Entity creation should not normally fail.

Entity destruction should not panic if:

* the ID is outside the allocator;
* the entity has already been destroyed;
* the generation is stale.

Instead:

```rust
destroy(...) -> false
```

This makes invalid lifecycle operations testable without crashing the engine.

For debugging builds, optional assertions or diagnostics can be added later.

---

# 13. Generation Overflow

Generation uses:

```rust
u32
```

A slot would need to be destroyed over four billion times before wrapping.

For the first implementation:

```rust
wrapping_add(1)
```

is acceptable.

Later debug builds may detect wraparound explicitly if desired.

---

# 14. Threading

Entity allocation should initially be single-writer.

Do not make the allocator lock-free yet.

Initial rule:

```text
Entity creation/destruction
        ↓
Structural ECS mutation phase
        ↓
Single controlled execution point
```

Systems may later queue entity creation/destruction commands.

This avoids unnecessary synchronisation in the first ECS implementation.

---

# 15. Future Command Buffer

Eventually systems should not mutate entity structure directly while parallel queries are running.

Instead:

```text
System threads
    │
    ├── Spawn command
    ├── Destroy command
    ├── Add component
    └── Remove component
            │
            ▼
       Command Buffer
            │
            ▼
      Structural Sync
            │
            ▼
          World
```

This is future work.

The entity allocator itself should not implement the command buffer.

---

# 16. Memory Model

Entity slots are contiguous:

```text
slots

0   1   2   3   4   5
│   │   │   │   │   │
▼   ▼   ▼   ▼   ▼   ▼
A   A   D   A   D   A
```

Where:

```text
A = alive
D = dead/free
```

Free IDs are stored separately:

```text
free_ids = [2, 4]
```

Creation can therefore reuse an existing slot without reallocating the entire structure.

---

# 17. Entity Equality

Two entities are equal only if both fields match:

```text
ID
AND
generation
```

Therefore:

```rust
Entity {
    id: 8,
    generation: 1,
}
```

is not equal to:

```rust
Entity {
    id: 8,
    generation: 2,
}
```

even though they use the same slot.

---

# 18. Tests

Entity management should have comprehensive tests before component work begins.

## Test 1 — Create Entity

```text
create()
    ↓
Entity is alive
```

Expected:

```text
alive_count == 1
```

---

## Test 2 — Unique IDs

Create multiple active entities.

Expected:

```text
No two live entities have the same ID.
```

---

## Test 3 — Destroy Entity

```text
create
destroy
```

Expected:

```text
is_alive == false
alive_count == 0
```

---

## Test 4 — Double Destroy

```text
create
destroy
destroy again
```

Expected:

```text
first destroy  == true
second destroy == false
```

No panic.

---

## Test 5 — Reuse ID

```text
A = create
destroy A
B = create
```

Expected:

```text
A.id == B.id
A.generation != B.generation
```

---

## Test 6 — Stale Handle

```text
A = create
destroy A
B = create
```

Expected:

```text
is_alive(A) == false
is_alive(B) == true
```

---

## Test 7 — Invalid ID

Construct an entity whose ID has never been allocated.

Expected:

```text
is_alive == false
destroy  == false
```

---

## Test 8 — Alive Count

Create 100 entities.

Destroy 35.

Expected:

```text
alive_count == 65
```

---

## Test 9 — Recycling

Repeatedly create and destroy entities.

Expected:

* allocator remains valid;
* generations change;
* IDs are safely reused;
* stale entities remain invalid.

---

# 19. Benchmarks

Initial benchmarks:

```text
Create 1,000 entities
Create 10,000 entities
Create 100,000 entities
Create 1,000,000 entities
```

Also benchmark:

```text
Create
Destroy
Recycle
is_alive lookup
```

Metrics:

```text
operations / second
allocation count
memory usage
```

These benchmarks establish the baseline before more complex ECS structures are introduced.

---

# 20. File Structure

Recommended:

```text
crates/ecs/src/
├── lib.rs
├── entity.rs
├── allocator.rs
├── component.rs
├── storage.rs
├── query.rs
└── world.rs
```

For this milestone, only:

```text
entity.rs
allocator.rs
```

need implementation.

---

# 21. `entity.rs`

Suggested contents:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

impl Entity {
    pub const fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }
}
```

---

# 22. `allocator.rs`

Suggested initial implementation:

```rust
use crate::entity::Entity;

#[derive(Debug, Clone)]
struct EntitySlot {
    generation: u32,
    alive: bool,
}

#[derive(Debug, Default)]
pub struct EntityAllocator {
    slots: Vec<EntitySlot>,
    free_ids: Vec<u32>,
    alive_count: usize,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self) -> Entity {
        if let Some(id) = self.free_ids.pop() {
            let slot = &mut self.slots[id as usize];

            debug_assert!(!slot.alive);

            slot.alive = true;
            self.alive_count += 1;

            return Entity::new(id, slot.generation);
        }

        let id = self.slots.len() as u32;

        self.slots.push(EntitySlot {
            generation: 0,
            alive: true,
        });

        self.alive_count += 1;

        Entity::new(id, 0)
    }

    pub fn destroy(&mut self, entity: Entity) -> bool {
        let Some(slot) = self.slots.get_mut(entity.id as usize) else {
            return false;
        };

        if !slot.alive || slot.generation != entity.generation {
            return false;
        }

        slot.alive = false;
        slot.generation = slot.generation.wrapping_add(1);

        self.free_ids.push(entity.id);
        self.alive_count -= 1;

        true
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.slots
            .get(entity.id as usize)
            .map(|slot| {
                slot.alive &&
                slot.generation == entity.generation
            })
            .unwrap_or(false)
    }

    pub fn alive_count(&self) -> usize {
        self.alive_count
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
}
```

---

# 23. `lib.rs`

Initial ECS exports:

```rust
mod allocator;
mod entity;

pub use allocator::EntityAllocator;
pub use entity::Entity;
```

---

# 24. First Implementation Boundary

Do not implement these yet:

```text
Components
Queries
Archetypes
GPU ECS
Parallel mutation
Command buffers
Serialization
Networking IDs
```

The entity allocator should first become boring, reliable infrastructure.

---

# 25. Exit Criteria

Entity management is complete when:

* [ ] entities can be created;
* [ ] entities can be destroyed;
* [ ] IDs can be safely recycled;
* [ ] generations invalidate stale handles;
* [ ] invalid handles do not panic;
* [ ] entity lifecycle tests pass;
* [ ] baseline benchmarks exist;
* [ ] no external ECS framework is involved.

Only then move to component registration and storage.
