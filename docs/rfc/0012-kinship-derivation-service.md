# RFC 0012: Kinship Derivation Service

- Status: `Draft`
- Last updated: `2026-04-25`

## Background

The current `GET /api/family-trees/{diagram_id}` path is intentionally lineage-only, but the family-tree domain requirements point toward richer read-side kinship behavior:

- inverse views such as `parent -> child`
- symmetric views such as `spouse`
- ancestor / descendant derivation from `tree_path`
- sibling classification
- uncle / aunt, nephew / niece, cousin, and in-law derivation
- source tracking such as `explicit`, `derived`, and `suggested`
- gender- and age-aware presentation metadata

At the same time, the workspace already has an existing `RelationshipService` with a clear shape:

- validate explicit relationship write payloads
- normalize write-side fields such as notes and dates
- persist, update, delete, and fetch stored `relationship` rows

If richer kinship logic is added without clarifying boundaries, two bad outcomes are likely:

1. `FamilyTreeUsecase` becomes a large graph-derivation module instead of an orchestration layer.
2. `RelationshipService` starts to mix CRUD for stored facts with read-only derivation for inferred facts.

This RFC defines a new service boundary that is intentionally loose-coupled and MECE with `RelationshipService`.

## Goals

- keep persisted relationship management separate from read-only kinship derivation
- define a service name based on role, not on one endpoint or one aggregate label
- draw an explicit MECE boundary between `RelationshipService`, the new derivation service, and `FamilyTreeUsecase`
- keep the new service loosely coupled to existing services
- support incremental rollout from the current lineage-only projection to richer kinship output

## Non-Goals

- changing the relationship CRUD API in the same change
- persisting derived relationships into the `relationship` table
- deciding final localized labels such as Japanese kinship strings
- replacing `tree_path` write-side maintenance
- solving cross-diagram merge or identity-linking concerns from RFC 0008

## Proposal

Introduce a dedicated service in `creation-service` for read-side kinship derivation.

Recommended name:

- `KinshipDerivationService`

Why this name:

- `FamilyTreeRelationshipService` sounds like CRUD ownership for family-tree relationships
- the real role is not "all family-tree relationships"
- the role is "derive kinship semantics from explicit facts"
- the same logic should be reusable beyond one endpoint, for example ancestors, descendants, relatives-of-person, or future merged genealogy reads

Recommended dependency direction:

- `FamilyTreeUsecase` depends on `ProvidesKinshipDerivationService`
- `RelationshipService` and `KinshipDerivationService` do not depend on each other
- `FamilyTreeUsecase` orchestrates both services and assembles the final response

This keeps the usecase thin while making the two services orthogonal instead of overlapping.

## MECE Service Boundary

### `RelationshipService` Owns

`RelationshipService` is the service for persisted explicit relationship facts.

It owns:

- validation of create / update / delete payload shape for stored rows
- write-side normalization such as blank-note cleanup and date-range checks
- enforcement of explicit relationship invariants needed for persistence
- repository-backed CRUD for `relationship`
- returning stored `Relationship` rows to callers as explicit facts

It does not own:

- inverse expansion such as showing `child` from `parent`
- symmetric expansion such as mirrored spouse views
- canonical family-tree orientation for read-side graph use
- sibling / cousin / in-law / ancestor derivation
- family-tree node adjacency or root detection
- read-side source tagging such as `Derived` or `Suggested`

### `KinshipDerivationService` Owns

`KinshipDerivationService` is the service for read-only transformation from explicit facts to kinship semantics.

It owns:

- canonicalization of explicit relationships into read-side graph form
- inverse and symmetric read expansion
- use of lineage closure data to derive ancestor / descendant semantics
- sibling classification and higher-order kinship derivation
- tagging outputs as `Explicit`, `Derived`, or `Suggested`
- filtering derived relations to the active visible scope supplied by the caller

It does not own:

- persistence of `relationship` rows
- create / update / delete validation for stored rows
- mutation of `tree_path`
- loading diagrams, entities, or persons for a request
- mapping domain outputs directly into HTTP response DTOs

### `FamilyTreeUsecase` Owns

`FamilyTreeUsecase` remains the application orchestration layer.

It owns:

1. validating `diagram_id`
2. loading the target diagram
3. rejecting non-`family_tree` diagrams
4. loading active persons in scope
5. loading explicit relationships through `RelationshipService`
6. loading any additional lineage closure input if the derivation service needs it
7. calling `KinshipDerivationService`
8. assembling `FamilyTree { nodes, edges, root_entity_ids, stats }`

It does not own:

- rule-by-rule kinship derivation
- stored relationship CRUD semantics

