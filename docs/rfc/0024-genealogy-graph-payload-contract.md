# RFC 0024: Genealogy Graph Payload Contract

- 状態: `下書き`
- 最終更新: `2026-05-09`

## 背景

`GET /api/genealogy/diagram/{diagram_id}` と `GET /api/genealogy/world/{world_id}` は、どちらも frontend から見ると「家系 graph を描画する read API」である。

しかし現在の response は diagram-local graph と world overview graph で top-level context の形が異なる。さらに、centered view 用の `center_entity_id` / `ancestor_depth` / `descendant_depth` は request contract に存在する一方で、response 側には center-relative な metadata がまだ明示されていない。

Frontend で同じ graph renderer を diagram / world の両方に使うには、backend 側で共通 graph payload を明示した方がよい。

この RFC は RFC 0007、RFC 0014、RFC 0015、RFC 0012 を補完し、genealogy read API の共通 response contract を定義する。

## 目標

- diagram / world の genealogy read endpoint が同じ `GenealogyGraphPayload` を返せるようにする。
- response に graph の文脈を表す `context` を追加する。
- centered genealogy read のために `center_entity_id` と node-level center metadata を明示する。
- world overview の統合 edge に安定した public id を持たせる。
- edge の裏付けの強さを frontend が表現できるようにする。
- 既存の source provenance を失わない。

## 非目標

- layout 座標や renderer 固有の配置 hint を決めること。
- localized kinship label を backend contract に固定すること。
- derived relationship を `relationship` table に永続化すること。
- identity matching や conflict resolution の最終仕様を決めること。
- ACL / publication scope の最終仕様を決めること。

## 提案

`GET /api/genealogy/diagram/{diagram_id}` と `GET /api/genealogy/world/{world_id}` の `data` を、共通の graph payload に置き換える。

これは破壊的変更として扱う。既存の `data.diagram`、`data.world`、diagram-only の `relationship_id` primary identity、diagram-only stats は廃止し、`context`、`edge_id`、共通 stats を authoritative contract にする。

推奨 TypeScript shape:

```ts
type GenealogyGraphPayload = {
  as_of: string | null;
  context: GenealogyGraphContextPayload;
  center_entity_id: number | null;
  root_entity_ids: number[];
  nodes: GenealogyGraphNodePayload[];
  edges: GenealogyGraphEdgePayload[];
  stats: GenealogyGraphStatsPayload;
};

type GenealogyGraphContextPayload =
  | {
      kind: "diagram";
      diagram_id: number;
      world_id: number;
      diagram_ids: number[];
      name: string;
    }
  | {
      kind: "world";
      world_id: number;
      diagram_ids: number[];
      name: string;
    };
```

HTTP response の envelope は既存方針と同じく維持する。

```json
{
  "status": "success",
  "data": {
    "as_of": null,
    "context": {
      "kind": "world",
      "world_id": 1,
      "diagram_ids": [10, 11],
      "name": "Hayashi family"
    },
    "center_entity_id": 20,
    "root_entity_ids": [1],
    "nodes": [],
    "edges": [],
    "stats": {
      "diagram_count": 2,
      "node_count": 0,
      "edge_count": 0,
      "root_count": 0
    }
  }
}
```

## Context 契約

`context` は graph がどの scope から作られたかを表す。

Diagram graph:

```json
{
  "kind": "diagram",
  "diagram_id": 10,
  "world_id": 1,
  "diagram_ids": [10],
  "name": "Main family tree"
}
```

World graph:

```json
{
  "kind": "world",
  "world_id": 1,
  "diagram_ids": [10, 11],
  "name": "Hayashi family"
}
```

Rules:

- `kind = "diagram"` の場合、`diagram_id` は必須。
- `kind = "world"` の場合、`world_id` は必須。
- `world_id` は diagram / world のどちらでも必須。認可、cache key、frontend state partitioning の基準を揃えるためである。
- `diagram_ids` は response に含まれる entity membership の対象 diagram を deterministic に昇順で返す。RFC 0025 採用後、relationship は world-level fact であり diagram に所有されない。
- diagram response の `diagram_ids` は `[diagram_id]` とする。
- `name` は graph scope の表示名であり、diagram graph では diagram 名、world graph では world 名を返す。

## Node 契約

Node は既存の person payload と provenance を維持し、center-relative metadata を追加する。

