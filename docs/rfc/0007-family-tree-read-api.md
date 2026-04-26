# RFC 0007: Family Tree Read API

- 状態: `下書き`
- 最終更新: `2026-03-23`

## 背景

現在の client は family tree を描画するために、既存 aggregate endpoint を個別に呼ぶ必要がある。

- `GET /api/persons`
- `GET /api/relationships`

その結果、次の処理を client 側が担っている。

- 対象 `diagram` が本当に `kind = family_tree` か確認する
- relationship を family-tree 用の親子 edge に正規化する
- forest view 用の root node を導出する
- projection に参加しない record を除外する

workspace は lineage-only closure table として `tree_path` を維持しているが、family-tree-oriented projection を返す public read API はまだない。

## 提案

認証付き read-only endpoint を追加する。

- `GET /api/family-trees/{diagram_id}`

この endpoint は active な `diagram(kind = family_tree)` 1 件について、正規化済み family-tree projection を返す。

`family-trees` は独立して書き込む aggregate ではなく、derived read model として扱う。そのため write operation は既存 resource の endpoint に残す。

- `POST /api/diagrams/create`
- `PATCH /api/diagrams/update/{diagram_id}`
- `DELETE /api/diagrams/delete/{diagram_id}`
- `POST /api/persons/create`
- `PATCH /api/persons/update/{entity_id}`
- `DELETE /api/persons/delete/{entity_id}`
- `POST /api/relationships/create`
- `PATCH /api/relationships/update/{relationship_id}`
- `DELETE /api/relationships/delete/{relationship_id}`

## レスポンス形状

```json
{
  "status": "success",
  "data": {
    "diagram": {
      "diagram_id": 1,
      "name": "Hayashi family",
      "kind": "family_tree",
      "description": "sample"
    },
    "root_entity_ids": [1],
    "nodes": [
      {
        "entity_id": 1,
        "diagram_id": 1,
        "name": "A",
        "description": null,
        "gender": "female",
        "birth_date": "1970-01-01",
        "death_date": null,
        "birthplace": null,
        "residence": null,
        "photo_url": null,
        "parent_entity_ids": [],
        "child_entity_ids": [2],
        "is_root": true
      }
    ],
    "edges": [
      {
        "relationship_id": 10,
        "parent_entity_id": 1,
        "child_entity_id": 2,
        "kind": "parent",
        "start_date": null,
        "end_date": null,
        "end_reason": null,
        "notes": null
      }
    ],
    "stats": {
      "person_count": 2,
      "edge_count": 1,
      "root_count": 1
    }
  }
}
```

## ノード契約

各 `node` は既存 `Person` payload を基礎にし、family-tree 固有の adjacency metadata を追加する。

- `parent_entity_ids: Vec<usize>`
- `child_entity_ids: Vec<usize>`
- `is_root: bool`

## エッジ契約

各 `edge` は UI で扱いやすい lineage-oriented な形にする。

- `relationship_id`
- `parent_entity_id`
- `child_entity_id`
- `kind`
- `start_date`
- `end_date`
- `end_reason`
- `notes`

RFC 0013 以降、stored tree-edge kind は `parent` と `adoptive_parent` を基本とする。response は常に `parent_entity_id -> child_entity_id` を表す。

## Validation と error mapping

- `diagram_id == 0` -> `400 BAD_REQUEST`
- diagram が存在しない -> `404 NOT_FOUND`
- diagram が soft-delete 済み -> `404 NOT_FOUND`
- diagram は存在するが `kind != family_tree` -> `400 BAD_REQUEST`
- 認証なし/不正 -> `401 UNAUTHORIZED`
- 予期しない DB / assembly failure -> `500 INTERNAL_SERVER_ERROR`

## レイヤ設計

### Driver 層

- `creation-driver/src/handler/family_tree.rs`
- `creation-driver/src/response.rs`
- `creation-driver/src/route/mod.rs`
- OpenAPI tag: `FamilyTrees`

### Usecase 層

`creation-usecase/src/usecase/family_tree.rs` を追加する。

責務:

1. `diagram_id` を検証する
2. 対象 diagram を読み込み、`family_tree` 以外を拒否する
3. diagram 内の person を読み込む
4. diagram 内の relationship を読み込む
5. kinship derivation service で lineage edge を正規化する
6. adjacency list と root を導出する
7. projection を返す

### Adapter 層

active diagram row を返す lookup を追加する。

- `get_diagram(body: GetDiagramSchema) -> Result<Option<Diagram>, ...>`

初期 read path は既存 person / relationship query を再利用できる。v1 では raw `tree_path` row を response に返さない。

## Projection アルゴリズム

1. active diagram を `diagram_id` で読み込む
2. `diagram.kind == family_tree` でなければ拒否する
3. diagram の active person を読み込む
4. diagram の active relationship を読み込む
5. lineage edge を parent-to-child 方向に正規化する
6. 各 node の `parent_entity_ids` / `child_entity_ids` を作る
7. parent を持たない node から `root_entity_ids` を導出する
8. 出力を deterministic に sort する

推奨 sort order:

- `nodes`: `entity_id`
- `edges`: `relationship_id`
- `root_entity_ids`: 昇順

## なぜ raw `tree_path` row を返さないか

- `tree_path` は direct edge より大きくなりやすい
- 多くの client が必要とするのは直接の親子 edge であり、全 ancestor/descendant pair ではない
- subtree や analytics が必要になれば、明示的な query option として後から追加できる

## テスト計画

- valid family-tree diagram で nodes / edges / root ids を返す
- missing / soft-deleted diagram は `404`
- `correlation` diagram は `400`
- deleted person / relationship を除外する
- 複数 root の forest view を返す
- 認証が必要

## 未解決事項

- richer kinship output をこの endpoint に足すか、別 endpoint にするか
- server-side layout hint を持つか
- descendants / ancestors など subtree endpoint を追加するか
