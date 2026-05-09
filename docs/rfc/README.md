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
- `docs/rfc/0008-world-aggregate-and-membership.md`: world aggregate、entity 所属、diagram 所属の方針
- `docs/rfc/0009-explicit-primary-key-column-names.md`: root table の汎用 `id` column を resource-scoped primary key 名へ変更する方針
- `docs/rfc/0010-explicit-id-fields-in-public-api.md`: public API の `id` field と path parameter を明示的な名前へ変更する方針
- `docs/rfc/0011-rust-workspace-build-time.md`: Rust workspace の local build / check loop 高速化方針
- `docs/rfc/0012-kinship-derivation-service.md`: stored relationship と read-side kinship derivation の MECE な service boundary
- `docs/rfc/0013-canonical-relationship-kinds-and-tree-path.md`: canonical stored relationship kind と tree-path maintenance policy
- `docs/rfc/0014-as-of-genealogy-projection.md`: as-of date 指定による genealogy diagram / overview / kinship projection policy
- `docs/rfc/0015-genealogy-overview-api.md`: world 内 diagram を統合する Genealogy Overview API
- `docs/rfc/0016-world-crud-api.md`: world CRUD API と lifecycle semantics
- `docs/rfc/0017-familytree-to-genealogy-naming.md`: FamilyTree から Genealogy への命名変更方針
- `docs/rfc/0018-resource-authorization-boundary.md`: world / diagram / entity / relationship の認可境界と外部 authorization 連携方針
- `docs/rfc/0019-session-and-jwt-hardening.md`: session cookie / JWT transport / revocation の hardening 方針
- `docs/rfc/0020-csrf-and-state-changing-requests.md`: cookie auth 利用時の CSRF と state-changing request policy
- `docs/rfc/0021-auth-abuse-and-account-enumeration.md`: login / register abuse protection と account enumeration 対策
- `docs/rfc/0022-error-response-and-observability-boundary.md`: client-facing error と server-side observability の境界
- `docs/rfc/0023-request-size-and-public-docs-hardening.md`: request body size limit と public docs / Swagger UI hardening 方針
- `docs/rfc/0024-genealogy-graph-payload-contract.md`: diagram / world 共通の Genealogy Graph response contract
- `docs/rfc/0025-tree-path-scope-and-maintenance.md`: world-scoped relationship / `tree_path` と mutation 時の maintenance policy
