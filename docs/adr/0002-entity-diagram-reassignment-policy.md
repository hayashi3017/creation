# ADR 0002: Entity Diagram Reassignment Policy

- Status: `Draft`
- Last updated: `2026-03-14`

## Context

`update_entity` currently accepts `diagram_id`, which means a generic update can move an entity across diagrams.

That choice is not neutral because other tables are diagram-scoped:

- `relationship.diagram_id`
- `tree_path`
- any future per-diagram invariants

Allowing reassignment without an explicit policy risks silent data inconsistency.

## Options

### Option A: Allow generic update to move entities across diagrams

Pros:

- fewer endpoints
- simple client behavior

Cons:

- unclear cascade policy for relationships and tree paths
- easy to move data accidentally

### Option B: Disallow reassignment in generic update

- `diagram_id` is fixed after create
- moving an entity requires a dedicated future workflow

Pros:

- safer default
- keeps generic update semantics simple
- avoids hidden cross-diagram side effects

Cons:

- dedicated move operation is needed if the product later requires it

### Option C: Allow reassignment only through a dedicated move endpoint

Pros:

- explicit intent
- room for transactional revalidation and cascade handling

Cons:

- still requires deciding the move semantics up front

## Proposed Decision

Choose **Option B** for the current API.

`update_entity` should not change `diagram_id`. If cross-diagram moves become necessary, add a dedicated endpoint after relationship and tree maintenance rules are defined.

## Consequences

- future resource-oriented API can take `diagram_id` from the create path only
- generic entity update should reject reassignment attempts
- validation RFCs should assume diagram existence checks without permitting moves by default
