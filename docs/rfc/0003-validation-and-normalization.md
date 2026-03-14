# RFC 0003: Validation And Normalization

- Status: `Accepted`
- Last updated: `2026-03-14`

## Background

Diagram / Entity validation is currently minimal:

- `id` / `diagram_id` must be non-zero
- `name` must not be an empty string

This leaves whitespace-only names, max length, description normalization, and foreign key checks underspecified.

## Proposal

Define shared write-time validation rules.

### Name

- trim leading and trailing whitespace before validation
- reject empty results after trimming
- enforce an explicit max length aligned with DB column size

### Description

- trim surrounding whitespace
- convert empty string to `NULL`
- document whether multi-line text is preserved as-is

### Diagram

- apply the shared name / description rules

### Entity

- apply the shared name / description rules
- validate that the target `diagram_id` exists and is not soft-deleted before create
- validate updates according to [ADR 0002](../adr/0002-entity-diagram-reassignment-policy.md)

## Implementation Notes

- normalization should happen before repository calls
- validation errors should use typed domain errors rather than relying on DB constraint failures

## Implementation Status

- `diagram` / `entity` write paths now trim surrounding whitespace from `name`.
- `name` is rejected when empty after trimming or when it exceeds the current DDL-backed `VARCHAR(255)` limit.
- `description` is now optional in write requests and is normalized to `NULL` when omitted, `null`, or blank after trimming.
- `diagram_id` existence checks and the final entity reassignment policy remain pending follow-up work tied to [ADR 0002](../adr/0002-entity-diagram-reassignment-policy.md).

## Review Points

- Should max length be checked by characters, bytes, or left to PostgreSQL column validation?
- Do we want to preserve intentionally blank descriptions as empty strings, or standardize on `NULL` everywhere?
