# RFC 0015: Genealogy Overview API

- 状態: `下書き`
- 最終更新: `2026-04-26`

## 背景

Genealogy Overview は、world 内の複数 genealogy diagram を統合した家系図を返す read API である。

単一 diagram の既存 `GET /api/family-trees/{diagram_id}` は移行前 API として diagram-local projection を返す。一方、overview では world-scoped `entity_id` を primary node とし、複数 diagram に配置された同じ entity を 1 つの人物 node として扱う。

この RFC は Genealogy Overview の API contract と projection rule を定義する。familytree から genealogy への repository-wide 命名変更方針は RFC 0017 で定義する。

## 目標

- world 内の複数 diagram を統合した genealogy graph を返す。
- diagram 設定の `genealogy_overview_enabled` に従い、overview 出力対象 diagram を制御する。
- 同一人物を world-scoped `entity_id` で統合した node として返す。
- relationship provenance を失わず、どの diagram 由来か追跡できるようにする。
- 既存の `KinshipDerivationService` と canonical relationship kind 方針を再利用する。
- 将来の `as_of` projection と両立する。

## 非目標

- World CRUD の詳細。
- person CRUD の詳細。
- 自動 identity matching。
- layout 座標計算。
- ACL / publication scope の最終仕様。

## 提案 API

World の overview を返す endpoint を追加する。`world_id` は path parameter ではなく JSON body で受け取る。

```http
POST /api/genealogy/overview
```

Request:

```json
{
  "world_id": 1,
  "center_entity_id": 10,
  "ancestor_depth": 3,
  "descendant_depth": 2,
  "diagram_ids": [1, 2],
  "as_of": "1995-01-01",
  "include_hidden": false
}
```

初期実装ではこの 1 endpoint に統一する。全体 overview が必要な場合は `center_entity_id`、`ancestor_depth`、`descendant_depth` を省略する。

## Response 形状

推奨 response:

```json
{
  "status": "success",
  "data": {
    "world": {
      "world_id": 1,
      "name": "Hayashi family"
    },
    "diagram_ids": [1, 2],
    "as_of": null,
    "nodes": [
      {
        "entity_id": 10,
        "name": "A",
        "gender": "unknown",
        "birth_date": null,
        "death_date": null,
        "source_diagram_ids": [1, 2]
      }
    ],
    "edges": [
      {
        "from_entity_id": 10,
        "to_entity_id": 11,
        "kind": "parent",
        "source": "explicit",
        "source_relationship_ids": [30, 42],
        "source_diagram_ids": [1, 2]
      }
    ],
    "root_entity_ids": [10],
    "stats": {
      "diagram_count": 2,
      "node_count": 1,
      "edge_count": 1
    }
  }
}
```

Node の primary identifier は `entity_id` とする。`source_diagram_ids` は、その entity がどの diagram に配置されているかを示す provenance である。

## Projection ルール

Overview は次の順序で組み立てる。

1. world を load する。
2. world に属する active `family_tree` diagram を load する。
3. query で `diagram_ids` が指定されていれば world 内に限定して filter する。
4. `genealogy_overview_enabled = true` の diagram のみに filter する。
5. filter 後の diagram が 0 件なら `409 CONFLICT` を返す。
6. 対象 diagram に含まれる active entity と person attributes を load する。
7. 同じ `entity_id` が複数 diagram に含まれる場合、1 node に統合する。
8. 対象 diagram の active canonical relationship を load する。
9. relationship endpoint が world 内 entity であり、対象 diagram に含まれることを確認する。
10. `source_entity_id = target_entity_id` になる relationship は overview edge から除外し、diagnostic として扱う。
11. 同じ entity pair / kind / validity range の edge を統合する。
12. `KinshipDerivationService` に overview 用 graph input を渡す。
13. root、stats、provenance を assemble する。

## Diagram 出力設定

Diagram は `genealogy_overview_enabled` を持つ。

```sql
genealogy_overview_enabled BOOLEAN NOT NULL DEFAULT true
```

意味:

- `true`: Genealogy Overview の統合対象に含める。
- `false`: diagram は編集可能なまま残すが、Genealogy Overview には出力しない。

`diagram_ids` が request で指定された場合も、この flag は必ず適用する。client が明示的に指定した diagram であっても、`genealogy_overview_enabled = false` なら出力対象にしない。

対象 world 内の `family_tree` diagram がすべて disabled の場合、または request の `diagram_ids` がすべて disabled だった場合は error を返す。

推奨 error:

```http
409 CONFLICT
```

```json
{
  "status": "error",
  "error": {
    "code": "NO_VISIBLE_GENEALOGY_DIAGRAMS",
    "message": "No genealogy diagrams are enabled for overview output."
  }
}
```

理由:

- world は存在し、request も構文的には正しい。
- しかし現在の diagram 設定では overview を構築できない。
- `404` だと world や diagram が存在しない状態と区別しにくい。

## Relationship 統合ルール

同じ relationship が複数 diagram で表現されている場合、overview では重複 edge をまとめる。

初期 dedupe key:

```text
source_entity_id
target_entity_id
kind
start_date
end_date
end_reason
```

統合された edge は `source_relationship_ids` と `source_diagram_ids` を配列で持つ。

Conflict がある場合:

- `parent` と `adoptive_parent` は別 edge として残す。
- 同じ pair に複数の tree-edge kind がある場合は、両方返し、将来 conflict warning を追加する。
- `spouse` / `partner` / `cohabitant` は symmetric rule に従い endpoint を正規化する。
- date range が違う relationship は別 edge として扱う。

## Root 検出

Root は overview graph 上の `entity_id` で計算する。

初期ルール:

- tree-edge kind は RFC 0013 と同じく `parent` と `adoptive_parent`。
- incoming tree-edge を持たない node を root とする。
- query で center が指定された場合、root は全体 root ではなく center-relative response metadata として扱ってもよい。

## As-Of との関係

RFC 0014 の as-of rule を world overview にも適用できる。

`as_of` が指定された場合:

- relationship は date range で filter する。
- `birth_date > as_of` の person entity は除外する。
- `death_date < as_of` の person entity は表示する。
- `tree_path` は使わず、request-local closure を構築する。

初期実装では `as_of` を optional とし、未指定時は current overview を返す。

## レイヤリング

- Driver: request parsing と response DTO mapping。
- Usecase: world / diagram / person / relationship loading と orchestration。
- Service: `KinshipDerivationService` が merged graph の kinship derivation を担当する。
- Adapter: world-scoped diagram、entity/person、overview relationship loading の repository を提供する。

`RelationshipService` は stored relationship CRUD の責務を維持し、overview の merge logic は持たない。

## Test Plan

最小 coverage:

- world 内の複数 diagram が 1 overview に含まれる。
- world 外の diagram は含まれない。
- `genealogy_overview_enabled = false` の diagram は overview に含まれない。
- すべての対象 diagram が disabled の場合は `409 CONFLICT` と `NO_VISIBLE_GENEALOGY_DIAGRAMS` を返す。
- 同じ `entity_id` が複数 diagram に含まれても 1 node に統合される。
- source diagram provenance が返る。
- relationship endpoint が world 内 entity として検証される。
- 重複 relationship が 1 edge に統合される。
- deleted world / diagram / entity / person / relationship は除外される。
- `as_of` 指定時に date range filter が適用される。
- `birth_date > as_of` の person entity が除外される。
- root が overview graph 上で計算される。

## 未解決事項

- Conflict warning を response に含めるか。
- source provenance を常に返すか、include flag で制御するか。
- Centered overview の response shape を通常 overview と同一にするか。
- Overview 専用の cache policy をどうするか。
