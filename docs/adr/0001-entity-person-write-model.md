# ADR 0001: Entity-Person Write Model

- Status: `Draft`
- Last updated: `2026-03-15`

## Context

`entity` CRUD is implemented, but `person` table fields are not yet writable through the API.

Current schema shape:

- `entity`: generic node row
- `person`: `entity(kind='person')` specialization row

If we postpone this decision too long, future API additions may force a breaking request/response change.

## Options

### Option A: Entity and Person are written separately

- generic entity endpoint manages only `entity`
- separate person endpoint manages `person`

Pros:

- clear table ownership
- simpler generic entity payload

Cons:

- clients must coordinate multiple writes
- partial write risk unless the server adds orchestration

### Option B: Single transactional write model with nested kind-specific payload

- generic entity create/update accepts common fields
- when `kind = person`, payload includes nested `person` details
- server writes both tables in one transaction

Pros:

- matches current domain shape better
- easier for clients to create complete person entities
- avoids partial write flows

Cons:

- generic entity API becomes kind-aware
- future entity kinds need extensible payload design

## Proposed Decision

Choose **Option B**.

Recommended request shape:

```json
{
  "kind": "person",
  "name": "Alice",
  "description": "example",
  "person": {
    "gender": "female",
    "birth_date": "1990-01-01"
  }
}
```

## Consequences

- service layer must validate common fields and kind-specific payloads together
- repository layer should use one transaction for `entity` + `person`
- future specializations should follow the same nested payload pattern rather than inventing unrelated write APIs

## Follow-Up

- current implementation exposes `person` as the public aggregate endpoint and writes `entity + person` together there, but generic `/api/entities` is no longer the public write surface
- internal transaction ownership for that aggregate write is tracked separately in `docs/adr/0003-transaction-port-for-aggregate-writes.md`
- define how reads should expose person-specific fields
- define whether non-`person` entity kinds require their own nested object keys when introduced
