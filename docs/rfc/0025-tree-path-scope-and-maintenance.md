# RFC 0025: World-Scoped Relationship And Tree Path

- 状態: `下書き`
- 最終更新: `2026-05-09`

## 背景

RFC 0008 では、`entity` は world に所属し、diagram は world 内 entity を選択する表示・編集単位として定義した。

この前提に立つなら、relationship も diagram-local fact ではなく world-level の正規 fact として扱う方が一貫する。

```text
entity:
  world-level canonical person/entity

relationship:
  world-level canonical explicit fact

diagram:
  world entity の表示範囲
  relationship の所有者ではない
```

この RFC は、`relationship.diagram_id` を廃止して `relationship.world_id` に寄せ、`tree_path` も `world_id` を持つ current-state lineage reachability cache として定義する。

## 目標

- relationship を world-level の正規 fact として扱う。
- diagram を relationship 所有者ではなく、entity 表示範囲として扱う。
- diagram に含まれる entity 同士の active relationship は、kind filter / `as_of` / display settings に従って自動表示する。
- `diagram_relationship` のような中間テーブルを導入しない。
- `tree_path` を world-scoped な current-state lineage reachability cache として定義する。
- relationship mutation と `tree_path` rebuild の transaction boundary を定義する。
- 並行 relationship update で cycle や stale closure が commit されないようにする。
- as-of projection では current `tree_path` を historical source of truth として使わないことを再確認する。

## 非目標

- temporal `tree_path` / historical closure cache を導入すること。
- all paths を永続化すること。
- localized kinship label を定義すること。
- `KinshipDerivationService` の全 derived relation algorithm をこの RFC で実装詳細まで固定すること。
- biological / legal / foster parentage subtype の最終 schema を決めること。
- diagram ごとに別の relationship 仮説を保存すること。

## 決定

`relationship` と `tree_path` は world-scoped とする。

```text
relationship:
  world-scoped explicit canonical fact

tree_path:
  world-scoped current-state lineage reachability cache
  relationship から再構築可能
  source of truth ではない

diagram:
  entity 表示範囲
  relationship は所有しない
```

Diagram graph の edge は次の条件で決まる。

```text
relationship.world_id = diagram.world_id
source_entity_id と target_entity_id がどちらも active diagram_entity
relationship が active
kind filter / as_of / display settings を満たす
```

このため、`diagram_relationship` は導入しない。

理由:

- relationship は world-level の正規 fact であり、diagram ごとの重複保存を避けたい。
- diagram は人物の表示範囲を定義すれば十分である。
- relationship の表示切り替えは diagram membership ではなく、kind filter / `as_of` / display settings で扱う方が直交する。
- world overview で relationship merge や diagram provenance dedupe を行う必要が減る。

## Schema

`relationship` は `diagram_id` ではなく `world_id` を持つ。

推奨 schema:

```sql
CREATE TABLE relationship (
    relationship_id BIGSERIAL PRIMARY KEY,
    world_id BIGINT NOT NULL REFERENCES world(world_id) ON DELETE CASCADE,
    source_entity_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    target_entity_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    kind relationship_kind NOT NULL,
    start_date DATE,
    end_date DATE,
    end_reason VARCHAR(32),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT chk_relationship_no_active_self_relation
        CHECK (deleted_at IS NOT NULL OR source_entity_id <> target_entity_id)
);
```

`tree_path` も `world_id` を持つ。

```sql
CREATE TABLE tree_path (
    world_id BIGINT NOT NULL REFERENCES world(world_id) ON DELETE CASCADE,
    ancestor_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    descendant_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    depth INT NOT NULL CHECK (depth >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (world_id, ancestor_id, descendant_id),
    CONSTRAINT chk_tree_path_self_depth CHECK (
        (ancestor_id = descendant_id AND depth = 0)
        OR
        (ancestor_id <> descendant_id AND depth > 0)
    )
);
```

Indexes:

