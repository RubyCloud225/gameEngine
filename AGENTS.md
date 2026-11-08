# Agent guidance

## Scope and project context

This file applies to the whole repository. Follow any more specific `AGENTS.md`
instructions in the directories you change.

gameEngine is a Rust 2024 Cargo workspace building a custom, data-oriented ECS
game engine. Read `STATUS.md` for the current development state, `ROADMAP.md`
for milestones, and `DESIGN.md` plus relevant documents in `docs/` for design
constraints. Check the code before relying on documentation that may be stale.

## Repository layout

- `crates/ecs`: entities, component registration, storage, worlds, and queries.
- Other crates in `crates/`: engine core, scheduler, GPU, renderer, physics, AI,
  input, and assets.
- `legacy/`: preserved Bevy prototype, excluded from the root workspace. Treat
  it as reference material and change it only when the task calls for it.
- `docs/`: subsystem documentation.
- `examples/`, `tests/`, and `benchmarks/`: supporting examples, tests, and
  benchmark material; inspect their Cargo configuration before running them.

## Implementation conventions

- Follow the established architecture in `DESIGN.md`, `architecture.md`, and
  the relevant subsystem documents in `docs/`. Do not make overarching
  architectural decisions or change the project's design direction independently.
  If a task requires such a decision, explain the issue and ask the user to
  decide before implementing the architectural change.
- Keep changes focused on the requested task and preserve unrelated work.
- Follow existing Rust style and use the stable toolchain configured in
  `rust-toolchain.toml`.
- Preserve type safety, entity generation checks, and data-oriented storage.
- Prefer safe Rust. Explain any necessary unsafe code and its safety invariants.
- Keep platform-specific APIs isolated behind backend-neutral interfaces.
- Use `f32` for realtime numeric data unless the design requires otherwise.
- Avoid new dependencies or expanding unfinished subsystems without a concrete
  need from the task.
- Add meaningful tests for new behavior and bug fixes, including relevant
  lifecycle and invalid-input cases. Documentation-only edits do not require
  running Rust tests.
- Update affected documentation when behavior, public APIs, or project status
  changes.
- After every change, update `CHANGELOG.md` in the same task with a concise
  description under the current dated Unreleased entry. Include code,
  documentation, and configuration changes; update an existing entry when it
  already describes the work rather than adding a duplicate.

## Validation

For code changes, start with the affected crate, for example:

```sh
cargo test -p ecs
```

For workspace-wide changes or final integration checks, use:

```sh
cargo check --workspace --all-targets
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

If changing the legacy prototype, validate it separately:

```sh
cargo test --manifest-path legacy/Cargo.toml
```

Report checks run, failures, and any warnings that affect confidence in the
change. Do not claim checks passed if they were not run.

## Git and handoff

Inspect the working tree before editing. Do not discard existing changes or
rewrite shared history. Commit and push when requested by the user.

In the final response, briefly describe what changed, how it was validated,
and any remaining limitations.
