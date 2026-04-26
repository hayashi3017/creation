# RFC 0014: As-Of Family Tree Projection

- 状態: `下書き`
- 最終更新: `2026-04-25`

## 背景

Family tree relationships は時点に依存する。

例:

- spouse relationship は 1990 年には active だが 2005 年には終了している可能性がある。
- cohabitant や partner relationship は限られた期間にだけ適用される可能性がある。
- adoptive parent や step-parent relationship は出生後に始まる可能性がある。
- person は要求された時点より後に生まれている可能性がある。
- person は要求された時点より前に死亡していても、historical family tree には表示され続けるべき場合がある。

現在の schema には既に `relationship.start_date` と `relationship.end_date` があるが、現在の read path は要求された historical date なしに relationships を active rows として扱っている。

そのため `GET /api/family-trees/{diagram_id}` は現在保存されている graph を表示できるが、次の質問には答えられない。

- "1995-01-01 時点でこの family tree はどう見えていたか"
- "その時点で spouse とみなされていたのは誰か"
- "どの parent / child / sibling / ancestor relationships がその時点で有効だったか"
- "その時点で center person から見た kinship は何か"

この RFC は、要求された時点で explicit relationships と derived kinship を評価する as-of read projection を提案する。

## 目標

- 特定の historical date に対する family-tree read を support する。
- 全 storage を event sourcing に書き換えず、まず read-side の temporal behavior として扱う。
- relationship `start_date` と `end_date` が explicit / derived kinship に与える影響を定義する。
- as-of read で `tree_path` をどう使うべきか明確にする。
- date が指定されない場合の現在の read behavior と互換性を保つ。
- RFC 0012 と RFC 0013 に揃える。

## 非目標

- full event sourcing の実装。
- すべての relationship row の過去 version を保存すること。
- UI timeline controls の最終決定。
- 同じ step で write API を変更すること。
- 初期 rollout で `tree_path` を完全な temporal history table にすること。
- 単純な date range を超える曖昧または概算の historical date を解くこと。

## 実現可能性

現在の方向性で実現可能だが、実装では 2 つの概念を区別する必要がある。

- current-state closure: active canonical lineage rows 用に維持される `tree_path`
- as-of projection: 要求 date を含む validity range を持つ rows から構築される read model

現在の `tree_path` table は validity range を持たないため、単独では historical closure query に答えられない。

初期実装では、as-of lineage closure は filter 済み canonical relationship rows から read time に導出する。これにより既存 `tree_path` の意味を壊さず、access pattern が証明される前に temporal closure table を導入することも避けられる。

## 提案 API

Family-tree read endpoint に任意 query parameter を追加する。

```http
GET /api/family-trees/{diagram_id}?as_of=1995-01-01
```

Semantics:

- `as_of` 省略時は現在の挙動を維持する。
- `as_of` 指定時は、その date に有効な projection を返す。
- `as_of` は ISO `YYYY-MM-DD` date とする。
- invalid date は `400 BAD_REQUEST` を返す。

推奨 response metadata:

```json
{
  "status": "success",
  "data": {
    "as_of": "1995-01-01",
    "temporal_mode": "as_of",
    "diagram": {},
    "root_entity_ids": [],
    "nodes": [],
    "edges": [],
    "stats": {}
  }
}
```

API churn を最小にしたい場合、初期 response は `temporal_mode` を省略してもよい。ただし `as_of` を返すと cache key と client state が明示的になるため有用である。

## 時点有効性ルール

Date range は inclusive に扱う。

Relationship は次を満たすとき `as_of` で有効である。

```text
(start_date IS NULL OR start_date <= as_of)
AND
(end_date IS NULL OR as_of <= end_date)
AND
deleted_at IS NULL
```

解釈:

- `start_date = NULL`: relationship は unknown beginning から有効である。
- `end_date = NULL`: relationship は start 後も有効である。
- `start_date = end_date`: relationship はその 1 日だけ有効である。
- `deleted_at`: administrative deletion であり、historical validity の一部ではない。

`deleted_at` は引き続き「この row は通常 read に参加しない」を意味する。削除済み fact の historical audit が必要な場合は、この projection の意味を変えるのではなく、別 archive または event log で扱う。

## Person 可視性ルール

初期実装では、historical family tree から deceased persons を隠さない。

