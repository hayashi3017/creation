# RFC 0010: Public API の明示的な ID Field

- 状態: `採用`
- 最終更新: `2026-04-11`

## 背景

RFC 0009 では、schema 側で root table の汎用 primary key 名 `id` を避ける方針を定義した。

一方で public API にはまだ汎用 `id` が残っている。

- response object: `user.id`, `diagram.id`, `relationship.id`
- request body: `UpdateDiagramSchema { id, ... }`
- path parameter: `/api/diagrams/update/{id}`, `/api/relationships/delete/{id}`

これは API 内の明示的な identifier 名と不整合である。

- `diagram_id`
- `entity_id`
- `source_entity_id`
- `target_entity_id`
- `relationship_id`

この project はまだ release 前なので、旧 `id` field の backward compatibility layer は不要とする。

## 目標

public API の identifier を resource-scoped かつ明示的な名前へ統一する。

## 対象外

- `/api/diagrams` や `/api/persons` など resource name の変更
- identifier 以外の field name の変更
- legacy client 向け versioned compatibility strategy

## 提案

public API contract の identifier field は、表している resource に合わせた明示名を使う。

推奨 rule:

- user: `user_id`
- diagram: `diagram_id`
- entity/person: `entity_id`
- relationship: `relationship_id`

path parameter も同じ名前を使う。

例:

- `/api/diagrams/update/{diagram_id}`
- `/api/diagrams/delete/{diagram_id}`
- `/api/persons/update/{entity_id}`
- `/api/persons/delete/{entity_id}`
- `/api/relationships/update/{relationship_id}`
- `/api/relationships/delete/{relationship_id}`

## Response 例

```json
{
  "diagram_id": 1,
  "name": "sample",
  "kind": "family_tree"
}
```

```json
{
  "relationship_id": 10,
  "diagram_id": 1,
  "source_entity_id": 1,
  "target_entity_id": 2,
  "kind": "parent"
}
```

## Migration 方針

- DTO/schema struct の field を explicit name に変更する
- handler path parameter 名を変更する
- OpenAPI docs を更新する
- tests を新 contract に合わせる
- release 前なので旧 `id` alias は提供しない

## 影響

- DB schema と public API の語彙が揃う
- client code が resource type を推測しやすくなる
- generic `id` の混乱を避けられる

## 未解決事項

- timestamp field の casing を将来揃えるか
- create response で作成済み resource id を返すか
