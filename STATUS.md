# gameEngine — Project Status

Last updated: 2026-09-19

## Current State

The project is being reorganised into a custom, portable, data-oriented Rust game engine.

The architectural direction is now defined in:

* `DESIGN.md`
* `ROADMAP.md`

The existing prototype code contains useful ideas but is not treated as production-ready engine code.

---

## Current Focus

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

## Working

At present:

* repository exists;
* original ECS/physics/gameplay prototype exists;
* architecture is documented;
* development direction is defined.

The old prototype should be treated as reference material rather than as stable engine code.

---

## Next Task

Create the new Cargo workspace and subsystem directory structure.

Initial crates:

```text
engine_core
ecs
scheduler
gpu
renderer
physics
ai
input
assets
```

Then confirm:

```bash
cargo check --workspace
```

passes successfully.

---

## After That

Begin:

### Milestone 1 — Custom ECS

First implementation task:

```text
Entity
  ↓
Entity allocator
  ↓
Generation tracking
  ↓
Entity destruction
```

Do not start component storage until entity lifecycle tests pass.

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

## Resume Here

When returning to this project:

1. Read this file.
2. Check the current milestone in `ROADMAP.md`.
3. Read the relevant subsystem document in `docs/`.
4. Run the workspace tests.
5. Continue the first unchecked task under **Current Focus**.

If this file disagrees with `ROADMAP.md`, this file represents the current development state.
