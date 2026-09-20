# gameEngine — Design Document

## 1. Purpose

`gameEngine` is a custom, data-oriented game engine written in Rust.

The engine is being built from first principles rather than on top of an existing game engine framework.

The primary goals are:

* predictable performance;
* efficient CPU cache utilisation;
* explicit CPU and GPU execution;
* parallel system execution;
* portable graphics and compute backends;
* clear subsystem boundaries;
* first-class AI support;
* support for large and complex game worlds;
* architecture that can eventually target consoles, including Xbox.

The engine should remain understandable enough that development can stop for an extended period and later resume without requiring the architecture to be reconstructed from memory.

---

# 2. Design Philosophy

The engine follows several core principles.

## 2.1 Data-Oriented Design

Game state is organised around the way systems process data rather than around object hierarchies.

Instead of:

```text
Player
 ├── Position
 ├── Velocity
 ├── Health
 ├── Mesh
 └── AI
```

the engine stores component data independently:

```text
Positions
[P0][P1][P2][P3]...

Velocities
[V0][V1][V2][V3]...

Health
[H0][H1][H2][H3]...
```

This allows systems to operate over contiguous memory.

Primary goals:

* minimise cache misses;
* improve SIMD suitability;
* simplify parallel execution;
* make component data suitable for GPU upload;
* avoid unnecessary pointer chasing.

---

## 2.2 Entity Component System

The ECS is the structural foundation of the engine.

### Entity

An entity is an identifier.

It contains no behaviour and should contain no gameplay state itself.

```rust
pub struct Entity {
    id: u64,
    generation: u32,
}
```

A generation value should eventually be used to protect against stale entity references.

### Component

Components contain data.

Examples:

```rust
Position
Velocity
Health
MeshInstance
Agent
Perception
MovementIntent
```

Components should contain minimal logic.

### System

Systems operate on component data.

Examples:

```text
MovementSystem
PhysicsSystem
PerceptionSystem
AnimationSystem
RenderExtractionSystem
```

Systems explicitly declare the data they read and write.

This information is used by the scheduler to determine which systems may run concurrently.

---

# 3. High-Level Architecture

```text
                         GAME
                           │
                           ▼
                          ECS
                           │
      ┌────────────────────┼────────────────────┐
      │                    │                    │
      ▼                    ▼                    ▼
     AI                 Physics              Renderer
      │                    │                    │
      └──────────────┬─────┴─────┬──────────────┘
                     │           │
                     ▼           ▼
                 Scheduler     Assets
                     │
              ┌──────┴──────┐
              │             │
              ▼             ▼
             CPU           GPU
                             │
                 ┌───────────┼────────────┐
                 ▼           ▼            ▼
               Metal       Vulkan       DirectX 12
                                           │
                                           ▼
                                        Xbox
```

The ECS owns world state.

Systems consume and modify ECS data.

The scheduler determines when systems execute.

The GPU layer determines how GPU workloads are submitted.

The renderer, physics engine and AI system sit above these lower-level foundations.

---

# 4. Engine Layers

The engine is divided into several logical layers.

```text
Gameplay / Game Code
        │
        ▼
AI / Physics / Animation / Rendering
        │
        ▼
ECS + Scheduler
        │
        ▼
GPU / Assets / Platform Services
        │
        ▼
Operating System / Hardware
```

Higher layers should not depend directly on platform-specific APIs unless absolutely necessary.

---

# 5. Engine Core

The engine core coordinates the lifetime of the engine.

Responsibilities include:

* engine startup;
* engine shutdown;
* configuration;
* subsystem initialisation;
* main loop;
* frame timing;
* fixed simulation timing;
* subsystem ownership.

The core should not contain gameplay-specific functionality.

Example conceptual structure:

```rust
Engine {
    world,
    scheduler,
    gpu,
    renderer,
    physics,
    ai,
    assets,
    input,
}
```

Subsystems may later be optional depending on application configuration.

---

# 6. ECS Architecture

The custom ECS is one of the central systems in the engine.

It must not depend on Bevy ECS or another engine framework.

The implementation may use supporting Rust crates where appropriate, but ownership of the ECS design remains within this project.

Core ECS modules:

```text
ecs/
├── entity
├── component
├── storage
├── query
└── world
```

---

## 6.1 Component Storage

Component storage should favour contiguous data.

The initial implementation should use a simple and understandable design before more complex archetype optimisation is introduced.

Possible progression:

```text
Typed component storage
        ↓
Sparse lookup
        ↓
Packed component arrays
        ↓
Archetype/chunk storage
```

Performance optimisations should be introduced only when benchmarks justify them.

---

