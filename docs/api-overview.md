# API Overview

Last updated: 2026-03-15

## Scope

This document reflects the current implementation in `creation-driver`.

Base path:

```text
/api
```

## Authentication

- Login issues JWT token in both:
  - response JSON (`token`)
  - HttpOnly cookie (`token`)
- Protected endpoints accept:
  - cookie token (`token`)
  - `Authorization: Bearer <jwt>`

Protected endpoints:

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

## Endpoints

| Method | Path | Auth | Summary |
| --- | --- | --- | --- |
| GET | `/api/healthchecker` | No | health check |
| POST | `/api/auth/register` | No | user registration |
| POST | `/api/auth/login` | No | login + token issue |
| GET | `/api/auth/logout` | Yes | clear auth cookie |
| GET | `/api/users/me` | Yes | current user profile |
| GET | `/api/diagrams` | Yes | list diagrams |
| POST | `/api/diagrams/create` | Yes | create diagram |
| PATCH | `/api/diagrams/update/{id}` | Yes | update diagram |
| DELETE | `/api/diagrams/delete/{id}` | Yes | soft delete diagram |
| GET | `/api/persons` | Yes | list persons in a diagram |
| POST | `/api/persons/create` | Yes | create entity + person |
| PATCH | `/api/persons/update/{entity_id}` | Yes | update entity + person |
| DELETE | `/api/persons/delete/{entity_id}` | Yes | soft delete entity + person |

## Request and response details

### GET `/api/healthchecker`

- Request body: none
- `200 OK`:

```json
{
  "status": "success",
  "message": "JWT Authentication in Rust using Axum, Postgres, and SQLX"
}
```

### POST `/api/auth/register`

- Request JSON:

```json
{
  "name": "alice",
  "email": "alice@example.com",
  "password": "secret"
}
```

- `200 OK`:

```json
{
  "status": "success",
  "data": { "response": "ok" }
}
```

- `409 CONFLICT` (duplicate email):

```json
{
  "status": "fail",
  "message": "User with that email already exists"
}
```

- `500 INTERNAL_SERVER_ERROR`: DB/hash failures

### POST `/api/auth/login`

- Request JSON:

```json
{
  "email": "alice@example.com",
  "password": "secret"
}
```

- `200 OK` + `Set-Cookie: token=...; HttpOnly; Path=/; SameSite=Lax`:

```json
{
  "status": "success",
  "token": "<jwt>"
}
```

- `400 BAD_REQUEST` (invalid credentials):

```json
{
  "status": "fail",
  "message": "Invalid email or password"
}
```

or

```json
{
  "status": "fail",
  "message": "Wrong password"
}
```

- `500 INTERNAL_SERVER_ERROR`: DB failures

### GET `/api/auth/logout`

- Auth required
- Request body: none
- `200 OK` + cookie clear header (`token` with negative max-age):

```json
{
  "status": "success"
}
```

### GET `/api/users/me`

- Auth required
- Request body: none
- `200 OK`:

```json
{
  "status": "success",
  "data": {
    "user": {
      "id": "uuid-string",
      "name": "alice",
      "email": "alice@example.com",
      "role": "user",
      "photo": "default.png",
      "createdAt": "2026-01-01T00:00:00Z",
      "updatedAt": "2026-01-01T00:00:00Z"
    }
  }
}
```

### GET `/api/diagrams`

- Auth required
- Request body: none

- `200 OK`:

```json
{
  "status": "success",
  "data": [
    {
      "id": 1,
      "name": "sample",
      "kind": "family_tree",
      "description": "optional"
    }
  ]
}
```

- `500 INTERNAL_SERVER_ERROR`: DB failures

### POST `/api/diagrams/create`

- Auth required
- Request JSON:

```json
{
  "name": "Family Tree",
  "kind": "family_tree",
  "description": "created from API"
}
```

- `description` is optional
- `name` is trimmed before validation and must fit `VARCHAR(255)`
- `description` is trimmed and stored as `NULL` when omitted, `null`, or blank

- `kind` enum values:
  - `family_tree`
  - `correlation`

- `200 OK`:
  - current handler returns empty body on success
- `400 BAD_REQUEST`:
  - invalid params (`name` empty)
  - duplicate/DB validation failures

### PATCH `/api/diagrams/update/{id}`

- Auth required
- Request JSON:

```json
{
  "name": "Family Tree v2",
  "kind": "correlation",
  "description": "updated from API"
}
```

