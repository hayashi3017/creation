# RFC 0014: As-Of Family Tree Projection

- Status: `Draft`
- Last updated: `2026-04-25`

## Background

Family-tree relationships are time-dependent.

Examples:

- a spouse relationship may be active in 1990 but ended by 2005
- a cohabitant or partner relationship may only apply for a limited period
- an adoptive or step-parent relationship may start after birth
- a person may be born after the requested point in time
- a person may have died before the requested point in time, while still remaining visible in a historical family tree

The current schema already has `relationship.start_date` and `relationship.end_date`, but current read paths treat relationships as active rows without a requested historical date.

This means `GET /api/family-trees/{diagram_id}` can show the current stored graph, but cannot answer questions such as:

- "What did this family tree look like on 1995-01-01?"
- "Who was considered a spouse at this time?"
- "Which parent / child / sibling / ancestor relationships were valid then?"
- "What was the kinship from a center person at that point in time?"

This RFC proposes an as-of read projection that evaluates explicit relationships and derived kinship at a requested point in time.

## Goals

- support family-tree reads for a specific historical date
- keep temporal behavior read-side first, without rewriting all storage to event sourcing
- define how relationship `start_date` and `end_date` affect explicit and derived kinship
- clarify how `tree_path` should be used for as-of reads
- keep current read behavior compatible when no date is requested
- align with RFC 0012 and RFC 0013

## Non-Goals

- implementing full event sourcing
- storing every past version of every relationship row
- deciding final UI timeline controls
- changing write APIs in the same step
- making `tree_path` a complete temporal history table in the initial rollout
- solving ambiguous or approximate historical dates beyond simple date ranges

## Feasibility

This is feasible with the current direction, but the implementation should distinguish two concepts:

- current-state closure: maintained `tree_path` for active canonical lineage rows
- as-of projection: a read model built from rows whose validity range includes the requested date

The current `tree_path` table does not store validity ranges, so it cannot answer historical closure queries by itself.

For initial implementation, as-of lineage closure should be derived at read time from filtered canonical relationship rows. This avoids corrupting the meaning of the existing `tree_path` and avoids introducing a temporal closure table before the access pattern is proven.

## Proposed API

Extend the family-tree read endpoint with an optional query parameter:

```http
GET /api/family-trees/{diagram_id}?as_of=1995-01-01
```

Semantics:

- if `as_of` is omitted, preserve the current behavior
- if `as_of` is present, return the projection valid on that date
- `as_of` should be an ISO `YYYY-MM-DD` date
- invalid dates return `400 BAD_REQUEST`

Recommended response metadata:

```json
{
  "status": "success",
  "data": {
    "as_of": "1995-01-01",
    "temporal_mode": "as_of",
    "diagram": {},
    "root_entity_ids": [],
    "nodes": [],
    "edges": [],
    "stats": {}
  }
}
```

For compatibility, the initial response may omit `temporal_mode` if API churn should be minimized. However, returning `as_of` is useful because it makes cache keys and client state explicit.

## Temporal Validity Rules

Use inclusive date ranges.

A relationship is valid at `as_of` when:

```text
(start_date IS NULL OR start_date <= as_of)
AND
(end_date IS NULL OR as_of <= end_date)
AND
deleted_at IS NULL
```

Interpretation:

- `start_date = NULL`: relationship is valid from an unknown beginning
- `end_date = NULL`: relationship remains valid after its start
- `start_date = end_date`: relationship is valid for that one date
- `deleted_at`: administrative deletion; not part of historical validity

`deleted_at` should continue to mean "this row should not participate in normal reads." If historical audit of deleted facts is needed later, that should be handled by a separate archive or event log, not by changing this projection.

## Person Visibility Rules

The initial implementation should not hide deceased persons from a historical family tree.

Recommended baseline:

- include active, non-deleted person/entity rows in the diagram
- if `person.birth_date` is known and `birth_date > as_of`, exclude that person from the as-of projection
- if `person.death_date` is known and `death_date < as_of`, keep the person visible but mark them as deceased through existing fields

Rationale:

- family trees usually include ancestors who are no longer alive
- showing deceased persons is necessary for historical ancestry
- excluding people not yet born prevents impossible edges from appearing before birth

Open policy:

- if the product later needs "living household at date" views, that should be a separate projection mode from a family-tree ancestry view

## Relationship Projection Rules

For as-of reads:

1. load active persons/entities in the diagram
2. filter persons by birth-date visibility
3. load canonical relationship rows valid at `as_of`
4. filter relationships whose endpoints are not visible in the as-of person set
5. normalize explicit rows into canonical graph input
6. derive lineage closure and kinship from the filtered graph
7. assemble the response

Examples:

- `spouse(A, B)` with `start_date = 1980-01-01`, `end_date = 2000-12-31` is present for `as_of=1995-01-01`
- the same spouse row is absent for `as_of=2005-01-01`
- `parent(A, B)` with no dates is considered valid unless an endpoint is not visible
- `adoptive_parent(A, B)` starts contributing to lineage only from its `start_date`

## Tree Path Strategy

Do not use the current `tree_path` table as the source of truth for as-of closure.

Reason:

- `tree_path` has only `ancestor_id`, `descendant_id`, and `depth`
- it has no `valid_from` or `valid_to`
- it is maintained from current active lineage rows
- a historical query may need closure for a relationship graph that differs from the current graph

Initial as-of strategy:

- load relationship rows valid at `as_of`
- keep only tree-edge kinds from RFC 0013, initially `parent` and `adoptive_parent`
- build ancestor / descendant closure in memory for that request
- detect cycles against the as-of graph
- use that request-local closure for derived kinship

