# RFC 0008: Genealogy Aggregate, Diagram Merge, and Entity Visibility

- Status: `Draft`
- Last updated: `2026-03-28`

## Background

The current model treats `diagram` as the top-level container for `entity`, `person`, `relationship`, and `tree_path`.

That works for single-diagram editing, but it does not yet define how to:

- manage multiple family-tree diagrams as one mergeable unit
- render a merged family-tree view without copying all source rows into a new diagram
- reflect source diagram updates in the merged view after the merge has been configured
- express publication scope for the container being shared
- hide a person from public output while keeping visibility ownership on `entity`
- avoid the inconsistent state where the same real-world person is visible in one source diagram and hidden in another
- request centered views with JSON request data instead of query parameters

The current single-diagram family-tree read API should remain valid, but it is not enough for these multi-diagram requirements.

## Goals

- keep `diagram` as the write unit for existing CRUD on diagrams, persons, and relationships
- add a higher-level aggregate that groups diagrams for merge and publication purposes
- avoid deep-copy merge semantics so source updates naturally flow into merged reads
- keep hidden-state ownership on `entity`
- guarantee consistent hidden-state behavior for linked entities that represent the same person
- support centered read requests through JSON request bodies

## Non-Goals

- automatic same-person matching heuristics
- final ACL or collaborator implementation details
- replacing the existing single-diagram `GET /api/family-trees/{diagram_id}` endpoint

## Naming Options

Several aggregate names are reasonable:

- `genealogy`: recommended
- `family-space`
- `family-network`
- `diagram-group`

Recommended choice: `genealogy`

Why:

- it is domain-oriented rather than UI-oriented
- it is broader than a single tree and can contain multiple roots and multiple source diagrams
- it is less vague than `family-space`
- it avoids the storage-oriented feel of `diagram-group`

Why not choose `family-space` as the primary term:

- `space` says "container" but not "shared genealogy"
- it does not strongly imply cross-diagram identity linking
- it reads more like a collaboration workspace than a genealogy domain object

## Proposal

Introduce a new top-level aggregate named `genealogy`.

Responsibilities:

- own the set of source diagrams that should be rendered together
- own publication metadata for the merged share unit
- own cross-diagram same-person linkage metadata
- provide merged family-tree read projections

Keep `diagram` as the source-of-truth write unit:

- `diagram` still owns diagram metadata
- `person` writes still mutate `entity` + `person`
- `relationship` writes still stay diagram-scoped
- `tree_path` maintenance remains diagram-scoped

Do not create a merged copy diagram as the primary model.

Instead, define merge as:

- a genealogy references multiple source diagrams
- a genealogy links entities across those diagrams when they represent the same real-world person
- merged reads are derived from source rows plus genealogy-owned linkage metadata

This keeps post-merge updates visible without requiring copy-back synchronization.

## Data Model

### New Tables

#### `genealogy`

Purpose:

- top-level merge and publication unit for multiple diagrams

Suggested columns:

- `id BIGSERIAL PRIMARY KEY`
- `name VARCHAR(255) NOT NULL`
- `description TEXT`
- `publication_scope publication_scope NOT NULL DEFAULT 'private'`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `deleted_at TIMESTAMPTZ`

#### `genealogy_diagram`

Purpose:

- attach source diagrams to a genealogy

Suggested columns:

- `genealogy_id BIGINT NOT NULL REFERENCES genealogy(id) ON DELETE CASCADE`
- `diagram_id BIGINT NOT NULL REFERENCES diagram(id) ON DELETE CASCADE`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `PRIMARY KEY (genealogy_id, diagram_id)`

#### `genealogy_identity_cluster`

Purpose:

- represent one same-person cluster inside a genealogy without creating a separate canonical person write model

Suggested columns:

