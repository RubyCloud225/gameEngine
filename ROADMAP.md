# gameEngine — Development Roadmap

## Purpose

This roadmap defines the intended development order for `gameEngine`.

The goal is to keep development incremental, testable and easy to resume after long pauses.

Each milestone should produce a working result before the next begins.

---

# Milestone 0 — Repository Recovery

## Goal

Turn the current repository into a clean development base without losing the original work.

## Tasks

* [x] Create new directory structure
* [x] Add `DESIGN.md`
* [x] Add `ROADMAP.md`
* [x] Add `STATUS.md`
* [x] Add subsystem documentation directory
* [x] Create `legacy/`
* [x] Move old prototype files into `legacy/`
* [x] Preserve original physics and gameplay experiments
* [x] Create Cargo workspace
* [x] Confirm workspace builds with empty crates
* [x] Add `.gitignore`
* [x] Add formatting configuration
* [x] Add lint configuration

## Exit Criteria

The repository:

* has a clear structure;
* builds successfully;
* contains no accidental dependency on old prototype code;
* preserves old work for reference.

---

# Milestone 1 — Custom ECS Foundation

## Goal

Create the custom data-oriented ECS that the rest of the engine will depend on.

## 1.1 Entity Management

* [x] Define `Entity`
* [x] Implement entity IDs
* [x] Add generation counters
* [x] Implement entity allocator
* [x] Implement entity destruction
* [x] Detect stale entity handles
* [x] Add entity lifecycle tests

## 1.2 Component Registration

* [ ] Define component type IDs
* [ ] Implement component registration
* [ ] Add typed component storage
* [ ] Add component insertion
* [ ] Add component removal
* [ ] Add component lookup
* [ ] Add component lifecycle tests

## 1.3 Component Storage

Initial implementation:

* [ ] Packed typed arrays
* [ ] Entity-to-component lookup
* [ ] Contiguous iteration
* [ ] Safe mutable access
* [ ] Safe immutable access

Later optimisation:

* [ ] Evaluate sparse-set storage
* [ ] Evaluate archetype storage
* [ ] Benchmark alternatives before changing design

## 1.4 World

* [ ] Create `World`
* [ ] Entity creation through `World`
* [ ] Entity deletion through `World`
* [ ] Component insertion through `World`
* [ ] Component removal through `World`
* [ ] Component retrieval through `World`

## 1.5 Queries

* [ ] Single-component query
* [ ] Multi-component query
* [ ] Mutable query
* [ ] Read-only query
* [ ] Query filtering
* [ ] Query tests

## Example

Create an ECS demo containing:

```text
10,000 entities

Position
Velocity
Health
```

Update all positions by iterating over `Position + Velocity`.

## Benchmarks

* [ ] Entity creation
* [ ] Entity destruction
* [ ] Component insertion
* [ ] Component removal
* [ ] Single-component iteration
* [ ] Multi-component iteration

## Exit Criteria

The engine can create entities, attach components and efficiently query component data without any external ECS framework.

---

# Milestone 2 — System Model

## Goal

Define how engine logic operates over ECS data.

## Tasks

* [ ] Define `System`
* [ ] Define system identifier
* [ ] Define system metadata
* [ ] Declare component reads
* [ ] Declare component writes
* [ ] Declare resource reads
* [ ] Declare resource writes
* [ ] Define execution target metadata
* [ ] Define explicit dependencies
* [ ] Define system frequency

Possible metadata:

```text
Name
Reads
Writes
Dependencies
Execution target
Tick rate
```

## Initial Execution Targets

```text
CPU
Auto
```

GPU execution is introduced later.

## Exit Criteria

Systems can declare their data access and execute sequentially against the ECS.

---

# Milestone 3 — Scheduler

## Goal

Create the system scheduler.

## 3.1 Sequential Scheduler

* [ ] Register systems
* [ ] Build execution order
* [ ] Respect explicit dependencies
* [ ] Detect invalid dependency graphs
* [ ] Execute systems sequentially
* [ ] Add deterministic ordering tests

## 3.2 Dependency Analysis

