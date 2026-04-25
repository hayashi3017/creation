# RFC Index

## Status

- `Draft`: review in progress
- `Accepted`: approved and ready for implementation
- `Replaced`: superseded by another RFC

## Files

- `docs/rfc/0001-resource-oriented-api-shape.md`: Diagram / Person API path and parameter shape
- `docs/rfc/0002-mutation-result-and-error-mapping.md`: Update/Delete result semantics and HTTP status mapping
- `docs/rfc/0003-validation-and-normalization.md`: Input validation and normalization policy
- `docs/rfc/0004-sqlx-workflow-and-test-environment.md`: SQLx offline workflow and DB-backed test setup
- `docs/rfc/0005-entity-person-soft-delete-consistency.md`: Soft-delete propagation policy between `entity` and `person`
- `docs/rfc/0006-relationship-soft-delete-and-tree-path-consistency.md`: Soft-delete propagation policy for `relationship` and `tree_path`
- `docs/rfc/0007-family-tree-read-api.md`: Family-tree read projection endpoint and contract
- `docs/rfc/0008-genealogy-aggregate-and-visibility.md`: Cross-diagram genealogy aggregate, merge semantics, publication scope, and entity visibility
- `docs/rfc/0009-explicit-primary-key-column-names.md`: Rename generic root-table `id` columns to explicit resource-scoped primary key names
- `docs/rfc/0010-explicit-id-fields-in-public-api.md`: Rename public API `id` fields and path parameters to explicit resource-scoped names
- `docs/rfc/0011-rust-workspace-build-time.md`: Workspace-level policy for faster local Rust build and check loops
- `docs/rfc/0012-kinship-derivation-service.md`: MECE service boundary for stored relationships and read-side kinship derivation
- `docs/rfc/0013-canonical-relationship-kinds-and-tree-path.md`: Canonical stored relationship kinds and tree-path maintenance policy
