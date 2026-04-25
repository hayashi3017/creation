# RFC 0013: Canonical Relationship Kinds And Tree Path Policy

- Status: `Accepted`
- Last updated: `2026-04-26`

## Background

The current `relationship_kind` enum mixes several different concepts:

- directed canonical facts: `parent`, `adopted_parent`, `step_parent`
- inverse facts that can be derived from directed facts: `child`, `adopted_child`, `step_child`
- symmetric facts: `spouse`, `cohabitant`
- derived kinship labels: `sibling`
- relationship lifecycle state encoded as kind: `divorced_spouse`

This creates a structural problem around `tree_path`.

`tree_path` is a closure table for ancestor / descendant traversal. It only has clear semantics when it is built from directed lineage edges. If storage allows both canonical directed facts and inverse or derived facts, every write path must repeatedly decide which rows participate in closure maintenance and how to normalize them.

The current implementation mitigates this by allowing only `parent` and `child` through public relationship writes and by normalizing both into a parent-to-child edge before rebuilding `tree_path`. That works as a temporary compatibility layer, but it keeps the storage enum broader than the domain facts that should actually be persisted.

This RFC narrows the persistence model so `tree_path` can be maintained from canonical stored facts instead of from a mixed relationship enum.

## Goals

- make `relationship_kind` represent only persisted canonical relationship facts
- remove inverse and graph-derived kinship from the storage enum
- define which canonical kinds participate in `tree_path`
- make `source_entity_id` / `target_entity_id` semantics unambiguous
- provide a migration path from existing `child` and other non-canonical enum values
- keep richer kinship labels as read-side derived output, aligned with RFC 0012

## Non-Goals

- implementing the migration in this RFC
- finalizing localized display labels such as Japanese kinship strings
- changing the public family-tree response contract from RFC 0007 in the same step
- persisting sibling, cousin, grandparent, or other derived kinship rows
- deciding the final API shape for richer kinship output

## Proposal

Treat `relationship` rows as explicit canonical facts only.

Recommended stored enum:

```sql
CREATE TYPE relationship_kind AS ENUM (
  'parent',
  'adoptive_parent',
  'step_parent',
  'spouse',
  'partner',
  'cohabitant'
);
```

Recommended conceptual grouping:

```text
Directed canonical relationships:
- parent
- adoptive_parent
- step_parent

Symmetric canonical relationships:
- spouse
- partner
- cohabitant

Derived kinship, not persisted:
- child
- adoptive_child
- step_child
- sibling
- ancestor
- descendant
- grandparent
- grandchild
- uncle_aunt
- nephew_niece
- cousin
```

The key rule is that storage records facts, while read models derive kinship views.

For example:

- `parent(A, B)` is stored as a fact: A is a parent of B.
- `child(B, A)` is not stored; it is the inverse view of `parent(A, B)`.
- `sibling(A, B)` is not stored; it is derived from shared parent sets.
- `grandparent(A, C)` is not stored; it is derived from `tree_path.depth = 2`.
- `divorced_spouse` is not stored as a kind; it is represented as an ended `spouse` relationship with an end reason.

## Source And Target Semantics

### Directed Kinds

For directed kinds, `source_entity_id -> target_entity_id` is meaningful:

- `parent(source, target)`: source is parent, target is child
- `adoptive_parent(source, target)`: source is adoptive parent, target is adoptive child
- `step_parent(source, target)`: source is step parent, target is step child

Directed relationships should not be auto-sorted by id because the direction is part of the fact.

### Symmetric Kinds

For symmetric kinds, the relationship has no semantic direction.

Recommended storage rule:

- normalize endpoints before persistence
- store `source_entity_id < target_entity_id`
- reject or merge duplicate active rows for the same diagram, endpoint pair, and kind

This prevents both `spouse(1, 2)` and `spouse(2, 1)` from being stored as separate active facts.

## Tree Path Policy

`tree_path` should be built only from canonical directed lineage facts.

Initial tree-edge policy:

- `parent`: participates in `tree_path`
- `adoptive_parent`: participates in `tree_path`
- `step_parent`: does not participate in `tree_path` initially
- `spouse`, `partner`, `cohabitant`: never participate in `tree_path`

Rationale:

