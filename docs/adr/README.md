# ADR 索引

## 状態

- `下書き`: 議論中
- `採用`: 合意済みで、実装時の判断基準にする
- `置き換え済み`: 新しい ADR または RFC によって置き換えられた

## ファイル

- `docs/adr/0001-entity-person-write-model.md`: `Entity` 書き込みと `person` 特化テーブルの関係
- `docs/adr/0002-entity-diagram-reassignment-policy.md`: 汎用 entity 更新で `diagram_id` の変更を許可するか
- `docs/adr/0003-transaction-port-for-aggregate-writes.md`: aggregate orchestration と transaction boundary の配置
- `docs/adr/0004-relationship-lineage-tree-maintenance.md`: relationship kind と `tree_path` maintenance に関する初期方針。現在は置き換え済み
- `docs/adr/0005-as-of-projection-tree-path-boundary.md`: as-of family-tree read が current `tree_path` ではなく request-local closure を使う理由
