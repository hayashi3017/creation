# RFC 0009: Explicit Primary Key Column Names

- Status: `Accepted`
- Last updated: `2026-04-11`

## Background

The current schema mixes two naming styles:

- primary key columns on root tables use a generic `id`
- foreign key columns already use explicit names such as `diagram_id`, `entity_id`, `source_entity_id`, and `target_entity_id`

Examples from the current schema:

- `users.id`
- `diagram.id`
- `entity.id`
- `relationship.id`
- `entity.diagram_id`
- `person.entity_id`
- `relationship.diagram_id`

That inconsistency is manageable in simple single-table queries, but it becomes harder to read as joins and aggregate reads grow:

- `SELECT id` is ambiguous without an alias in almost every join
- SQL and SQLx row mappings need more aliasing to explain which `id` is being read
- future multi-diagram and genealogy reads will join more tables and make generic `id` harder to track

## Goal

Define one naming policy for primary key columns so the database schema is self-descriptive.

## Non-Goals

- changing public HTTP field names in this RFC
- renaming routes or resource names
- changing tables whose primary key is already semantically explicit

## Proposal

Rename generic primary key columns on root tables from `id` to explicit resource-scoped names.

Recommended rule:

- if a table has a dedicated surrogate primary key, that column should be named with an explicit resource-scoped name such as `user_id`, `diagram_id`, `entity_id`, or `relationship_id`

Examples:

- `users.id` -> `user_id`
- `diagram.id` -> `diagram_id`
- `entity.id` -> `entity_id`
- `relationship.id` -> `relationship_id`

## Why Use Resource-Scoped Names

Pros:

- deterministic and mechanical
- avoids introducing one-off singularization rules
- keeps the requested `user_id` / `diagram_id` pattern consistent

Cons:

- `user_id` does not mirror the plural table name `users` exactly

That mismatch is acceptable because the primary objective is identifier clarity at the column and API boundary, not literal repetition of the table name.

## Scope

### Tables That Should Change

- `users`
- `diagram`
- `entity`
- `relationship`

### Tables That Should Not Change

- `person.entity_id`
- `tree_path.ancestor_id`
- `tree_path.descendant_id`

Reason:

- these columns are already explicit
- they are not generic root-table `id` columns
- `tree_path` uses semantic role names rather than a synthetic row id

### Future Tables

Any new table introduced later should follow the same rule if it has a dedicated surrogate primary key.

Examples for future work:

- `genealogy.genealogy_id`
- `genealogy_identity_cluster.genealogy_identity_cluster_id`

Join tables or membership tables may continue to use composite keys when that better represents the data model.

## Naming Matrix

Recommended target names:

| Table | Current PK | Target PK |
| --- | --- | --- |
| `users` | `id` | `user_id` |
| `diagram` | `id` | `diagram_id` |
| `entity` | `id` | `entity_id` |
| `relationship` | `id` | `relationship_id` |
| `person` | `entity_id` | unchanged |
| `tree_path` | `(ancestor_id, descendant_id)` | unchanged |

Representative FK effects:

- `entity.diagram_id` should reference `diagram.diagram_id`
- `person.entity_id` should reference `entity.entity_id`
- `relationship.diagram_id` should reference `diagram.diagram_id`
- `relationship.source_entity_id` and `relationship.target_entity_id` should reference `entity.entity_id`
- `tree_path.ancestor_id` and `tree_path.descendant_id` should reference `entity.entity_id`

## SQL And Code Impact

This is a schema-level naming change, but it affects multiple layers.

### Database

Migration work will need to update:

- primary key column names
- foreign key references
- index names when they embed old column names
- sequence names and defaults where naming policy requires them

### Adapter

SQL queries and SQLx macros will need to move from generic `id` selection to explicit column names.

Examples:

- `SELECT diagram_id, name, kind FROM diagram`
- `SELECT entity_id, diagram_id, kind FROM entity`

This should reduce alias noise in joins where `id` is currently overloaded.

### Service And Usecase Layers

This RFC does not by itself require public domain structs to stop exposing `id` immediately.

Two implementation choices are possible:

- minimal migration: keep service/API fields as `id` and alias SQL columns at repository boundaries
- full internal rename: use explicit names in adapter and internal domain structs, then update the public API contract under a separate RFC

Recommended first step:

- rename DB columns and adapter-facing row structs first
- keep public API contracts stable unless a separate RFC chooses external field renames

That keeps the schema consistent without forcing an HTTP breaking change into the same rollout.

Public API field renames are covered by `docs/rfc/0010-explicit-id-fields-in-public-api.md`.

## API Compatibility

No public API change is required by this RFC.

For example:

- `GET /api/diagrams` may continue returning `{"id": 1, ...}`
- `GET /api/users/me` may continue returning `{"user": {"id": "...", ...}}`

The storage column name and the external JSON field name do not have to be identical.

If we later want public JSON to use resource-scoped names such as `diagram_id`, that should be reviewed separately because it is an external contract change.

## Migration Strategy

Recommended rollout:

1. add one migration that renames root-table primary key columns
2. update dependent foreign keys and schema object names
3. update adapter SQL and SQLx metadata
4. update tests and fixtures
5. update `docs/data-model.md`
6. verify all DB-backed tests against the renamed schema

Prefer one focused migration rather than gradually mixing old and new naming in the same runtime code.

## Benefits

- more readable SQL joins
- less ambiguity in query review and debugging
- clearer schema documentation
- a better base for upcoming multi-diagram and genealogy tables

## Drawbacks

- broad mechanical churn across queries, fixtures, and tests
- temporary migration risk if any raw SQL path is missed
- `user_id` does not mirror the plural table name `users` exactly

## Open Questions

- Should index and constraint names be fully renamed to match the new PK column names, or only when required for clarity?
- Should adapter/domain structs keep generic `id` fields for now, or should internal Rust names also become explicit in the same change?
- Should the `users` table remain the only plural table name long-term, or should table naming itself be reconsidered in a separate RFC?
