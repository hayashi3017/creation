# RFC 0005: Entity-Person Soft Delete Consistency

- Status: `Draft`
- Last updated: `2026-03-15`

## Background

`person` CRUD is now implemented as a separate API on top of the `person` specialization table.

Current behavior:

- public `DELETE /api/persons/delete/{entity_id}` sets both `entity.deleted_at` and `person.deleted_at`
- internal `entity_repository.delete_entity(...)` still sets only `entity.deleted_at`
- `person` reads and writes require both `person.deleted_at IS NULL` and `entity.deleted_at IS NULL`

This means deleting an `entity` hides the related `person` row from the API, but does not mark the `person` row as logically deleted in the database.

## Problem

The current behavior leaves lifecycle semantics implicit:

- DB state can contain `person.deleted_at IS NULL` rows whose parent `entity` is already soft-deleted
- future restore / recreate semantics for the same `entity_id` become unclear
- future specialization tables would likely repeat the same ambiguity

## Options

### Option A: Parent soft delete does not propagate

- keep current behavior
- child specialization rows remain physically present and logically active
- API hides them by filtering on parent activity

Pros:

- smallest implementation
- no cross-table update on parent delete

Cons:

- DB state is harder to reason about
- restore / recreate semantics stay ambiguous

### Option B: Parent soft delete propagates to specialization rows

- deleting `entity` also marks `person.deleted_at`
- optionally, deleting `diagram` would later cascade logical delete into `entity` and specialization rows by the same policy

Pros:

- lifecycle state is explicit in every table
- easier to reason about restore / audit semantics

Cons:

- more cross-table mutation logic
- broader policy decision if generalized beyond `person`

## Open Questions

- should `diagram` soft delete also logically delete its `entity` and `person` descendants in the same pass?
- if restore APIs are added later, should restore also propagate from parent to child?
- should child-specific delete remain allowed after parent delete, or become a no-op / `404` only?