推奨 baseline:

- diagram 内の active, non-deleted person/entity rows を含める。
- `person.birth_date` が既知で `birth_date > as_of` の場合、その person を as-of projection から除外する。
- `person.death_date` が既知で `death_date < as_of` の場合、その person は表示し続け、既存 field で deceased として示す。

理由:

- family tree は通常、既に亡くなった ancestors を含む。
- historical ancestry には deceased persons の表示が必要である。
- まだ生まれていない人を除外することで、出生前の不可能な edge を防ぐ。

未決ポリシー:

- 将来「date 時点の living household」view が必要な場合、family-tree ancestry view とは別の projection mode とする。

## Relationship Projection ルール

As-of read では次の順序で処理する。

1. diagram 内の active persons/entities を load する。
2. birth-date visibility で persons を filter する。
3. `as_of` で有効な canonical relationship rows を load する。
4. endpoint が as-of person set に含まれない relationships を filter する。
5. explicit rows を canonical graph input に normalize する。
6. filter 済み graph から lineage closure と kinship を derive する。
7. response を assemble する。

例:

- `start_date = 1980-01-01`, `end_date = 2000-12-31` の `spouse(A, B)` は `as_of=1995-01-01` で present である。
- 同じ spouse row は `as_of=2005-01-01` では absent である。
- date がない `parent(A, B)` は endpoint が visible である限り valid とみなす。
- `adoptive_parent(A, B)` は `start_date` 以降のみ lineage に寄与する。

## Tree Path 戦略

As-of closure の source of truth として現在の `tree_path` table を使わない。

理由:

- `tree_path` には `ancestor_id`, `descendant_id`, `depth` しかない。
- `valid_from` や `valid_to` がない。
- 現在 active な lineage rows から維持されている。
- historical query では現在 graph と異なる relationship graph の closure が必要になる可能性がある。

初期 as-of strategy:

- `as_of` で有効な relationship rows を load する。
- RFC 0013 の tree-edge kinds、初期は `parent` と `adoptive_parent` のみを残す。
- request 内で ancestor / descendant closure を memory 上に構築する。
- as-of graph に対して cycle を detect する。
- request-local closure を derived kinship に使う。

Diagram-scoped family tree は request-local graph derivation で扱える程度に小さい想定のため、初期実装としては許容できる。

性能問題が出た場合は、後で dedicated temporal closure table を追加する。

将来 table 案:

```sql
CREATE TABLE temporal_tree_path (
  diagram_id BIGINT NOT NULL,
  ancestor_id BIGINT NOT NULL,
  descendant_id BIGINT NOT NULL,
  depth INT NOT NULL,
  valid_from DATE,
  valid_to DATE,
  PRIMARY KEY (diagram_id, ancestor_id, descendant_id, valid_from)
);
```

Temporal closure の正しい維持は current-state closure よりかなり複雑なため、明確な必要性が出るまでこの future table は導入しない。

## 親族関係導出の扱い

RFC 0012 の `KinshipDerivationService` は persistence を所有せず、temporal input を受け取れるようにする。

推奨 input shape:

```rust
struct DeriveKinshipInput {
    active_person_ids: Vec<usize>,
    explicit_relationships: Vec<Relationship>,
    lineage_paths: Vec<KinshipLineagePath>,
    as_of: Option<NaiveDate>,
}
```

Usecase は loading と temporal filtering を所有する。

Derivation service は graph interpretation のみを所有する。

- `child` のような inverse relationships
- spouse / partner のような symmetric expansion
- shared as-of parents からの sibling classification
- as-of lineage closure からの ancestor / descendant
- uncle/aunt や cousin のような higher-order kinship

これにより RFC 0012 の責務分割を維持する。

## Relationship Lifecycle と End Reasons

RFC 0013 は、終了した spouse relationships を `divorced_spouse` kind ではなく `end_date` と `end_reason` で表すことを推奨する。

As-of projection は validity に `end_reason` ではなく `end_date` を使う。

例:

- `spouse`, `end_date = 2000-01-01`, `end_reason = divorce`
- `1999-12-31` では spouse として valid
- `2001-01-01` では spouse として absent

UI が end date 後に "former spouse" を表示したい場合、それは as-of active relation projection ではなく別の historical relationship view とする。

