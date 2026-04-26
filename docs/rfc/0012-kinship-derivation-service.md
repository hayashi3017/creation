# RFC 0012: Kinship Derivation Service

- 状態: `採用`
- 最終更新: `2026-04-26`

## 背景

現在の `GET /api/family-trees/{diagram_id}` は意図的に lineage のみを扱うが、family tree domain ではより豊かな read-side kinship が必要になる。

- `parent -> child` のような inverse view
- `spouse` のような symmetric view
- `tree_path` からの ancestor / descendant 導出
- sibling classification
- uncle / aunt、nephew / niece、cousin、in-law の導出
- `explicit`、`derived`、`suggested` などの source tracking
- gender と age を考慮した presentation metadata

一方、既存の `RelationshipService` には明確な責務がある。

- 明示的な relationship write payload の検証
- notes や dates など write-side field の正規化
- 保存済み `relationship` row の作成、更新、削除、取得

境界を曖昧にしたまま richer kinship logic を追加すると、次の問題が起きやすい。

1. `FamilyTreeUsecase` が orchestration layer ではなく巨大な graph derivation module になる。
2. `RelationshipService` が保存済み fact の CRUD と inferred fact の read-only derivation を混在させる。

この RFC は、`RelationshipService` と MECE で loose-coupled な新しい service boundary を定義する。

## 目標

- 永続化された relationship 管理と read-only kinship derivation を分離する。
- endpoint 名や aggregate 名ではなく、役割に基づく service 名を定義する。
- `RelationshipService`、新しい derivation service、`FamilyTreeUsecase` の MECE 境界を明示する。
- 新 service を既存 service と疎結合に保つ。
- 現在の lineage-only projection から richer kinship output へ段階的に展開できるようにする。

## 非目標

- 同じ変更で relationship CRUD API を変えること。
- derived relationships を `relationship` table に永続化すること。
- 日本語親族表現など最終的な localized label を決めること。
- `tree_path` の write-side maintenance を置き換えること。
- RFC 0008 の cross-diagram merge や identity-linking concern を解くこと。

## 提案

`creation-service` に read-side kinship derivation 専用 service を導入する。

推奨名:

- `KinshipDerivationService`

この名前を選ぶ理由:

- `FamilyTreeRelationshipService` は family tree relationship の CRUD 所有に見える。
- 実際の役割は「family tree relationship 全般」ではない。
- 役割は「明示的 fact から kinship semantics を導出すること」である。
- 同じ logic は ancestor、descendant、relatives-of-person、将来の merged genealogy read など endpoint をまたいで再利用される。

推奨する依存方向:

- `FamilyTreeUsecase` は `ProvidesKinshipDerivationService` に依存する。
- `RelationshipService` と `KinshipDerivationService` は相互に依存しない。
- `FamilyTreeUsecase` が両 service を orchestrate し、最終 response を組み立てる。

これにより usecase を薄く保ち、2 つの service を重複ではなく直交した責務にできる。

## MECE な Service 境界

### `RelationshipService` の責務

`RelationshipService` は永続化された明示的 relationship fact の service である。

責務:

- 保存 row に対する create / update / delete payload shape の検証
- blank note cleanup や date range check など write-side normalization
- 永続化に必要な明示的 relationship invariant の enforcement
- repository-backed CRUD for `relationship`
- stored `Relationship` rows を explicit fact として caller に返すこと

責務ではないこと:

- `parent` から `child` を見せるような inverse expansion
- mirrored spouse view のような symmetric expansion
- read-side graph 用の canonical family-tree orientation
- sibling / cousin / in-law / ancestor derivation
- family-tree node adjacency や root detection
- `Derived` や `Suggested` など read-side source tagging

### `KinshipDerivationService` の責務

`KinshipDerivationService` は explicit fact から kinship semantics への read-only transformation を担当する service である。

責務:

- 明示的 relationship を read-side graph form に canonicalize すること
- inverse / symmetric read expansion
- lineage closure data を使った ancestor / descendant semantics の導出
- sibling classification と higher-order kinship derivation
- output を `Explicit`、`Derived`、`Suggested` として tag すること
- caller が渡した active visible scope に derived relation を絞り込むこと

責務ではないこと:

- `relationship` row の永続化
- 保存 row の create / update / delete validation
- `tree_path` の mutation
- request の diagram、entity、person loading
- domain output を HTTP response DTO に直接 mapping すること

### `FamilyTreeUsecase` の責務

`FamilyTreeUsecase` は application orchestration layer のままにする。

責務:

1. `diagram_id` を検証する。
2. 対象 diagram を load する。
3. 非 `family_tree` diagram を reject する。
4. scope 内の active persons を load する。
5. `RelationshipService` 経由で explicit relationships を load する。
6. derivation service が必要なら追加の lineage closure input を load する。
7. `KinshipDerivationService` を呼ぶ。
8. `FamilyTree { nodes, edges, root_entity_ids, stats }` を assemble する。

責務ではないこと:

- rule-by-rule kinship derivation
- stored relationship CRUD semantics

### この分割が MECE である理由