```ts
type GenealogyGraphNodePayload = {
  entity_id: number;
  name: string;
  description: string | null;
  gender: "male" | "female" | "other" | "unknown" | null;
  birth_date: string | null;
  death_date: string | null;
  birthplace: string | null;
  residence: string | null;
  photo_url: string | null;
  source_diagram_ids: number[];
  parent_entity_ids: number[];
  child_entity_ids: number[];
  is_root: boolean;
  relation_to_center: GenealogyRelationToCenter | null;
  relation_path_to_center: GenealogyRelationPathStepPayload[] | null;
  generation_offset_from_center: number | null;
};
```

`relation_to_center` は localized label ではなく、machine-readable な semantic value とする。

初期 enum:

```ts
type GenealogyRelationToCenter =
  | "self"
  | "parent"
  | "child"
  | "ancestor"
  | "descendant"
  | "sibling"
  | "spouse"
  | "partner"
  | "cohabitant"
  | "uncle_or_aunt"
  | "nibling"
  | "cousin"
  | "in_law"
  | "step_parent"
  | "step_child"
  | "relative"
  | "unrelated"
  | "unknown";
```

`relation_to_center` は「center から見た graph 上の構造的位置」を返す。localized display label や性別・年齢で細分化された続柄名は返さない。

Backend が返すべきもの:

- graph traversal から安定して計算できる構造 category。
- direct relation: `parent`、`child`、`sibling`、`spouse`、`partner`、`cohabitant`。
- lineage relation: `ancestor`、`descendant`。
- higher-order relation: `uncle_or_aunt`、`nibling`、`cousin`、`in_law`。
- step relation の入口: `step_parent`、`step_child`。
- 分類できるが詳細 category をまだ持たない場合の fallback: `relative`。

Frontend が決めるべきもの:

- localized label。例: 父、母、兄、弟、祖父、祖母、伯父、叔父、義兄、義弟。
- gender / age / locale で決まる表示語。例: `parent + male = father`、`sibling + male + older = older_brother`。
- product-specific な親しみ表現。例: 家族、親戚、身内。

`father`、`mother`、`son`、`daughter`、`grandfather`、`grandmother`、`older_brother`、`younger_sister` のような表示派生語は `relation_to_center` に入れない。

理由:

- enum が locale-specific な表示辞書になり、API contract が肥大化する。
- gender や birth_date の欠損、non-binary gender、年齢不明の扱いを backend enum に固定してしまう。
- `generation_offset_from_center` と node attributes で十分に表現できる。

例:

```json
{
  "relation_to_center": "ancestor",
  "generation_offset_from_center": -2
}
```

この node の `gender` が `male` なら frontend は「祖父」と表示できる。`gender` が `female` なら「祖母」、不明なら「祖父母」または neutral label を表示できる。

`adoptive_parent`、`biological_parent`、`legal_parent` のような relation nature は、`relation_to_center` ではなく `relation_path_to_center[].kind` または edge metadata で表す。

例:

```json
{
  "relation_to_center": "parent",
  "relation_path_to_center": [
    {
      "from_entity_id": 10,
      "to_entity_id": 20,
      "edge_id": "diagram:1:edge:abc",
      "kind": "adoptive_parent",
      "direction": "reverse"
    }
  ],
  "generation_offset_from_center": -1
}
```

この分割により、`relation_to_center` は構造 category、`relation_path_to_center[].kind` は関係の性質、frontend は表示語彙という責務に分けられる。

`relation_path_to_center` は center から対象 node へ到達する代表 path を返す。

```ts
type GenealogyRelationPathStepPayload = {
  from_entity_id: number;
  to_entity_id: number;
  edge_id: string;
  kind: string;
  direction: "forward" | "reverse";
};
```

Rules:

- `center_entity_id` が指定されない場合、`relation_to_center`、`relation_path_to_center`、`generation_offset_from_center` は `null` にする。
- center node は `relation_to_center = "self"`、`relation_path_to_center = []`、`generation_offset_from_center = 0`。
- 直系 ancestor は `generation_offset_from_center < 0` とする。例: parent は `-1`、grandparent は `-2`。
- 直系 descendant は `generation_offset_from_center > 0` とする。例: child は `1`、grandchild は `2`。
- sibling / spouse / partner / cohabitant は同世代として `0` を返してよい。ただし lineage distance が定義できない場合は `null` でもよい。
- uncle / aunt、nibling、cousin、in-law の詳細分類は `relation_to_center` の coarse category と path から frontend が表示する。
- center から同じ node へ複数 path がある場合、初期実装では最短 path を返す。同距離の場合は edge id / entity id の deterministic sort で選ぶ。
- `relation_to_center` は core API では英語の stable token に固定し、表示文言は frontend が locale ごとに決める。
- `parent_entity_ids` / `child_entity_ids` は node-local adjacency cache として残す。renderer が最初の描画で lineage adjacency を再計算しなくて済むためである。
- `source_diagram_ids` は relationship の所有元ではなく、node / endpoint entity がどの diagram membership から見えているかを示す provenance として扱う。RFC 0025 採用後、relationship 自体は world-level fact である。