## 6.2 Query System

Systems should be able to request combinations of components.

Conceptually:

```rust
Query<(&Position, &mut Velocity)>
```

The exact syntax is not fixed.

The query system must provide:

* efficient iteration;
* safe read access;
* exclusive write access;
* filtering by component presence;
* predictable iteration behaviour.

---

# 7. Scheduler

The scheduler manages engine systems and tasks.

Its purpose is to maximise parallelism while preserving correctness.

Systems declare:

* components read;
* components written;
* resources read;
* resources written;
* execution dependencies;
* preferred execution target.

Example:

```text
PlayerInput
    │
    ▼
MovementIntent
    │
    ▼
Physics
    │
    ▼
TransformUpdate
    │
    ▼
RenderExtraction
```

Systems without conflicting data dependencies may execute concurrently.

---

## 7.1 Execution Targets

The engine should eventually support heterogeneous execution.

Possible targets:

```rust
ExecutionTarget::Cpu
ExecutionTarget::Gpu
ExecutionTarget::Auto
```

`Auto` may eventually allow the scheduler to choose an execution path according to workload size and available hardware.

This is a future capability.

The first scheduler implementation should prioritise correctness and clarity.

---

## 7.2 Fixed Simulation

Simulation systems such as physics should operate on a fixed timestep.

Example:

```text
Rendering
Variable timestep

Physics
Fixed timestep

AI
Fixed or lower-frequency timestep
```

The engine must not assume every subsystem needs to run once per rendered frame.

---

# 8. GPU Architecture

GPU execution is a first-class part of the engine.

However, platform-specific APIs must not define the architecture of higher-level systems.

The engine exposes its own GPU abstraction.

```text
Renderer / Compute Systems
           │
           ▼
      Engine GPU API
           │
       GPU Backend
           │
 ┌─────────┼──────────┐
 ▼         ▼          ▼
Metal    Vulkan     DirectX 12
```

---

## 8.1 Development Backend

The primary development platform is Apple Silicon macOS.

The initial GPU backend will therefore target Metal.

The initial implementation may use `wgpu` to provide access to Metal while preserving portability.

The rest of the engine must not depend directly on `wgpu` types.

Correct:

```rust
GpuBuffer
GpuTexture
GpuPipeline
GpuCommandBuffer
```

Avoid exposing:

```text
wgpu::Buffer
MTLBuffer
VkBuffer
ID3D12Resource
```

outside the GPU implementation layer.

---

## 8.2 Target Backends

Planned targets:

```text
macOS      → Metal
Linux      → Vulkan
Windows    → DirectX 12
Xbox       → DirectX 12 / Xbox graphics APIs
```

Xbox support is a design requirement but not an initial implementation target.

The architecture must not make future Xbox support unnecessarily difficult.

---

# 9. Numerical Representation

Realtime game simulation should primarily use `f32`.

Reasons include:

* lower memory use;
* higher memory bandwidth;
* broad GPU support;
* efficient SIMD execution;
* typical GPU-native vector formats.

Example:

```rust
#[repr(C)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub _padding: f32,
}
```

GPU-facing structures should use predictable layouts and explicit alignment.

`f64` should only be introduced where precision requirements justify the additional cost.

---

# 10. Renderer

The renderer sits above the GPU abstraction.

Its responsibility is to translate engine scene state into rendering commands.

```text
ECS Scene Data
      │
      ▼
Render Extraction
      │
      ▼
Renderer
      │
      ▼
Engine GPU API
      │
      ▼
Metal / Vulkan / DX12
```

Initial renderer capabilities should remain simple.

Possible progression:

```text
Triangle
   ↓
Mesh
   ↓
Camera
   ↓
Materials
   ↓
Lighting
   ↓
Visibility / culling
   ↓
Advanced rendering
```

Rendering should not be required for ECS or simulation tests.

---

# 11. Physics

Physics is an independent engine subsystem.

It interacts with world state through ECS components.

Typical components include:

```text
Position
Velocity
Acceleration
Collider
Mass
Grounded
CollisionLayer
```

Physics responsibilities include:

* integration;
* collision detection;
* collision resolution;
* spatial partitioning;
* raycasting;
* trigger volumes;
* moving platforms;
* future rigid-body simulation.

The physics engine remains authoritative over physical motion.

Gameplay and AI should express desired motion through intent rather than directly overriding physics state where possible.

---

# 12. AI System

AI is a first-class engine subsystem.

AI should be able to operate without requiring machine learning.

The baseline AI architecture supports conventional game AI, while allowing learned models to be introduced later.

---

## 12.1 Core Principle

> AI consumes world state and produces intent.