```sql
CREATE INDEX idx_relationship_world
ON relationship(world_id);

CREATE INDEX idx_relationship_source_entity
ON relationship(source_entity_id);

CREATE INDEX idx_relationship_target_entity
ON relationship(target_entity_id);

CREATE INDEX idx_tree_path_world_ancestor
ON tree_path(world_id, ancestor_id);

CREATE INDEX idx_tree_path_world_descendant
ON tree_path(world_id, descendant_id);

CREATE INDEX idx_tree_path_world_depth
ON tree_path(world_id, depth);
```

PostgreSQL の通常 FK だけでは、`relationship.world_id = source_entity.world_id = target_entity.world_id` を直接保証しづらい。実装時は次のどちらかを選ぶ。

Option A: composite FK 用の unique constraint を追加する。

```sql
ALTER TABLE entity
ADD CONSTRAINT uq_entity_world_entity
UNIQUE (world_id, entity_id);

ALTER TABLE relationship
ADD CONSTRAINT fk_relationship_source_world_entity
FOREIGN KEY (world_id, source_entity_id)
REFERENCES entity(world_id, entity_id);

ALTER TABLE relationship
ADD CONSTRAINT fk_relationship_target_world_entity
FOREIGN KEY (world_id, target_entity_id)
REFERENCES entity(world_id, entity_id);

ALTER TABLE tree_path
ADD CONSTRAINT fk_tree_path_ancestor_world_entity
FOREIGN KEY (world_id, ancestor_id)
REFERENCES entity(world_id, entity_id);

ALTER TABLE tree_path
ADD CONSTRAINT fk_tree_path_descendant_world_entity
FOREIGN KEY (world_id, descendant_id)
REFERENCES entity(world_id, entity_id);
```

Option B: FK は `entity(entity_id)` のままにし、service layer で world consistency を必ず検証する。

推奨は Option A である。DB が world boundary を guardrail として持てるため、relationship / tree_path の cross-world 混入を防ぎやすい。

`deleted_at IS NULL` は通常の FK では保証できないため、active scope は service layer でも検証する。

## Diagram Projection

Diagram は relationship を所有しない。

Diagram graph を返す read path は次で組み立てる。

1. active diagram を load する。
2. diagram の active `diagram_entity` から visible entity set を作る。
3. `relationship.world_id = diagram.world_id` の active relationship を load する。
4. source / target がどちらも visible entity set に含まれる relationship だけを edge として表示する。
5. kind filter / `as_of` / display settings を適用する。
6. root / adjacency / center metadata を response scope 内で計算する。

重要:

- diagram に entity A と entity B が含まれていれば、A-B 間の active relationship は自動表示対象である。
- relationship を diagram ごとに明示 include/exclude する中間テーブルは持たない。
- 特定 kind を隠したい場合は diagram display settings で filter する。
- 時点で切り替えたい場合は `as_of` を使う。

## World Projection

World graph は world-level relationship をそのまま使う。

RFC 0015 の初期 overview のように、同じ relationship が複数 diagram に重複保存される前提の merge は不要になる。

World graph の provenance は次のように整理する。

- `source_relationship_ids`: world-level relationship ids。
- `source_diagram_ids`: node/entity が表示対象 diagram から来た場合の membership provenance。relationship の所有元ではない。
- edge identity は `relationship_id` または RFC 0024 の `edge_id` で安定化する。

Diagram filter 付き overview を返す場合も、filter は relationship provenance ではなく visible entity set を作るために使う。

## Depth Semantics

`tree_path.depth` は active world lineage graph における最短 path length とする。

Rules:

- `ancestor_id = descendant_id` の row は `depth = 0`。
- `ancestor_id <> descendant_id` の row は `depth > 0`。
- same ancestor / descendant pair に複数 path がある場合、`tree_path` は最短 depth だけを保持する。
- multiple paths 自体が重要な read では、`KinshipDerivationService` が `MultiplePaths` / `InferredFromShortestPath` のような qualifier を返す。
- all paths は初期実装では保存しない。

この定義により、`tree_path` は reachability cache としては deterministic になる。一方で、複数経路を含む親族説明の完全な source ではない。

## Tree Edge Policy

RFC 0013 と同じく、初期 tree-edge kind は次だけにする。

- `parent`
- `adoptive_parent`

`step_parent` は保存可能な explicit relationship だが、初期 `tree_path` には含めない。

理由:

- `parent` と `adoptive_parent` は ancestry-like lineage として扱いやすい。
- `step_parent` を ancestor / descendant closure に含めるかは product / culture / UI policy に依存する。
- step relation は direct relationship graph と `relation_path_to_center` で表現できる。

将来 opt-in projection で `step_parent` を lineage traversal に含める場合、current `tree_path` に混ぜず、projection option または別 closure cache として扱う。

## Maintenance

relationship mutation と `tree_path` rebuild は同一 transaction 内で実行する。

推奨 flow:

```text
begin transaction
  acquire world-scoped advisory lock

  validate world is active
  validate source / target are active entities in the world
  validate no self relationship
  validate date range
  normalize symmetric orientation
  validate duplicate / parent-family constraints
  validate no lineage cycle

  insert / update / delete relationship

  delete tree_path where world_id = ?
  rebuild tree_path from active parent/adoptive_parent relationships in the world
  insert self paths and shortest-depth closure rows for active world entities

commit
```

初期実装では world 単位 rebuild を採用する。

差分更新は高速化余地として残すが、削除時に本来残る alternate path まで消してしまうリスクがある。

例:

```text
A -> B -> D
A -> C -> D
```

`A -> B` を削除しても `A -> D` は `A -> C -> D` 経由で残る。このようなケースを安全に扱うまで、差分削除は導入しない。

## Concurrency

同一 world の relationship mutation は transaction-level advisory lock で直列化する。

```sql
SELECT pg_advisory_xact_lock($world_id);
```

理由:

- concurrent transaction が互いの uncommitted relationship を見ないと、cycle validation をすり抜ける可能性がある。
- relationship row と `tree_path` cache の commit order を world 単位で一貫させる必要がある。

Lock key は namespace collision を避けるため、実装では fixed namespace と `world_id` を組み合わせる。

## Cycle Policy

`tree_path` に参加する lineage graph は DAG でなければならない。

新規 tree edge `source -> target` を追加する前に、同じ world 内で `target` が `source` の ancestor でないことを検証する。

`tree_path` が最新である前提なら次で検出できる。

```sql
SELECT 1
FROM tree_path
WHERE world_id = $world_id
  AND ancestor_id = $target_entity_id
  AND descendant_id = $source_entity_id
LIMIT 1;
```

ただし mutation transaction 内では relationship change 後に full rebuild と DAG validation を行い、cache stale に依存しすぎない。

Cycle detected の場合は `409 CONFLICT` を返す。

## Relationship Constraints

RFC 0013 の active directed / symmetric duplicate constraints は world scope に変更する。

```sql
CREATE UNIQUE INDEX uq_relationship_directed_active
ON relationship (
    world_id,
    source_entity_id,
    target_entity_id,
    kind
)
WHERE deleted_at IS NULL
  AND kind IN ('parent', 'adoptive_parent', 'step_parent');

CREATE UNIQUE INDEX uq_relationship_symmetric_active
ON relationship (
    world_id,
    LEAST(source_entity_id, target_entity_id),
    GREATEST(source_entity_id, target_entity_id),
    kind
)
WHERE deleted_at IS NULL
  AND kind IN ('spouse', 'partner', 'cohabitant');
```

追加検討:

```sql
CREATE UNIQUE INDEX uq_relationship_parent_family_pair_active
ON relationship (
    world_id,
    source_entity_id,
    target_entity_id
)
WHERE deleted_at IS NULL
  AND kind IN ('parent', 'adoptive_parent', 'step_parent');
```

この制約は同一 parent-child pair に複数 parent-family kind を保存することを防ぐ。

ただし、同一人物が biological parent でも legal/adoptive parent でもあるような richer parentage model を扱う場合、この制約は強すぎる可能性がある。初期実装では product policy として「同一 pair の parent-family kind は高々 1 つ」を採用するか、service layer warning に留めるかを実装 RFC で決める。

## As-Of Projection

ADR 0005 と同じく、as-of mode では current `tree_path` を historical source of truth として使わない。

as-of read は次を source of truth とする。

- `as_of` で filter した canonical relationship rows
- `as_of` で visible と判断した person/entity rows
- request-local lineage closure

