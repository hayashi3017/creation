# RFC 0010: Explicit ID Fields In Public API

- Status: `Accepted`
- Last updated: `2026-04-11`

## Background

RFC 0009 defines a schema-side move away from generic root-table primary key names such as `id`.

The public API still uses generic `id` in multiple places:

- response objects such as `user.id`, `diagram.id`, and `relationship.id`
- request bodies such as `UpdateDiagramSchema { id, ... }`
- path parameters such as `/api/diagrams/update/{id}` and `/api/relationships/delete/{id}`

That leaves the external contract inconsistent with the explicit identifier names already used elsewhere in the API:

- `diagram_id`
- `entity_id`
- `source_entity_id`
- `target_entity_id`
- `relationship_id`

Because this project has not been released yet, we do not need a backward-compatibility layer for the old `id` fields.

## Goal

Make public API identifiers resource-scoped and explicit everywhere instead of exposing generic `id`.

## Non-Goals

- changing resource names such as `/api/diagrams` or `/api/persons`
- changing non-identifier field names
- defining a versioned compatibility strategy for legacy clients

## Proposal

Change public API contracts so identifier fields use explicit names that match the represented resource.

Recommended rule:

- every externally visible identifier field should use a resource-scoped name such as `user_id`, `diagram_id`, `entity_id`, or `relationship_id`

This applies to:

- JSON response fields
- JSON request fields
- path parameter names
- OpenAPI parameter names and schemas

Because compatibility is not required, old `id` names should be removed rather than aliased.

## Naming Rule

Use singular resource-scoped names in the API contract.

Examples:

- `user_id`
- `diagram_id`
- `entity_id`
- `relationship_id`

Do not expose table-name-shaped identifiers such as `users_id` in the public API.

## Scope

### Response Models

The following response fields should become explicit:

- `FilteredUser.id` -> `user_id`
- `Diagram.id` -> `diagram_id`
- `Entity.id` -> `entity_id`
- `Relationship.id` -> `relationship_id`

Derived read models should also follow the same rule.

Examples:

- `FamilyTree.diagram.id` -> `FamilyTree.diagram.diagram_id`
- any future merged genealogy node ids should use `node_id`, not generic `id`

### Request Models

Request-body identifiers should become explicit.

Examples:

- `UpdateDiagramSchema.id` -> `diagram_id`
- `DeleteDiagramSchema.id` -> `diagram_id`
- `ExistsActiveDiagramSchema.id` -> `diagram_id`
- `GetDiagramSchema.id` -> `diagram_id`
- `UpdateEntitySchema.id` -> `entity_id`
- `DeleteEntitySchema.id` -> `entity_id`
- `UpdateRelationshipSchema.id` -> `relationship_id`
- `DeleteRelationshipSchema.id` -> `relationship_id`
- `LoadRelationshipDiagramIdSchema.id` -> `relationship_id`

Where a schema already uses an explicit identifier such as `entity_id` or `diagram_id`, leave it unchanged.

### Path Parameters

Path parameter names should also become explicit.

Examples:

- `/api/diagrams/update/{id}` -> `/api/diagrams/update/{diagram_id}`
- `/api/diagrams/delete/{id}` -> `/api/diagrams/delete/{diagram_id}`
- `/api/relationships/update/{id}` -> `/api/relationships/update/{relationship_id}`
- `/api/relationships/delete/{id}` -> `/api/relationships/delete/{relationship_id}`

Endpoints already using explicit names, such as `/api/persons/update/{entity_id}`, should remain as they are.

## Example Contract Changes

### User Response

Before:

```json
{
  "status": "success",
  "data": {
    "user": {
      "id": "uuid-string",
      "name": "alice"
    }
  }
}
```

After:

```json
{
  "status": "success",
  "data": {
    "user": {
      "user_id": "uuid-string",
      "name": "alice"
    }
  }
}
```

### Diagram Response

Before:

```json
{
  "id": 1,
  "name": "sample",
  "kind": "family_tree"
}
```

After:

```json
{
  "diagram_id": 1,
  "name": "sample",
  "kind": "family_tree"
}
```

### Relationship Response

Before:

```json
{
  "id": 10,
  "diagram_id": 1,
  "source_entity_id": 1,
  "target_entity_id": 2
}
```

After:

```json
{
  "relationship_id": 10,
  "diagram_id": 1,
  "source_entity_id": 1,
  "target_entity_id": 2
}
```

## Path And Body Consistency

When a route uses a resource identifier in the path, the path variable name and the body field name should match.

Examples:

- `PATCH /api/diagrams/update/{diagram_id}` should use `diagram_id` throughout the handler and usecase boundary
- `PATCH /api/relationships/update/{relationship_id}` should use `relationship_id` throughout the handler and usecase boundary

This avoids unnecessary translation between public names and internal names.

## No Compatibility Layer

Because the API has not been released yet:

- do not keep both `id` and `diagram_id`
- do not accept both `{id}` and `{diagram_id}`
- do not add serde aliases only for compatibility

Prefer one clean cutover to the explicit names.

## Implementation Notes

### Driver

Update:

- handler path parameter names
- OpenAPI path parameter descriptions
- response DTO examples
- API docs in `docs/api-overview.md` and `docs/request-flow.md`

### Service And Usecase

Update public request and response models to explicit identifier field names.

Where internal helper structs are not externally visible, it is still preferable to align them to avoid mixed naming inside the codebase.

### Adapter

Repository row mappings should align with RFC 0009 schema naming to avoid repeated aliasing back to generic `id`.

## Rollout Plan

1. finalize RFC 0009 schema-side naming
2. rename public API model fields to explicit ids
3. rename path parameters to explicit ids
4. update OpenAPI docs and examples
5. update all tests and fixtures that assert JSON field names
6. update `docs/api-overview.md` and `docs/request-flow.md`

This should be implemented as one focused change set rather than as piecemeal aliases.

## Benefits

- clearer API contracts
- better alignment between schema, Rust models, and OpenAPI
- less ambiguity in handlers and tests
- easier onboarding for future genealogy and merged-graph APIs

## Drawbacks

- broad mechanical changes across handlers, tests, OpenAPI, and docs
- existing local clients must be updated immediately once the change lands

## Open Questions

- Should create endpoints that currently return only `{ response: "ok" }` continue doing so, or should they eventually return explicit resource ids after creation?
- Should timestamp naming also be revisited for consistency, for example `createdAt` vs `created_at`, or should that stay out of scope?
