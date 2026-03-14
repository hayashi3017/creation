# Workspace Architecture

Last updated: 2026-02-18

## Goal

This workspace is a Rust monorepo for an HTTP API service with layered modules:

- `creation-driver`: HTTP routing, handlers, middleware, runtime bootstrap
- `creation-usecase`: application use cases
- `creation-service`: domain service and repository traits
- `creation-adapter`: database implementation (PostgreSQL + sqlx)
- `xtask`: developer operations (migrate/test/sqlx/docker wrappers)

Related directory map: `docs/directory-structure.md`

## Dependency structure (current)

`Cargo.toml` workspace members:

```text
creation-driver
creation-usecase
creation-service
creation-adapter
xtask
```

Current crate dependencies (observed from `Cargo.toml`):

```text
creation-driver  -> creation-usecase, creation-service, creation-adapter
creation-usecase -> creation-service
creation-adapter -> creation-service, creation-usecase   # TODO in code comment
creation-service -> (no local crate dependency)
xtask            -> (standalone)
```

Target layer direction in repository guidance:

```text
driver -> usecase -> service -> adapter
```

Implementation note:
- The current runtime path is functional, but not fully aligned with the target direction because `creation-driver` directly uses `creation-adapter::RepositoryImpl` in handlers/state.

## Runtime composition

Entry point: `creation-driver/src/bin/main.rs`

Startup flow:

1. Load environment variables (`dotenv` + `Config::init`).
2. Initialize tracing subscriber (JSON logs).
3. Build `AppModule` (`RepositoryImpl<UserTable>` and `RepositoryImpl<DiagramTable>`).
4. Build Axum router with shared `AppState`.
5. Attach CORS middleware and auth middleware per protected route.
6. Bind listener (`listenfd` if supplied, fallback to manual bind).
7. Run server with graceful shutdown on Ctrl+C.

Core state:

- `AppState` (`creation-driver/src/lib.rs`)
  - `driver: AppModule`
  - `env: Config`

## HTTP and application flow

Router definition: `creation-driver/src/route/mod.rs`

Public routes:

- `GET /api/healthchecker`
- `POST /api/auth/register`
- `POST /api/auth/login`

Protected routes (JWT auth middleware):

- `GET /api/auth/logout`
- `GET /api/users/me`
- `GET /api/diagrams`
- `POST /api/diagrams/create`
- `POST /api/diagrams/update`
- `POST /api/diagrams/delete`
- `GET /api/entities`
- `POST /api/entities/create`
- `POST /api/entities/update`
- `POST /api/entities/delete`

Request processing pattern (current):

1. Handler receives JSON/body and shared `AppState`.
2. Handler calls repository-backed methods via `data.driver.*_repository`.
3. Adapter performs SQLx query / transaction.
4. Domain/service/usecase error enums are mapped to HTTP responses.

## Authentication model

Files:

- `creation-driver/src/handler/user.rs`
- `creation-driver/src/jwt_auth.rs`

Behavior:

1. Login verifies credentials (Argon2 hashed password check in adapter repository).
2. Server issues JWT (`sub`, `exp`, `iat`) using `JWT_SECRET`.
3. Token is set as HttpOnly cookie (`token`) and also returned in response body.
4. Protected routes accept token from cookie or `Authorization: Bearer`.
5. Middleware decodes token, loads user from DB, and inserts user into request extensions.

## Data and persistence

Persistence implementation:

- DB pool wrapper: `creation-adapter/src/persistence/postgres.rs`
- User repository: `creation-adapter/src/repository/user.rs`
- Diagram repository: `creation-adapter/src/repository/diagram.rs`

Migrations:

- `creation-adapter/migrations/`

Schema/tooling commands are orchestrated by `xtask`:

- `migrate`
- `migrate-info`
- `sqlx-prepare`
- `test`
- `docker`

## Testing architecture

Main integration tests:

- `creation-driver/tests/`

Repository tests:

- `creation-adapter/tests/`

Fixtures:

- `creation-driver/tests/fixtures/`
- `creation-adapter/tests/fixtures/`

The suite uses `sqlx` test patterns and fixture SQL files for data setup.

## Current design gaps to track

1. Layer boundary mismatch:
   `creation-driver` directly depends on and calls adapter repositories.
2. Reverse dependency in adapter:
   `creation-adapter` currently depends on `creation-usecase` (marked TODO in `creation-adapter/Cargo.toml`).
3. Trait layering exists (`Provides*` / `Uses*`) but is not fully enforced by module boundaries at runtime wiring.

These are refactor targets if strict clean architecture boundaries are required.
