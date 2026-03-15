# Improvements

Last updated: 2026-03-15

Detailed proposals and decisions:

- RFC index: `docs/rfc/README.md`
- ADR index: `docs/adr/README.md`

## Diagram API

- Diagram API は list の `GET /api/diagrams` だけ resource-oriented 側に揃っていて、write 系は `POST /api/diagrams/create`, `PATCH /api/diagrams/update/{id}`, `DELETE /api/diagrams/delete/{id}` の action-style な形が残っている。resource-oriented な path へ寄せるか、この形を正式仕様にするかを決めた方が API の一貫性が上がる。See `docs/rfc/0001-resource-oriented-api-shape.md`.

## Person API

- Person API は `GET /api/persons` が body に依存し、write 系は `POST /api/persons/create`, `PATCH /api/persons/update/{id}`, `DELETE /api/persons/delete/{id}` の action-style な形になっている。resource-oriented な path と parameter policy に揃えるか、この形を正式仕様にするかを決めた方が API の一貫性が上がる。See `docs/rfc/0001-resource-oriented-api-shape.md`.
- 現在の公開 API は `person` に `entity` の共通項目を取り込んだ aggregate write model になっている。将来 `person` 以外の `entity_kind` を追加するなら、generic `/api/entities` を復活させるのか、kind ごとの aggregate endpoint を増やすのかを早めに決めた方が拡張しやすい。See `docs/adr/0001-entity-person-write-model.md`.
- `person` aggregate write は `PersonWriteUnitOfWork` で transaction を張る形になった。将来 specialization が増えた時に、この abstraction を generic な aggregate write transaction に一般化するか、specialization ごとに専用 UoW を増やすかは早めに決めた方が内部設計を揃えやすい。See `docs/adr/0003-person-aggregate-transaction-boundary.md`.

## Validation And Error Handling

- `entity` の write validation では、まだ `diagram_id` の存在確認をしていない。作成先 / 更新先の diagram が存在し、かつ論理削除されていないことをどのレイヤで保証するかを決めた方が仕様が安定する。See `docs/rfc/0003-validation-and-normalization.md`.
- `update_entity` で diagram 間移動を許可するかどうかは未確定。validation rule と update semantics を `docs/adr/0002-entity-diagram-reassignment-policy.md` に沿って固めた方が安全。See `docs/rfc/0003-validation-and-normalization.md` and `docs/adr/0002-entity-diagram-reassignment-policy.md`.
- 公開 `person` API は entity/person を同時に論理削除するが、内部の `entity_repository.delete_entity(...)` は依然として親 `entity` だけを論理削除する。specialization row の削除伝播方針は内部 API も含めて明文化した方がよい。See `docs/rfc/0005-entity-person-soft-delete-consistency.md`.
- User API にも Diagram / Person と同じ status mapping policy を適用するかは未整理。`register/login/me/logout` を同じ観点で揃えるか、User API だけ別ポリシーにするかを決めた方がエラー契約の見通しが良くなる。See `docs/rfc/0002-mutation-result-and-error-mapping.md`.

## Tooling And Tests

- `xtask` は依然として外部の `sqlx-cli` バイナリに依存している。初回セットアップでの詰まりを減らすなら、前提ツールの検査や bootstrap コマンドを追加してもよい。 See `docs/rfc/0004-sqlx-workflow-and-test-environment.md`.
- `.env` と `.env.docker` で host / container 向けの `DATABASE_URL` が混ざりやすい。`localhost` を前提にした `.env.example` や bootstrap 時の設定チェックを追加すると、`cargo run` / `cargo test` の接続失敗を減らせる。 See `docs/rfc/0004-sqlx-workflow-and-test-environment.md`.
