# RFC 0011: Rust Workspace Local Build Time Policy

- Status: `Draft`
- Last updated: `2026-04-07`

## Background

As this Rust workspace grows, local feedback loops become increasingly sensitive to configuration choices:

- workspace-wide recompilation cascades across multiple crates
- build scripts can rerun more often than necessary
- developers often use `cargo build` even when `cargo check` would be sufficient
- release-oriented settings are heavier than needed for day-to-day verification
- build cache configuration is not yet standardized across the workspace

The repository already uses a Cargo workspace at the root, but today it does not define:

- explicit workspace-level `profile.dev`
- a fast verification profile between `dev` and `release`
- workspace-level `sccache` integration in `.cargo/config.toml`
- a documented policy for `default-members`
- a documented policy for `build.rs` rerun conditions

At the time of writing:

- root [`Cargo.toml`](/home/hayashi3017/git/creation/Cargo.toml) defines `[workspace]` and `[workspace.dependencies]`, but no `[profile.*]`
- no repository `.cargo/config.toml` is present
- no `build.rs` files were found in the workspace

## Goal

Reduce local development build time in a reproducible, workspace-wide way without weakening the final production `release` profile.

## Non-Goals

- maximizing production runtime performance
- redesigning CI from scratch
- linker-specific or target-specific tuning
- large-scale dependency pruning
- broad feature-flag restructuring

Those may still be useful later, but they should be handled by separate work.

## Proposal

Adopt the following local-build policy for this workspace:

1. define workspace-level build profiles only at the root `Cargo.toml`
2. make `profile.dev` explicitly optimized for edit-compile-check loops
3. add a `release-fast` profile for fast local optimized builds
4. add optional workspace-level `sccache` support via `.cargo/config.toml`
5. review whether `workspace.default-members` should narrow default build scope
6. require explicit `rerun-if-*` rules for any future `build.rs`
7. document `cargo check` as the default day-to-day verification command
8. use `cargo --timings` when investigating slow builds

## Current State In This Repository

Observed from the current workspace:

- root [`Cargo.toml`](/home/hayashi3017/git/creation/Cargo.toml) has no `[profile.dev]`
- root [`Cargo.toml`](/home/hayashi3017/git/creation/Cargo.toml) has no custom release-like profile such as `release-fast`
- root [`Cargo.toml`](/home/hayashi3017/git/creation/Cargo.toml) has no `default-members`
- no repository `.cargo/config.toml` exists
- no `build.rs` files exist today

That means this RFC is mostly about establishing policy and a clean baseline before the workspace grows further.

## Profile Policy

### Root-Only Profiles

Build profiles should be defined only in the workspace root [`Cargo.toml`](/home/hayashi3017/git/creation/Cargo.toml).

Reason:

- Cargo profile behavior is clearer when one file owns the policy
- member crates should not silently drift into different local-build behavior
- future reviews stay small and centralized

### `profile.dev`

Recommended baseline:

```toml
[profile.dev]
incremental = true
debug = "line-tables-only"
```

Intent:

- keep incremental compilation enabled for frequent edits
- reduce debug info generation cost while preserving usable backtraces

Fallback:

- if `debug = "line-tables-only"` is not viable on the toolchain in use, prefer `debug = 1`

Do not standardize these in `dev`:

- custom `opt-level`
- `lto`
- aggressive `codegen-units` tuning

The default development profile should stay optimized for fast iteration, not benchmark-like behavior.

### `profile.release-fast`

Add a local optimized verification profile:

```toml
[profile.release-fast]
inherits = "release"
incremental = true
lto = "off"
codegen-units = 64
debug = "line-tables-only"
```

Intent:

- provide a faster alternative to full `release`
- keep production `release` untouched
- support local validation of roughly optimized behavior without full release build cost

Use cases:

- quick local smoke tests under optimization
- rough binary-size or speed checks
- local reproduction of release-only behavior without paying full release build cost

## Cache Policy

### `.cargo/config.toml`

If it does not conflict with existing tooling, add:

```toml
[build]
rustc-workspace-wrapper = "sccache"
```

Intent:

- improve reuse across crates
- reduce rebuild cost after branch switches or dependency churn

Constraints:

- `sccache` should remain optional, not a hard prerequisite for development
- if wrapper configuration already exists, compatibility must be reviewed before changing it
- any environment workaround should be documented with a short rationale comment near the setting

## Workspace Scope Policy

### `default-members`

Review whether daily development actually needs every workspace member by default.

Possible future direction:

```toml
[workspace]
members = ["creation-adapter", "creation-driver", "creation-service", "creation-usecase", "xtask"]
default-members = ["creation-driver", "creation-adapter", "creation-service", "creation-usecase"]
```

This is only a policy recommendation for review, not an automatic decision.

Use `default-members` only if it reflects real daily usage. Over-narrowing the default scope can surprise contributors and hide breakage in less common crates.

## Build Script Policy

There are no `build.rs` files in the repository today, but the policy should be established now.

Any future `build.rs` must declare explicit rerun conditions when practical.

Examples:

```rust
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}
```

```rust
fn main() {
    println!("cargo:rerun-if-changed=schema/openapi.yaml");
    println!("cargo:rerun-if-env-changed=MY_CODEGEN_MODE");
}
```

Intent:

- avoid unnecessary reruns
- keep code generation inputs explicit
- make future build-time cost easier to reason about

Do not narrow rerun conditions unless the true inputs are known. Incorrectly small rerun scopes are worse than conservative reruns.

## Command Policy

For routine local development, prefer:

1. `cargo check`
2. `cargo build` only when a binary or artifact is actually needed
3. `cargo build --profile release-fast` for quick optimized verification

Examples:

```bash
cargo check --workspace
cargo check -p creation-driver
cargo build -p creation-driver --profile release-fast
```

Reason:

- `cargo check` skips unnecessary code generation for faster feedback
- developers should pay full build cost only when the output artifact is needed

## Measurement Policy

When build time feels slow, use timings instead of guessing.

Recommended command:

```bash
cargo build --workspace --profile release-fast --timings
```

This should be used to identify:

- slow crates
- expensive build scripts
- unexpectedly wide rebuild scopes

## Alternatives Considered

### Lighten `profile.release` Directly

Rejected because:

- it weakens the meaning of the production profile
- it couples local convenience to release expectations

### Raise `opt-level` In `dev`

Rejected as the default because:

- it often slows compilation
- it fights the primary goal of fast iteration

### Aggressive Linker Or `RUSTFLAGS` Tuning

Deferred because:

- it is more environment-specific
- it is harder to keep reproducible across contributors
- troubleshooting gets more complex

## Rollout Plan

1. confirm current build configuration in the root workspace
2. add root `profile.dev`
3. add root `profile.release-fast`
4. evaluate `.cargo/config.toml` and optional `sccache` support
5. review whether `default-members` helps or harms the current workflow
6. document command guidance in developer-facing docs
7. measure with `cargo --timings`

## Benefits

- faster local rebuilds
- clearer ownership of build settings
- better separation between local verification and production release
- lower risk of accidental build-policy drift between crates
- a better baseline before more crates or code generation are added

## Drawbacks

- more explicit Cargo configuration to maintain
- `release-fast` may hide some issues that only appear in full `release`
- `sccache` adds another tool to explain and troubleshoot if adopted
- `default-members` can confuse contributors if it stops matching common workflows

## Review Points

- Should this repository adopt `default-members` now, or wait until there is a clearer pain point?
- Should `sccache` support be committed in-repo, or documented as an optional local setup first?
- Should a future RFC also cover test execution latency, such as `nextest`, scoped test workflows, or fixture DB startup cost?
