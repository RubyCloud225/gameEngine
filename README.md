# gameEngine — Data-Oriented ECS Engine in Rust
 
A high-performance game engine built from first principles in Rust, structured
around the Entity Component System (ECS) paradigm and data-oriented design.
Targets maximum CPU cache utilisation and safe multithreading via Rust's
ownership model.
 
---
 
## Design Philosophy
 
Traditional object-oriented game engines scatter related data across memory
by coupling data and logic inside objects. An `Actor` containing its own
`Position`, `Velocity`, and `Mesh` means iterating over actors pulls
unrelated data into cache on every frame — a performance ceiling that cannot
be engineered around at scale.
 
This engine rejects that model entirely. Data-oriented design (DOD) separates
*what things are* (components) from *what happens to them* (systems), grouping
data by type rather than by entity. The result is contiguous memory layouts
that the CPU prefetcher can actually use.
 
---
 
## Architecture
 
The engine is structured across two layers.
 
### Layer 1 — Engine Core
 
The low-level foundation handling parallelism, rendering, and I/O.
 
```
┌─────────────────────────────────────────────────────┐
│                    Engine Core                      │
│                                                     │
│  ┌──────────────┐  ┌─────────────┐  ┌───────────┐  │
│  │ Task         │  │ Renderer    │  │ Async I/O │  │
│  │ Scheduler    │  │             │  │           │  │
│  │              │  │ Vulkan /    │  │ Asset     │  │
│  │ Thread pool  │  │ DirectX     │  │ streaming │  │
│  │ Load balance │  │ Draw calls  │  │ No stalls │  │
│  └──────────────┘  └─────────────┘  └───────────┘  │
└─────────────────────────────────────────────────────┘
```
 
| System | Role |
|---|---|
| **Task Scheduler** | Thread pool distributing independent tasks across all CPU cores. Handles load balancing and synchronisation between parallel workloads. |
| **Renderer** | Translates scene data into GPU draw calls. Render thread decoupled from game logic thread — frame N-1 renders while frame N is computed. |
| **Async I/O** | Loads world chunks, textures, and assets from disk without blocking the game loop. |
 
### Layer 2 — Gameplay Framework
 
High-level systems built on top of the core's parallel infrastructure.
 
| System | Function |
|---|---|
| **Animation System** | Skeletal bone blending and Inverse Kinematics, parallelised across the Task Scheduler. |
| **World Streaming** | Loads and unloads open-world chunks via Async I/O, with no game thread stalls. |
| **Material / Shader System** | Surface rendering pipeline, GPU shader execution via the Renderer core. |
 
---
 
## Entity Component System (ECS)
 
ECS is the structural foundation of the engine. Every object in the world is
decomposed into three concepts:
 
### Entities
 
An entity is nothing more than an integer ID — a pure index into component
storage. It carries no data and no logic of its own.
 
```rust
type Entity = u64;
```
 
### Components
 
Components are pure data structs with no methods. All components of the same
type are stored contiguously — a Structure of Arrays (SOA) layout.
 
```rust
struct Position { x: f32, y: f32, z: f32 }
struct Velocity { dx: f32, dy: f32, dz: f32 }
struct Mass     { kg: f32 }
```
 
Contiguous storage means a system iterating over 10,000 `Position` components
reads a single linear block of memory. Cache miss rate approaches zero.
 
### Systems
 
Systems are pure functions. They declare which components they read or write,
query all entities that possess those components, and operate on them.
 
```rust
fn physics_system(positions: &mut [Position], velocities: &[Velocity], dt: f32) {
    for (pos, vel) in positions.iter_mut().zip(velocities.iter()) {
        pos.x += vel.dx * dt;
        pos.y += vel.dy * dt;
        pos.z += vel.dz * dt;
    }
}
```
 
Because systems declare their data access explicitly, the scheduler can run
independent systems in parallel with no data races. In Rust, the borrow
checker enforces this at compile time — a system that reads `Position` cannot
run concurrently with a system that writes `Position`.
 
---
 
## Memory Layout
 
The contrast between OOP and DOD at the memory level:
 
```
OOP — Array of Structs (AoS)
──────────────────────────────────────────────────────
│ Entity 0                    │ Entity 1              │
│ pos.x pos.y pos.z vel.x ... │ pos.x pos.y pos.z ... │
──────────────────────────────────────────────────────
  ↑ Physics system loads irrelevant data on every read
 
DOD — Structure of Arrays (SoA)
──────────────────────────────────────────────────────
Positions: │ x0 y0 z0 │ x1 y1 z1 │ x2 y2 z2 │ ...
Velocities:│ dx0 dy0  │ dx1 dy1  │ dx2 dy2  │ ...
──────────────────────────────────────────────────────
  ↑ Physics system reads one contiguous block. Cache hot.
```
 
---
 
## HPC Parallel
 
The ECS model maps directly onto GPU compute concepts:
 
| ECS Concept | HPC / CUDA Equivalent |
|---|---|
| Entity | Thread ID / Block ID |
| Component (SoA) | Contiguous GPU memory buffer |
| System | CUDA kernel |
| Task Scheduler | CUDA grid / warp scheduler |
 
Systems operating on large contiguous component arrays are structurally
equivalent to CUDA kernels operating on device memory — the same principles
of cache locality, memory coalescing, and parallel independence apply.
 
---
 
## Why Rust
 
Rust's ownership and borrowing rules enforce the ECS contract at compile time:
 
- Mutable access to a component type is exclusive — no data races by
  construction
- Zero-cost abstractions mean iterator-based system queries compile to the
  same machine code as hand-written C loops
- No garbage collector — frame timing is deterministic
---
 
## Structure
 
```
gameEngine/
├── engine/          # Core ECS, scheduler, renderer abstractions
├── main.rs          # Entry point and system registration
├── architecture.md  # Detailed architecture reference
└── TODO.md          # Active development roadmap
```
 
---
 
## Status
 
Active development. Core ECS implementation and task scheduler are the
current focus. Physics and rendering systems follow once the parallel
infrastructure is stable.
 
See [TODO.md](TODO.md) for the current roadmap and
[architecture.md](architecture.md) for the full architectural reference.
 
---
 
*Original implementation by Catherine Earl, 2025. Rust.*
 
