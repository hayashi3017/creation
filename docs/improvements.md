# Improvements

Last updated: 2026-03-14

Detailed proposals and decisions:

- RFC index: `docs/rfc/README.md`
- ADR index: `docs/adr/README.md`

## Diagram API

- Diagram API は list の `GET /api/diagrams` だけ resource-oriented 側に揃っていて、write 系は `POST /api/diagrams/create`, `PATCH /api/diagrams/update/{id}`, `DELETE /api/diagrams/delete/{id}` の action-style な形が残っている。resource-oriented な path へ寄せるか、この形を正式仕様にするかを決めた方が API の一貫性が上がる。See `docs/rfc/0001-resource-oriented-api-shape.md`.
- `update_diagram` と `delete_diagram` は現在、対象が存在しない場合や既に `deleted_at` が入っている場合でも `200` を返す。更新件数を確認して `404 Not Found` か `409 Conflict` を返すようにした方がクライアント側で扱いやすい。See `docs/rfc/0002-mutation-result-and-error-mapping.md`.

## Entity API

- Entity API は `GET /api/entities` が body に依存し、作成は `/api/entities/create`、更新は `/api/entities/update/{id}`、削除だけ `/api/entities/{id}` という mixed な形になっている。list/create/update/delete の path policy と parameter の置き場所を揃えた方が API の一貫性が上がる。See `docs/rfc/0001-resource-oriented-api-shape.md`.
- `update_entity` と `delete_entity` は現在、対象が存在しない場合や既に削除済みの場合でも `200` を返す。更新件数を確認して `404 Not Found` か `409 Conflict` を返すようにした方がクライアント側で扱いやすい。See `docs/rfc/0002-mutation-result-and-error-mapping.md`.
- `entity.kind = person` だけ先に CRUD 化されていて、`person` テーブルの詳細項目はまだ API から扱えない。`entity` と `person` を同時に作成・更新する入力モデルを定義するか、責務を分けた API にするかを早めに決めた方が後方互換を壊しにくい。See `docs/adr/0001-entity-person-write-model.md`.

## Validation And Error Handling

- `diagram` のサービス層バリデーションは最小限で、`name` の空文字と `id == 0` しか見ていない。長さ上限、前後空白の扱い、`description` の空文字を `NULL` に寄せるかどうかを決めると API 契約が安定する。See `docs/rfc/0003-validation-and-normalization.md`.
- `entity` も同様に `diagram_id` / `id` と `name` の最小検証しかしていない。`diagram_id` の存在確認、diagram 間移動を許可するかどうか、`description` の扱いを決めると仕様が安定する。See `docs/rfc/0003-validation-and-normalization.md` and `docs/adr/0002-entity-diagram-reassignment-policy.md`.
- Diagram 系ハンドラの DB エラーは一部 `400 Bad Request` に丸められている。`500 Internal Server Error` と `400` の使い分けを整理した方が、他ハンドラとの整合も取りやすい。See `docs/rfc/0002-mutation-result-and-error-mapping.md`.

## Tooling And Tests

- このリポジトリは `SQLX_OFFLINE=true` 前提のビルドパスがあるため、新しい `query!` / `query_as!` を追加すると SQLx キャッシュ更新が必要になる。`sqlx prepare` を CI に組み込むか、追加方針を明文化した方が変更時の詰まりを減らせる。 See `docs/rfc/0004-sqlx-workflow-and-test-environment.md`.
- `sqlx::test` を使うテストは、実行環境によってはセットアップ用 DB に接続できず失敗する。ローカル/CI で必ず到達可能な Postgres 起動手順を `xtask` かテスト手順に寄せておくと再現性が上がる。 See `docs/rfc/0004-sqlx-workflow-and-test-environment.md`.
