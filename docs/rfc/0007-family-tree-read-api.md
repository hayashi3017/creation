# RFC 0007: Family Tree Read API

- Status: `Draft`
- Last updated: `2026-03-23`

## Background

Current clients must call the existing aggregate endpoints separately to render a family tree:

- `GET /api/persons`
- `GET /api/relationships`

That leaves client code responsible for behavior that already belongs to the server-side domain model:

- verifying that the target `diagram` is actually `kind = family_tree`
- normalizing `relationship.kind = parent | child` into one consistent parent-to-child direction
- deriving root nodes for a forest view
- filtering out records that should not participate in a family-tree projection

The workspace already maintains `tree_path` as a lineage-only closure table, but there is no public read API that exposes a family-tree-oriented projection.

## Proposal

Add a protected read-only endpoint:

- `GET /api/family-trees/{diagram_id}`

This endpoint returns a normalized family-tree projection for one active `diagram(kind = family_tree)`.

`family-trees` should be treated as a derived read model, not as an independently writable aggregate.
Because of that, write operations should remain on the existing resources and path conventions:

- `POST /api/diagrams/create`
- `PATCH /api/diagrams/update/{id}`
- `DELETE /api/diagrams/delete/{id}`
- `POST /api/persons/create`
- `PATCH /api/persons/update/{entity_id}`
- `DELETE /api/persons/delete/{entity_id}`
- `POST /api/relationships/create`
- `PATCH /api/relationships/update/{id}`
- `DELETE /api/relationships/delete/{id}`

Do not add `create/update/delete` routes under `/api/family-trees` in v1. Those routes would duplicate ownership that already belongs to `diagram`, `person`, and `relationship`.

## Response Shape

```json
{
  "status": "success",
  "data": {
    "diagram": {
      "id": 1,
      "name": "Hayashi family",
      "kind": "family_tree",
      "description": "sample"
    },
    "root_entity_ids": [1],
    "nodes": [
      {
        "entity_id": 1,
        "diagram_id": 1,
        "name": "A",
        "description": null,
        "gender": "female",
        "birth_date": "1970-01-01",
        "death_date": null,
        "birthplace": null,
        "residence": null,
        "photo_url": null,
        "parent_entity_ids": [],
        "child_entity_ids": [2],
        "is_root": true
      }
    ],
    "edges": [
      {
        "relationship_id": 10,
        "parent_entity_id": 1,
        "child_entity_id": 2,
        "kind": "parent",
        "start_date": null,
        "end_date": null,
        "notes": null
      }
    ],
    "stats": {
      "person_count": 2,
      "edge_count": 1,
      "root_count": 1
    }
  }
}
```

### Node Contract

Each `node` should be based on the existing `Person` payload and extended with family-tree-specific adjacency metadata:

- `parent_entity_ids: Vec<usize>`
- `child_entity_ids: Vec<usize>`
- `is_root: bool`

This keeps the current person fields reusable while removing graph reconstruction work from the client.

### Edge Contract

Each `edge` should be lineage-oriented and stable for UI use:

- `relationship_id`
- `parent_entity_id`
- `child_entity_id`
- `kind` (the stored relationship kind for traceability)
- `start_date`
- `end_date`
- `notes`

Normalization rule:

- stored `parent` means `source_entity_id -> target_entity_id`
- stored `child` means `target_entity_id -> source_entity_id`

The response should always expose `parent_entity_id -> child_entity_id`, regardless of how the write API payload was expressed.

## Validation And Error Mapping

Recommended behavior:

- `diagram_id == 0` -> `400 BAD_REQUEST`
- diagram does not exist -> `404 NOT_FOUND`
- diagram is soft-deleted -> `404 NOT_FOUND`
- diagram exists but `kind != family_tree` -> `400 BAD_REQUEST`
- authentication missing/invalid -> `401 UNAUTHORIZED`
- unexpected DB or assembly failure -> `500 INTERNAL_SERVER_ERROR`

Read filtering rules:

- only active `person` aggregates are returned as nodes
- only active lineage relationships are returned as edges
- relationships whose endpoints are no longer active should be ignored by the projection

If legacy data ever bypasses write-side cycle protection and a cycle is detected during projection assembly, returning `409 CONFLICT` would be more precise than silently emitting a broken tree. This can be added if we decide to validate topology on read.

## Layer Design

### Driver

Add a dedicated handler and response DTO:

- `creation-driver/src/handler/family_tree.rs`
- `creation-driver/src/response.rs`
- `creation-driver/src/route/mod.rs`

Public contract:

- `GET /api/family-trees/:diagram_id`
- protected by the existing auth middleware
- no request body

OpenAPI:

- add a `FamilyTrees` tag
- keep English operation text externalized under `creation-driver/src/openapi_docs/en/operations/`

### Usecase

Add `creation-usecase/src/usecase/family_tree.rs`.

Responsibilities:

1. validate `diagram_id`
2. load the target diagram and reject non-`family_tree` kinds
3. load the persons in the diagram
4. load the relationships in the diagram
5. normalize lineage edges into one parent-to-child direction
6. derive roots and adjacency lists
7. return the assembled projection

This should be a dedicated read use case instead of calling existing HTTP handlers or duplicating driver-level response mapping.

### Service

Two implementation choices are reasonable:

- minimal v1: keep projection assembly inside the family-tree use case as pure helper functions
- cleaner reuse: extract person-list assembly and lineage normalization into a dedicated service module

The second option is preferable if we expect additional read models such as subtree, ancestors, or descendants.

### Adapter

Add one diagram lookup that returns the actual active diagram row, not just an existence flag.
Recommended shape:

- `get_diagram(body: GetDiagramSchema) -> Result<Option<Diagram>, ...>`

The initial read path can reuse existing person and relationship queries. No new `tree_path` read SQL is required in v1.

`tree_path` remains part of the write-side consistency model in v1, not part of the response payload.

## Projection Algorithm

1. Load the active diagram by `diagram_id`.
2. Reject unless `diagram.kind == family_tree`.
3. Load all active persons for the diagram.
4. Load all active relationships for the diagram.
5. Keep only lineage edges (`parent`, `child`).
6. Normalize each lineage edge into `parent_entity_id -> child_entity_id`.
7. Build `parent_entity_ids` and `child_entity_ids` for every node.
8. Derive `root_entity_ids` from nodes with zero parents.
9. Sort output deterministically.

Recommended sort order:

- `nodes` by `entity_id`
- `edges` by `relationship_id`
- `root_entity_ids` ascending

## Why Not Return `tree_path` Rows

Returning closure rows in the public response is not recommended for v1:

- `tree_path` size grows faster than direct edges
- most clients need direct parent-child edges, not every ancestor-descendant pair
- closure data can still be added later behind an explicit query option if a subtree or analytics use case needs it

## Test Plan

Add integration coverage under `creation-driver/tests/family_tree.rs`.

Minimum cases:

- returns normalized nodes, edges, and root ids for a valid family-tree diagram
- returns `404` for a missing or soft-deleted diagram
- returns `400` for a `correlation` diagram
- normalizes stored `child` relationships into parent-to-child output edges
- excludes deleted persons or deleted relationships from the projection
- supports multiple roots and returns a forest view
- requires authentication

## Open Questions

- Should v1 remain lineage-only even after the public relationship API starts accepting `spouse`, `sibling`, or `step_*` kinds?
- Do we want server-side layout hints later, such as generation depth or grouping, or should layout stay fully client-side?
- Should future subtree endpoints be added, for example `/api/family-trees/{diagram_id}/entities/{entity_id}/descendants`?
