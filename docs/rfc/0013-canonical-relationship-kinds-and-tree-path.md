# RFC 0013: Canonical Relationship Kinds と Tree Path ポリシー

- 状態: `採用`
- 最終更新: `2026-04-26`

## 背景

現在の `relationship_kind` enum は複数の概念を混在させている。

- directed canonical facts: `parent`, `adopted_parent`, `step_parent`
- directed facts から導出できる inverse facts: `child`, `adopted_child`, `step_child`
- symmetric facts: `spouse`, `cohabitant`
- derived kinship labels: `sibling`
- kind として表現された relationship lifecycle state: `divorced_spouse`

これは `tree_path` 周辺に構造的な問題を作る。

`tree_path` は ancestor / descendant traversal 用の closure table である。directed lineage edge から構築される場合にだけ意味が明確になる。storage が canonical directed facts と inverse / derived facts の両方を許すと、write path はどの row が closure maintenance に参加するか、どう正規化するかを毎回判断する必要がある。

現在の実装は public relationship writes で `parent` と `child` のみを許可し、両方を parent-to-child edge に正規化してから `tree_path` を rebuild することで緩和している。これは一時的な互換 layer としては機能するが、storage enum が実際に永続化すべき domain fact より広いままになる。

この RFC は persistence model を狭め、mixed relationship enum ではなく canonical stored facts から `tree_path` を維持できるようにする。

## 目標

- `relationship_kind` を永続化される canonical relationship facts のみを表すものにする。
- inverse と graph-derived kinship を storage enum から取り除く。
- どの canonical kind が `tree_path` に参加するか定義する。
- `source_entity_id` / `target_entity_id` semantics を曖昧でなくする。
- 既存の `child` や他の non-canonical enum values からの migration path を用意する。
- richer kinship label は RFC 0012 と揃え、read-side derived output として扱う。

## 非目標

- この RFC 内で migration を実装すること。
- 日本語親族表現など final localized display labels を決めること。
- 同じ step で RFC 0007 の public family-tree response contract を変えること。
- sibling、cousin、grandparent など derived kinship rows を永続化すること。
- richer kinship output の最終 API shape を決めること。

## 提案

`relationship` row は明示的な canonical fact のみとして扱う。

推奨 stored enum:

```sql
CREATE TYPE relationship_kind AS ENUM (
  'parent',
  'adoptive_parent',
  'step_parent',
  'spouse',
  'partner',
  'cohabitant'
);
```

概念上の grouping:

```text
Directed canonical relationships:
- parent
- adoptive_parent
- step_parent

Symmetric canonical relationships:
- spouse
- partner
- cohabitant

Derived kinship, not persisted:
- child
- adoptive_child
- step_child
- sibling
- ancestor
- descendant
- grandparent
- grandchild
- uncle_aunt
- nephew_niece
- cousin
```

重要なルールは、storage は fact を記録し、read model が kinship view を導出することである。

- `parent(A, B)` は「A が B の parent である」という fact として保存する。
- `child(B, A)` は保存しない。これは `parent(A, B)` の inverse view である。
- `sibling(A, B)` は保存しない。shared parent sets から導出する。
- `grandparent(A, C)` は保存しない。`tree_path.depth = 2` から導出する。
- `divorced_spouse` は kind として保存しない。end reason 付きの終了した `spouse` relationship として表す。

## Source と Target の意味

### 有向種別

Directed kind では `source_entity_id -> target_entity_id` に意味がある。

- `parent(source, target)`: source は parent、target は child
- `adoptive_parent(source, target)`: source は adoptive parent、target は adoptive child
- `step_parent(source, target)`: source は step parent、target は step child

方向は fact の一部なので、directed relationships を id で自動 sort してはいけない。

### 対称種別

Symmetric kind では relationship に意味的な方向はない。

推奨 storage rule:

- persistence 前に endpoints を normalize する。
- `source_entity_id < target_entity_id` として保存する。
- 同じ diagram、endpoint pair、kind の duplicate active rows を reject または merge する。

これにより `spouse(1, 2)` と `spouse(2, 1)` が別々の active fact として保存されることを防ぐ。

## Tree Path ポリシー

`tree_path` は canonical directed lineage facts のみから構築する。

初期 tree-edge policy:

- `parent`: `tree_path` に参加する。
- `adoptive_parent`: `tree_path` に参加する。
- `step_parent`: 初期状態では `tree_path` に参加しない。
- `spouse`, `partner`, `cohabitant`: `tree_path` には決して参加しない。

理由:

