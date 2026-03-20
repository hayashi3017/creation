# RFC 0006: Relationship Soft-Delete And Tree Path Consistency

- Status: `Draft`
- Last updated: `2026-03-20`

## Background

`relationship` writes currently rebuild `tree_path` inside the same transaction, so create / update / delete of lineage edges stays consistent.

That still leaves one unresolved path:

- an `entity` or `person` can be soft-deleted without touching `relationship`
- `tree_path` is not rebuilt on that path today

Because `relationship` rows reference entities directly, soft-deleting an entity can leave active relationship rows and previously computed closure rows that no longer represent visible data.

## Proposal

Decide a single policy for soft-delete propagation across:

- `entity`
- `person`
- `relationship`
- `tree_path`

## Options

### Option A: Cascade soft-delete to relationships and rebuild tree_path immediately

Behavior:

- soft-delete the target `entity`
- soft-delete active `relationship` rows that reference that entity
- rebuild `tree_path` for the affected diagram in the same transaction

Pros:

- read models stay physically consistent
- tree traversal never depends on filtering out stale rows later

Cons:

- delete flow becomes heavier
- restoring data later is more complex

### Option B: Keep relationships as-is and filter deleted entities at read/rebuild time

Behavior:

- soft-delete only the target `entity` / `person`
- leave `relationship` rows active
- ensure all reads and `tree_path` rebuilds ignore deleted entities

Pros:

- lighter write path
- preserves relationship history more directly

Cons:

- stale active rows remain in storage
- every consumer must remember to filter deleted entities

### Option C: Add a dedicated archival state for relationship visibility

Pros:

- separates “entity deleted” from “relationship historically invalid”
- allows future restore/archive behavior

Cons:

- more schema and application complexity

## Review Points

- Should entity/person delete trigger relationship soft-delete in the same transaction?
- If not, should `tree_path` be rebuilt proactively anyway?
- Do we want historical relationship rows to remain queryable after entity soft-delete, and if so through which endpoint?
