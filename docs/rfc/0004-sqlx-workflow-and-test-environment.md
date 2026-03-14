# RFC 0004: SQLx Workflow And Test Environment

- Status: `Accepted`
- Last updated: `2026-03-14`

## Background

This repository already has two recurring workflow issues:

- `SQLX_OFFLINE=true` means new `query!` / `query_as!` usage requires query cache refresh
- `sqlx::test` depends on a reachable setup database, but the bootstrap path is not documented tightly enough for all environments

## Proposal

Treat `xtask` as the documented entrypoint for SQLx and DB-backed test workflows.

### Recommended commands

- start local DB: `cargo run -p xtask -- docker`
- run migrations: `cargo run -p xtask -- migrate`
- refresh SQLx cache: `cargo run -p xtask -- sqlx-prepare`
- run scoped tests: `cargo run -p xtask -- test -p creation-driver --test entity`

### CI expectations

- provide a reachable PostgreSQL instance
- run migrations before DB-backed tests
- run SQLx cache verification as a dedicated step
- fail fast when `.env` / `DATABASE_URL` is missing

### Documentation updates

- document required environment variables in one place
- document the minimum local bootstrap order for contributors

## Rationale

`xtask` already contains project-specific behavior such as the SQLx prepare workaround for cross-device link failures, so duplicating raw cargo/sqlx commands in multiple places increases drift.

## Implementation Status

- CI now runs database migrations, verifies the checked-in SQLx cache with `cargo run -p xtask -- sqlx-prepare --check`, and executes tests with the same `cargo run -p xtask -- test` entrypoint documented for local development.
- Local contributor setup is documented in `docs/local-development.md`.
- `xtask test` now loads `.env` before spawning `cargo test`, so local runs inherit `DATABASE_URL` the same way other xtask database commands do.
- `xtask` host-side DB commands now normalize `DATABASE_URL=...@host.docker.internal...` to `localhost` before spawning `sqlx` / `cargo test`, avoiding setup DB failures from container-only hostnames.
- `xtask test` now uses `cargo test --no-fail-fast`, so one failing test binary does not stop the rest of the workspace suite from running.
- Installing `sqlx-cli` is still a manual prerequisite.

## Review Points

- Should CI run `cargo run -p xtask -- sqlx-prepare` directly, or should we add a dedicated `--check` mode first?
- Do we want a single contributor setup guide in `README.md`, or a dedicated `docs/local-development.md`?
