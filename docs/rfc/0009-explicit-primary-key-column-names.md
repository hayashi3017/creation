# RFC 0009: 明示的な Primary Key Column 名

- 状態: `採用`
- 最終更新: `2026-04-11`

## 背景

現在の schema は 2 つの命名 style が混在している。

- root table の primary key column は汎用的な `id`
- foreign key column は `diagram_id`、`entity_id`、`source_entity_id`、`target_entity_id` のように明示的

例:

- `users.id`
- `diagram.id`
- `entity.id`
- `relationship.id`
- `entity.diagram_id`
- `person.entity_id`
- `relationship.diagram_id`

join や aggregate read が増えると、汎用 `id` は読みにくくなる。

- `SELECT id` は join ではほぼ必ず ambiguous
- SQL / SQLx mapping に alias が増える
- genealogy read ではさらに多くの table を join する

## 目標

DB schema が自己説明的になるよう、primary key column の命名方針を統一する。

## 対象外

- public HTTP field name の変更
- route や resource name の変更
- 既に意味が明確な primary key を持つ table の変更

## 提案

専用 surrogate primary key を持つ table は、resource-scoped な名前を使う。

- `users.id` -> `users.user_id`
- `diagram.id` -> `diagram.diagram_id`
- `entity.id` -> `entity.entity_id`
- `relationship.id` -> `relationship.relationship_id`

`person.entity_id` は entity specialization の primary key なので維持する。

`tree_path` は semantic role name を持つ composite key なので維持する。

- `tree_path.ancestor_id`
- `tree_path.descendant_id`

## Migration 方針

- column rename migration を追加する
- foreign key reference を新 column 名に合わせる
- Rust table model と repository query を更新する
- public API field name は RFC 0010 で扱う

## 影響

- SQL の可読性が上がる
- join 時の alias が減る
- code 上の `id` ambiguity が減る
- migration は DB schema と Rust mapping の両方に影響する

## 変更表

| table | 旧 column | 新 column |
|---|---|---|
| `users` | `id` | `user_id` |
| `diagram` | `id` | `diagram_id` |
| `entity` | `id` | `entity_id` |
| `relationship` | `id` | `relationship_id` |
| `person` | `entity_id` | 変更なし |
| `tree_path` | `(ancestor_id, descendant_id)` | 変更なし |
