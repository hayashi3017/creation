# RFC 0005: Entity-Person Soft Delete Consistency

- 状態: `下書き`
- 最終更新: `2026-03-15`

## 背景

`person` CRUD は `person` specialization table 上の API として実装された。

現在の挙動は次の通り。

- public `DELETE /api/persons/delete/{entity_id}` は `entity.deleted_at` と `person.deleted_at` の両方を設定する
- internal `entity_repository.delete_entity(...)` は `entity.deleted_at` だけを設定する
- `person` read/write は `person.deleted_at IS NULL` と `entity.deleted_at IS NULL` の両方を要求する

つまり、`entity` を削除すると API からは関連 `person` row が見えなくなるが、DB 上では `person` row 自体は logical delete されていない。

## 問題

現在の挙動では lifecycle semantics が暗黙的になる。

- `person.deleted_at IS NULL` だが親 `entity` は soft-delete 済み、という状態が残る
- 同じ `entity_id` の restore / recreate semantics が不明確になる
- 将来 specialization table が増えたときに同じ曖昧さが繰り返される

## 選択肢

### Option A: 親の soft delete を specialization row に伝播しない

- 現状維持
- child specialization row は物理的にも論理的にも active のまま残る
- API は親 entity の active 条件で非表示にする

利点:

- 実装変更が最小
- parent delete 時に cross-table update が不要

欠点:

- DB state と API visibility がずれる
- restore policy が曖昧

### Option B: 親 entity の soft delete を specialization row に伝播する

- entity/person を同じ transaction で soft-delete する
- 内部 repository も aggregate delete path 経由に寄せる

利点:

- DB state が API visibility と一致する
- future specialization にも同じ方針を適用しやすい

欠点:

- delete orchestration が必要
- 汎用 entity repository 単体では完結しない

## 提案

Option B を採用する方向で整理する。public aggregate delete と internal delete の semantics を揃え、specialization row の lifecycle を親 entity と一貫させる。

## 未解決事項

- restore API を追加する場合、restore も parent から child へ伝播させるか
- parent delete 後の child-specific delete は no-op にするか `404` にするか
- specialization が増えたときの共通 orchestration をどう表現するか
