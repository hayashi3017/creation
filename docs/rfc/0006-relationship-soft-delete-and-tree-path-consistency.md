# RFC 0006: Relationship Soft-Delete と Tree Path Consistency

- 状態: `下書き`
- 最終更新: `2026-03-20`

## 背景

`relationship` write は同じ transaction 内で `tree_path` を rebuild するため、lineage edge の create / update / delete は整合する。

ただし未解決の path がある。

- `entity` または `person` は `relationship` に触れずに soft-delete できる
- その path では現在 `tree_path` が rebuild されない

`relationship` row は entity を直接参照するため、entity を soft-delete すると active relationship row と古い closure row が残り、visible data を表さない状態になりうる。

## 提案

次の範囲について soft-delete propagation policy を 1 つに決める。

- `entity`
- `person`
- `relationship`
- `tree_path`

## 選択肢

### Option A: relationship へ cascade soft-delete し、即時に tree_path を rebuild する

挙動:

- 対象 `entity` を soft-delete する
- その entity を参照する active `relationship` row を soft-delete する
- affected diagram の `tree_path` を同じ transaction で rebuild する

利点:

- read model が物理的にも整合する
- traversal が stale row filtering に依存しない

欠点:

- delete path が重くなる
- historical relationship row の扱いを別途整理する必要がある

### Option B: relationship は残し、read/rebuild 時に deleted entity を除外する

挙動:

- active relationship row はそのまま残す
- read と `tree_path` rebuild は deleted entity を無視する

利点:

- relationship history を直接残しやすい
- delete path の write が少ない

欠点:

- 全 read/rebuild path で filtering を徹底する必要がある
- stale row が DB 上に残る

### Option C: relationship visibility 用の archival state を追加する

挙動:

- entity deletion と relationship archival を別 state として扱う

利点:

- history と visibility を分けられる

欠点:

- model が増える
- 初期実装としては重い

## 未解決事項

- entity/person delete は同じ transaction で relationship soft-delete を起こすべきか
- そうしない場合でも `tree_path` は proactive に rebuild すべきか
- entity soft-delete 後も historical relationship row を query 可能にするか
