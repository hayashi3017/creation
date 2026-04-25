# ADR Index

## Status

- `Draft`: under discussion
- `Accepted`: agreed and should guide implementation
- `Superseded`: replaced by a newer ADR

## Files

- `docs/adr/0001-entity-person-write-model.md`: How Entity writes should interact with the `person` specialization table
- `docs/adr/0002-entity-diagram-reassignment-policy.md`: Whether generic entity update may change `diagram_id`
- `docs/adr/0003-transaction-port-for-aggregate-writes.md`: Where aggregate orchestration and transaction boundaries should live
- `docs/adr/0004-relationship-lineage-tree-maintenance.md`: Superseded initial policy for relationship kinds and `tree_path` maintenance
- `docs/adr/0005-as-of-projection-tree-path-boundary.md`: Why as-of family-tree reads derive lineage closure request-locally instead of using current `tree_path`
