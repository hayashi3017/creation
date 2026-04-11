# Improvements

Last updated: 2026-04-11

Detailed proposals and decisions:

- RFC index: `docs/rfc/README.md`
- ADR index: `docs/adr/README.md`

## Diagram API

- Diagram API は list の `GET /api/diagrams` だけ resource-oriented 側に揃っていて、write 系は `POST /api/diagrams/create`, `PATCH /api/diagrams/update/{id}`, `DELETE /api/diagrams/delete/{id}` の action-style な形が残っている。resource-oriented な path へ寄せるか、この形を正式仕様にするかを決めた方が API の一貫性が上がる。See `docs/rfc/0001-resource-oriented-api-shape.md`.

## Person API

- Person API は `GET /api/persons` が body に依存し、write 系は `POST /api/persons/create`, `PATCH /api/persons/update/{id}`, `DELETE /api/persons/delete/{id}` の action-style な形になっている。resource-oriented な path と parameter policy に揃えるか、この形を正式仕様にするかを決めた方が API の一貫性が上がる。See `docs/rfc/0001-resource-oriented-api-shape.md`.
- 現在の公開 API は `person` に `entity` の共通項目を取り込んだ aggregate write model になっている。将来 `person` 以外の `entity_kind` を追加するなら、generic `/api/entities` を復活させるのか、kind ごとの aggregate endpoint を増やすのかを早めに決めた方が拡張しやすい。See `docs/adr/0001-entity-person-write-model.md`.
- aggregate write は service-side の transaction port で transaction-aware な service container を作り、その配下の repository が内部の tx 有無を見て pool / transaction を切り替える形になった。将来 specialization が増えた時に、transaction-aware repository の状態管理と commit 後の振る舞いをどこまで共通化するかを決めておくと内部設計を揃えやすい。See `docs/adr/0003-transaction-port-for-aggregate-writes.md`.

## Relationship API

- Relationship API も `GET /api/relationships` の body 依存と `POST /api/relationships/create`, `PATCH /api/relationships/update/{id}`, `DELETE /api/relationships/delete/{id}` の action-style な path を採用している。Diagram / Person と同じ観点なので、resource-oriented に寄せるか現行形を正式化するかを RFC 0001 でまとめて整理した方がよい。See `docs/rfc/0001-resource-oriented-api-shape.md`.
- 現在の公開 API と `tree_path` 再構築は lineage 用に限定し、`parent` / `child` だけを受け付けている。`sibling` / `spouse` / `adopted_*` / `step_*` をいつ公開 API に出すか、そのとき `tree_path` に影響させるかを別途決めた方がよい。See `docs/adr/0004-relationship-lineage-tree-maintenance.md`.

## Validation And Error Handling

- `entity` の write validation では、まだ `diagram_id` の存在確認をしていない。作成先 / 更新先の diagram が存在し、かつ論理削除されていないことをどのレイヤで保証するかを決めた方が仕様が安定する。See `docs/rfc/0003-validation-and-normalization.md`.
- `update_entity` / `update_person` は diagram 間移動を拒否するようになったが、現行 API は更新 payload に `diagram_id` を残したまま「現在の所属 diagram と一致していること」を guard として使っている。resource-oriented な path に寄せるなら、この guard を残すか path / server-side lookup に寄せるかを整理した方がよい。See `docs/rfc/0001-resource-oriented-api-shape.md` and `docs/adr/0002-entity-diagram-reassignment-policy.md`.
- 公開 `person` API は entity/person を同時に論理削除するが、内部の `entity_repository.delete_entity(...)` は依然として親 `entity` だけを論理削除する。specialization row の削除伝播方針は内部 API も含めて明文化した方がよい。See `docs/rfc/0005-entity-person-soft-delete-consistency.md`.
- User API にも Diagram / Person と同じ status mapping policy を適用するかは未整理。`register/login/me/logout` を同じ観点で揃えるか、User API だけ別ポリシーにするかを決めた方がエラー契約の見通しが良くなる。See `docs/rfc/0002-mutation-result-and-error-mapping.md`.

## Schema Naming

- Public API responses, request bodies, and path parameters still expose generic `id` names in several places. Since this API is not released yet, the contract can be cleaned up in one pass without compatibility aliases. See `docs/rfc/0010-explicit-id-fields-in-public-api.md`.

## API Documentation

- OpenAPI text is currently authored in English under `creation-driver/src/openapi_docs/en/`. If Japanese or other locales are needed, decide whether to publish multiple localized specs such as `/api-docs/openapi.en.json` and `/api-docs/openapi.ja.json`, or to rewrite summaries and descriptions at render time with a locale-aware `Modify` step. Until that policy is chosen, the generated spec is English-only.

## Genealogy Merge And Visibility

- Cross-diagram merge, publication scope, entity-level hidden state, and centered merged-family-tree reads are not defined yet. Before implementing merged genealogy views, review the aggregate boundary, naming, and visibility propagation policy in `docs/rfc/0008-genealogy-aggregate-and-visibility.md`.

## Tooling And Tests

- `xtask` は依然として外部の `sqlx-cli` バイナリに依存している。初回セットアップでの詰まりを減らすなら、前提ツールの検査や bootstrap コマンドを追加してもよい。 See `docs/rfc/0004-sqlx-workflow-and-test-environment.md`.
- `.env` と `.env.docker` で host / container 向けの `DATABASE_URL` が混ざりやすい。`localhost` を前提にした `.env.example` や bootstrap 時の設定チェックを追加すると、`cargo run` / `cargo test` の接続失敗を減らせる。 See `docs/rfc/0004-sqlx-workflow-and-test-environment.md`.
- The workspace does not yet define an explicit local build-time policy around root-level profiles, `release-fast`, optional `sccache`, `default-members`, or future `build.rs` rerun conditions. See `docs/rfc/0011-rust-workspace-build-time.md`.
