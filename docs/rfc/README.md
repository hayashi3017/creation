# RFC 索引

## 状態

- `下書き`: レビュー中
- `採用`: 承認済みで、実装に進めてよい
- `置き換え済み`: 別の RFC によって置き換えられた

## ファイル

- `docs/rfc/0001-resource-oriented-api-shape.md`: Diagram / Person API の path と parameter shape
- `docs/rfc/0002-mutation-result-and-error-mapping.md`: Update/Delete の結果 semantics と HTTP status mapping
- `docs/rfc/0003-validation-and-normalization.md`: 入力 validation と normalization policy
- `docs/rfc/0004-sqlx-workflow-and-test-environment.md`: SQLx offline workflow と DB-backed test setup
- `docs/rfc/0005-entity-person-soft-delete-consistency.md`: `entity` と `person` の soft-delete propagation policy
- `docs/rfc/0006-relationship-soft-delete-and-tree-path-consistency.md`: `relationship` と `tree_path` の soft-delete propagation policy
- `docs/rfc/0007-family-tree-read-api.md`: family-tree read projection endpoint と contract
- `docs/rfc/0008-genealogy-aggregate-and-visibility.md`: cross-diagram genealogy aggregate、merge semantics、publication scope、entity visibility
- `docs/rfc/0009-explicit-primary-key-column-names.md`: root table の汎用 `id` column を resource-scoped primary key 名へ変更する方針
- `docs/rfc/0010-explicit-id-fields-in-public-api.md`: public API の `id` field と path parameter を明示的な名前へ変更する方針
- `docs/rfc/0011-rust-workspace-build-time.md`: Rust workspace の local build / check loop 高速化方針
- `docs/rfc/0012-kinship-derivation-service.md`: stored relationship と read-side kinship derivation の MECE な service boundary
- `docs/rfc/0013-canonical-relationship-kinds-and-tree-path.md`: canonical stored relationship kind と tree-path maintenance policy
- `docs/rfc/0014-as-of-family-tree-projection.md`: as-of date 指定による family-tree / kinship projection policy
