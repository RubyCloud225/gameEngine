# gameEngine — Project Status

Last updated: 2026-10-01

## Current State

The repository recovery and initial architecture phase are complete.

The project is now in:

**Milestone 1 - Custom ECS Foundation**

The architectural direction is now defined in:

* `DESIGN.md`
* `ROADMAP.md`

The existing prototype code remains available under `legacy/` as reference material only.

---
## Current Focus

### Milestone 1 - Custom ECS Foundation

Completed:

* [x] Define `Entity`
* [x] Implement Entity ID's
* [x] Add generation counters
* [x] Implement entity allocator
* [x] Implement entity destruction
* [x] Detect stale entity handles
* [x] Add entity lifecycle tests

Current task:

### Component Registration

* [x] Define component type IDs
* [ ] Implement component registration
* [ ] Add typed component storage
* [ ] Add component insertion
* [ ] Add component removal
* [ ] Add component lookup
* [ ] Add component lifecycle tests

---

## Current Task

Implement Component type identification and registration.

### Objective

Introduce a mechanism for identifying and registering ECS component types

### Acceptance Criteria

- Each registered component type has a stable identifier
- Component Registration is type-safe
- Registered component types can be looked up
- Duplicate registeration is handled correctly
- Unit tests cover registration behaviour

### Out of Scope

- World integration
- Queries
- Scheduler integration
- GPU component Storage

### Validation

* [ ] `cargo test --workspace`
* [ ] `cargo clippy --workspace`
* [ ] `cargo fmt --check`

---

## Working

At present:

* repository exists;
* original ECS/physics/gameplay prototype exists;
* architecture is documented;
* development direction is defined.

The old prototype should be treated as reference material rather than as stable engine code.

---

## Do Not Work On Yet

Avoid expanding these systems until the foundation is stable:

* advanced rendering;
* complex physics;
* neural AI;
* GPU scheduling heuristics;
* asset streaming;
* animation;
* world streaming;
* Linux optimisation;
* DirectX 12 implementation;
* Xbox implementation;
* editor tooling.

Small experiments are acceptable when needed to validate an architectural decision.

---

## Important Architectural Decisions

* Custom ECS
* Rust engine core
* Data-oriented component storage
* CPU and GPU are both execution targets
* Metal is the first GPU backend
* GPU API remains backend-neutral
* Linux target: Vulkan
* Windows target: DirectX 12
* Xbox is a future first-class platform target
* AI is a dedicated subsystem
* AI produces intent rather than controlling physics directly
* Physics remains authoritative for movement
* `f32` is the default realtime numeric representation
* Platform-specific APIs remain isolated
* Existing prototype code will be preserved under `legacy/`

---

## Milestones Cleared

### Milestone 0 — Repository Recovery

In progress:

* [x] Define overall engine architecture
* [x] Define long-term roadmap
* [x] Add AI as a first-class subsystem
* [x] Define GPU portability strategy
* [x] Define Xbox as a future platform target
* [x] Create new repository directory structure
* [x] Create Cargo workspace
* [x] Create empty subsystem crates
* [x] Move prototype code into `legacy/`
* [x] Confirm the reorganised workspace builds cleanly
---

## Resume Here

When returning to this project:

1. Read this file.
2. Check the current milestone in `ROADMAP.md`.
3. Read the relevant subsystem document in `docs/`.
4. Run the workspace tests.
5. Continue the first unchecked task under **Current Focus**.

If this file disagrees with `ROADMAP.md`, this file represents the current development state.