### Why This Split Is MECE

- persisted explicit fact lifecycle belongs only to `RelationshipService`
- read-only kinship inference belongs only to `KinshipDerivationService`
- request orchestration and response assembly belong only to `FamilyTreeUsecase`

No responsibility needs to be duplicated across the three.

## Stored vs Derived Relationships

This RFC recommends a policy rather than an immediate schema rewrite:

- explicit relationships are the rows stored in `relationship`
- derived relationships are computed at read time and are never persisted
- suggested relationships are heuristics that may be returned to clients later, but are not treated as canonical facts

Current read/write baseline:

- explicit lineage: `parent`, `child`
- future explicit kinds may include `spouse`, `cohabitant`, `adopted_parent`, `adopted_child`, `step_parent`, `step_child`
- derived-only kinds should include `sibling`, `ancestor`, `descendant`, `uncle_aunt`, `nephew_niece`, `cousin`, and `in_law`

Do not store `sibling`, `ancestor`, `cousin`, or similar graph-expanded kinship as independent rows.

The current Rust enum already contains some kinds such as `Sibling`, but this RFC treats that as an implementation detail of the current model, not as approval to persist sibling rows going forward.

## Relationship Source

The distinction between explicit, derived, and suggested kinship is useful and should be preserved as a read-model concept.

Recommended enum:

```rust
enum FamilyTreeRelationshipSource {
    Explicit,
    Derived,
    Suggested,
}
```

Meaning:

- `Explicit`: directly backed by a stored `relationship` row
- `Derived`: deterministically implied by explicit rows plus lineage closure
- `Suggested`: heuristic output that may be useful in UI or review flows, but should not be treated as already confirmed domain truth

## Read Model Shape

Do not reuse the storage-oriented `Relationship` struct as the main output shape for richer kinship output.

The derivation service should own its own domain output type, for example:

```rust
struct KinshipRelation {
    from_entity_id: usize,
    to_entity_id: usize,
    kind: KinshipRelationKind,
    source: FamilyTreeRelationshipSource,
    explicit_relationship_id: Option<usize>,
    sibling_kind: Option<FamilyTreeSiblingKind>,
    generation_distance: Option<usize>,
}
```

Recommended supporting enum:

```rust
enum FamilyTreeSiblingKind {
    Full,
    Half,
    Adoptive,
    Step,
}
```

Important design points:

- localized strings such as `兄`, `弟`, `父`, or `母` should not be the primary domain output
- the derivation service may return semantic relations that are richer than the current `FamilyTreeEdge`
- `FamilyTreeUsecase` decides how much of that output is exposed in a given endpoint contract

Instead, the service should return semantic data, and any final localized label should be rendered later from:

- relationship kind
- gender
- age ordering when needed
- locale / presentation rules

This avoids locking the core domain contract to one language or one UI wording policy.

## Derivation Rules

### Canonical Normalization

`KinshipDerivationService` should normalize stored rows into a canonical internal graph before deriving higher-level kinship.

Examples:

- `parent(A, B)` implies canonical lineage edge `A -> B`
- `child(A, B)` implies canonical lineage edge `B -> A`
- `spouse(A, B)` is symmetric
- `cohabitant(A, B)` is symmetric
- `divorced_spouse(A, B)` is symmetric as a historical relationship, not a current spouse guarantee

### Tree Path

`tree_path` remains the authoritative lineage closure table for ancestor / descendant reachability.

`RelationshipService` does not interpret `tree_path`.

`KinshipDerivationService` may consume lineage closure input for derivation such as:

- `depth = 1`: direct parent / child
- `depth = 2`: grandparent / grandchild metadata
- `depth >= 1`: ancestor / descendant relationships with optional distance metadata

Do not expose raw `tree_path` rows as the main public contract just to support these derivations.

### Siblings

Sibling derivation should be read-only.

Baseline rule:

- two people are siblings when they share at least one parent in the canonical lineage graph

Recommended classification:

- `full`: same two explicit parents
- `half`: one shared explicit parent
- `adoptive`: derived through adoptive lineage once those explicit kinds are supported
- `step`: derived through step-parent semantics, not through blood/adoptive lineage

### Uncle / Aunt, Nephew / Niece, Cousin

These are second-order derived relationships and should not be materialized in storage.

Examples:

- parent of A is sibling of B -> B is uncle/aunt of A
- sibling of A has child B -> B is nephew/niece of A
- parent of A is sibling of parent of B -> A and B are cousins

The service should compute them from previously normalized lineage and sibling data rather than from ad hoc client-side logic.

### In-Law

In-law relationships should remain derived-only.

Example:

