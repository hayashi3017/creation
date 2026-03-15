# ADR 0003: Person Aggregate Transaction Boundary

- Status: `Accepted`
- Last updated: `2026-03-15`

## Context

公開 `person` API は `entity` の共通項目と `person` の詳細項目を 1 リクエストで扱う aggregate write model になっている。

その一方で、adapter の `person_repository` が `entity` と `person` の両テーブルを直接更新すると、repository の責務が table boundary をまたぎやすい。

今回決めたいのは次の 2 点:

- `entity` / `person` の table-focused repository を維持すること
- それでも `person` aggregate write の原子性を失わないこと

## Decision

`person` aggregate write の orchestration は usecase 層で行い、transaction boundary は `PersonWriteUnitOfWork` abstraction で表現する。

具体的には:

- `entity_repository` は `entity` テーブルだけを更新する
- `person_repository` は `person` テーブルだけを更新する
- `PersonUsecase` が create/update/delete の手順を決める
- adapter が `PersonWriteUnitOfWork` を実装し、SQLx transaction を隠蔽する

usecase は「この操作は 1 transaction」と決めるが、`sqlx::Transaction` 自体は握らない。

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

## Consequences

- `person` の create/update/delete は usecase で validation 済み payload を組み立ててから `PersonWriteUnitOfWork` を呼ぶ
- `person_repository` の write test は `person` テーブル専用の振る舞いだけを確認する
- aggregate write の end-to-end は driver test で担保する
- 将来 `person` 以外の specialization が増えたら、`PersonWriteUnitOfWork` を一般化するか、specialization ごとに専用 UnitOfWork を持つかを再検討する