* [ ] Detect read/write conflicts
* [ ] Detect write/write conflicts
* [ ] Build system dependency graph
* [ ] Detect independent systems

## 3.3 Parallel Scheduler

* [ ] Create worker thread pool
* [ ] Dispatch independent systems
* [ ] Add synchronisation barriers
* [ ] Handle worker errors
* [ ] Ensure clean shutdown
* [ ] Add race-condition tests

## Benchmarks

* [ ] Sequential system execution
* [ ] 2-thread execution
* [ ] 4-thread execution
* [ ] Full available CPU execution
* [ ] Scheduler overhead

## Exit Criteria

Independent systems execute safely in parallel.

---

# Milestone 4 — Time and Engine Loop

## Goal

Create a stable engine runtime loop.

## Tasks

* [ ] Engine startup
* [ ] Engine shutdown
* [ ] Frame timer
* [ ] Delta time
* [ ] Fixed timestep
* [ ] Accumulator
* [ ] Frame counter
* [ ] Simulation tick counter
* [ ] Time scaling
* [ ] Pause support

## Simulation Model

```text
Input
  ↓
Variable update
  ↓
Fixed simulation steps
  ↓
Rendering preparation
  ↓
Render
```

## Tests

* [ ] Stable fixed timestep
* [ ] Multiple simulation steps in one render frame
* [ ] Large-frame recovery
* [ ] Pause
* [ ] Time scaling

## Exit Criteria

The engine can run deterministic fixed-step simulation independently of rendering rate.

---

# Milestone 5 — GPU Abstraction

## Goal

Introduce a backend-independent GPU interface.

## Tasks

* [ ] Create `gpu` crate
* [ ] Define GPU adapter abstraction
* [ ] Define GPU device abstraction
* [ ] Define buffer abstraction
* [ ] Define texture abstraction
* [ ] Define shader abstraction
* [ ] Define compute pipeline abstraction
* [ ] Define render pipeline abstraction
* [ ] Define command encoder abstraction
* [ ] Define command buffer abstraction
* [ ] Define queue abstraction

## Architectural Rule

No backend-specific API type may escape the `gpu` crate.

## Exit Criteria

Higher layers can describe GPU work without depending on Metal, Vulkan or DirectX APIs.

---

# Milestone 6 — Metal Development Backend

## Goal

Get GPU execution running on Apple Silicon.

Initial implementation may use `wgpu` internally.

## Tasks

* [ ] Initialise GPU adapter
* [ ] Initialise Metal-backed device
* [ ] Create storage buffer
* [ ] Upload CPU data
* [ ] Execute compute shader
* [ ] Read GPU result
* [ ] Validate result against CPU implementation
* [ ] Add GPU error reporting
* [ ] Add GPU capability reporting

## First Compute Experiment

Use:

```text
Position
Velocity
Delta Time
```

Compute:

```text
Position += Velocity × DeltaTime
```

on the GPU.

## Test Sizes

* [ ] 1,000 entities
* [ ] 10,000 entities
* [ ] 100,000 entities
* [ ] 1,000,000 entities

## Benchmark

Compare:

```text
CPU sequential
CPU parallel
GPU compute
```

## Exit Criteria

The same simulation workload can execute correctly on both CPU and GPU.

---

# Milestone 7 — CPU/GPU Execution Model

## Goal

Allow systems to declare where they can execute.

## Tasks

* [ ] Add `ExecutionTarget::Cpu`
* [ ] Add `ExecutionTarget::Gpu`
* [ ] Add `ExecutionTarget::Auto`
* [ ] Define GPU-system interface
* [ ] Add GPU task submission
* [ ] Add CPU/GPU synchronisation
* [ ] Add resource state tracking
* [ ] Prevent unsafe overlapping access
* [ ] Measure CPU/GPU transfer overhead

## Later

* [ ] Runtime scheduling heuristics
* [ ] Workload-size thresholds
* [ ] Hardware capability detection
* [ ] Automatic execution-target selection

## Exit Criteria

The scheduler can safely coordinate CPU and GPU workloads.

---

# Milestone 8 — Basic Renderer

