# Improvements

Last updated: 2026-03-14

Detailed proposals and decisions:

- RFC index: `docs/rfc/README.md`
- ADR index: `docs/adr/README.md`

## Diagram API

- Diagram API は list の `GET /api/diagrams` だけ resource-oriented 側に揃っていて、write 系は `POST /api/diagrams/create`, `PATCH /api/diagrams/update/{id}`, `DELETE /api/diagrams/delete/{id}` の action-style な形が残っている。resource-oriented な path へ寄せるか、この形を正式仕様にするかを決めた方が API の一貫性が上がる。See `docs/rfc/0001-resource-oriented-api-shape.md`.

## Entity API

- Entity API は `GET /api/entities` が body に依存し、作成は `/api/entities/create`、更新は `/api/entities/update/{id}`、削除だけ `/api/entities/{id}` という mixed な形になっている。list/create/update/delete の path policy と parameter の置き場所を揃えた方が API の一貫性が上がる。See `docs/rfc/0001-resource-oriented-api-shape.md`.
- `entity.kind = person` だけ先に CRUD 化されていて、`person` テーブルの詳細項目はまだ API から扱えない。`entity` と `person` を同時に作成・更新する入力モデルを定義するか、責務を分けた API にするかを早めに決めた方が後方互換を壊しにくい。See `docs/adr/0001-entity-person-write-model.md`.

## Validation And Error Handling

- `entity` の write validation では、まだ `diagram_id` の存在確認をしていない。作成先 / 更新先の diagram が存在し、かつ論理削除されていないことをどのレイヤで保証するかを決めた方が仕様が安定する。See `docs/rfc/0003-validation-and-normalization.md`.
- `update_entity` で diagram 間移動を許可するかどうかは未確定。validation rule と update semantics を `docs/adr/0002-entity-diagram-reassignment-policy.md` に沿って固めた方が安全。See `docs/rfc/0003-validation-and-normalization.md` and `docs/adr/0002-entity-diagram-reassignment-policy.md`.
- User API にも Diagram / Entity と同じ status mapping policy を適用するかは未整理。`register/login/me/logout` を同じ観点で揃えるか、User API だけ別ポリシーにするかを決めた方がエラー契約の見通しが良くなる。See `docs/rfc/0002-mutation-result-and-error-mapping.md`.

## Tooling And Tests

- `xtask` は依然として外部の `sqlx-cli` バイナリに依存している。初回セットアップでの詰まりを減らすなら、前提ツールの検査や bootstrap コマンドを追加してもよい。 See `docs/rfc/0004-sqlx-workflow-and-test-environment.md`.
- `.env` と `.env.docker` で host / container 向けの `DATABASE_URL` が混ざりやすい。`localhost` を前提にした `.env.example` や bootstrap 時の設定チェックを追加すると、`cargo run` / `cargo test` の接続失敗を減らせる。 See `docs/rfc/0004-sqlx-workflow-and-test-environment.md`.
