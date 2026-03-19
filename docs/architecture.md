# Workspace Architecture

Last updated: 2026-03-15

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
- `AppModule` still owns adapter-backed concrete implementations, but handlers now call usecase traits on that module rather than reaching into repositories directly.

## Runtime composition

Entry point: `creation-driver/src/bin/main.rs`

Startup flow:

1. Load environment variables (`dotenv` + `Config::init`).
2. Initialize tracing subscriber (JSON logs).
3. Build `AppModule` (`RepositoryImpl<UserTable>`, `RepositoryImpl<DiagramTable>`, `RepositoryImpl<EntityTable>`, `RepositoryImpl<PersonTable>`).
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
- `PATCH /api/diagrams/update/{id}`
- `DELETE /api/diagrams/delete/{id}`
- `GET /api/persons`
- `POST /api/persons/create`
- `PATCH /api/persons/update/{entity_id}`
- `DELETE /api/persons/delete/{entity_id}`

Request processing pattern (current):

1. Handler receives JSON/body and shared `AppState`.
2. Handler calls usecase methods via `data.driver`.
3. Usecase/service layers validate input and coordinate reads/writes.
4. Adapter performs SQLx query / transaction.
5. Domain/service/usecase error enums are mapped to HTTP responses.

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
- Entity repository: `creation-adapter/src/repository/entity.rs`
- Person repository: `creation-adapter/src/repository/person.rs`
- Transaction port: `creation-adapter/src/repository/transaction.rs`
  This provides the shared SQLx transaction state used by transaction-aware repositories. `begin_transaction()` returns a service container whose repositories switch between pool and transaction internally.

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

1. `creation-driver` still constructs adapter-backed concrete modules directly in `AppModule`, even though request execution now goes through usecase traits.
2. Reverse dependency in adapter:
   `creation-adapter` currently depends on `creation-usecase` (marked TODO in `creation-adapter/Cargo.toml`).
3. Trait layering exists (`Provides*` / `Uses*`) but is not fully enforced by module boundaries at runtime wiring.

These are refactor targets if strict clean architecture boundaries are required.
