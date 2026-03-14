# RFC 0001: Resource-Oriented API Shape

- Status: `Draft`
- Last updated: `2026-03-14`

## Background

Current Diagram / Entity endpoints have two inconsistencies:

- update and delete use `POST /.../update` and `POST /.../delete`
- `GET /api/diagrams` and `GET /api/entities` currently depend on JSON bodies

This makes client generation, caching, and route discovery harder than necessary.

## Proposal

Adopt resource-oriented paths in the next API revision.

### Diagram

- `GET /api/diagrams`
- `POST /api/diagrams`
- `PATCH /api/diagrams/:id`
- `DELETE /api/diagrams/:id`

### Entity

- `GET /api/diagrams/:diagram_id/entities`
- `POST /api/diagrams/:diagram_id/entities`
- `PATCH /api/entities/:id`
- `DELETE /api/entities/:id`

### Request shape

- Stop sending JSON bodies on `GET`
- Use path parameters for parent-child ownership such as `diagram_id`
- Reserve query parameters for future filters such as `kind`, pagination, or sort order

## Migration Plan

1. Add the new routes alongside the current ones.
2. Mark `/create`, `/update`, `/delete` style routes as deprecated in docs.
3. Remove body-dependent `GET` routes after clients migrate.

## Review Points

- Should entity update/delete also be nested under `/api/diagrams/:diagram_id/...`, or is a global `/api/entities/:id` better once the ID is known?
- Do we want to add `GET /api/diagrams/:id` and `GET /api/entities/:id` in the same revision for consistency?