- 永続化された explicit fact lifecycle は `RelationshipService` のみに属する。
- read-only kinship inference は `KinshipDerivationService` のみに属する。
- request orchestration と response assembly は `FamilyTreeUsecase` のみに属する。

3 者で重複させる必要がある責務はない。

## Stored と Derived の関係

この RFC は即時の schema rewrite ではなく、方針を推奨する。

- explicit relationships は `relationship` に保存される rows である。
- derived relationships は read time に計算され、永続化しない。
- suggested relationships は将来 client に返す可能性がある heuristic output だが、canonical fact として扱わない。

RFC 0013 後の read/write baseline:

- explicit tree-edge lineage: `parent`, `adoptive_parent`
- explicit non-tree-edge canonical kinds: `step_parent`, `spouse`, `partner`, `cohabitant`
- derived-only kinds: `sibling`, `ancestor`, `descendant`, `uncle_aunt`, `nephew_niece`, `cousin`, `in_law`

`sibling`、`ancestor`、`cousin` など graph-expanded kinship を独立 row として保存しない。storage-facing Rust enum に `Sibling` など derived-only kind を含めない。

## 関係ソースの扱い

explicit、derived、suggested の区別は有用であり、read model concept として残す。

```rust
enum FamilyTreeRelationshipSource {
    Explicit,
    Derived,
    Suggested,
}
```

意味:

- `Explicit`: 保存済み `relationship` row に直接裏付けられる。
- `Derived`: explicit row と lineage closure から決定論的に含意される。
- `Suggested`: UI や review flow で有用な可能性がある heuristic output だが、確認済み domain truth として扱わない。

## Read Model 形状

richer kinship output の主 output shape として、storage-oriented な `Relationship` struct を再利用しない。

Derivation service は独自の domain output type を持つ。

```rust
struct KinshipRelation {
    from_entity_id: usize,
    to_entity_id: usize,
    kind: KinshipRelationKind,
    source: FamilyTreeRelationshipSource,
    explicit_relationship_id: Option<usize>,
    sibling_kind: Option<FamilyTreeSiblingKind>,
    generation_distance: Option<usize>,
}
```

補助 enum の例:

```rust
enum FamilyTreeSiblingKind {
    Full,
    Half,
    Adoptive,
    Step,
}
```

重要な設計点:

- `兄`、`弟`、`父`、`母` のような localized string を主 domain output にしない。
- derivation service は現在の `FamilyTreeEdge` より豊かな semantic relation を返してよい。
- `FamilyTreeUsecase` が、endpoint contract でどこまで expose するかを決める。

最終的な localized label は、次をもとに後段で render する。

- relationship kind
- gender
- 必要に応じた age ordering
- locale / presentation rules

これにより core domain contract を特定言語や UI wording policy に固定しない。

## 導出ルール

### Canonical 正規化

`KinshipDerivationService` は higher-level kinship を導出する前に、stored rows を canonical internal graph に正規化する。

例:

- `parent(A, B)` は canonical lineage edge `A -> B` を含意する。
- `adoptive_parent(A, B)` は canonical lineage edge `A -> B` を含意する。
- `spouse(A, B)` は symmetric である。
- `cohabitant(A, B)` は symmetric である。
- 終了した spouse state は別 kind ではなく、`spouse + end_date + end_reason` で表す。

### Tree Path の扱い

`tree_path` は ancestor / descendant reachability の authoritative lineage closure table のままである。

`RelationshipService` は `tree_path` を解釈しない。

`KinshipDerivationService` は次のような導出のために lineage closure input を消費してよい。

- `depth = 1`: direct parent / child
- `depth = 2`: grandparent / grandchild metadata
- `depth >= 1`: optional distance metadata 付き ancestor / descendant relationships

これらの derivation を支えるためだけに raw `tree_path` rows を public contract の主形状として expose しない。

### Sibling の導出

Sibling derivation は read-only とする。

基準ルール:

- canonical lineage graph で少なくとも 1 人の parent を共有する 2 人は siblings である。

推奨 classification:

- `full`: 同じ 2 人の explicit parents を共有する。
- `half`: 1 人の explicit parent を共有する。
- `adoptive`: explicit kind が対応した後、adoptive lineage 経由で導出する。
- `step`: blood/adoptive lineage ではなく step-parent semantics 経由で導出する。

### Uncle / Aunt、Nephew / Niece、Cousin

これらは second-order derived relationships であり、storage に materialize しない。

例:

- A の parent が B の sibling である場合、B は A の uncle/aunt である。
- A の sibling に child B がいる場合、B は A の nephew/niece である。
- A の parent と B の parent が siblings である場合、A と B は cousins である。

Service はこれらを ad hoc な client-side logic ではなく、正規化済み lineage と sibling data から計算する。

### In-Law

In-law relationships は derived-only のままにする。

例:

- `spouse(A, B) + parent(B, C)` なら A は C の parent-in-law である。

これは `FamilyTreeUsecase` ではなく derivation service に置くべき graph expansion である。

### Suggested Step Relationship の扱い

Suggested step relationships は有用だが、過剰推論しやすい。

推奨ポリシー:

