# gameEngine — Data-Oriented ECS Engine in Rust
 
A high-performance game engine built from first principles in Rust, structured
around the Entity Component System (ECS) paradigm and data-oriented design.
Targets maximum CPU cache utilisation and safe multithreading via Rust's
ownership model.
 
## Run and test

The root is a Cargo workspace with nine empty subsystem crates: `engine_core`,
`ecs`, `scheduler`, `gpu`, `renderer`, `physics`, `ai`, `input`, and `assets`.
Their source modules are registered, but the new engine is not implemented yet.
The Bevy prototype is preserved as a separate package under `legacy/` and is
excluded from the new workspace. The new crates have no dependency on it.

```sh
cargo check --workspace --all-targets
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

The empty new crates currently have no behaviour tests. Run the prototype's
19 existing tests and headless demo explicitly:

```sh
cargo test --manifest-path legacy/Cargo.toml
cargo run --manifest-path legacy/Cargo.toml
cargo test --manifest-path legacy/Cargo.toml triggers
```

If an already-open terminal cannot find Cargo, run `source "$HOME/.cargo/env"` or
open a new terminal. In VS Code, run **Developer: Reload Window** after setup.
The rust-analyzer workspace settings link to the root `Cargo.toml`.
The empty files under `benchmarks/` remain placeholders, not Cargo benchmark targets.

The legacy library exposes `components`, `resources`, and `systems`. Start with
`new_world()` to install resources and `physics_schedule()` to construct the
fixed-step schedule. Call `advance_frame(&mut world, &mut schedule, elapsed_seconds)`
to run complete 1/60-second ticks. `legacy/main.rs` is a working example;
`legacy/tests/systems.rs` demonstrates each prototype system independently.

### Legacy prototype systems and conventions

- Movement/input, gravity, acceleration, friction, AABB collision and small step-up.
- Broad-phase box pairs, spatial-grid rebuilding, collision layers and raycasts.
- Trigger enter/stay/exit, with actor and target IDs and next-tick event cleanup.
- Moving-platform carry and one-time momentum transfer on departure.
- Damage resistance, invulnerability, death markers, damage effects and hit-stop.
- Animation state selection and blend-request queue, camera shake and time management.

`Position` is the centre of a `Hitbox`; width, height and depth are full dimensions.
`BoundingBox` min/max values are local offsets from Position. Bodies processed by
`physics_prediction_systems` need `FixedPhysics`, `Position`, `Velocity`,
`Acceleration`, `Hitbox` and `Grounded`. Add `AffectedByGravity` for gravity,
`Collidable` to act as an obstacle, and `TriggerTracker` to receive trigger events.
`Player` plus `MovementSpeed` enables input-driven horizontal velocity. Resistance
values are fractions (0–1); friction is the fraction of horizontal velocity retained
per second while grounded (0 stops, 1 retains all speed).

The simple `movement` and y=0 `collision_resolution_system` are alternatives to the
full prediction system. Do not schedule both movement implementations on the same
body. Queue damage in `DamageRequests.events`; collision does not inflict arbitrary
example damage. Consume trigger events after detection and before the next physics
tick. `shoot_ray_system` is a query helper called inside a system; rebuild the grid
before calling it.

Physics uses discrete axis-separated collision, so fast bodies can tunnel through
thin obstacles. Collider snapshots and raycast candidates currently use straightforward
scans; the grid is available for future optimization. Moving-platform carry assumes
kinematic platform velocity and does not implement a rigid-body solver. Animation
queues blend requests but has no skeletal animation backend. `DamageEffect` entities
are data for a future effects consumer, which must remove them when finished.
Components such as inventory, lifetime and knockback do not yet have corresponding
systems; the existing TODO list remains future work.

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
 