- `parent` と `adoptive_parent` は ancestry-like な directed lineage edge を表す。
- `step_parent` は family-relevant だが、ancestor / descendant traversal に含めるべきかは product と文化に依存する。
- symmetric relationships は ancestor / descendant の方向を定義しない。
- `cohabitant` は生活状況または social relationship であり、kinship traversal に影響させてはいけない。

この結果、`tree_path.depth` は狭い意味を保てる。

- `depth = 1`: adoptive lineage を含めた direct parent / child
- `depth = 2`: grandparent / grandchild-style lineage
- `depth >= 1`: ancestor / descendant reachability

後続 RFC が明示的に変更するまで、`tree_path` に spouse、partner、cohabitant、sibling、cousin、step-parent traversal の row を保存しない。

## ライフサイクル状態の扱い

Lifecycle state を `relationship_kind` に encode しない。

`divorced_spouse` は canonical relationship と lifecycle fields に置き換える。

推奨 table addition:

```sql
end_reason VARCHAR(32)
```

将来 enum を導入する場合の推奨値:

```text
divorce
death
separation
unknown
```

例:

- current spouse: `kind = spouse`, `end_date = NULL`, `end_reason = NULL`
- divorced spouse: `kind = spouse`, `end_date IS NOT NULL`, `end_reason = divorce`
- widowed spouse: `kind = spouse`, `end_date IS NOT NULL`, `end_reason = death`

これにより relationship kind と relationship history を分離できる。

## 制約

Self-relation check を追加する。

```sql
CHECK (source_entity_id <> target_entity_id)
```

Directed canonical facts 用の active-row unique index を追加する。

```sql
CREATE UNIQUE INDEX uq_relationship_directed_active
ON relationship (
  diagram_id,
  source_entity_id,
  target_entity_id,
  kind
)
WHERE deleted_at IS NULL
  AND kind IN ('parent', 'adoptive_parent', 'step_parent');
```

Symmetric canonical facts 用の active-row unique index を追加する。

```sql
CREATE UNIQUE INDEX uq_relationship_symmetric_active
ON relationship (
  diagram_id,
  LEAST(source_entity_id, target_entity_id),
  GREATEST(source_entity_id, target_entity_id),
  kind
)
WHERE deleted_at IS NULL
  AND kind IN ('spouse', 'partner', 'cohabitant');
```

Application service は書き込み前に normalize と validate を行う。Database constraints は最終 guardrail であり、唯一の validation layer ではない。

## Rust Model の方向性

Storage-facing enum は stored canonical facts のみを含める。