- spouse(A, B) + parent(B, C) -> A is parent-in-law of C

This is precisely the kind of graph expansion that should live in the derivation service rather than in `FamilyTreeUsecase`.

### Suggested Step Relationships

Suggested step relationships are useful, but they are also the easiest place to over-infer.

Recommended policy:

- do not infer spouse from shared children
- do not infer spouse from co-residence alone
- do not treat `end_date` or divorce history as enough to conclude current family membership
- only emit step-parent / step-child as `Suggested` unless there is an explicit canonical relationship kind or a stronger product rule

If we later infer step relationships from `parent + spouse/cohabitant + date overlap`, that output should stay opt-in until product semantics are reviewed.

## Loose Coupling Rules

To keep the new service loosely coupled:

- `KinshipDerivationService` should not call `RelationshipService`
- `RelationshipService` should not call `KinshipDerivationService`
- `FamilyTreeUsecase` should pass explicit relationships in as domain input
- if lineage closure is needed, the usecase should pass it in or depend on a dedicated read port for that data
- do not route closure-table concerns through `RelationshipService`, because that would blur persistence concerns with derivation concerns

Recommended interface direction:

- use a domain input/output struct, not HTTP-specific schema structs
- let the caller pass already loaded scope data for deterministic unit tests
- if later needed, add a dedicated tree-path read port instead of expanding `RelationshipService`

Example shape:

```rust
struct DeriveKinshipInput {
    active_person_ids: Vec<usize>,
    explicit_relationships: Vec<Relationship>,
    lineage_paths: Vec<KinshipLineagePath>,
}

struct KinshipDerivation {
    lineage_edges: Vec<CanonicalLineageEdge>,
    relations: Vec<KinshipRelation>,
}
```

## Why A Service Is Better Than Keeping This In The Usecase

This RFC explicitly recommends the service boundary suggested in the user note, but with a narrower and clearer role than `FamilyTreeRelationshipService`.

Reasons:

- kinship derivation is domain logic, not application orchestration
- the algorithms will likely grow faster than the endpoint count
- the same logic will be needed by future reads such as ancestors, descendants, relatives-of-person, or merged genealogy projections
- service-level unit tests can focus on graph inputs and outputs without bootstrapping the full usecase stack
- `FamilyTreeUsecase` stays readable and aligned with the rest of the workspace layering
- `RelationshipService` keeps one job: manage persisted explicit relationship facts

In short:

- `FamilyTreeUsecase` should ask for a kinship derivation result
- `RelationshipService` should keep owning stored relationship rows
- neither should absorb the other's job

## Incremental Rollout

To keep implementation tractable, use staged rollout.

### Phase 1

Keep the public `/api/family-trees/{diagram_id}` response unchanged.

Introduce `KinshipDerivationService` with only the read-side canonicalization needed by the current endpoint:

- `parent` / `child` normalization into canonical lineage edges
- filtering to active visible scope

Keep in `FamilyTreeUsecase` for now:

- node assembly
- adjacency list materialization
- root detection
- stats assembly

This keeps the boundary MECE:

- derivation service derives kinship facts
- usecase assembles endpoint-specific response shape

### Phase 2

Extend `KinshipDerivationService` to carry:

- `Explicit` vs `Derived` source
- optional sibling classification
- optional generation distance metadata from `tree_path`

The existing public response may still stay lineage-only at this phase.

### Phase 3

Add richer derived kinship output only after deciding the API contract:

- extend `/api/family-trees/{diagram_id}`
- or add a dedicated kinship-focused read under `/api/family-trees/{diagram_id}/relationships`

Do not force the richer contract into the current read API until the public shape is reviewed.

## Benefits

- keeps complex graph semantics out of the usecase layer
- makes kinship rules reusable across multiple read models
- preserves a clean separation between stored facts and read-only derivation
- reduces pressure to persist mechanically derivable rows such as siblings or cousins
- gives a safe place to add future heuristics without polluting CRUD services

## Drawbacks

- introduces more domain types and one more service trait
- may require a dedicated tree-path read input later
- some kinship terminology does not exactly match the current `RelationshipKind` enum and must be normalized carefully

## Open Questions

- Should richer derived kinship use a new read-only enum instead of extending the storage-facing `RelationshipKind`?
- Should `Suggested` relationships be excluded by default unless an explicit include flag is added later?
- Should localized kinship labels be rendered in the driver or left entirely to clients?
- When the public write API starts accepting non-lineage kinds, which of them are canonical stored facts and which should stay derived-only?
- If lineage closure input is needed, should the usecase load it directly from a dedicated read repository, or should `KinshipDerivationService` gain its own read-port dependency?