- `id BIGSERIAL PRIMARY KEY`
- `genealogy_id BIGINT NOT NULL REFERENCES genealogy(id) ON DELETE CASCADE`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`

#### `genealogy_identity_member`

Purpose:

- attach source entities to a same-person cluster

Suggested columns:

- `genealogy_id BIGINT NOT NULL REFERENCES genealogy(id) ON DELETE CASCADE`
- `cluster_id BIGINT NOT NULL REFERENCES genealogy_identity_cluster(id) ON DELETE CASCADE`
- `entity_id BIGINT NOT NULL REFERENCES entity(id) ON DELETE CASCADE`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `PRIMARY KEY (genealogy_id, entity_id)`

This keeps one entity in at most one identity cluster per genealogy.

### Column Additions

#### `diagram`

Add:

- `publication_scope publication_scope NOT NULL DEFAULT 'private'`

This controls whether the diagram may participate in public or authenticated reads when accessed directly or through a genealogy.

#### `entity`

Add:

- `visibility_mode entity_visibility_mode NOT NULL DEFAULT 'visible'`

Recommended enum shape:

- `visible`
- `hidden`

The hidden-state owner remains `entity`, not the identity cluster.

### New Enums

#### `publication_scope`

Recommended initial values:

- `private`
- `authenticated`
- `public`

`authenticated` is useful even before a richer collaborator model exists because the system already distinguishes authenticated access from public access.

#### `entity_visibility_mode`

Recommended initial values:

- `visible`
- `hidden`

If the product later needs placeholder nodes instead of omission, add a future `masked` mode instead of overloading `hidden`.

## Merge Semantics

Merge should be logical, not physical.

Algorithm:

1. attach source diagrams to a genealogy
2. create identity clusters for entities that represent the same person across attached diagrams
3. load active entities and relationships from attached diagrams
4. collapse linked entities into one merged node per identity cluster
5. carry unlinked entities through as standalone merged nodes
6. normalize lineage relationships into parent-to-child direction
7. collapse duplicate edges created by linked-entity unification
8. filter by publication scope and entity visibility
9. return the merged projection

The merged view should never become the source-of-truth write model.

## Why This Reflects Source Updates

No deep-copy merged diagram is stored.

Instead:

- source `entity` and `relationship` rows stay in their original diagrams
- genealogy-owned metadata only describes which diagrams belong together and which entities should be treated as the same person
- merged family-tree reads are rebuilt from live source data

That means:

- editing a person in diagram A changes the next merged read automatically
- editing a relationship in diagram B changes the next merged read automatically
- hiding an entity changes the next merged read automatically

If response caching or materialized projections are added later, invalidate them by affected `genealogy_id` on source writes rather than changing the source-of-truth model.

## Visibility And Publication Rules

### Ownership

- publication scope is owned by `diagram` and `genealogy`
- hidden-state is owned by `entity`

### Effective Publication Rule

Merged reads should include a source diagram only when both of these allow the requested audience:

- the genealogy publication scope
- the source diagram publication scope

Use the more restrictive result.

Example:

- genealogy = `public`
- diagram A = `private`
- diagram B = `public`

Public merged reads may include diagram B data but not diagram A data.

### Effective Hidden Rule

The write owner remains `entity.visibility_mode`, but linked same-person entities must not diverge in a merged genealogy.

Recommended rule:

1. visibility writes target a single entity id
2. the write use case resolves its genealogy identity cluster memberships
3. the same visibility value is propagated to all active member entities in the same cluster, in the same transaction

Read-side safety rule:

- if legacy data or manual DB edits leave a cluster inconsistent, the merged read should treat the cluster as hidden if any active member entity is hidden

This keeps ownership on `entity` while preventing the "diagram A visible / diagram B hidden" split for the same person.

### Hidden-Entity Output Behavior

For v1 merged public reads, hidden entities should be omitted from the graph rather than returned as placeholders.

This is simpler than placeholder masking and avoids accidentally leaking names or metadata.

Future masking can be added later as a separate mode.

## Read Model Shape

Do not reuse the single-diagram `FamilyTreeNode.entity_id` as the merged node identifier.

In a merged genealogy, one node may represent:

- one unlinked entity, or
- multiple linked entities across diagrams

The merged read model should therefore use a stable merged node id.

Recommended shape:

```json
{
  "status": "success",
  "data": {
    "genealogy": {
      "id": 10,
      "name": "Hayashi genealogy",
      "publication_scope": "public"
    },
    "nodes": [
      {
        "node_id": "cluster:42",
        "source_entity_ids": [100, 240],
        "source_diagram_ids": [1, 4],
        "representative_entity_id": 100,
        "name": "A",
        "parent_node_ids": [],
        "child_node_ids": ["entity:302"],
        "is_root": true
      }
    ],
    "edges": [
      {
        "edge_id": "cluster:42->entity:302",
        "source_relationship_ids": [500, 880],
        "parent_node_id": "cluster:42",
        "child_node_id": "entity:302"
      }
    ]
  }
}
```

Notes:

- `source_entity_ids` preserves traceability back to source entities
- `representative_entity_id` gives the frontend one concrete entity id for drill-down
- `source_relationship_ids` preserves traceability when multiple source edges collapse into one merged edge

## API Shape

### Management Endpoints

Suggested write endpoints:

- `POST /api/genealogies/create`
- `PATCH /api/genealogies/update/{id}`
- `DELETE /api/genealogies/delete/{id}`
- `POST /api/genealogies/{id}/diagrams/attach`
- `POST /api/genealogies/{id}/diagrams/detach`
- `POST /api/genealogies/{id}/identity-clusters/link`
- `POST /api/genealogies/{id}/identity-clusters/unlink`

Suggested visibility endpoint:

- `PATCH /api/persons/visibility/{entity_id}`

That endpoint still mutates `entity.visibility_mode` under the current public aggregate model.

### Read Endpoints

Keep the existing simple read:

- `GET /api/family-trees/{diagram_id}`

Add a merged read endpoint for genealogy projections:

- `POST /api/genealogies/{id}/family-trees/view`

Do not use query parameters for centered views or publication previews.

Pass read controls as JSON request data:

```json
{
  "data": {
    "center_entity_id": 240,
    "ancestor_depth": 2,
    "descendant_depth": 3,
    "included_diagram_ids": [1, 4],
    "audience": "public"
  }
}
```

Recommended request fields:

- `center_entity_id: Option<usize>`
- `ancestor_depth: Option<usize>`
- `descendant_depth: Option<usize>`
- `included_diagram_ids: Option<Vec<usize>>`
- `audience: Option<publication_scope>`

Behavior:

- if `center_entity_id` is omitted, return the full merged genealogy view
- if `center_entity_id` is present, resolve it to its identity cluster if linked
- then return only the requested ancestor / descendant window around that center

Using `POST` for this read is acceptable because the request carries non-trivial structured view instructions and must not be encoded as query parameters.

## Centered View Semantics

Centered views should be defined at the genealogy layer, not as a mutation of the single-diagram family-tree endpoint.

Recommended behavior:

- input center is an `entity_id`
- if that entity belongs to an identity cluster in the target genealogy, use the merged node for that cluster as the center
- `ancestor_depth = 0` means no ancestors
- `descendant_depth = 0` means no descendants
- if both depths are omitted, return the full merged graph

This keeps the API compatible with the current entity-centric data model while still giving the frontend a merged-person experience.

## Layer Design

### Driver

Add new handlers and OpenAPI docs for:

- genealogy CRUD
- diagram attach / detach
- identity-cluster link / unlink
- merged family-tree view
- person visibility update

### Usecase

Add a dedicated genealogy use case layer that handles:

- diagram membership validation
- identity cluster validation
- cross-entity visibility propagation
- merged graph assembly
- centered-graph trimming

### Service

Extract projection logic from the current family-tree use case into reusable graph assembly helpers once genealogy view work starts.

This aligns with the existing expectation that subtree or descendant read models may be added later.

### Adapter

Add repositories for:

- genealogy
- genealogy_diagram
- genealogy_identity_cluster
- genealogy_identity_member

Add bulk read helpers to:

- load all active entities by multiple diagram ids
- load all active person records by entity ids
- load all active relationships by multiple diagram ids
- resolve entity memberships to genealogy clusters

## Validation And Error Mapping

Recommended behavior:

- missing genealogy -> `404 NOT_FOUND`
- soft-deleted genealogy -> `404 NOT_FOUND`
- attach diagram not found or deleted -> `404 NOT_FOUND`
- attach diagram of wrong kind -> `400 BAD_REQUEST`
- link entities from diagrams not attached to the genealogy -> `400 BAD_REQUEST`
- link non-person entities -> `400 BAD_REQUEST`
- request body with invalid depth or ids -> `400 BAD_REQUEST`
- visibility update on missing entity -> `404 NOT_FOUND`
- unauthorized access for requested audience -> `401` or `403` depending on auth policy
- unexpected DB or assembly failure -> `500 INTERNAL_SERVER_ERROR`

## Rollout Plan

1. add new enums and columns for `publication_scope` and `entity.visibility_mode`
2. add genealogy and membership tables
3. add attach / detach APIs
4. add identity-cluster link / unlink APIs
5. add visibility update API with cluster-wide propagation
6. add merged family-tree `view` endpoint
7. optionally add projection cache invalidation if live assembly becomes too expensive

## Open Questions

- Should one `diagram` be attachable to multiple genealogies, or should membership be exclusive?
- Do we need explicit cross-diagram relationships beyond same-person clustering?
- Should hidden nodes be fully omitted in all audiences, or should authenticated private reads be able to request placeholders later?
- Should `audience` stay as an explicit read-body field, or should it eventually be inferred only from auth context plus endpoint choice?
