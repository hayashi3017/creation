# ADR 0004: Relationship Lineage Tree Maintenance

- Status: `Superseded`
- Last updated: `2026-04-26`
- Superseded by: `docs/rfc/0013-canonical-relationship-kinds-and-tree-path.md`

## Context

The database schema already has two separate concepts:

- `relationship`: general links between entities
- `tree_path`: closure table for ancestor / descendant traversal

However, the `relationship_kind` enum contains both lineage and non-lineage kinds:

- lineage-like: `parent`, `child`
- non-lineage: `sibling`, `spouse`, `adopted_*`, `step_*`, `cohabitant`, `divorced_spouse`

`tree_path` only makes clear sense for directed ancestry. If every relationship kind were allowed to participate in closure maintenance, the meaning of ancestor / descendant would become ambiguous.

## Decision

For the implementation at the time this ADR was accepted:

- public `relationship` write APIs only accept `parent` and `child`
- `tree_path` is rebuilt only from active lineage edges
- `parent` is interpreted as `source_entity_id -> target_entity_id`
- `child` is interpreted as `target_entity_id -> source_entity_id`
- non-lineage relationship kinds are not yet exposed through the public API

## Consequences

- ancestor traversal semantics stay narrow and predictable
- cycle detection can be implemented on a directed lineage graph
- future support for `sibling`, `spouse`, `adopted_*`, or `step_*` needs a separate review of:
  - whether those kinds should be accepted by the public API
  - whether they affect `tree_path`
  - whether a different traversal structure is needed
