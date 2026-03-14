# RFC 0002: Mutation Result And Error Mapping

- Status: `Accepted`
- Last updated: `2026-03-14`

## Background

Current soft-delete based update/delete handlers have two gaps:

- they return `200 OK` even when no active row matched the target ID
- some database failures are mapped to `400 Bad Request`

This makes it hard for clients to distinguish invalid input, missing resources, and server failures.

## Proposal

Standardize mutation outcomes for Diagram / Entity handlers as follows.

### HTTP status mapping

- `400 Bad Request`: malformed request or validation failure
- `404 Not Found`: target row does not exist or is already soft-deleted
- `409 Conflict`: business rule violation such as an unsupported state transition
- `500 Internal Server Error`: database or infrastructure failure

### Repository / service behavior

- update and delete should inspect `rows_affected()`
- when `rows_affected() == 0`, repository or service should return a typed `NotFound` error
- validation errors should remain separate from database errors

## Scope

- `update_diagram`
- `delete_diagram`
- `update_entity`
- `delete_entity`

## Migration Plan

1. Add `NotFound` variants to repository/service/usecase errors.
2. Update handlers to map typed errors to `404`.
3. Move database-originated unexpected errors to `500`.

## Implementation Status

- `update_diagram`, `delete_diagram`, `update_entity`, and `delete_entity` now inspect `rows_affected()` through typed repository errors.
- When no active row matches the target ID, including already soft-deleted rows, handlers return `404 Not Found`.
- Database-originated failures in these update/delete handlers now map to `500 Internal Server Error`.
- User APIs remain out of scope for this pass.

## Review Points

- Is `already soft-deleted` best represented as `404`, or do we want a distinct `409` path?
- Do we want the same status mapping policy applied to User APIs in the same pass?
