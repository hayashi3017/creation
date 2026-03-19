# ADR 0003: Transaction Port For Aggregate Writes

- Status: `Accepted`
- Last updated: `2026-03-15`

## Context

公開 `person` API は `entity` の共通項目と `person` の詳細項目を 1 リクエストで扱う aggregate write model になっている。

その一方で、adapter の `person_repository` が `entity` と `person` の両テーブルを直接更新すると、repository の責務が table boundary をまたぎやすい。

今回決めたいのは次の 2 点:

- `entity` / `person` の table-focused repository を維持すること
- それでも `person` aggregate write の原子性を失わないこと

## Decision

aggregate write の orchestration は usecase 層で行い、transaction boundary は service 側の port である `ProvidesTransactionManager` / `TransactionContext` で表現する。

具体的には:

- `entity_repository` は `entity` テーブルだけを更新する
- `person_repository` は `person` テーブルだけを更新する
- `PersonUsecase` が create/update/delete の手順を決める
- usecase は `begin_transaction()` で transaction-aware な service container を受け取る
- usecase は repository ではなく service を呼ぶ
- repository は内部に shared transaction state を持ち、tx が有効なら transaction、なければ pool を使う
- adapter が SQLx transaction を shared state として保持し、service container に注入する

usecase は「この操作は 1 transaction」と決めるが、`sqlx::Transaction` 自体は握らない。

`BEGIN` / `COMMIT` の SQLx 実装は adapter に閉じ込め、usecase からは `ProvidesTransactionManager::begin_transaction()` 経由で使う。

理由:

- transaction は domain rule ではなく persistence concern
- usecase は service port にだけ依存し、adapter 実装詳細を知らなくてよい
- usecase は boundary を決めるだけで、DB 実装詳細からは切り離せる

## Rejected Alternatives

### Option A: `person_repository` が `entity + person` を直接更新する

Pros:

- 実装は単純
- usecase は薄いままで済む

Cons:

- repository の責務が specialization aggregate に寄りすぎる
- table-focused な再利用がしにくい

### Option B: usecase が `sqlx::Transaction` を直接扱う

Pros:

- transaction boundary は明示的

Cons:

- usecase が adapter / SQLx 実装詳細に依存する
- layered boundary が崩れる

### Option C: transaction ごとに tx-scoped repository を組み立てる

Cons:

- resource が増えるたびに transaction 専用 repository 型が増える
- repository module の外側で CRUD 実装が重複しやすい

## Consequences

- `person` の create/update/delete は usecase で validation 済み payload を組み立ててから transaction port を開始し、transaction-aware な service container を通して順に呼ぶ
- `person_repository` の write test は `person` テーブル専用の振る舞いだけを確認する
- aggregate write の end-to-end は driver test で担保する
- 将来 specialization が増えても、同じ transaction-aware container の上に service を追加する形で拡張できる