- shared children から spouse を推論しない。
- co-residence だけから spouse を推論しない。
- `end_date` や divorce history だけで current family membership を結論しない。
- explicit canonical relationship kind または強い product rule がない限り、step-parent / step-child は `Suggested` としてのみ emit する。

将来 `parent + spouse/cohabitant + date overlap` から step relationship を推論する場合も、product semantics が review されるまで opt-in にする。

## 疎結合ルール

新 service を疎結合に保つ。

- `KinshipDerivationService` は `RelationshipService` を呼ばない。
- `RelationshipService` は `KinshipDerivationService` を呼ばない。
- `FamilyTreeUsecase` は explicit relationships を domain input として渡す。
- lineage closure が必要なら、usecase が渡すか、その data 用の dedicated read port に依存する。
- closure-table concern を `RelationshipService` 経由にしない。永続化 concern と derivation concern が曖昧になる。

推奨 interface:

- HTTP-specific schema struct ではなく domain input/output struct を使う。
- deterministic unit tests のため、caller が既に load 済みの scope data を渡す。
- 必要になったら `RelationshipService` を拡張するのではなく、dedicated tree-path read port を追加する。

```rust
struct DeriveKinshipInput {
    active_person_ids: Vec<usize>,
    explicit_relationships: Vec<Relationship>,
    lineage_paths: Vec<KinshipLineagePath>,
}

struct KinshipDerivation {
    lineage_edges: Vec<CanonicalLineageEdge>,
    relations: Vec<KinshipRelation>,
}
```

## Usecase に置かず Service にする理由

この RFC は、ユーザーのメモにある service boundary を推奨する。ただし `FamilyTreeRelationshipService` より狭く明確な役割にする。

理由:

- kinship derivation は application orchestration ではなく domain logic である。
- algorithm は endpoint 数より速く複雑化する可能性が高い。
- 同じ logic は ancestor、descendant、relatives-of-person、merged genealogy projection など将来 read で必要になる。
- service-level unit tests は full usecase stack を起動せず、graph input/output に集中できる。
- `FamilyTreeUsecase` を読みやすく保ち、workspace layering と揃えられる。
- `RelationshipService` は保存済み explicit relationship fact の管理に集中できる。

要するに、`FamilyTreeUsecase` は kinship derivation result を要求し、`RelationshipService` は stored relationship rows を所有し続ける。どちらも相手の責務を吸収しない。

## 段階的な展開

### Phase 1

public `/api/family-trees/{diagram_id}` response は変更しない。

現在の endpoint に必要な read-side canonicalization だけを持つ `KinshipDerivationService` を導入する。

- `parent` / `adoptive_parent` を canonical lineage edges に正規化する。
- active visible scope に絞り込む。

当面 `FamilyTreeUsecase` に残すもの:

- node assembly
- adjacency list materialization
- root detection
- stats assembly

これにより境界は MECE に保たれる。

- derivation service は kinship fact を derive する。
- usecase は endpoint-specific response shape を assemble する。

実装状況:

- `creation-service/src/service/kinship_derivation.rs` に実装済み。
- `FamilyTreeUsecase` は load 済み explicit relationships と active person ids を service に渡す。
- service は現在の family-tree projection 用 canonical lineage edges を返す。
- service は内部的に explicit parent/adoptive-parent relations と derived inverse child/adoptive-child relations も emit する。
- public `/api/family-trees/{diagram_id}` response は変更していない。

### Phase 2

`KinshipDerivationService` を拡張し、次を扱う。

- `Explicit` と `Derived` の source
- optional sibling classification
- `tree_path` 由来の optional generation distance metadata

この段階でも既存 public response は lineage-only のままでよい。

### Phase 3

API contract を決めた後に richer derived kinship output を追加する。

- `/api/family-trees/{diagram_id}` を拡張する。
- または `/api/family-trees/{diagram_id}/relationships` に kinship-focused read を追加する。

public shape を review する前に、現在の read API へ richer contract を押し込まない。

## 利点

- 複雑な graph semantics を usecase layer から切り出せる。
- kinship rule を複数 read model で再利用できる。
- stored fact と read-only derivation の分離を保てる。
- siblings や cousins のような機械的に導出できる row を保存する圧力を下げる。
- CRUD service を汚さず、将来 heuristic を追加する場所を用意できる。

## 欠点

- domain type と service trait が増える。
- 将来 dedicated tree-path read input が必要になる可能性がある。
- 一部の kinship terminology は現在の `RelationshipKind` enum と一致しないため、慎重な正規化が必要である。

## 未解決事項

- richer derived kinship は storage-facing `RelationshipKind` を拡張するのではなく、新しい read-only enum を使うべきか。
- `Suggested` relationships は将来 explicit include flag が追加されるまで既定で除外すべきか。
- localized kinship labels は driver で render すべきか、完全に client に任せるべきか。
- public write API が non-lineage kinds を受け付け始める場合、どれを canonical stored fact とし、どれを derived-only にすべきか。
- lineage closure input が必要な場合、usecase が dedicated read repository から直接 load すべきか、`KinshipDerivationService` が自身の read-port dependency を持つべきか。