## Goal

Render a basic scene through the engine GPU abstraction.

## 8.1 Window

* [ ] Create window
* [ ] Connect surface to GPU backend
* [ ] Handle resize
* [ ] Handle close events

## 8.2 Basic Rendering

* [ ] Clear screen
* [ ] Render triangle
* [ ] Vertex buffers
* [ ] Index buffers
* [ ] Camera
* [ ] Transform matrices
* [ ] Render mesh component

## 8.3 Scene Rendering

* [ ] Mesh instances
* [ ] Materials
* [ ] Texture loading
* [ ] Basic depth testing
* [ ] Basic lighting

## Exit Criteria

A scene containing ECS entities can be rendered through the custom renderer.

---

# Milestone 9 — Input System

## Goal

Create platform-neutral input.

## Tasks

* [ ] Keyboard
* [ ] Mouse
* [ ] Controller
* [ ] Input state resource
* [ ] Input mapping
* [ ] Action abstraction
* [ ] Dead-zone support
* [ ] Controller reconnect support

## Example

```text
Physical Input
     ↓
Input Mapping
     ↓
Action
     ↓
MovementIntent
```

## Exit Criteria

Gameplay systems operate on engine actions rather than platform key codes.

---

# Milestone 10 — Physics Foundation

## Goal

Migrate the useful concepts from the original physics prototype into clean engine systems.

## 10.1 Integration

* [ ] Position
* [ ] Velocity
* [ ] Acceleration
* [ ] Gravity
* [ ] Fixed-step integration

## 10.2 Collision Components

* [ ] Collider
* [ ] AABB
* [ ] Collision layers
* [ ] Collision masks
* [ ] Trigger volumes

## 10.3 Broad Phase

* [ ] Spatial grid
* [ ] Insert entities
* [ ] Query neighbouring cells
* [ ] Grid rebuild
* [ ] Benchmark broad phase

## 10.4 Narrow Phase

* [ ] AABB overlap
* [ ] Collision pair generation
* [ ] Collision resolution
* [ ] Ground detection
* [ ] Ceiling collision
* [ ] Wall collision

## 10.5 Character Movement

* [ ] Movement intent
* [ ] Step-up logic
* [ ] Grounding
* [ ] Moving platforms
* [ ] Momentum transfer

## 10.6 Queries

* [ ] Raycast
* [ ] Trigger enter
* [ ] Trigger stay
* [ ] Trigger exit

## Exit Criteria

A character can move through a collision world with stable fixed-step physics.

---

# Milestone 11 — AI Foundation

## Goal

Create conventional game AI that works independently of ML.

## 11.1 Agent

* [ ] `Agent` component
* [ ] AI state
* [ ] Current goal
* [ ] Current target

## 11.2 Perception

* [ ] Perception component
* [ ] Nearby-entity detection
* [ ] Vision interface
* [ ] Hearing/event interface
* [ ] Perception update frequency

## 11.3 Memory

* [ ] Agent memory
* [ ] Last-known position
* [ ] Target history
* [ ] Memory expiry

## 11.4 Blackboard

* [ ] Blackboard structure
* [ ] Typed values
* [ ] Read/write access
* [ ] Debug inspection

## 11.5 Behaviour

Initial:

* [ ] Finite-state machine
* [ ] Behaviour tree

Later:

* [ ] Utility AI
* [ ] Goal-oriented planning

## 11.6 AI Output

AI should produce intent components.

Examples:

```text
MovementIntent
AttackIntent
InteractionIntent
LookIntent
```

## Exit Criteria

An NPC can perceive a target, make a decision and issue movement intent without directly modifying physics state.

---

# Milestone 12 — Navigation

## Goal

Allow AI agents to navigate the world.

## Tasks

* [ ] Navigation graph interface
* [ ] Path representation
* [ ] A* pathfinding
* [ ] Path following
* [ ] Navigation intent
* [ ] Path invalidation
* [ ] Dynamic obstacle response

## Later

* [ ] Navigation mesh
* [ ] Hierarchical pathfinding
* [ ] GPU-assisted pathfinding
* [ ] Crowd navigation