AI should generally avoid directly modifying physical state.

Example:

```text
Perception
     │
     ▼
AI Memory
     │
     ▼
Decision
     │
     ▼
MovementIntent
     │
     ▼
Physics
```

This preserves subsystem boundaries.

---

## 12.2 AI Architecture

```text
AI
├── Agent
├── Perception
├── Memory
├── Blackboard
├── Behaviour
├── Planner
├── Navigation
└── Inference
```

### Agent

Represents an entity capable of autonomous decision-making.

### Perception

Collects relevant information from the game world.

Examples:

* visible entities;
* audible events;
* nearby threats;
* environmental conditions.

### Memory

Stores relevant historic information.

Examples:

* last known player position;
* previous attacker;
* visited location;
* current objective.

### Blackboard

Provides structured working state shared between AI behaviours.

### Behaviour

Supports deterministic decision systems such as:

* finite-state machines;
* behaviour trees;
* utility-based decision systems.

### Planner

May later support goal-oriented planning.

### Navigation

Responsible for high-level movement planning.

Pathfinding is separate from physics.

Navigation determines:

```text
Where should the entity move?
```

Physics determines:

```text
Can the entity actually move there?
```

### Inference

Provides an optional interface for learned models.

The engine must not require ML inference in order to function.

---

## 12.3 AI Execution

Different AI workloads may have different execution frequencies.

Example:

```text
Player perception        60 Hz
Nearby NPC decisions     20 Hz
Distant NPC decisions     5 Hz
Strategic world AI        1 Hz
```

This allows AI workload to scale with world size.

Future AI workloads may also be batched for GPU execution.

---

# 13. Asset System

The asset subsystem handles external game data.

Responsibilities include:

* loading;
* caching;
* reference tracking;
* asynchronous I/O;
* unloading;
* GPU upload;
* asset lifetime.

Potential asset types:

```text
Meshes
Textures
Shaders
Audio
Animations
World chunks
Navigation data
AI data
```

Asset loading must not block the main simulation thread.

---

# 14. Input

Input should be separated from gameplay logic.

The input layer converts platform input into engine-level input state.

```text
Keyboard / Controller / Mouse
              │
              ▼
          Input Layer
              │
              ▼
         Input State
              │
              ▼
       Gameplay Systems
```

Game code should not depend directly on platform key codes.

This will be particularly important for eventual console support.

---

# 15. Platform Layer

Platform-specific code must be isolated.

Potential responsibilities:

```text
Window creation
Input devices
Filesystem
Thread primitives
Timing
Console APIs
GPU backend initialisation
```

Platform-specific implementation should remain below the engine's portable APIs.

---

# 16. Engine Data Flow

A typical frame may eventually resemble:

```text
                INPUT
                  │
                  ▼
             ECS updates
                  │
                  ▼
            AI perception
                  │
                  ▼
             AI decision
                  │
                  ▼
              Intents
                  │
                  ▼
              Physics
                  │
                  ▼
          Transform update
                  │
                  ▼
         Render extraction
                  │
                  ▼
              Renderer
                  │
                  ▼
                 GPU
```

The scheduler controls the actual execution order and parallelism.

---

# 17. CPU/GPU Data Flow

The engine should minimise unnecessary transfers between CPU and GPU.

Ideal workloads for GPU execution include:

* large parallel transforms;
* particles;
* animation skinning;
* visibility calculations;
* crowd simulation;
* some physics workloads;
* AI inference;
* rendering.

CPU workloads include:

* entity creation/destruction;
* game rules;
* event management;
* small irregular workloads;
* system scheduling;
* high-level AI decisions;
* operating-system interaction.

The correct execution target should be determined by workload characteristics rather than by assuming the GPU is always faster.

---

# 18. Debugging and Observability

Debugging support should be treated as part of engine architecture.

The engine should eventually expose:

* entity inspection;
* component inspection;
* system execution timing;
* scheduler graph inspection;
* GPU timing;
* physics debug drawing;
* AI state inspection;
* memory statistics;
* frame timing.

A custom engine becomes difficult to maintain if internal state cannot be observed.

---

# 19. Serialization

Engine state should eventually support serialization.

Uses include:

* save games;
* world loading;
* editor tooling;
* testing;
* debugging;
* deterministic replay.

AI state must also be serializable where practical.

---

# 20. Testing Philosophy

Every core subsystem should be usable without launching a graphical application.

Examples:

```text
ECS tests
Scheduler tests
Physics tests
AI tests
GPU compute tests
Asset tests
```

Graphics-dependent tests should be isolated from CPU-only tests.

---

# 21. Benchmarking