## Query と Repository の変更

`as_of` を持つ read schema を追加する。

```rust
struct GetFamilyTreeSchema {
    diagram_id: usize,
    as_of: Option<NaiveDate>,
}
```

Relationship repository は date-filtered read path を support する。

```rust
struct GetRelationshipsAtDateSchema {
    diagram_id: usize,
    as_of: NaiveDate,
}
```

推奨 SQL predicate:

```sql
WHERE
  r.diagram_id = $1
  AND r.deleted_at IS NULL
  AND (r.start_date IS NULL OR r.start_date <= $2)
  AND (r.end_date IS NULL OR $2 <= r.end_date)
```

Person loading は初期状態では現在の person list query を再利用し、usecase で `birth_date` filter を行ってよい。非効率になった場合は、同じ visibility rule を SQL で適用する repository query を追加する。

## Current Mode と Historical Mode

Lineage closure を得る時点で 2 つの code path を分ける。

- current mode: 維持済み `tree_path` を使ってよい。
- as-of mode: date-filtered relationships から request-local lineage closure を構築する。

As-of read 中に `tree_path` を mutate しない。Historical date 用に `tree_path` を一時 rebuild しようとしない。

維持済み `tree_path` は、現在 active graph の write-side consistency optimization のままである。

## 段階的な展開

### Phase 1

`GET /api/family-trees/{diagram_id}` に `as_of` を追加する。

この phase では次を行う。

- explicit relationships を date で filter する。
- public response shape はほぼ変更しない。
- as-of relationship set から direct edges を derive する。
- as-of edges から roots と adjacency を構築する。
- richer derived kinship はまだ expose しない。

### Phase 2

`KinshipDerivationService` が as-of input を受け取れるように接続する。

この phase では次を行う。

- as-of lineage に対して sibling / ancestor / descendant を derive する。
- source metadata を内部的に含める。
- 別 response contract が accepted されるまで public response を安定させる。

### Phase 3

必要に応じて richer temporal kinship endpoint を追加する。

```http
GET /api/family-trees/{diagram_id}/kinships?as_of=1995-01-01&center_entity_id=10
```

この endpoint は現在の family-tree projection を過負荷にせず、richer labels と center-person-relative kinship を expose できる。

### Phase 4

Profiling により request-local derivation が遅いと分かった場合にのみ、temporal closure caching を検討する。

## 利点

- event sourcing なしに historical family-tree rendering を support できる。
- 既存の `start_date` と `end_date` を再利用できる。
- `tree_path` semantics を clean に保てる。
- RFC 0013 の canonical relationship storage と整合する。
- UI に timeline control 用の明確な `as_of` parameter を提供できる。
- derived kinship を特定時点で一貫して評価できる。

## 欠点

- as-of read は request ごとに closure を derive するため current-state read より遅くなる可能性がある。
- unknown start / end を持つ date range は依然として意味的に曖昧である。
- date がない既存 rows は常に valid として扱われるため、historical accuracy が低い可能性がある。
- archival support が追加されるまで、deleted rows は historical projection に使えない。
- richer temporal semantics には将来的に event history や versioned facts が必要になる可能性がある。

## テスト計画

最小 coverage:

- `as_of` 省略時、現在の family-tree response behavior が維持される。
- invalid `as_of` は `400 BAD_REQUEST` を返す。
- `start_date` より前の relationship は除外される。
- `start_date` 当日の relationship は含まれる。
- `end_date` 当日の relationship は含まれる。
- `end_date` より後の relationship は除外される。
- `birth_date > as_of` の person は除外される。
- deceased person は `death_date` 後も visible のままである。
- roots と adjacency が as-of graph から再計算される。
- as-of projection は `tree_path` を mutate しない。
- cycle detection は as-of lineage graph に対して走る。

## 未解決事項

- `deleted_at` rows は audit mode を通じて historical as-of read に参加すべきか。
- Historical fact は year-only や month-only の date precision を support すべきか。
- Ancestry、household、legal family state に別々の projection mode を用意すべきか。
- Former relationships は `end_date` 後に historical annotation として表示すべきか、active as-of relation output から除外すべきか。
- Temporal closure caching は diagram-wide、date ごと、または profiling で必要になるまで導入しない方針のどれにすべきか。