## Exit Criteria

AI agents can navigate around obstacles toward a target.

---

# Milestone 13 — AI Inference

## Goal

Provide an optional learned-model execution path.

## Rules

The core engine must remain functional without this subsystem.

## Tasks

* [ ] Define inference interface
* [ ] Define model input
* [ ] Define model output
* [ ] CPU inference backend
* [ ] GPU inference backend
* [ ] Batched inference
* [ ] Model lifecycle management
* [ ] AI fallback behaviour

## Possible Uses

* [ ] NPC decision models
* [ ] Animation selection
* [ ] Navigation prediction
* [ ] Procedural behaviour
* [ ] Learned steering
* [ ] Future generative systems

## Exit Criteria

An AI agent can optionally use a learned model without changing the rest of the AI architecture.

---

# Milestone 14 — Asset System

## Goal

Create asynchronous asset management.

## Tasks

* [ ] Asset IDs
* [ ] Asset handles
* [ ] Asset registry
* [ ] Async file loading
* [ ] Cache
* [ ] Reference counting or ownership model
* [ ] Asset unloading
* [ ] Mesh loading
* [ ] Texture loading
* [ ] Shader loading

## Later

* [ ] Streaming world chunks
* [ ] Background decompression
* [ ] GPU upload queue
* [ ] Hot reload

## Exit Criteria

Assets can load without blocking the main simulation loop.

---

# Milestone 15 — Animation

## Goal

Add skeletal animation.

## Tasks

* [ ] Skeleton representation
* [ ] Bone transforms
* [ ] Animation clips
* [ ] Animation state
* [ ] Animation blending
* [ ] Animation controller

## GPU Work

* [ ] GPU skinning
* [ ] Batched animation processing

## AI Integration

AI may select animation state.

AI does not execute animation directly.

## Exit Criteria

Animated entities can transition smoothly between states.

---

# Milestone 16 — Debug Tooling

## Goal

Make the custom engine observable.

## Tasks

* [ ] Frame timing
* [ ] System timing
* [ ] Scheduler timing
* [ ] ECS statistics
* [ ] Entity inspector
* [ ] Component inspector
* [ ] Physics debug rendering
* [ ] Spatial-grid rendering
* [ ] AI state display
* [ ] GPU timing
* [ ] Memory statistics

## Exit Criteria

Core engine state can be inspected without adding temporary print statements everywhere.

---

# Milestone 17 — Serialization

## Goal

Allow world and engine state to be persisted.

## Tasks

* [ ] Entity serialization
* [ ] Component serialization
* [ ] World serialization
* [ ] Asset references
* [ ] AI state serialization
* [ ] Save game format
* [ ] Load game
* [ ] Versioning strategy

## Later

* [ ] Deterministic replay
* [ ] Network snapshots
* [ ] Editor integration

## Exit Criteria

A running world can be saved and restored.

---

# Milestone 18 — Advanced Renderer

## Possible Work

* [ ] Frustum culling
* [ ] GPU culling
* [ ] Instancing
* [ ] Deferred rendering
* [ ] Shadow mapping
* [ ] PBR materials
* [ ] HDR
* [ ] Post-processing
* [ ] Temporal effects
* [ ] Compute-based lighting

These should be developed according to actual game requirements.

---

# Milestone 19 — Large World Support

## Goal

Support significantly larger scenes.

## Tasks

* [ ] World partitioning
* [ ] Chunk representation
* [ ] Async chunk loading
* [ ] Chunk unloading
* [ ] Entity migration
* [ ] Asset streaming
* [ ] Navigation streaming

## Later

* [ ] Predictive streaming
* [ ] GPU-driven visibility
* [ ] Background world simulation

---

# Milestone 20 — Linux Backend Validation

## Goal

Validate architecture outside macOS.

## Tasks

* [ ] Build on Linux
* [ ] Vulkan backend
* [ ] Input validation
* [ ] Window validation
* [ ] Rendering tests
* [ ] Compute tests
* [ ] Performance comparison

## Exit Criteria

Core engine examples run correctly on Linux.

---

