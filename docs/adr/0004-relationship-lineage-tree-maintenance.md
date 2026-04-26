# ADR 0004: Relationship Lineage Tree Maintenance

- 状態: `置き換え済み`
- 最終更新: `2026-04-26`
- 置き換え先: `docs/rfc/0013-canonical-relationship-kinds-and-tree-path.md`

## 背景

DB schema には次の 2 つの概念がある。

- `relationship`: entity 間の汎用 link
- `tree_path`: ancestor / descendant traversal 用の closure table

当時の `relationship_kind` enum には lineage と non-lineage が混在していた。

- lineage 的なもの: `parent`, `child`
- non-lineage: `sibling`, `spouse`, `adopted_*`, `step_*`, `cohabitant`, `divorced_spouse`

`tree_path` は directed ancestry に対してだけ意味が明確である。すべての relationship kind を closure maintenance に参加させると、ancestor / descendant の意味が曖昧になる。

## 決定

この ADR が採用された時点の実装では、次を方針とした。

- public `relationship` write API は `parent` と `child` だけを受け付ける
- `tree_path` は active lineage edge だけから rebuild する
- `parent` は `source_entity_id -> target_entity_id` と解釈する
- `child` は `target_entity_id -> source_entity_id` と解釈する
- non-lineage relationship kind は public API にまだ公開しない

## 影響

- ancestor traversal semantics は狭く予測しやすい
- cycle detection は directed lineage graph に対して実装できる
- `sibling`, `spouse`, `adopted_*`, `step_*` の将来対応では次を再検討する必要があった
- その kind を public API で受け付けるか
- `tree_path` に影響させるか
- 別の traversal structure が必要か

## 現在の状態

この ADR は RFC 0013 によって置き換えられた。現在は canonical relationship kind と `parent` / `adoptive_parent` を中心にした `tree_path` policy を採用する。