- `parent` and `adoptive_parent` both express an ancestry-like directed lineage edge.
- `step_parent` is family-relevant but whether it should contribute to ancestor / descendant traversal is product- and culture-dependent.
- symmetric relationships do not define ancestor / descendant direction.
- `cohabitant` is a living arrangement or social relationship and must not affect kinship traversal.

This means `tree_path.depth` can retain a narrow meaning:

- `depth = 1`: direct parent / child, including adoptive lineage if accepted
- `depth = 2`: grandparent / grandchild-style lineage
- `depth >= 1`: ancestor / descendant reachability

`tree_path` should not store rows for spouse, partner, cohabitant, sibling, cousin, or step-parent traversal unless a later RFC explicitly changes the policy.

## Lifecycle State

Do not encode lifecycle state in `relationship_kind`.

Replace `divorced_spouse` with a canonical relationship plus lifecycle fields.

Recommended table additions:

```sql
end_reason VARCHAR(32)
```

Recommended values, if an enum is introduced later:

```text
divorce
death
separation
unknown
```

Examples:

- current spouse: `kind = spouse`, `end_date = NULL`, `end_reason = NULL`
- divorced spouse: `kind = spouse`, `end_date IS NOT NULL`, `end_reason = divorce`
- widowed spouse: `kind = spouse`, `end_date IS NOT NULL`, `end_reason = death`

This keeps relationship kind and relationship history separate.

## Constraints

Add a self-relation check:

```sql
CHECK (source_entity_id <> target_entity_id)
```

Add an active-row unique index for directed canonical facts:

```sql
CREATE UNIQUE INDEX uq_relationship_directed_active
ON relationship (
  diagram_id,
  source_entity_id,
  target_entity_id,
  kind
)
WHERE deleted_at IS NULL
  AND kind IN ('parent', 'adoptive_parent', 'step_parent');
```

Add an active-row unique index for symmetric canonical facts:

```sql
CREATE UNIQUE INDEX uq_relationship_symmetric_active
ON relationship (
  diagram_id,
  LEAST(source_entity_id, target_entity_id),
  GREATEST(source_entity_id, target_entity_id),
  kind
)
WHERE deleted_at IS NULL
  AND kind IN ('spouse', 'partner', 'cohabitant');
```

The application service should still normalize and validate before writing. Database constraints are the final guardrail, not the only validation layer.

## Rust Model Direction

The storage-facing enum should only include stored canonical facts.

