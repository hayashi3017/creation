# Improvements

Last updated: 2026-03-14

## Diagram API

- `POST /api/diagrams/update` / `POST /api/diagrams/delete` は動作するが、将来的には `PATCH /api/diagrams/:id` / `DELETE /api/diagrams/:id` のような REST 形に寄せた方が API の一貫性が上がる。
- `update_diagram` と `delete_diagram` は現在、対象が存在しない場合や既に `deleted_at` が入っている場合でも `200` を返す。更新件数を確認して `404 Not Found` か `409 Conflict` を返すようにした方がクライアント側で扱いやすい。
- `GET /api/diagrams` だけ JSON body を要求している。検索条件が不要なら body なしにするか、必要なら query parameter に寄せた方が扱いやすい。

## Entity API

- `POST /api/entities/update` / `POST /api/entities/delete` も `diagram` と同様に、将来的には `PATCH /api/entities/:id` / `DELETE /api/entities/:id` に寄せた方が API の一貫性が上がる。
- `GET /api/entities` は `diagram_id` を JSON body で受けている。`GET` の body 依存を避けて query parameter か `/api/diagrams/:id/entities` のようなネストした path に寄せた方が扱いやすい。
- `update_entity` と `delete_entity` は現在、対象が存在しない場合や既に削除済みの場合でも `200` を返す。更新件数を確認して `404 Not Found` か `409 Conflict` を返すようにした方がクライアント側で扱いやすい。
- `entity.kind = person` だけ先に CRUD 化されていて、`person` テーブルの詳細項目はまだ API から扱えない。`entity` と `person` を同時に作成・更新する入力モデルを定義するか、責務を分けた API にするかを早めに決めた方が後方互換を壊しにくい。

## Validation And Error Handling

- `diagram` のサービス層バリデーションは最小限で、`name` の空文字と `id == 0` しか見ていない。長さ上限、前後空白の扱い、`description` の空文字を `NULL` に寄せるかどうかを決めると API 契約が安定する。
- `entity` も同様に `diagram_id` / `id` と `name` の最小検証しかしていない。`diagram_id` の存在確認、diagram 間移動を許可するかどうか、`description` の扱いを決めると仕様が安定する。
- Diagram 系ハンドラの DB エラーは一部 `400 Bad Request` に丸められている。`500 Internal Server Error` と `400` の使い分けを整理した方が、他ハンドラとの整合も取りやすい。

## Tooling And Tests

- このリポジトリは `SQLX_OFFLINE=true` 前提のビルドパスがあるため、新しい `query!` / `query_as!` を追加すると SQLx キャッシュ更新が必要になる。`sqlx prepare` を CI に組み込むか、追加方針を明文化した方が変更時の詰まりを減らせる。
- `sqlx::test` を使うテストは、実行環境によってはセットアップ用 DB に接続できず失敗する。ローカル/CI で必ず到達可能な Postgres 起動手順を `xtask` かテスト手順に寄せておくと再現性が上がる。
