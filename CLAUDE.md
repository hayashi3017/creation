# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Start DB container
cargo run -p xtask -- docker

# Drop, create, and migrate database
cargo run -p xtask -- migrate

# Run API server (reads .env)
cargo run -p creation-driver

# Run all tests
cargo run -p xtask -- test

# Run scoped tests (convention-based: resolves creation-driver/tests/<scope>.rs and creation-adapter/tests/<scope>_repository.rs)
cargo run -p xtask -- test-scope <scope>
cargo run -p xtask -- test-scope diagram
cargo run -p xtask -- test-scope entity auth     # multiple scopes
cargo run -p xtask -- test-scope full            # full workspace
cargo run -p xtask -- test-scope package:xtask   # single crate
cargo run -p xtask -- test-scope driver:<test>   # explicit driver test target
cargo run -p xtask -- test-scope adapter:<test>  # explicit adapter test target

# Scoped test with output
cargo run -p xtask -- test -p creation-driver --test auth -- --nocapture

# Refresh sqlx query cache after changing query!/query_as! macros
cargo run -p xtask -- sqlx-prepare

# Check sqlx cache is current (CI verification)
cargo run -p xtask -- sqlx-prepare --check

# Format
cargo fmt

# Migration status
cargo run -p xtask -- migrate-info
```

Port: `8001` (debug) / `8000` (release), set via `RUNTIME_MODE` env var.
OpenAPI JSON: `/api-docs/openapi.json`, Swagger UI: `/swagger-ui`.

## Architecture

Five workspace crates with strict layering:

```
creation-driver  →  creation-usecase  →  creation-service  →  creation-adapter
     (HTTP)           (app use cases)      (domain logic)        (PostgreSQL)
```

- **creation-driver**: Axum handlers, JWT auth middleware, router, runtime bootstrap. Entry point: `src/bin/main.rs`. State: `AppState { driver: AppModule, env: Config }`.
- **creation-usecase**: Coordinates multi-step flows (e.g., `create_person` writes entity + diagram_entity + person in one transaction).
- **creation-service**: Domain trait definitions (`Provides*`, `Uses*`) and validation helpers. No external dependencies.
- **creation-adapter**: SQLx implementations of service traits. Migrations in `migrations/`. Transaction port in `src/repository/transaction.rs` provides a transaction-aware service container.
- **xtask**: Dev tooling (migrate, test, docker, sqlx-prepare wrappers).

### Data model

Core entities: `world` → `diagram`, `entity` (polymorphic: `kind=person`), `person`, `relationship`, `tree_path` (closure table for parent/adoptive_parent edges, rebuilt on each relationship write).

Entities and persons are written together in a single transaction; soft-delete via `deleted_at`.

### Request flow

```
HTTP → driver handler → usecase → service validation → adapter repository/transaction → sqlx → HTTP response
```

Auth middleware reads JWT from cookie `token` or `Authorization: Bearer`, loads user from DB, and injects `UserTable` into request extensions.

### Testing

- **API integration tests**: `creation-driver/tests/` — use `sqlx::test` with fixture SQL files in `tests/fixtures/`.
- **Repository tests**: `creation-adapter/tests/` with fixtures in `tests/fixtures/`.
- After any implementation, run the smallest relevant `test-scope` before reporting done (unless docs-only).

### Configuration

`.env` (local) and `.env.docker` (container). Required vars: `DATABASE_URL`, `JWT_SECRET`, `JWT_EXPIRED_IN`, `JWT_MAXAGE`, `RUNTIME_MODE`.

`xtask` normalizes `host.docker.internal` → `localhost` and appends `sslmode=disable` for host-side commands; direct `sqlx` CLI commands should use the explicit local URL.

## Conventions

- Commit messages: `type: summary` (e.g., `add: ...`, `fix: ...`, `chore: ...`).
- Follow-up work → `docs/improvements.md`; design proposals → `docs/rfc/`; architectural decisions → `docs/adr/`.
- When updating an RFC after implementation, only update status/tracking sections — do not rewrite Background or other review context.
- `creation-adapter` currently has a reverse dependency on `creation-usecase` (marked TODO); this is a known refactor target.
