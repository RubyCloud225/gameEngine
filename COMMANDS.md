# Commands

Run these commands from the repository root. The root workspace uses the stable
Rust toolchain configured in `rust-toolchain.toml`.

## Workspace

```sh
# Build all workspace crates.
cargo build --workspace

# Check all workspace targets without producing executables.
cargo check --workspace --all-targets

# Run workspace tests, including documentation tests.
cargo test --workspace

# Check formatting.
cargo fmt --all --check

# Apply formatting.
cargo fmt --all

# Run Clippy and treat warnings as errors.
cargo clippy --workspace --all-targets -- -D warnings

# Generate and open API documentation.
cargo doc --workspace --no-deps --open
```

The root workspace contains subsystem libraries. Use the legacy command below
to run the preserved prototype.

## Work on a single crate

These examples target the ECS crate. Replace `ecs` with another workspace
package name as needed.

```sh
cargo check -p ecs --all-targets
cargo test -p ecs
cargo clippy -p ecs --all-targets -- -D warnings

# Run tests whose names contain the filter.
cargo test -p ecs component::tests

# Show output from passing tests.
cargo test -p ecs -- --nocapture
```

## Legacy prototype

`legacy/` is excluded from the root workspace and must be checked separately.

```sh
# Run the headless prototype.
cargo run --manifest-path legacy/Cargo.toml

# Run all prototype tests.
cargo test --manifest-path legacy/Cargo.toml

# Run prototype tests matching a filter.
cargo test --manifest-path legacy/Cargo.toml triggers

# Check prototype formatting.
cargo fmt --manifest-path legacy/Cargo.toml --check
```

## Review local changes

```sh
git status --short --branch
git diff
git diff --staged
git diff --check
```

`git diff` shows tracked changes. Use `git status` to also see newly created,
untracked files.
