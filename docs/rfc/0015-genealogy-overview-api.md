# RFC 0015: Genealogy Overview API

- 状態: `下書き`
- 最終更新: `2026-04-26`

## 背景

Genealogy Overview は、world 内の複数 genealogy diagram を統合した家系図を返す read API である。

単一 diagram の `GET /api/genealogy/diagram/{diagram_id}` は diagram-local projection を返す。一方、overview では world-scoped `entity_id` を primary node とし、複数 diagram に登録された同じ entity を 1 つの人物 node として扱う。

この RFC は Genealogy Overview の API contract と projection rule を定義する。familytree から genealogy への repository-wide 命名変更方針は RFC 0017 で定義する。

Relationship scope は RFC 0025 で world-level canonical fact に更新する。RFC 0025 採用後の overview は diagram-local relationship の merge ではなく、world-level relationship と optional diagram entity filter から構築する。

## 目標

- world 内の複数 diagram を統合した genealogy graph を返す。
- diagram 設定の `genealogy_overview_enabled` に従い、overview 出力対象 diagram を制御する。
- 同一人物を world-scoped `entity_id` で統合した node として返す。
- relationship は world-level fact として扱い、diagram filter 適用時は endpoint entity の diagram membership provenance を追跡できるようにする。
- 既存の `KinshipDerivationService` と canonical relationship kind 方針を再利用する。
- 将来の `as_of` projection と両立する。

## 非目標

- World CRUD の詳細。
- person CRUD の詳細。
- 自動 identity matching。
- layout 座標計算。
- ACL / publication scope の最終仕様。

## 提案 API

World の overview を返す endpoint を追加する。`world_id` は path parameter で受け取る。

```http
GET /api/genealogy/world/{world_id}
```

Query:

```http
GET /api/genealogy/world/1?diagram_ids=1,2&center_entity_id=10&ancestor_depth=3&descendant_depth=2&as_of=1995-01-01
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
        "source_entity_id": 10,
        "target_entity_id": 11,
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

Node の primary identifier は `entity_id` とする。`source_diagram_ids` は、その entity がどの diagram に登録されているかを示す provenance である。

## Projection ルール

Overview は次の順序で組み立てる。

1. world を load する。
2. world に属する active `family_tree` diagram を load する。
3. query で `diagram_ids` が指定されていれば world 内に限定して filter する。
4. `genealogy_overview_enabled = true` の diagram のみに filter する。
5. filter 後の diagram が 0 件なら `409 CONFLICT` を返す。
6. 対象 diagram の active `diagram_entity` と entity / person attributes を load する。
7. 同じ `entity_id` が複数 diagram に登録されている場合、1 node に統合する。
8. world-level active canonical relationship を load する。
9. `diagram_ids` filter がある場合は、対象 diagram の visible entity set の内側に source / target がどちらも含まれる relationship だけを overview edge として扱う。
10. `source_entity_id = target_entity_id` になる relationship は overview edge から除外し、diagnostic として扱う。
11. world-level relationship は重複保存されない前提なので、同じ `relationship_id` を primary provenance として扱う。legacy migration 期間に重複 row がある場合だけ dedupe する。
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

## Relationship 表示ルール

RFC 0025 採用後、relationship は world-level の正規 fact である。同じ relationship を複数 diagram に保存して overview で merge する前提は採用しない。

Overview edge の基本 identity は world-level `relationship_id`、または RFC 0024 の `edge_id` とする。

`diagram_ids` query が指定された場合は、relationship provenance ではなく visible entity set を絞る。

```text
visible_entity_ids = active diagram_entity rows in selected diagrams
edge is visible when source_entity_id and target_entity_id are both in visible_entity_ids
```

`source_diagram_ids` は relationship の所有元ではなく、endpoint entity がどの selected diagram に含まれていたかを示す membership provenance として扱う。

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
- Service: `KinshipDerivationService` が world graph の kinship derivation を担当する。
- Adapter: world-scoped diagram、entity/person、overview relationship loading の repository を提供する。

`RelationshipService` は world-level stored relationship CRUD の責務を維持し、overview-specific filtering / assembly は持たない。

## Test Plan

最小 coverage:

- world 内の複数 diagram が 1 overview に含まれる。
- world 外の diagram は含まれない。
- `genealogy_overview_enabled = false` の diagram は overview に含まれない。
- すべての対象 diagram が disabled の場合は `409 CONFLICT` と `NO_VISIBLE_GENEALOGY_DIAGRAMS` を返す。
- 同じ `entity_id` が複数 diagram に登録されていても 1 node に統合される。
- source diagram provenance が返る。
- relationship endpoint が world 内 entity として検証される。
- world-level relationship が overview edge として返る。
- `diagram_ids` filter が指定された場合、selected diagram の visible entity set 外の relationship は除外される。
- deleted world / diagram / entity / person / relationship は除外される。
- `as_of` 指定時に date range filter が適用される。
- `birth_date > as_of` の person entity が除外される。
- root が overview graph 上で計算される。

## 未解決事項

- Conflict warning を response に含めるか。
- source provenance を常に返すか、include flag で制御するか。
- Centered overview の response shape を通常 overview と同一にするか。
- Overview 専用の cache policy をどうするか。
