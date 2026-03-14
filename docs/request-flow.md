# Request Flow

Last updated: 2026-02-18

## Overview

This document describes the current request execution path from HTTP entry to database access.

## Global startup flow

Entry point: `creation-driver/src/bin/main.rs`

1. `dotenv()` loads `.env`.
2. `Config::init()` reads required env vars:
   - `DATABASE_URL`
   - `JWT_SECRET`
   - `JWT_EXPIRED_IN`
   - `JWT_MAXAGE`
   - `RUNTIME_MODE`
3. `AppModule::new()` creates repository implementations.
4. Router is built in `create_router(...)`.
5. CORS layer is attached.
6. Server listens on:
   - `8001` for debug mode
   - `8000` for release mode
7. Graceful shutdown is handled on Ctrl+C.

## Layer execution (current implementation)

Current runtime path:

```text
HTTP route
  -> driver handler
  -> adapter repository (RepositoryImpl<...>)
  -> sqlx query / transaction
  -> driver error mapping
  -> HTTP response
```

Note:

- Traits for usecase/service/repository layers exist, but handlers currently call adapter repositories directly via `AppState.driver`.

## Auth middleware flow

Middleware: `creation-driver/src/jwt_auth.rs::auth`

1. Read token from cookie `token`, else from `Authorization: Bearer`.
2. Decode JWT with `JWT_SECRET`.
3. Parse `claims.sub` as UUID.
4. Query `users` table by UUID.
5. Insert `UserTable` into request extensions.
6. Continue to protected handler.

Failure paths:

- No token -> `401` (`You are not logged in, please provide token`)
- Decode/UUID parse failure -> `401` (`Invalid token`)
- Missing user -> `401` (`The user belonging to this token no longer exists`)
- DB query error -> `500`

## Endpoint flows

### `POST /api/auth/register`

Handler: `creation-driver/src/handler/user.rs::register_user_handler`

1. Parse JSON into `RegisterUserSchema`.
2. Call `user_repository.regist_user(...)`.
3. Repository flow (`creation-adapter/src/repository/user.rs`):
   - Generate Argon2 hash
   - Begin transaction
   - Insert into `users`
   - Commit
4. Return:
   - `200` on success
   - `409` for duplicate user
   - `500` for DB/hash failure

### `POST /api/auth/login`

Handler: `creation-driver/src/handler/user.rs::login_user_handler`

1. Parse JSON into `LoginUserSchema`.
2. Call `user_repository.login_user(...)`.
3. Repository flow:
   - Query user by lowercase email
   - Verify Argon2 password
4. On success:
   - Create JWT (`sub`, `iat`, `exp`)
   - Set HttpOnly cookie `token`
   - Return JSON with `token`
5. Return:
   - `200` success
   - `400` invalid user/password
   - `500` DB failure

### `GET /api/users/me` (protected)

Handler: `creation-driver/src/handler/user.rs::get_me_handler`

1. Auth middleware injects `UserTable` into extensions.
2. Handler maps to `FilteredUser`.
3. Return `200` with user profile JSON.

### `GET /api/diagrams` (protected)

Handler: `creation-driver/src/handler/diagram.rs::get_diagrams`

1. Auth middleware validates token.
2. Parse JSON body into `GetDiagramsSchema` (currently `{}`).
3. Repository queries `diagram` where `deleted_at IS NULL`.
4. Rows are mapped to `Vec<Diagram>`.
5. Return:
   - `200` with list
   - `500` DB failure

### `POST /api/diagrams/create` (protected)

Handler: `creation-driver/src/handler/diagram.rs::create_diagram`

1. Auth middleware validates token.
2. Parse JSON into `CreateDiagramSchema`.
3. Service-level validation (through trait impl) checks `name` not empty.
4. Repository inserts into `diagram`.
5. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params / DB error mapping

### `POST /api/diagrams/update` (protected)

Handler: `creation-driver/src/handler/diagram.rs::update_diagram`

1. Auth middleware validates token.
2. Parse JSON into `UpdateDiagramSchema`.
3. Service-level validation checks:
   - `id != 0`
   - `name` not empty
4. Repository updates the active `diagram` row and refreshes `updated_at`.
5. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params / DB error mapping

### `POST /api/diagrams/delete` (protected)

Handler: `creation-driver/src/handler/diagram.rs::delete_diagram`

1. Auth middleware validates token.
2. Parse JSON into `DeleteDiagramSchema`.
3. Service-level validation checks `id != 0`.
4. Repository soft-deletes the active `diagram` row by setting `deleted_at`.
5. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params / DB error mapping

### `GET /api/entities` (protected)

Handler: `creation-driver/src/handler/entity.rs::get_entities`

1. Auth middleware validates token.
2. Parse JSON body into `GetEntitiesSchema`.
3. Service-level validation checks `diagram_id != 0`.
4. Repository queries `entity` where `deleted_at IS NULL` and `diagram_id = ?`.
5. Rows are mapped to `Vec<Entity>`.
6. Return:
   - `200` with list
   - `400` invalid params
   - `500` DB failure

### `POST /api/entities/create` (protected)

Handler: `creation-driver/src/handler/entity.rs::create_entity`

1. Auth middleware validates token.
2. Parse JSON into `CreateEntitySchema`.
3. Service-level validation checks:
   - `diagram_id != 0`
   - `name` not empty
4. Repository inserts into `entity`.
5. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params / DB error mapping

### `POST /api/entities/update` (protected)

Handler: `creation-driver/src/handler/entity.rs::update_entity`

1. Auth middleware validates token.
2. Parse JSON into `UpdateEntitySchema`.
3. Service-level validation checks:
   - `id != 0`
   - `diagram_id != 0`
   - `name` not empty
4. Repository updates the active `entity` row and refreshes `updated_at`.
5. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params / DB error mapping

### `POST /api/entities/delete` (protected)

Handler: `creation-driver/src/handler/entity.rs::delete_entity`

1. Auth middleware validates token.
2. Parse JSON into `DeleteEntitySchema`.
3. Service-level validation checks `id != 0`.
4. Repository soft-deletes the active `entity` row by setting `deleted_at`.
5. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params / DB error mapping

## Sequence snapshot (login -> me)

```text
Client -> POST /api/auth/login
  Driver handler -> User repository -> users table
  Driver handler -> issue JWT + Set-Cookie(token)
Client -> GET /api/users/me (with token)
  Auth middleware -> decode JWT -> users table
  Handler -> return filtered user
```

## Known behavior notes

- `GET /api/diagrams` expects a JSON body because the handler uses `Json<GetDiagramsSchema>`.
- `POST /api/diagrams/create` returns `200` with empty body in success path.
- `POST /api/diagrams/update` returns `200` with empty body in success path.
- `POST /api/diagrams/delete` returns `200` with empty body in success path.
- `GET /api/entities` expects a JSON body because the handler uses `Json<GetEntitiesSchema>`.
- `POST /api/entities/create` returns `200` with empty body in success path.
- `POST /api/entities/update` returns `200` with empty body in success path.
- `POST /api/entities/delete` returns `200` with empty body in success path.
- Error mapping status codes are not fully uniform across handlers yet.