Performance claims must be measured.

Important benchmarks include:

```text
Entity creation
Component insertion
Component query iteration
System scheduling
Thread scaling
Collision broad phase
AI batch processing
CPU vs GPU transforms
CPU vs GPU simulation workloads
GPU upload bandwidth
```

Optimisation should follow profiling rather than assumption.

---

# 22. Repository Structure

```text
gameEngine/
│
├── README.md
├── DESIGN.md
├── STATUS.md
├── ROADMAP.md
├── Cargo.toml
│
├── crates/
│   ├── engine_core/
│   ├── ecs/
│   ├── scheduler/
│   ├── gpu/
│   ├── renderer/
│   ├── physics/
│   ├── ai/
│   ├── input/
│   └── assets/
│
├── shaders/
│   ├── compute/
│   ├── vertex/
│   └── fragment/
│
├── examples/
│   ├── minimal/
│   ├── ecs_demo/
│   ├── gpu_compute/
│   ├── physics_demo/
│   └── ai_demo/
│
├── tests/
│
├── benchmarks/
│
├── docs/
│   ├── ECS.md
│   ├── SCHEDULER.md
│   ├── GPU.md
│   ├── RENDERER.md
│   ├── PHYSICS.md
│   ├── AI.md
│   ├── ASSETS.md
│   └── PLATFORM.md
│
└── legacy/
```

---

# 23. Dependency Rules

The intended dependency direction is:

```text
                engine_core
                     │
                     ▼
                  scheduler
                     │
                     ▼
                     ecs

AI ───────────────► ECS
Physics ──────────► ECS
Renderer ─────────► ECS

Renderer ─────────► GPU
GPU ──────────────► Platform

Assets ───────────► Platform
Input ────────────► Platform
```

Circular subsystem dependencies should be avoided.

Communication between higher-level systems should generally happen through ECS state, events or explicit interfaces.

---

# 24. Non-Goals

The following are not immediate goals:

* competing with Unreal Engine feature-for-feature;
* building an editor before the runtime works;
* implementing every graphics backend simultaneously;
* putting every workload on the GPU;
* building advanced ML agents before conventional AI works;
* implementing complex physics before the ECS and scheduler are stable;
* premature optimisation.

The engine should grow from a stable core.

---

# 25. Development Order

The intended implementation sequence is:

```text
1. Repository structure
        ↓
2. Custom ECS
        ↓
3. Scheduler
        ↓
4. Fixed timestep
        ↓
5. GPU abstraction
        ↓
6. Metal-backed GPU execution
        ↓
7. Basic renderer
        ↓
8. Physics
        ↓
9. AI
        ↓
10. Asset streaming
        ↓
11. Additional platforms
        ↓
12. Advanced GPU execution
        ↓
13. Xbox investigation / backend
```

Individual experiments may happen earlier, but dependencies should be stabilised in approximately this order.

---

# 26. Current Platform Strategy

Primary development environment:

```text
macOS
Apple Silicon
Rust
Metal
```

Portability targets:

```text
Linux   → Vulkan
Windows → DirectX 12
Xbox    → DirectX 12 / Xbox platform APIs
```

Cross-platform design begins at the API boundary, not after the Mac implementation has already been completed.

---

# 27. Core Architectural Rules

These rules should remain visible during development.

1. The ECS is custom.
2. Components primarily contain data.
3. Systems contain behaviour.
4. Subsystems communicate through explicit interfaces or ECS state.
5. The scheduler owns execution ordering.
6. Physics owns physical movement.
7. AI produces intent.
8. The renderer owns rendering.
9. GPU-specific types stay inside the GPU layer.
10. Platform-specific types stay inside the platform layer.
11. `f32` is the default realtime numeric representation.
12. CPU and GPU execution are both first-class.
13. GPU execution is chosen because it is appropriate, not merely because it is available.
14. Old experimental code is preserved in `legacy/`, not mixed with production code.
15. Architecture should favour clarity before sophistication.
16. Every major optimisation should eventually have a benchmark.
17. Every core subsystem should be independently testable.

---

# 28. Long-Term Direction

The long-term goal is a custom, portable, data-oriented engine capable of running large interactive worlds with efficient use of modern heterogeneous hardware.

The engine should be able to coordinate:

```text
CPU cores
   +
GPU compute
   +
Rendering
   +
Physics
   +
AI
   +
Streaming
```

through one understandable architecture.

The aim is not merely to render a game.

The aim is to build a runtime capable of efficiently deciding:

```text
What needs to happen?
What data does it require?
When may it run?
Where should it execute?
```

That is the central architectural idea behind `gameEngine`.
