# Request Flow

Last updated: 2026-04-11

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
  -> usecase
  -> service validation / coordination
  -> adapter repository or unit of work
  -> sqlx query / transaction
  -> driver error mapping
  -> HTTP response
```

Note:

- `AppState.driver` is still an adapter-backed concrete module, but handlers call usecase traits on that module.

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
2. Handler constructs `GetDiagramsSchema` without requiring a request body.
3. Repository queries `diagram` where `deleted_at IS NULL`.
4. Rows are mapped to `Vec<Diagram>`.
5. Return:
   - `200` with list
   - `500` DB failure

### `POST /api/diagrams/create` (protected)

Handler: `creation-driver/src/handler/diagram.rs::create_diagram`

1. Auth middleware validates token.
2. Parse JSON into `CreateDiagramSchema`.
3. Service-level validation trims `name`, rejects empty / overlong values, and normalizes blank or missing `description` to `NULL`.
4. Repository inserts into `diagram`.
5. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params / DB error mapping

### `POST /api/diagrams/entities/create` (protected)

Handler: `creation-driver/src/handler/diagram.rs::create_diagram_entity_membership`

1. Auth middleware validates token.
2. Parse JSON into `CreateDiagramEntityMembershipSchema`.
3. Service-level validation checks `diagram_id != 0` and `entity_id != 0`.
4. Repository inserts or reactivates `diagram_entity` only when the diagram and entity are active and belong to the same world.
5. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params
   - `404` missing, deleted, or cross-world diagram/entity
   - `500` DB failure

### `PATCH /api/diagrams/update/{diagram_id}` (protected)

Handler: `creation-driver/src/handler/diagram.rs::update_diagram_by_diagram_id`

1. Auth middleware validates token.
2. Read `diagram_id` from path and JSON body into the update request payload.
3. Service-level validation checks:
   - `diagram_id != 0`
   - trimmed `name` is not empty
   - trimmed `name` fits `VARCHAR(255)`
   - blank or missing `description` is normalized to `NULL`
4. Repository updates the active `diagram` row and refreshes `updated_at`.
5. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params
   - `404` target missing or already soft-deleted
   - `500` DB failure

### `DELETE /api/diagrams/delete/{diagram_id}` (protected)

Handler: `creation-driver/src/handler/diagram.rs::delete_diagram_by_diagram_id`

1. Auth middleware validates token.
2. Read `diagram_id` from path.
3. Service-level validation checks `diagram_id != 0`.
4. Repository soft-deletes the active `diagram` row by setting `deleted_at`.
5. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params
   - `404` target missing or already soft-deleted
   - `500` DB failure

### `GET /api/persons` (protected)

Handler: `creation-driver/src/handler/person.rs::get_persons_by_diagram`

1. Auth middleware validates token.
2. Parse JSON body into `GetPersonsSchema`.
3. Usecase validates `diagram_id != 0`.
4. Usecase loads active `entity(kind=person)` rows through `entity_service`.
5. Usecase loads active `person` rows through `person_service`.
6. Usecase merges both results into `Vec<Person>`.
7. Return:
   - `200` with list
   - `400` invalid params
   - `500` DB failure

### `POST /api/persons/create` (protected)

Handler: `creation-driver/src/handler/person.rs::create_person`

1. Auth middleware validates token.
2. Parse JSON body into `CreatePersonSchema`.
3. Usecase normalizes the aggregate payload using service-level validation helpers:
   - `world_id != 0`
   - optional `diagram_id`, when provided, must identify an active diagram in the same world
   - trimmed `name` is not empty
   - trimmed `name` fits `VARCHAR(255)`
   - blank or missing `description` is normalized to `NULL`
   - `birthplace` / `residence` fit `VARCHAR(255)` after trim
   - `photo_url` fits `VARCHAR(512)` after trim
   - blank optional strings are normalized to `NULL`
4. Usecase begins the service-side transaction port and gets a transaction-aware service container.
5. The transactional `entity` service inserts `entity(world_id, kind=person)` and returns `entity_id`.
6. When `diagram_id` is provided, the transactional `entity` service creates the `diagram_entity` membership.
7. The transactional `person` service inserts the matching `person` row.
8. The transaction context commits.
9. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params
   - `500` DB failure

### `PATCH /api/persons/update/{entity_id}` (protected)

Handler: `creation-driver/src/handler/person.rs::update_person_by_entity_id`

1. Auth middleware validates token.
2. Read `entity_id` from path and parse JSON body into the update request payload.
3. Usecase normalizes the aggregate payload using service-level validation helpers:
   - `entity_id != 0`
   - trimmed `name` is not empty
   - trimmed `name` fits `VARCHAR(255)`
   - blank or missing `description` is normalized to `NULL`
   - provided string fields fit DDL limits after trim
   - blank optional strings are normalized to `NULL`
4. Usecase begins the service-side transaction port and gets a transaction-aware service container.
5. The transactional `entity` service updates the active `entity(kind=person)` row.
6. The transactional `person` service updates the active `person` row.
7. The transaction context commits.
8. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params
   - `404` missing / soft-deleted `entity` or `person`
   - `500` DB failure

### `DELETE /api/persons/delete/{entity_id}` (protected)

Handler: `creation-driver/src/handler/person.rs::delete_person_by_entity_id`

1. Auth middleware validates token.
2. Read `entity_id` from path.
3. Usecase validates `entity_id != 0`.
4. Usecase begins the service-side transaction port and gets a transaction-aware service container.
5. The transactional `entity` service soft-deletes the active `entity` row.
6. The transactional `person` service soft-deletes the active `person` row.
7. The transaction context commits.
8. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params
   - `404` missing / soft-deleted `entity` or `person`
   - `500` DB failure

### `GET /api/relationships` (protected)

Handler: `creation-driver/src/handler/relationship.rs::get_relationships_by_diagram`

1. Auth middleware validates token.
2. Parse JSON body into `GetRelationshipsSchema`.
3. Service validates `diagram_id != 0`.
4. Repository loads active `relationship` rows in that diagram.
5. Return:
   - `200` with list
   - `400` invalid params
   - `500` DB failure

### `POST /api/relationships/create` (protected)

Handler: `creation-driver/src/handler/relationship.rs::create_relationship`

1. Auth middleware validates token.
2. Parse JSON body into `CreateRelationshipSchema`.
3. Service-level validation checks:
   - `diagram_id != 0`
   - `source_entity_id != 0`
   - `target_entity_id != 0`
   - `source_entity_id != target_entity_id`
   - only canonical stored relationship kinds are accepted
   - symmetric kinds normalize endpoints so `source_entity_id < target_entity_id`
   - if both dates are present, `start_date <= end_date`
   - blank or missing `end_reason` is normalized to `NULL`
   - blank or missing `notes` is normalized to `NULL`
4. Usecase begins the service-side transaction port and gets a transaction-aware service container.
5. Transactional repository inserts the active `relationship` row only if the diagram and both entities exist, are active, and belong to the same diagram.
6. Transactional `tree_path` repository rebuilds the closure table for that diagram from active `parent` and `adoptive_parent` edges.
7. If cycle detection fails, the transaction is rolled back.
8. The transaction context commits.
9. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params / cycle detected
   - `404` missing / soft-deleted diagram or entity
   - `500` DB failure

### `PATCH /api/relationships/update/{relationship_id}` (protected)

Handler: `creation-driver/src/handler/relationship.rs::update_relationship_by_relationship_id`

1. Auth middleware validates token.
2. Read `relationship_id` from path and parse JSON body into the update request payload.
3. Service-level validation checks:
   - `relationship_id != 0`
   - `source_entity_id != 0`
   - `target_entity_id != 0`
   - `source_entity_id != target_entity_id`
   - only canonical stored relationship kinds are accepted
   - symmetric kinds normalize endpoints so `source_entity_id < target_entity_id`
   - if both dates are present, `start_date <= end_date`
   - blank or missing `end_reason` is normalized to `NULL`
   - blank or missing `notes` is normalized to `NULL`
4. Usecase begins the service-side transaction port and gets a transaction-aware service container.
5. Transactional repository updates the active `relationship` row while keeping its `diagram_id` unchanged.
6. Transactional `tree_path` repository rebuilds the closure table for that diagram.
7. If cycle detection fails, the transaction is rolled back.
8. The transaction context commits.
9. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params / cycle detected
   - `404` missing / soft-deleted relationship or entity
   - `500` DB failure

### `DELETE /api/relationships/delete/{relationship_id}` (protected)

Handler: `creation-driver/src/handler/relationship.rs::delete_relationship_by_relationship_id`

1. Auth middleware validates token.
2. Read `relationship_id` from path.
3. Service-level validation checks `relationship_id != 0`.
4. Usecase begins the service-side transaction port and gets a transaction-aware service container.
5. Transactional repository soft-deletes the active `relationship` row.
6. Transactional `tree_path` repository rebuilds the closure table for that relationship's diagram.
7. The transaction context commits.
8. Return:
   - `200` on success (empty body in current handler)
   - `400` invalid params
   - `404` missing / soft-deleted relationship
   - `500` DB failure

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

- `POST /api/diagrams/create` returns `200` with empty body in success path.
- `PATCH /api/diagrams/update/{diagram_id}` returns `200` with empty body in success path.
- `DELETE /api/diagrams/delete/{diagram_id}` returns `200` with empty body in success path.
- `GET /api/persons` expects a JSON body because the handler uses `Json<GetPersonsSchema>`.
- `GET /api/relationships` also expects a JSON body because the handler uses `Json<GetRelationshipsSchema>`.
- `POST /api/persons/create` returns `200` with empty body in success path.
- `PATCH /api/persons/update/{entity_id}` returns `200` with empty body in success path.
- `DELETE /api/persons/delete/{entity_id}` returns `200` with empty body in success path.
- `POST /api/relationships/create` returns `200` with empty body in success path.
- `PATCH /api/relationships/update/{relationship_id}` returns `200` with empty body in success path.
- `DELETE /api/relationships/delete/{relationship_id}` returns `200` with empty body in success path.
- Diagram / Person の update/delete は `404` / `500` を返し分けるが、create や User API を含めた全体の status mapping はまだ完全には統一されていない。