## Edge 契約

Edge は diagram graph / world graph の両方で安定 id を持つ。

```ts
type GenealogyGraphEdgePayload = {
  edge_id: string;
  source_entity_id: number;
  target_entity_id: number;
  kind: string;
  source: "explicit" | "derived" | "suggested";
  source_confidence: "confirmed" | "inferred" | "conflicting" | "unknown";
  start_date: string | null;
  end_date: string | null;
  end_reason: string | null;
  notes: string | null;
  source_relationship_ids: number[];
  source_diagram_ids: number[];
};
```

`certainty` ではなく `source_confidence` を採用する。

理由:

- `certainty` は事実そのものの真偽を backend が断定する印象が強い。
- `source_confidence` は「response に含めた edge がどの程度の裏付けを持つか」を表し、explicit / derived / conflicting provenance と相性がよい。

初期 semantics:

- `confirmed`: 1 つ以上の active explicit relationship row に直接裏付けられる。
- `inferred`: explicit row から read-time に導出されたが、保存済み relationship row はない。
- `conflicting`: graph 内に同じ対象をめぐる矛盾候補がある。edge は返すが UI は注意表示できる。
- `unknown`: legacy import や不完全 source など、裏付けが分類できない。

`relationship_id` は edge の primary field として返さない。diagram graph でも `source_relationship_ids = [relationship_id]` として表現し、frontend は常に `edge_id` を key にする。

`source_relationship_ids` / `source_diagram_ids` は常に必須である。derived / suggested edge で直接の保存 row がない場合、`source_relationship_ids` は空配列にする。

## Edge の安定 ID

RFC 0025 採用後、relationship は world-level の正規 fact である。diagram graph / world graph のどちらでも `edge_id` を public stable id とし、frontend は `relationship_id` を直接 key にしない。

この RFC では diagram / world の両方で `edge_id: string` を public stable id とする。

Diagram graph:

```text
diagram:{diagram_id}:edge:{hash}
```

World graph:

```text
world:{world_id}:edge:{hash}
```

`hash` の input は次の canonical key を deterministic に serialize したものとする。

```text
source_entity_id
target_entity_id
kind
source
start_date
end_date
end_reason
sorted(source_relationship_ids)
```

Rules:

- hash algorithm は implementation detail だが、同じ input から同じ output になる必要がある。
- `source_relationship_ids` は昇順に sort してから hash input に入れる。
- `source_diagram_ids` は hash input に入れない。diagram membership filter が変わっても、同じ world-level relationship の edge identity は変わらないようにする。
- source relationship が増減した場合は別 edge とみなし、`edge_id` が変わってよい。
- diagram graph でも `relationship:{relationship_id}` は使わない。explicit / derived / suggested を同じ identity rule で扱うためである。
- 同じ world-level relationship edge が再取得時に同じ `edge_id` になることを API test で保証する。
- Client は world overview edge の identity として `relationship_id` ではなく `edge_id` を使う。

## Stats 契約

Stats は graph payload 全体で同じ field を返す。

```ts
type GenealogyGraphStatsPayload = {
  node_count: number;
  edge_count: number;
  root_count: number;
  diagram_count: number;
};
```

Rules:

- `node_count` は `nodes.length` と一致させる。
- `edge_count` は `edges.length` と一致させる。
- `root_count` は `root_entity_ids.length` と一致させる。
- `diagram_count` は `context.diagram_ids.length` と一致させる。

`person_count` は廃止する。genealogy graph payload は person 以外の entity kind を将来含められる read model なので、graph cardinality は `node_count` で表す。

## Breaking Change Policy

この RFC は互換維持よりも response contract の一貫性を優先する。

廃止する response field:

- `data.diagram`
- `data.world`
- `data.person_count`
- `edges[].relationship_id`

置き換え:

- `data.diagram` / `data.world` -> `data.context`
- `edges[].relationship_id` -> `edges[].edge_id` and `edges[].source_relationship_ids`
- `person_count` -> `node_count`

理由:

- diagram / world の分岐を frontend の graph renderer から消せる。
- world graph edge と diagram graph edge の identity model を統一できる。
- optional legacy field が増えるほど OpenAPI と client type が曖昧になる。
- `context` を authoritative にすることで cache key、breadcrumb、scope switcher の実装が単純になる。

## Projection Rules

Centered request の response assembly は次の順序で行う。

1. RFC 0007 / RFC 0015 のルールに従って graph scope を load する。
2. `as_of` が指定されていれば RFC 0014 の projection rule を適用する。
3. explicit relationship を canonical graph edge に正規化する。
4. world graph では relationship を RFC 0015 の dedupe key で統合する。
5. 各 edge に `edge_id` と `source_confidence` を付与する。
6. root を計算する。
7. `center_entity_id` が指定されていれば、center からの shortest path を計算する。
8. 各 node に `relation_to_center`、`relation_path_to_center`、`generation_offset_from_center` を付与する。
9. `nodes`、`edges`、`root_entity_ids`、`diagram_ids` を deterministic に sort する。

`ancestor_depth` / `descendant_depth` が指定された場合、初期実装では response graph を center-relative subgraph に絞る。depth filtering の exact traversal は lineage edge を優先し、spouse / partner など非 lineage edge を含めるかは別 RFC または本 RFC の更新で詰める。

## Error Semantics

既存 endpoint の status mapping を維持する。

- invalid id / invalid query -> `400 BAD_REQUEST`
- missing or invisible diagram / world -> `404 NOT_FOUND`
- visible world だが overview 対象 diagram がない -> `409 CONFLICT` + `NO_VISIBLE_GENEALOGY_DIAGRAMS`
- `center_entity_id` が graph scope に含まれない -> `400 BAD_REQUEST`

`center_entity_id` が別 user / 別 world の entity である場合も、resource existence を漏らさないため `400 BAD_REQUEST` または `404 NOT_FOUND` のどちらにするかは RFC 0018 の authorization policy に従う。

## Implementation Plan

1. `creation-service` に `GenealogyGraphPayload` 相当の共通 graph model を追加する。
2. `GenealogyDiagramGraph` と `GenealogyOverview` を public response model としては廃止し、usecase output を共通 graph model に統一する。
3. diagram / overview usecase の assembly を共通 helper に寄せる。
4. world edge の `edge_id` 生成 helper を追加する。
5. center-relative metadata を計算する read-side helper を追加する。
6. driver response / OpenAPI schema を `GenealogyGraphResponse` に統一する。
7. API tests を新 contract に置き換える。

## Test Plan

- diagram graph response に `context.kind = "diagram"` と `context.diagram_id` が含まれる。
- world graph response に `context.kind = "world"`、`context.world_id`、`context.diagram_ids` が含まれる。
- diagram / world response のどちらにも `data.diagram` / `data.world` が含まれない。
- `center_entity_id` 未指定時、node の center metadata は `null`。
- center node は `self`、empty path、generation offset `0`。
- parent / child / ancestor / descendant の generation offset が符号付きで返る。
- sibling / uncle_or_aunt / nibling / cousin / in_law が graph path から coarse category として返る。
- father / mother / older_brother などの display label は response enum に含まれない。
- `relation_path_to_center` が deterministic に返る。
- diagram edge の `edge_id` が `diagram:{diagram_id}:edge:{hash}` 形式で返る。
- diagram edge の `source_relationship_ids` が元 relationship id を含む。
- edge response に top-level `relationship_id` が含まれない。
- world graph edge の `edge_id` が再取得しても変わらない。
- explicit edge は `source_confidence = "confirmed"`。
- derived edge を返す場合は `source = "derived"` かつ `source_confidence = "inferred"`。
- conflict edge を返す場合は `source_confidence = "conflicting"`。
- `stats.node_count` / `edge_count` / `root_count` が配列長と一致する。
- `stats.diagram_count` が `context.diagram_ids.length` と一致する。

## 未解決事項

- cousin removed、parallel/cross cousin、older/younger uncle などの細分類を frontend-only 表示に留めるか、将来 `relation_detail_to_center` のような別 field を追加するか。
- sibling / cousin / in-law など higher-order kinship の path 選択 rule。複数 path がある場合の priority をどこまで domain policy にするか。
- `ancestor_depth` / `descendant_depth` が spouse / sibling edge を含むか。
- conflict warning を edge field だけで足りるか、top-level diagnostics として返すか。
- world edge の hash algorithm を OpenAPI description に明記するか。
