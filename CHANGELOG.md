# Changelog

## Unrealeased - 2026-10-07

### Added
- Function to register component ID which will be attached to the entity created in `entity.rs`
- add unit tests to validate this.

## Unreleased — 2026-09-20

### Added

- Cargo workspace with nine subsystem crates: `engine_core`, `ecs`, `scheduler`,
  `gpu`, `renderer`, `physics`, `ai`, `input`, and `assets`.
- Custom ECS entity handles and entity management: creation, destruction,
  generation checks, free-slot reuse, live entity counts, and allocated slot counts.
- Thirteen entity lifecycle tests covering invalid handles, stale generations,
  repeated destruction, and slot recycling.
- Rust toolchain configuration, shared workspace lints, Cargo lockfiles, VS Code
  rust-analyzer settings, and build-output ignore rules.
- Architecture, roadmap, project status, subsystem documentation, and benchmark
  placeholders for future development.

### Changed

- Moved the Bevy ECS prototype, demo, tests, and notes into `legacy/` with its own
  Cargo package, excluded from the new workspace.
- Registered the new subsystem source modules and updated README setup and test
  commands for the reorganised repository.
- Made the legacy prototype runnable with a fixed-step simulation and a headless
  falling-body demo; added nineteen tests for its existing systems.

### Fixed

- Repaired prototype component definitions, resource access, system scheduling,
  and incomplete movement, collision, trigger, raycast, damage, platform,
  animation-state, camera-shake, and time-management code.
- Completed both allocation paths in `EntityManager::create()` so each returns
  an entity handle.
- Moved `destroy()` into the entity manager implementation and corrected its
  entity ID lookup and documentation formatting.
- Implemented the missing `is_alive()`, `alive_count()`, and `capacity()` methods
  used by the entity lifecycle tests.

### Current scope

- New subsystems beyond entity management remain scaffolds. The legacy physics
  uses discrete collision detection, and animation has no rendering backend.