current `tree_path` は period information を持たないため、as-of projection で使うと historical truth を誤る。

## Kinship Derivation Boundary

`KinshipDerivationService` は単一 label を返す関数にしない。

推奨 output:

```rust
pub struct KinshipDerivation {
    pub relations: Vec<KinshipRelation>,
    pub diagnostics: Vec<KinshipDiagnostic>,
}
```

意味:

- 関係がない: `relations = []`
- 関係がある: `relations` に 1 件以上
- 複数関係がある: `relations` に複数件
- 情報不足: qualifier に Unknown / Incomplete を付ける
- データ不整合: diagnostics に入れる

これにより、「計算不能」を panic や arbitrary label selection ではなく型で表現する。

Localized label selection は RFC 0024 と同じく presentation concern とする。

## Diagnostics

実装時は、少なくとも次の diagnostic を表現できるようにする。

```rust
pub enum KinshipDiagnostic {
    RelationshipOutsideActiveScope,
    TreePathOutOfSync,
    CycleDetected,
    DuplicateCanonicalFact,
    InconsistentDateRange,
    MissingSelfPath,
}
```

Diagnostics は normal read response に常時含めるか、debug / validation endpoint に限定するかを別途決める。

## Migration Plan

1. `relationship.world_id` を追加し、既存 `relationship.diagram_id` から `diagram.world_id` を backfill する。
2. `tree_path_v2` または migration-safe な table rewrite で `world_id` 付き schema を作る。
3. active world ごとに current active `parent` / `adoptive_parent` relationship から closure を rebuild する。
4. self path を active world entity ごとに insert する。
5. relationship endpoint entity が relationship world に属することを検査し、不正 row を修復または reject list に出す。
6. relationship / tree_path に world-entity consistency guardrail を追加する。
7. `relationship.diagram_id` を削除する。
8. `tree_path` repository / service model に `world_id` を追加する。
9. delete / rebuild API を world-id based へ寄せる。
10. relationship mutation transaction に world advisory lock を追加する。
11. diagram graph read は diagram entity set で world relationship を filter する形へ変更する。

## Test Plan

- relationship は world-level fact として作成される。
- relationship endpoint entity が relationship world に属さない場合は write が失敗する。
- diagram graph は diagram に含まれる entity 同士の active relationship を自動表示する。
- diagram graph は diagram 外 entity を endpoint に持つ relationship を表示しない。
- kind filter / display settings により diagram edge 表示を切り替えられる。
- `tree_path` は world ごとに self path を持つ。
- `ancestor_id = descendant_id` iff `depth = 0`。
- parent / adoptive_parent 追加後に world-scoped `tree_path` が rebuild される。
- parent / adoptive_parent 削除後に alternate path がある場合は reachability が残る。
- `step_parent` / `spouse` / `partner` / `cohabitant` は current `tree_path` に入らない。
- cycle を作る relationship write は `409 CONFLICT`。
- concurrent update で cycle を作れない。
- as-of read は current `tree_path` ではなく request-local closure を使う。

## 既存 RFC / ADR との関係

- RFC 0008 を更新する。entity は world-scoped、relationship と `tree_path` も world-scoped、diagram は entity 表示範囲とする。
- RFC 0013 を補完する。RFC 0013 は stored relationship kind と tree-edge kind を定義し、この RFC は relationship / `tree_path` の aggregate scope、schema、maintenance boundary を定義する。
- RFC 0015 を更新する。world overview は diagram-local relationship merge ではなく、world-level relationship と optional diagram entity filter から構築する。
- RFC 0012 を補完する。`KinshipDerivationService` は `tree_path` を source of truth ではなく cache input として消費する。
- ADR 0005 と整合する。as-of projection は current `tree_path` を historical source of truth として使わない。

## 未解決事項

- same parent-child pair に `parent` と `adoptive_parent` の両方を許可するか。
- child ごとの biological parent 最大数を storage constraint にするか、service validation に留めるか。
- overlapping spouse / partner / cohabitant を reject するか、diagnostic に留めるか。
- diagnostics を通常 API response に含めるか、validation endpoint に分離するか。
- future opt-in projection で `step_parent` を lineage traversal に含める場合の API shape。