Recommended shape:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredRelationshipKind {
    Parent,
    AdoptiveParent,
    StepParent,
    Spouse,
    Partner,
    Cohabitant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipTopology {
    Directed,
    Symmetric,
}

impl StoredRelationshipKind {
    pub fn topology(self) -> RelationshipTopology {
        match self {
            Self::Parent | Self::AdoptiveParent | Self::StepParent => {
                RelationshipTopology::Directed
            }
            Self::Spouse | Self::Partner | Self::Cohabitant => {
                RelationshipTopology::Symmetric
            }
        }
    }

    pub fn is_tree_edge(self) -> bool {
        matches!(self, Self::Parent | Self::AdoptiveParent)
    }
}
```

Derived read-side kinship should use a different enum and should not be reused as the database enum:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerivedKinship {
    Child,
    AdoptiveChild,
    StepChild,
    Sibling,
    Ancestor { depth: u32 },
    Descendant { depth: u32 },
    Grandparent,
    Grandchild,
    UncleAunt,
    NephewNiece,
    Cousin,
}
```

This split keeps `RelationshipService` responsible for persisted facts and keeps derived kinship in the read-side service boundary from RFC 0012.

## Migration Strategy

Use a staged migration because PostgreSQL enum value removal is not a simple `ALTER TYPE` operation.

### Phase 1: Add Compatibility Support

- add `partner` and `adoptive_parent` if choosing the new spelling
- add `end_reason`
- add the self-relation check after cleaning invalid rows
- add service-level topology helpers
- continue accepting existing public payloads only where backward compatibility is required

### Phase 2: Normalize Existing Rows

Normalize inverse rows into canonical direction:

- `child(source, target)` -> `parent(target, source)`
- `adopted_child(source, target)` -> `adoptive_parent(target, source)`
- `step_child(source, target)` -> `step_parent(target, source)`

Normalize lifecycle rows:

- `divorced_spouse(source, target)` -> `spouse(source, target)` with `end_reason = divorce`

Normalize symmetric rows:

- if `kind IN ('spouse', 'partner', 'cohabitant')` and `source_entity_id > target_entity_id`, swap endpoints

Resolve duplicate active rows before adding unique indexes.

### Phase 3: Rebuild Tree Path

After relationship rows are canonicalized:

1. delete affected `tree_path` rows for the migrated diagrams or rebuild all closure rows
2. rebuild from active `parent` and `adoptive_parent` rows only
3. verify cycle detection still uses only tree-edge kinds

The repository methods that load relationship edges for closure maintenance should filter by `StoredRelationshipKind::is_tree_edge()` semantics instead of special-casing `parent` and `child`.

### Phase 4: Replace The Database Enum

Create a new enum type and cast through text, or use a similar safe migration pattern:

```sql
CREATE TYPE relationship_kind_v2 AS ENUM (
  'parent',
  'adoptive_parent',
  'step_parent',
  'spouse',
  'partner',
  'cohabitant'
);
```

Then migrate `relationship.kind` to the new type after all rows use valid canonical values.

The exact SQL should be implemented in a dedicated migration with tests, because failed enum casts can leave the schema in a partially migrated state.

## API Compatibility

The write API should move toward accepting only canonical stored kinds.

Compatibility options:

- strict option: reject `child`, `adopted_child`, `step_child`, `sibling`, and `divorced_spouse` with `400 BAD_REQUEST`
- compatibility option: accept inverse kinds temporarily, normalize them before persistence, and return canonical stored kinds on read

The strict option is cleaner once clients are updated.

The compatibility option is safer during rollout if existing clients already send `child`.

Regardless of rollout option, persistence should converge on canonical stored rows only.

## Interaction With Existing RFCs And ADRs

This RFC extends ADR 0004.

ADR 0004 intentionally limited the current implementation to `parent` and `child` while deferring broader relationship semantics. This RFC answers that deferred question by removing `child` from long-term storage and making `tree_path` depend on canonical tree-edge kinds.

This RFC complements RFC 0012.

RFC 0012 defines the read-side `KinshipDerivationService`. This RFC defines the storage-side facts that service should consume.

## Benefits

- `tree_path` has one stable meaning: canonical ancestor / descendant closure
- inverse relationships no longer create duplicate storage facts
- derived kinship such as sibling and cousin cannot drift from stored parent facts
- symmetric relationship duplication is prevented consistently
- divorce and other relationship endings can be represented without multiplying kind values
- future read models can derive richer kinship without expanding the storage enum

## Drawbacks

- requires a non-trivial enum migration
- may require API compatibility handling for clients that send `child`
- `adopted_parent` vs `adoptive_parent` spelling must be decided and migrated consistently
- excluding `step_parent` from `tree_path` may need revisiting if the product wants step-family ancestry traversal

## Test Plan

Minimum coverage for implementation:

- migration converts `child` rows into swapped `parent` rows
- migration converts `adopted_child` rows into swapped adoptive-parent rows
- migration converts `divorced_spouse` into `spouse` plus `end_reason = divorce`
- symmetric relationship writes normalize endpoint order
- duplicate active symmetric relationships are rejected
- duplicate active directed relationships are rejected
- self-relationships are rejected
- `tree_path` rebuild uses `parent` and `adoptive_parent`
- `tree_path` rebuild ignores `step_parent`, `spouse`, `partner`, and `cohabitant`
- public relationship writes reject or normalize non-canonical legacy kinds according to the rollout option

## Resolved Decisions

- stored enum uses `adoptive_parent`
- public API rejects legacy inverse/derived kinds by removing them from the storage-facing enum
- migration normalizes existing inverse rows before replacing the PostgreSQL enum
- `end_reason` is stored as `VARCHAR(32)` for the initial implementation

## Open Questions

- Should `step_parent` become a tree-edge kind in a later opt-in projection, or should it stay outside ancestor / descendant closure permanently?
- Should `end_reason` eventually become a PostgreSQL enum or a separate event/history table?