推奨形:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredRelationshipKind {
    Parent,
    AdoptiveParent,
    StepParent,
    Spouse,
    Partner,
    Cohabitant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipTopology {
    Directed,
    Symmetric,
}

impl StoredRelationshipKind {
    pub fn topology(self) -> RelationshipTopology {
        match self {
            Self::Parent | Self::AdoptiveParent | Self::StepParent => {
                RelationshipTopology::Directed
            }
            Self::Spouse | Self::Partner | Self::Cohabitant => {
                RelationshipTopology::Symmetric
            }
        }
    }

    pub fn is_tree_edge(self) -> bool {
        matches!(self, Self::Parent | Self::AdoptiveParent)
    }
}
```

Derived read-side kinship は database enum として再利用せず、別 enum を使う。

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerivedKinship {
    Child,
    AdoptiveChild,
    StepChild,
    Sibling,
    Ancestor { depth: u32 },
    Descendant { depth: u32 },
    Grandparent,
    Grandchild,
    UncleAunt,
    NephewNiece,
    Cousin,
}
```

この分割により、`RelationshipService` は persisted facts を担当し、derived kinship は RFC 0012 の read-side service boundary に残る。

## Migration 戦略

PostgreSQL enum value removal は単純な `ALTER TYPE` ではないため、段階的 migration を使う。

### Phase 1: Compatibility Support を追加

- 新しい spelling を選ぶ場合は `partner` と `adoptive_parent` を追加する。
- `end_reason` を追加する。
- invalid rows を clean した後に self-relation check を追加する。
- service-level topology helpers を追加する。
- backward compatibility が必要な範囲でのみ既存 public payload を受け付け続ける。

### Phase 2: 既存 rows の正規化

Inverse rows を canonical direction に正規化する。

- `child(source, target)` -> `parent(target, source)`
- `adopted_child(source, target)` -> `adoptive_parent(target, source)`
- `step_child(source, target)` -> `step_parent(target, source)`

Lifecycle rows を正規化する。

- `divorced_spouse(source, target)` -> `end_reason = divorce` を持つ `spouse(source, target)`

Symmetric rows を正規化する。

- `kind IN ('spouse', 'partner', 'cohabitant')` かつ `source_entity_id > target_entity_id` の場合、endpoints を swap する。

Unique index を追加する前に duplicate active rows を解消する。

### Phase 3: Tree Path の再構築

Relationship rows を canonicalize した後に実行する。

1. migration 対象 diagram の `tree_path` rows を削除するか、すべての closure rows を rebuild する。
2. active `parent` と `adoptive_parent` rows のみから rebuild する。
3. cycle detection が tree-edge kinds のみを使うことを検証する。

Closure maintenance 用 relationship edge を load する repository methods は、`parent` と `child` を special-case するのではなく、`StoredRelationshipKind::is_tree_edge()` semantics で filter する。

### Phase 4: Database Enum の置き換え

新しい enum type を作り、text 経由で cast するなど safe migration pattern を使う。

```sql
CREATE TYPE relationship_kind_v2 AS ENUM (
  'parent',
  'adoptive_parent',
  'step_parent',
  'spouse',
  'partner',
  'cohabitant'
);
```

すべての rows が有効な canonical values になった後、`relationship.kind` を新 type に migrate する。

失敗した enum cast は schema を部分移行状態に残す可能性があるため、正確な SQL は dedicated migration と tests で実装する。

## API 互換性

Write API は canonical stored kinds のみを受け付ける方向へ移行する。

互換 options:

- strict option: `child`, `adopted_child`, `step_child`, `sibling`, `divorced_spouse` を `400 BAD_REQUEST` で reject する。
- compatibility option: inverse kinds を一時的に受け付け、persistence 前に normalize し、read では canonical stored kinds を返す。

Client 更新後は strict option の方が明快である。既存 client が `child` を送っている場合、rollout 中は compatibility option の方が安全である。

どちらの rollout option でも、persistence は canonical stored rows のみに収束させる。

## 既存 RFC / ADR との関係

この RFC は ADR 0004 を拡張する。

ADR 0004 は現在の実装を意図的に `parent` と `child` に限定し、より広い relationship semantics を延期していた。この RFC は `child` を長期 storage から取り除き、`tree_path` を canonical tree-edge kinds に依存させることで、その延期事項に答える。

この RFC は RFC 0012 を補完する。

RFC 0012 は read-side `KinshipDerivationService` を定義する。この RFC は、その service が消費すべき storage-side facts を定義する。

## 利点

- `tree_path` は canonical ancestor / descendant closure という安定した意味を持つ。
- inverse relationships による duplicate storage facts がなくなる。
- sibling や cousin など derived kinship が stored parent facts から乖離しなくなる。
- symmetric relationship duplication を一貫して防げる。
- divorce など relationship ending を kind value 増殖なしに表現できる。
- future read models は storage enum を広げずに richer kinship を導出できる。

## 欠点

- 非自明な enum migration が必要になる。
- `child` を送る client に対して API compatibility handling が必要になる可能性がある。
- `adopted_parent` と `adoptive_parent` の spelling を決め、一貫して migrate する必要がある。
- product が step-family ancestry traversal を求める場合、`step_parent` を `tree_path` から除外する方針を見直す必要がある。

## テスト計画

実装時の最小 coverage:

- migration が `child` rows を source/target 反転済み `parent` rows に変換する。
- migration が `adopted_child` rows を source/target 反転済み adoptive-parent rows に変換する。
- migration が `divorced_spouse` を `end_reason = divorce` 付き `spouse` に変換する。
- symmetric relationship writes が endpoint order を normalize する。
- duplicate active symmetric relationships が reject される。
- duplicate active directed relationships が reject される。
- self-relationships が reject される。
- `tree_path` rebuild が `parent` と `adoptive_parent` を使う。
- `tree_path` rebuild が `step_parent`, `spouse`, `partner`, `cohabitant` を無視する。
- public relationship writes が rollout option に従い non-canonical legacy kinds を reject または normalize する。

## 決定済み事項

- stored enum は `adoptive_parent` を使う。
- public API は storage-facing enum から legacy inverse/derived kinds を除くことで reject する。
- migration は PostgreSQL enum を置き換える前に既存 inverse rows を正規化する。
- 初期実装では `end_reason` を `VARCHAR(32)` として保存する。

## 未解決事項

- `step_parent` は将来 opt-in projection で tree-edge kind にすべきか、それとも ancestor / descendant closure の外に恒久的に置くべきか。
- `end_reason` は最終的に PostgreSQL enum または separate event/history table にすべきか。