This is acceptable for the first implementation because diagram-scoped family trees are expected to be small enough for request-local graph derivation.

If performance becomes a problem, add a dedicated temporal closure table later.

Possible future table:

```sql
CREATE TABLE temporal_tree_path (
  diagram_id BIGINT NOT NULL,
  ancestor_id BIGINT NOT NULL,
  descendant_id BIGINT NOT NULL,
  depth INT NOT NULL,
  valid_from DATE,
  valid_to DATE,
  PRIMARY KEY (diagram_id, ancestor_id, descendant_id, valid_from)
);
```

This future table should not be introduced until there is a clear need, because maintaining temporal closure correctly is significantly more complex than current-state closure.

## Kinship Derivation

`KinshipDerivationService` from RFC 0012 should accept temporal input without owning persistence.

Recommended input shape:

```rust
struct DeriveKinshipInput {
    active_person_ids: Vec<usize>,
    explicit_relationships: Vec<Relationship>,
    lineage_paths: Vec<KinshipLineagePath>,
    as_of: Option<NaiveDate>,
}
```

The usecase should own loading and temporal filtering.

The derivation service should own only the graph interpretation:

- inverse relationships such as `child`
- symmetric expansion such as spouse / partner
- sibling classification from shared as-of parents
- ancestor / descendant from as-of lineage closure
- higher-order kinship such as uncle/aunt and cousin

This keeps the responsibility split from RFC 0012 intact.

## Relationship Lifecycle And End Reasons

RFC 0013 recommends representing ended spouse relationships with `end_date` and `end_reason` rather than a `divorced_spouse` kind.

As-of projection should use `end_date` for validity, not `end_reason`.

Examples:

- `spouse`, `end_date = 2000-01-01`, `end_reason = divorce`
- valid as spouse on `1999-12-31`
- absent as spouse on `2001-01-01`

If the UI wants to display "former spouse" after the end date, that should be a separate historical relationship view, not the as-of active relation projection.

## Query And Repository Changes

Add read schemas that carry `as_of`.

Example:

```rust
struct GetFamilyTreeSchema {
    diagram_id: usize,
    as_of: Option<NaiveDate>,
}
```

Relationship repository should support a date-filtered read path:

```rust
struct GetRelationshipsAtDateSchema {
    diagram_id: usize,
    as_of: NaiveDate,
}
```

Recommended SQL predicate:

```sql
WHERE
  r.diagram_id = $1
  AND r.deleted_at IS NULL
  AND (r.start_date IS NULL OR r.start_date <= $2)
  AND (r.end_date IS NULL OR $2 <= r.end_date)
```

Person loading can initially reuse the current person list query and filter `birth_date` in the usecase. If this becomes inefficient, add a repository query that applies the same visibility rule in SQL.

## Current Versus Historical Modes

There should be two separate code paths at the point where lineage closure is obtained:

- current mode: may use maintained `tree_path`
- as-of mode: builds request-local lineage closure from date-filtered relationships

Do not mutate `tree_path` during an as-of read.

Do not try to temporarily rebuild `tree_path` for a historical date.

The maintained `tree_path` remains a write-side consistency optimization for the current active graph.

## Incremental Rollout

### Phase 1

Add `as_of` to `GET /api/family-trees/{diagram_id}`.

For this phase:

- filter explicit relationships by date
- keep the public response shape mostly unchanged
- derive direct edges from the as-of relationship set
- build roots and adjacency from those as-of edges
- do not expose richer derived kinship yet

### Phase 2

Wire `KinshipDerivationService` to accept as-of input.

For this phase:

- derive sibling / ancestor / descendant against as-of lineage
- include source metadata internally
- keep public response stable unless a separate response contract is accepted

### Phase 3

Add a richer temporal kinship endpoint if needed:

```http
GET /api/family-trees/{diagram_id}/kinships?as_of=1995-01-01&center_entity_id=10
```

This endpoint can expose richer labels and center-person-relative kinship without overloading the current family-tree projection.

### Phase 4

Consider temporal closure caching only if profiling shows request-local derivation is too slow.

## Benefits

- supports historical family-tree rendering without event sourcing
- reuses existing `start_date` and `end_date`
- keeps `tree_path` semantics clean
- works with canonical relationship storage from RFC 0013
- gives UI a clear `as_of` parameter for timeline controls
- allows derived kinship to be evaluated consistently at a point in time

## Drawbacks

- as-of reads may be slower than current-state reads because closure is derived per request
- date ranges with unknown starts or ends can still be semantically ambiguous
- existing rows without dates are treated as always valid, which may be historically inaccurate
- deleted rows are not available for historical projection unless archival support is added later
- richer temporal semantics may eventually require event history or versioned facts

## Test Plan

Minimum coverage:

- omitting `as_of` preserves current family-tree response behavior
- invalid `as_of` returns `400 BAD_REQUEST`
- relationship before `start_date` is excluded
- relationship on `start_date` is included
- relationship on `end_date` is included
- relationship after `end_date` is excluded
- person with `birth_date > as_of` is excluded
- deceased person remains visible after `death_date`
- roots and adjacency are recomputed from the as-of graph
- as-of projection does not mutate `tree_path`
- cycle detection runs against the as-of lineage graph

## Open Questions

- Should `deleted_at` rows ever participate in historical as-of reads through an audit mode?
- Should date precision support year-only or month-only historical facts?
- Should there be separate projection modes for ancestry, household, and legal family state?
- Should former relationships be shown as historical annotations after `end_date`, or excluded from active as-of relation output?
- Should temporal closure caching be diagram-wide, per date, or not introduced until required by profiling?