- `description` is optional
- `name` is trimmed before validation and must fit `VARCHAR(255)`
- `description` is trimmed and stored as `NULL` when omitted, `null`, or blank

- `200 OK`:
  - current handler returns empty body on success
- `400 BAD_REQUEST`:
  - invalid params (`id` is `0` or `name` empty)
- `404 NOT_FOUND`:
  - target diagram does not exist
  - target diagram is already soft-deleted
- `500 INTERNAL_SERVER_ERROR`:
  - DB failures

### DELETE `/api/diagrams/delete/{id}`

- Auth required
- Request body: none

- `200 OK`:
  - current handler returns empty body on success
- `400 BAD_REQUEST`:
  - invalid params (`id` is `0`)
- `404 NOT_FOUND`:
  - target diagram does not exist
  - target diagram is already soft-deleted
- `500 INTERNAL_SERVER_ERROR`:
  - DB failures

### GET `/api/persons`

- Auth required
- Request JSON:

```json
{
  "diagram_id": 1
}
```

- `200 OK`:

```json
{
  "status": "success",
  "data": [
    {
      "entity_id": 1,
      "diagram_id": 1,
      "name": "sample person",
      "description": "optional",
      "gender": "female",
      "birth_date": "1995-03-10",
      "death_date": null,
      "birthplace": "Tokyo",
      "residence": "Nagoya",
      "photo_url": "https://example.com/person-1.png"
    }
  ]
}
```

- `400 BAD_REQUEST`: invalid `diagram_id`
- `500 INTERNAL_SERVER_ERROR`: DB failure

### POST `/api/persons/create`

- Auth required
- Request JSON:

```json
{
  "diagram_id": 1,
  "name": "Alice",
  "description": "created from API",
  "gender": "male",
  "birth_date": "2001-01-01",
  "death_date": null,
  "birthplace": "Yokohama",
  "residence": "Kobe",
  "photo_url": "https://example.com/create.png"
}
```

- `description` and all person-specific fields are optional
- `name` is trimmed before validation and must fit `VARCHAR(255)`
- `description` is trimmed and stored as `NULL` when omitted, `null`, or blank
- `birthplace` / `residence` are trimmed and must fit `VARCHAR(255)`
- `photo_url` is trimmed and must fit `VARCHAR(512)`
- blank optional strings are stored as `NULL`

- `200 OK`:
  - current handler returns empty body on success
- `400 BAD_REQUEST`:
  - invalid params (`diagram_id` is `0` or `name` empty)
  - DB failures

### PATCH `/api/persons/update/{entity_id}`

- Auth required
- Request JSON:

```json
{
  "diagram_id": 1,
  "name": "Alice Updated",
  "description": "updated from API",
  "gender": "unknown",
  "birth_date": "1996-04-01",
  "death_date": "2024-04-01",
  "birthplace": "Fukuoka",
  "residence": "Sendai",
  "photo_url": "https://example.com/update.png"
}
```

- all aggregate fields are supplied in the JSON body
- `name` is trimmed before validation and must fit `VARCHAR(255)`
- `description` is trimmed and stored as `NULL` when omitted, `null`, or blank
- provided string fields are trimmed and validated against DDL length limits
- blank optional strings are stored as `NULL`

- `200 OK`:
  - current handler returns empty body on success
- `400 BAD_REQUEST`:
  - invalid params (`entity_id` or `diagram_id` is `0`, `name` empty)
- `404 NOT_FOUND`:
  - target person aggregate does not exist
  - target entity or person row is already soft-deleted
- `500 INTERNAL_SERVER_ERROR`:
  - DB failures

### DELETE `/api/persons/delete/{entity_id}`

- Auth required
- Request body: none

- `200 OK`:
  - current handler returns empty body on success
- `400 BAD_REQUEST`:
  - invalid params (`entity_id` is `0`)
- `404 NOT_FOUND`:
  - target person aggregate does not exist
  - target entity or person row is already soft-deleted
- `500 INTERNAL_SERVER_ERROR`:
  - DB failures

## Common auth error responses

When auth middleware rejects a request:

- `401 UNAUTHORIZED`:

```json
{
  "status": "fail",
  "message": "You are not logged in, please provide token"
}
```

or

```json
{
  "status": "fail",
  "message": "Invalid token"
}
```

or

```json
{
  "status": "fail",
  "message": "The user belonging to this token no longer exists"
}
```

- `500 INTERNAL_SERVER_ERROR` if user fetch fails in middleware.