# Milestone 21 — Windows / DirectX 12 Validation

## Goal

Validate the engine on the graphics architecture required for the future Xbox path.

## Tasks

* [ ] Build on Windows
* [ ] DirectX 12 backend
* [ ] Renderer validation
* [ ] Compute validation
* [ ] Input validation
* [ ] Controller validation
* [ ] Performance tests

## Exit Criteria

The engine operates successfully using DirectX 12 on Windows.

---

# Milestone 22 — Xbox Preparation

## Goal

Prepare the engine architecture for console integration.

This milestone begins only once the desktop DirectX 12 path is stable.

## Tasks

* [ ] Review Xbox GDK requirements
* [ ] Audit unsupported platform calls
* [ ] Confirm filesystem abstraction
* [ ] Confirm input abstraction
* [ ] Confirm graphics abstraction
* [ ] Confirm threading assumptions
* [ ] Confirm memory assumptions
* [ ] Add console build configuration
* [ ] Investigate DirectX 12.x differences
* [ ] Investigate Xbox controller and platform APIs

## Exit Criteria

A clear porting plan exists without requiring major engine redesign.

---

# Milestone 23 — Performance Pass

## Goal

Profile the engine as a complete system.

## Benchmarks

* [ ] ECS iteration
* [ ] Scheduler scaling
* [ ] Physics
* [ ] AI
* [ ] Navigation
* [ ] GPU compute
* [ ] Renderer
* [ ] Asset streaming
* [ ] Frame latency
* [ ] Memory use

## Profiling Questions

For each subsystem:

```text
Where is the time being spent?
Is the workload CPU bound?
Is it GPU bound?
Is it memory bound?
Is synchronisation the bottleneck?
Would moving it to the GPU help?
```

Optimise only with measured evidence.

---

# Milestone 24 — Engine Integration Demo

## Goal

Demonstrate the major engine systems working together.

The demo should include:

* [ ] Custom ECS
* [ ] Parallel scheduler
* [ ] GPU compute
* [ ] Rendering
* [ ] Player input
* [ ] Physics
* [ ] AI-controlled NPC
* [ ] Navigation
* [ ] Asset loading
* [ ] Debug tools

Example scene:

```text
Player
   │
   ├── moves through environment
   ├── collides with world
   └── interacts with NPC

NPC
   │
   ├── perceives player
   ├── selects behaviour
   ├── calculates path
   ├── produces movement intent
   └── follows player
```

This becomes the reference integration test for the engine.

---

# Future Research

These are intentionally outside the main roadmap.

## GPU

* GPU-driven ECS processing
* persistent GPU simulation
* indirect rendering
* GPU physics
* unified simulation/render data
* asynchronous compute

## AI

* learned NPC policies
* GPU-batched agents
* procedural decision models
* world-level simulation
* generative NPC behaviour
* local language models
* adaptive AI

## Physics

* rigid bodies
* constraints
* joints
* continuous collision detection
* ragdolls
* vehicle physics

## Rendering

* ray tracing
* path tracing
* GPU-driven scene submission
* mesh shaders
* advanced global illumination

## Engine

* networking
* deterministic multiplayer
* scripting
* editor
* plugin system
* mod support

---

# Working Rule

At any time:

```text
STATUS.md
```

defines the current task.

This roadmap defines what comes next.

Do not begin multiple major milestones simultaneously unless a small experiment is required to validate an architectural decision.

---

# Current Intended Sequence

```text
Repository Recovery
        ↓
Custom ECS
        ↓
System Model
        ↓
Scheduler
        ↓
Engine Loop
        ↓
GPU Abstraction
        ↓
Metal Compute
        ↓
CPU/GPU Scheduling
        ↓
Basic Renderer
        ↓
Input
        ↓
Physics
        ↓
AI
        ↓
Navigation
        ↓
AI Inference
        ↓
Assets
        ↓
Animation
        ↓
Debugging
        ↓
Serialization
        ↓
Large World
        ↓
Linux
        ↓
Windows / DX12
        ↓
Xbox
```

The priority is always a small working engine over a large collection of unfinished subsystems.
