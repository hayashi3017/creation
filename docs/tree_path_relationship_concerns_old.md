tree_path と relationship による親族関係計算の懸念まとめ

- Status: Draft
- Last updated: 2026-05-09
- 対象: 家系図管理アプリケーション / kinship derivation / tree_path / relationship

## 1. この文書の目的

この文書は、`relationship` テーブルと `tree_path` テーブルを併用して親族関係を永続化・導出する設計について、次を漏れなく整理する。

1. 計算できない、または計算結果が不定になるケース
2. データ競合・矛盾が起きるケース
3. 現在の schema で証明できない理由
4. 計算不能をなくすための設計方針
5. 証明可能な不変条件
6. DB 制約、サービス層検証、テストで守るべき項目

結論として、現在の `relationship + tree_path` 構成のままでは「計算できない場合がない」ことは証明できない。

ただし、次の方針を採用すれば、アプリケーション仕様の範囲では証明可能にできる。

```text
relationship = 正規の保存済み explicit fact
tree_path    = relationship から再構築可能な派生キャッシュ
親族導出      = relationship + tree_path から read-only に計算
表示ラベル    = 親族導出結果から後段で選択する presentation concern
```

特に重要なのは、親族計算を「必ず 1 つの日本語ラベルを返す関数」にしないことである。

```rust
// 避けたい形
fn calculate_relation(a: EntityId, b: EntityId) -> KinshipLabel;

// 推奨する形
fn derive_kinship(input: DeriveKinshipInput) -> KinshipDerivation;
```

`KinshipDerivation` は、関係がなければ空集合、関係が複数あれば複数件、情報不足があれば qualifier、データ矛盾があれば diagnostics を返す。

---

## 2. 前提整理

### 2.1 現在の保存対象

現在の schema では、`relationship_kind` は次の保存済み関係を持つ。

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

また、`relationship` は `diagram_id` を持つ。

```sql
CREATE TABLE relationship (
    relationship_id BIGSERIAL PRIMARY KEY,
    diagram_id BIGINT NOT NULL REFERENCES diagram(diagram_id) ON DELETE CASCADE,
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

一方、`tree_path` は `diagram_id` を持たない。

```sql
CREATE TABLE tree_path (
    ancestor_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    descendant_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    depth INT NOT NULL,
    PRIMARY KEY (ancestor_id, descendant_id)
);
```

### 2.2 現在の RFC 方針

既存 RFC では、次の分離が提案されている。

- `RelationshipService`: 保存済み explicit relationship fact の CRUD と write-side validation を担当する
- `KinshipDerivationService`: explicit fact から read-only に親族 semantics を導出する
- `FamilyTreeUsecase`: diagram / entity / relationship / tree_path の読み込みと response assembly を担当する

また、`sibling`, `ancestor`, `descendant`, `uncle_aunt`, `nephew_niece`, `cousin`, `in_law` は derived-only とし、`relationship` table に保存しない方針である。

この方針は妥当であり、計算不能をなくすためにも維持するべきである。

---

## 3. 計算不能・不定結果につながる懸念

## 3.1 `tree_path` に `diagram_id` がない

### 発生条件

`relationship` は `diagram_id` を持つが、`tree_path` は持たない。

同じ `entity` が複数 diagram に所属する、または world 単位で人物を共有する場合、`tree_path` がどの diagram の lineage closure なのかを区別できない。

### 例

```text
diagram A:
  太郎 parent 一郎

diagram B:
  太郎 parent 花子
```

このとき `tree_path` が `diagram_id` を持たないと、diagram A の表示で diagram B の祖先・子孫関係が混入する可能性がある。

### 影響

- diagram scope が破れる
- 中心人物 view で別 diagram の人物が出る
- tree_path を authoritative として使うほど誤導出が増える
- family tree merge / genealogy overview の導入時にさらに曖昧になる

### 対策

`tree_path` に `diagram_id` を追加する。

```sql
CREATE TABLE tree_path (
    diagram_id BIGINT NOT NULL REFERENCES diagram(diagram_id) ON DELETE CASCADE,
    ancestor_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    descendant_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    depth INT NOT NULL CHECK (depth >= 0),
    PRIMARY KEY (diagram_id, ancestor_id, descendant_id),
    CONSTRAINT chk_tree_path_self_depth CHECK (
        (ancestor_id = descendant_id AND depth = 0)
        OR
        (ancestor_id <> descendant_id AND depth > 0)
    )
);
```

可能なら、`diagram_entity` との複合 FK も追加する。

```sql
ALTER TABLE tree_path
ADD CONSTRAINT fk_tree_path_ancestor_diagram_entity
FOREIGN KEY (diagram_id, ancestor_id)
REFERENCES diagram_entity(diagram_id, entity_id);

ALTER TABLE tree_path
ADD CONSTRAINT fk_tree_path_descendant_diagram_entity
FOREIGN KEY (diagram_id, descendant_id)
REFERENCES diagram_entity(diagram_id, entity_id);
```

---

## 3.2 `relationship` の source / target が diagram 内 entity である保証が弱い

### 発生条件

`relationship.diagram_id` は diagram を参照するが、`source_entity_id` / `target_entity_id` は単に `entity(entity_id)` を参照している。

そのため、DB 制約上は次が起き得る。

```text
relationship.diagram_id = 1
source_entity_id        = diagram 1 に所属する entity
target_entity_id        = diagram 2 に所属する entity
```

### 影響

- diagram scope 外の人物が親族導出に混ざる
- tree_path 再構築時に scope 外 entity を ancestor / descendant に含める
- `active_person_ids` で後から絞る場合、関係の片側が欠落して導出結果が不完全になる
- 「計算できない」よりも危険で、「計算できるが間違っている」状態になる

### 対策

`relationship` に対して、source / target が同じ diagram に所属することを保証する。

```sql
ALTER TABLE relationship
ADD CONSTRAINT fk_relationship_source_diagram_entity
FOREIGN KEY (diagram_id, source_entity_id)
REFERENCES diagram_entity(diagram_id, entity_id);

ALTER TABLE relationship
ADD CONSTRAINT fk_relationship_target_diagram_entity
FOREIGN KEY (diagram_id, target_entity_id)
REFERENCES diagram_entity(diagram_id, entity_id);
```

ただし `diagram_entity.deleted_at IS NULL` は通常の FK では保証できないため、active scope はサービス層でも必ず検証する。

---

## 3.3 `tree_path` が lineage 種別を失う

### 発生条件

`relationship` は次を区別する。

```text
parent
adoptive_parent
step_parent
```

しかし `tree_path` は ancestor / descendant / depth しか持たない。

### 例

```text
A parent B
A adoptive_parent B
A step_parent B
```

これらはすべて `tree_path(A, B, 1)` になってしまう。

### 影響

`tree_path` だけでは次を計算できない。

- biological parent か adoptive parent か
- step parent か
- full sibling か half sibling か adoptive sibling か step sibling か
- biological ancestor か legal ancestor か
- 「祖父母」ではあるが実系統か養子系統か

### 対策

`tree_path` は到達性キャッシュに限定する。

```text
ancestor / descendant / depth:
  tree_path を使ってよい

parent / adoptive_parent / step_parent:
  relationship を使う

sibling classification:
  direct parent map から計算する
```

分類が必要な親族関係は、必ず direct な `relationship` を見る。

---

## 3.4 複数経路があると depth が一意にならない

### 発生条件

`tree_path` の主キーが `(ancestor_id, descendant_id)` のため、同じ ancestor / descendant に対して depth を 1 つしか保持できない。

### 例

```text
A -> B -> D
A -> C -> E -> D
```

この場合、A から D への経路は複数ある。

```text
A -> D depth = 2
A -> D depth = 3
```

現在の `tree_path` ではどちらか一方しか保存できない。

### 影響

- 「祖先かどうか」だけなら問題は小さい
- 「何世代上か」「何親等か」「祖父母か曾祖父母か」を厳密に扱う場合は情報が落ちる
- 複雑な婚姻・再婚・養子縁組・近親婚・重複系譜では、単一 depth が誤解を生む

### 対策

基本方針として、`tree_path.depth` は「最短 depth」と定義する。

```text
tree_path.depth = active lineage graph における最短 path length
```

複数経路の存在が重要な場合は、`KinshipRelation` に qualifier を付ける。

```rust
enum KinshipQualifier {
    MultiplePaths,
    InferredFromShortestPath,
}
```

全経路列挙は経路数が爆発するため、初期設計では避ける。

---

## 3.5 論理削除と `tree_path` がずれる

### 発生条件

`relationship` や `entity` は `deleted_at` を持つが、`tree_path` は持たない。

### 例

```text
relationship(A parent B) は deleted_at 設定済み
tree_path(A, B, 1) は残っている
```

### 影響

- 削除済み relationship から ancestor / descendant が導出される
- 表示上は削除したはずの親族関係が残る
- logical delete の正確性プロパティが崩れる

### 対策

`tree_path` を source of truth にしない。

```text
relationship 更新・削除
  -> 同一 transaction 内で対象 diagram の tree_path を再構築
  -> commit
```

差分更新は高速だが削除時の正しさ証明が難しい。
まずは diagram 単位で再構築する方が安全である。

---

## 3.6 `start_date` / `end_date` と `tree_path` が対応していない

### 発生条件

`relationship` には `start_date` / `end_date` があるが、`tree_path` には期間情報がない。

### 例

```text
A spouse B: 1990-01-01 .. 2000-12-31
A spouse C: 2005-01-01 .. NULL
```

また、親子関係や養子縁組も、仕様上は時点によって active / inactive が変わり得る。

### 影響

- 1995 年時点と 2026 年時点の家系図を同じ `tree_path` では表現できない
- as-of mode で誤った ancestor / descendant を返す
- 離婚・再婚・養子縁組解消などに弱くなる

### 対策

as-of 指定がある場合は、`tree_path` を authoritative にしない。

```text
as_of 指定あり:
  relationship を日付で絞る
  メモリ上で active lineage graph を作る
  必要範囲だけ closure を計算する

as_of 指定なし:
  現在時点用 tree_path を cache として使ってよい
```

将来、as-of 性能が必要になった場合のみ、`tree_path_history` や期間付き closure を検討する。

---

## 3.7 `tree_path` の自己参照不変条件が DB で保証されていない

### 発生条件

設計上は「すべての entity は自分自身への depth 0 path を持つ」とされているが、現在の `tree_path` schema では次を保証していない。

```text
ancestor_id = descendant_id なら depth = 0
depth = 0 なら ancestor_id = descendant_id
ancestor_id <> descendant_id なら depth > 0
```

### 影響

- 自己参照が欠落すると root detection や ancestor query が不安定になる
- depth 0 の非自己 path が入ると、同一人物判定に近い誤導出が起きる
- プロパティベーステストでは検出できても、DB が不正状態を拒否しない

### 対策

DB constraint を追加する。

```sql
ALTER TABLE tree_path
ADD CONSTRAINT chk_tree_path_depth
CHECK (
    (ancestor_id = descendant_id AND depth = 0)
    OR
    (ancestor_id <> descendant_id AND depth > 0)
);
```

また、entity 作成時に必ず同一 transaction で自己参照 path を作成する。

---

## 3.8 循環があると lineage closure が定義しづらい

### 発生条件

次のような cyclic parent relationship が入る。

```text
A parent B
B parent C
C parent A
```

### 影響

- ancestor / descendant が無限に循環する概念になる
- closure table の depth が定義できない、または最短距離だけは出るが家系図として矛盾する
- root detection ができない
- layout で上下関係が破綻する

### 対策

parent / adoptive_parent / step_parent のうち、lineage graph に含める関係については DAG を不変条件にする。

```text
新規 edge source -> target を追加する前に、target が source の ancestor ではないことを検証する
```

`tree_path` が正しい前提なら、追加前検証は次でよい。

```sql
SELECT 1
FROM tree_path
WHERE diagram_id = $diagram_id
  AND ancestor_id = $target_entity_id
  AND descendant_id = $source_entity_id
LIMIT 1;
```

存在する場合、追加すると循環になる。

---

## 3.9 step_parent を lineage closure に含めるかが曖昧

### 発生条件

現在の保存 kind には `step_parent` がある。

しかし、`step_parent` を `tree_path` に含めるかどうかは設計判断である。

### 影響

`step_parent` を closure に含める場合:

- step parent が ancestor として扱われる
- descendant / ancestor query に法的・同居的な関係が混ざる
- biological lineage と household lineage が混ざる

`step_parent` を closure に含めない場合:

- step-child / step-sibling の導出には別途 relationship graph を見に行く必要がある
- central view で継親を上下配置するか横配置するか決める必要がある

### 対策

lineage edge を複数カテゴリに分ける。

```rust
enum LineageEdgeKind {
    BiologicalParent,
    AdoptiveParent,
    StepParent,
}
```

`tree_path` が保持する closure の意味を明確にする。

推奨は次のどちらか。

```text
案 A:
  tree_path は parent + adoptive_parent の closure
  step_parent は direct relationship として別処理

案 B:
  tree_path に lineage_kind を持たせる
  biological / adoptive / step の closure を分ける
```

初期実装では案 A が安全である。

---

## 3.10 sibling を保存すると導出結果と競合する

### 発生条件

storage-facing enum に `sibling` を含めたり、兄弟姉妹関係を relationship row として保存する。

### 例

```text
保存済み:
  A sibling B

親子関係からの導出:
  A と B は parent を共有しない
```

または逆に、

```text
A parent C
A parent D
```

から C と D は sibling と導出できるのに、保存済み sibling が存在しない。

### 影響

- 保存済み sibling と導出 sibling のどちらを正とするか曖昧になる
- 更新時に sibling row の同期が必要になる
- 親子関係変更時に derived row の削除漏れが起きる

### 対策

`sibling` は保存しない。

```text
保存する:
  parent
  adoptive_parent
  step_parent
  spouse
  partner
  cohabitant

保存しない:
  child
  sibling
  ancestor
  descendant
  grandparent
  grandchild
  uncle_aunt
  nephew_niece
  cousin
  in_law
```

---

## 3.11 `child` / `adoptive_child` / `step_child` を保存すると inverse と競合する

### 発生条件

`parent(A, B)` と `child(B, A)` の両方を保存する。

### 影響

- 片方だけ削除・更新された場合に矛盾する
- start_date / end_date がずれる
- notes が異なる場合、どちらが canonical fact か不明になる
- `tree_path` 更新が二重になる

### 対策

保存方向を canonical に固定する。

```text
parent side だけ保存する:
  parent(A, B)
  adoptive_parent(A, B)
  step_parent(A, B)

child view は read-side derived relation として返す。
```

---

## 3.12 symmetric 関係の向き違い

### 発生条件

次の両方が登録される。

```text
spouse(A, B)
spouse(B, A)
```

### 現状

現在の unique index は `LEAST(source_entity_id, target_entity_id)` と `GREATEST(...)` を使っており、同一ペアの active symmetric duplicate は防げる。

### 残る懸念

- アプリケーションコード上では向きの違いを毎回考慮する必要がある
- source / target に意味がないにもかかわらず、DTO 上は direction を持つ
- start_date / end_date が異なる overlapping spouse row は別種の競合として残る

### 対策

保存時に canonical orientation へ正規化する。

```text
spouse(min(A, B), max(A, B))
partner(min(A, B), max(A, B))
cohabitant(min(A, B), max(A, B))
```

read-side では必要に応じて双方向 edge を emit する。

---

## 3.13 overlapping spouse / partner の扱いが仕様化されていない

### 発生条件

同一人物が同時期に複数 spouse を持つ。

```text
A spouse B: 2020-01-01 .. NULL
A spouse C: 2021-01-01 .. NULL
```

### 影響

- 一夫一婦制を前提にする UI では破綻する
- in-law derivation が大量に増える
- family tree layout で spouse 配置が曖昧になる

### 対策

プロダクト仕様として選択する。

```text
案 A:
  overlapping spouse を禁止する

案 B:
  overlapping spouse を許可し、複数配偶者を表現できる UI にする
```

日本の一般的な戸籍・家系図表現に寄せるなら、初期実装では案 A が安全である。

サービス層で次を検証する。

```text
同一 diagram 内で、同一 person に対する active spouse period が overlap しない
```

partner / cohabitant については spouse より緩く扱うか、同様に overlap 制約をかけるかを別途決める。

---

## 3.14 実親・養親・継親の同時登録制約が曖昧

### 発生条件

同一 child に対して、複数の parent 種別が登録される。

```text
A parent B
A adoptive_parent B
A step_parent B
```

または、実親が 3 人以上登録される。

```text
A parent C
B parent C
D parent C
```

### 影響

- full / half sibling の分類が不安定になる
- 法的親・生物学的親・継親が混ざる
- UI で親ノードの配置が破綻する

### 対策

次の制約を仕様化する。

```text
同一 parent-child pair に対して、parent 系 kind は高々 1 種類にする。

biological parent は child ごとに最大 2 人までにする。
adoptive_parent は child ごとに最大 2 人までにするか、制約しないかを仕様で決める。
step_parent は spouse/partner/cohabitant から suggested に留めるか、explicit として許可するかを仕様で決める。
```

---

## 3.15 sibling classification が親情報不足で決められない

### 発生条件

親が 1 人しか登録されていない、または親情報が欠落している。

### 例

```text
A parent C
A parent D
```

C と D は少なくとも 1 人の parent を共有する。だが、もう片方の親が未登録なので full sibling か half sibling か確定できない。

### 影響

- full / half を断定すると誤る
- 兄弟姉妹であること自体は言えるが分類は unknown になる

### 対策

`sibling_kind` に unknown / partial を入れる。

```rust
enum SiblingKind {
    Full,
    Half,
    Adoptive,
    Step,
    UnknownSharedParentOnly,
}
```

または qualifier で表す。

```rust
KinshipQualifier::ParentSetIncomplete
```

---

## 3.16 gender / birth_date 不足で日本語ラベルが決められない

### 発生条件

`gender = unknown`、または `birth_date` が未入力。

### 影響

次が決められない。

- 兄 / 弟
- 姉 / 妹
- 叔父 / 叔母
- 甥 / 姪
- 父 / 母
- 祖父 / 祖母

### 対策

domain output では localized label を返さない。

```text
core domain:
  sibling
  parent
  uncle_aunt

presentation:
  gender / age / locale を見て 兄・姉・弟・妹 などに変換
```

情報が足りない場合は、汎用ラベルを返す。

```text
兄弟姉妹
親
祖父母
おじ・おば
```

---

## 3.17 複数の親族関係が同時に成立する

### 発生条件

現実の家系では、1 組の人物間に複数の関係が同時に成立し得る。

### 例

```text
A は B の cousin でもあり spouse でもある
A は B の adoptive sibling でもあり in-law でもある
A は B の ancestor だが、別経路では in-law でもある
A は B の uncle でもあり step-parent でもある
```

### 影響

単一ラベルを返す API では計算不能または恣意的選択になる。

### 対策

`Vec<KinshipRelation>` を返す。

```rust
struct KinshipDerivation {
    relations: Vec<KinshipRelation>,
    diagnostics: Vec<KinshipDiagnostic>,
}
```

UI で primary label が必要な場合は、別途 ranking policy を定義する。

```rust
fn choose_primary_label(relations: &[KinshipRelation]) -> DisplayKinshipLabel;
```

このとき証明できるのは「必ず 1 つ選べる」ことであり、「唯一正しい親族関係である」ことではない。

---

## 3.18 suggested relationship を canonical fact と混ぜる懸念

### 発生条件

shared children, co-residence, date overlap などから step relationship や spouse を推論する。

### 例

```text
A parent C
B parent C
```

ここから A spouse B を推論すると、未婚・離婚・認知・養育などの可能性を無視することになる。

### 影響

- 事実ではない relationship を canonical fact のように扱ってしまう
- UI が「確定した関係」と「提案」を混同する
- tree_path や kinship derivation に誤った edge が混ざる

### 対策

`Suggested` は explicit / derived と明確に分ける。

```rust
enum FamilyTreeRelationshipSource {
    Explicit,
    Derived,
    Suggested,
}
```

suggested は既定では tree_path に反映しない。

---

## 3.19 relationship と tree_path の二重管理による同期漏れ

### 発生条件

relationship 作成・更新・削除と tree_path 更新が別処理になる。

### 影響

- relationship はあるが tree_path がない
- relationship は削除済みだが tree_path が残る
- tree_path の depth が古い
- parent 種別変更後に closure が更新されない

### 対策

更新ルールを一本化する。

```text
relationship を変更できるのは RelationshipService のみ
tree_path を直接変更できるのは TreePathMaintenanceService のみ
同一 transaction 内で relationship 更新と tree_path 再構築を行う
```

ただし責務上、`RelationshipService` が `KinshipDerivationService` を呼ぶべきではない。

---

## 3.20 差分更新方式の削除バグ

### 発生条件

parent edge 削除時に、その edge に依存する descendant path だけを差分削除する。

### 例

```text
A -> B -> D
A -> C -> D
```

`A -> B` を削除しても、`A -> D` は `A -> C -> D` 経由で残るべきである。

単純に `A` の ancestors と `B` の descendants の組み合わせを削除すると、本来残る path まで消える。

### 影響

- ancestor / descendant が欠落する
- root detection が誤る
- 中心人物 view が不完全になる

### 対策

初期実装では、差分更新ではなく diagram 単位再構築にする。

```text
1. relationship を変更する
2. 対象 diagram の tree_path を全削除する
3. active lineage relationships から closure を再計算する
4. self path を含めて insert する
5. commit する
```

規模が大きくなってから差分更新を導入する。

---

## 3.21 transaction isolation と並行更新

### 発生条件

同じ diagram に対して複数ユーザーが同時に relationship を更新する。

### 例

```text
Tx1: A parent B を追加
Tx2: B parent A を追加
```

それぞれの transaction が相手の未 commit edge を見ないと、両方 commit されて循環が発生し得る。

### 影響

- write-side validation を通過したのに commit 後に不変条件が壊れる
- tree_path が循環を含む

### 対策

diagram 単位で advisory lock を取る。

```sql
SELECT pg_advisory_xact_lock($diagram_id);
```

relationship 更新と tree_path 再構築は同一 transaction で行う。

---

## 3.22 world / diagram / entity の責務境界が曖昧になる

### 発生条件

`entity` は `world_id` を持ち、`diagram_entity` で diagram に所属する。relationship は diagram scoped。

### 懸念

- entity は world に属するのか diagram に属するのか
- 同じ entity を複数 diagram で共有するのか
- tree_path は diagram scoped なのか world scoped なのか
- genealogy overview では diagram 間の relationship をどう扱うのか

### 影響

- tree_path の scope が曖昧になる
- diagram merge / overview で同一人物リンクと親族 relation が混ざる
- relationship の diagram_id と entity.world_id の整合性チェックが必要になる

### 対策

初期実装では次を明文化する。

```text
entity は world に属する。
diagram_entity は diagram への表示・編集 scope を表す。
relationship は diagram scoped な explicit fact である。
tree_path は diagram scoped な derived cache である。
world scoped な overview は別 read model とする。
```

---

## 3.23 frontend 側の RelationshipKind と backend storage enum がずれる

### 発生条件

フロントエンド型に `child`, `sibling`, `adoptive_child`, `step_child`, `ex_spouse` が含まれる一方、現在の DB enum はそれらを保存対象にしていない。

### 影響

- FE から保存できない kind が送信される
- backend API が rejected kind を返す
- storage-facing enum と read-facing enum が混ざる

### 対策

型を分ける。

```typescript
type StoredRelationshipKind =
  | 'parent'
  | 'adoptive_parent'
  | 'step_parent'
  | 'spouse'
  | 'partner'
  | 'cohabitant';

type KinshipRelationKind =
  | StoredRelationshipKind
  | 'child'
  | 'adoptive_child'
  | 'step_child'
  | 'sibling'
  | 'ancestor'
  | 'descendant'
  | 'uncle_aunt'
  | 'nephew_niece'
  | 'cousin'
  | 'in_law';
```

---

## 4. データ競合として reject するべきケース一覧

## 4.1 必ず reject するケース

| ケース | 例 | 理由 | 推奨エラー |
|---|---|---|---|
| 自己関係 | `A parent A` | 意味が破綻する | 400 or 409 |
| 親子循環 | `A parent B`, `B parent A` | DAG 不変条件違反 | 409 |
| 祖先循環 | `A parent B`, `B parent C`, `C parent A` | closure が破綻 | 409 |
| diagram scope 外 entity | diagram 1 の relationship が diagram 2 の entity を参照 | scope 分離違反 | 400 or 409 |
| active duplicate directed | 同一 diagram, source, target, kind の重複 | 保存 fact 重複 | 409 |
| active duplicate symmetric | `spouse(A,B)` と `spouse(B,A)` | symmetric duplicate | 409 |
| start_date > end_date | `2026-01-01 .. 2025-01-01` | 期間矛盾 | 400 |
| deleted entity との relationship 作成 | 削除済み人物を参照 | active scope 違反 | 404 or 409 |

## 4.2 仕様判断が必要なケース

| ケース | 推奨初期方針 | 理由 |
|---|---|---|
| 同一 child に biological parent 3 人以上 | reject | 一般的な家系図表現を単純に保つ |
| 同一 pair に parent と adoptive_parent | reject | 分類が曖昧になる |
| 同一 pair に parent と step_parent | reject | 実親と継親が同時成立しにくい |
| overlapping spouse | reject | 初期 UI を単純にする |
| overlapping partner | allow or warn | spouse より緩い可能性がある |
| cohabitant 複数 | allow or warn | 実態として複数同居はあり得る |
| shared child から spouse 推論 | suggested only | 過剰推論を避ける |
| co-residence から spouse 推論 | suggested only | 過剰推論を避ける |
| step_parent の tree_path 反映 | 初期は反映しない | lineage closure の意味を保つ |

---

## 5. 親族計算で「計算不能」をなくすための型設計

## 5.1 推奨 output

```rust
pub struct KinshipDerivation {
    pub relations: Vec<KinshipRelation>,
    pub diagnostics: Vec<KinshipDiagnostic>,
}

pub struct KinshipRelation {
    pub from_entity_id: EntityId,
    pub to_entity_id: EntityId,
    pub kind: KinshipRelationKind,
    pub source: KinshipRelationSource,
    pub generation_distance: Option<i32>,
    pub explicit_relationship_id: Option<RelationshipId>,
    pub qualifiers: Vec<KinshipQualifier>,
}

pub enum KinshipRelationSource {
    Explicit,
    Derived,
    Suggested,
}

pub enum KinshipQualifier {
    GenderUnknown,
    BirthDateUnknown,
    BirthOrderUnknown,
    ParentSetIncomplete,
    MultiplePaths,
    InferredFromShortestPath,
    DateAmbiguous,
}

pub enum KinshipDiagnostic {
    RelationshipOutsideActiveScope,
    TreePathOutOfSync,
    CycleDetected,
    DuplicateCanonicalFact,
    InconsistentDateRange,
    MissingSelfPath,
}
```

## 5.2 意味

| 状態 | 返し方 |
|---|---|
| 関係がない | `relations = []` |
| 関係がある | `relations` に 1 件以上 |
| 複数関係がある | `relations` に複数件 |
| 情報不足 | `qualifiers` に Unknown 系を付ける |
| 複数経路 | `MultiplePaths` を付ける |
| データ不整合 | `diagnostics` に入れる |
| 表示ラベル未確定 | domain では未確定のまま返し、presentation で処理 |

これにより、関数の戻り値として「計算不能」を表現する必要がなくなる。

---

## 6. 証明可能性

## 6.1 現在の schema のまま証明できない理由

現在の schema のままでは、次の理由で「計算できない場合がない」ことは証明できない。

1. `relationship` は `diagram_id` を持つが、`tree_path` は持たない
2. `relationship` は期間を持つが、`tree_path` は期間を持たない
3. `relationship` は親種別を持つが、`tree_path` は種別を持たない
4. `tree_path` は同一 ancestor / descendant に depth を 1 つしか持てない
5. `relationship.source_entity_id` / `target_entity_id` が同一 diagram 内 entity である DB 制約が弱い
6. `tree_path` が relationship から再構築可能な cache であると明文化されていない
7. `tree_path` の自己参照・depth 不変条件が DB 制約で守られていない
8. 並行更新時に循環チェックがすり抜ける可能性がある
9. FE / BE / DB の relationship kind がずれている

## 6.2 条件付きで証明できる命題

次の命題なら証明できる。

> 任意の有効な diagram と任意の active person pair `(A, B)` について、`derive_kinship(A, B)` は必ず停止し、`Vec<KinshipRelation>` または空集合を返す。情報不足や複数候補は qualifier / diagnostics として表現され、panic や計算不能状態にはならない。

## 6.3 必要な不変条件

```text
I1. すべての relationship は同一 diagram 内の active entity 同士を結ぶ。
I2. active relationship は自己関係を持たない。
I3. parent / adoptive_parent / step_parent は有向 canonical fact である。
I4. spouse / partner / cohabitant は無向 canonical fact として保存時に正規化される。
I5. tree_path の対象とする lineage graph は DAG である。
I6. tree_path は active lineage edge から計算された transitive closure である。
I7. tree_path は relationship から再構築可能であり、手編集されない。
I8. derived relationship は relationship table に保存しない。
I9. as_of 指定がある場合、active relationship は日付条件で決まる。
I10. gender / birth_date など presentation 情報が欠けても Unknown として扱う。
I11. 親族導出は有限集合上の探索・join・set 演算だけで構成する。
I12. relationship 更新と tree_path 再構築は同一 transaction で実行する。
I13. 同一 diagram に対する relationship 更新は transaction-level lock で直列化する。
```

## 6.4 証明の骨子

### 停止性

- diagram 内の active entity 数は有限である
- relationship 数は有限である
- tree_path 数は有限である
- 導出アルゴリズムは有限集合に対する探索・join・集合演算である
- 再帰探索を使う場合も visited set を持つ

したがって必ず停止する。

### 健全性

各 derived relation は保存済み explicit fact または closure に対応する導出規則からのみ生成する。

例:

```text
Parent(A, B):
  relationship(parent, A, B) が存在する場合のみ emit

Child(B, A):
  relationship(parent, A, B) が存在する場合のみ emit

Sibling(A, B):
  A と B が少なくとも 1 人の parent を共有する場合のみ emit

Ancestor(A, B):
  tree_path(A, B, depth > 0) が存在する場合のみ emit
```

したがって、返した関係は必ず根拠を持つ。

### 完全性

仕様で定義した範囲に限れば、全 person pair に対して導出規則を適用することで漏れなく列挙できる。

ただし、現実世界のすべての親族概念を完全に表すという意味ではない。

### tree_path 整合性

relationship 更新後、同一 transaction 内で対象 diagram の `tree_path` を relationship から再構築するなら、commit 後の `tree_path` は active lineage graph の closure である。

---

## 7. 推奨する schema 修正

## 7.1 `tree_path` に `diagram_id` を追加

```sql
DROP TABLE IF EXISTS tree_path;

CREATE TABLE tree_path (
    diagram_id BIGINT NOT NULL REFERENCES diagram(diagram_id) ON DELETE CASCADE,
    ancestor_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    descendant_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    depth INT NOT NULL CHECK (depth >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (diagram_id, ancestor_id, descendant_id),
    CONSTRAINT chk_tree_path_self_depth CHECK (
        (ancestor_id = descendant_id AND depth = 0)
        OR
        (ancestor_id <> descendant_id AND depth > 0)
    ),
    CONSTRAINT fk_tree_path_ancestor_diagram_entity
        FOREIGN KEY (diagram_id, ancestor_id)
        REFERENCES diagram_entity(diagram_id, entity_id),
    CONSTRAINT fk_tree_path_descendant_diagram_entity
        FOREIGN KEY (diagram_id, descendant_id)
        REFERENCES diagram_entity(diagram_id, entity_id)
);

CREATE INDEX idx_tree_path_diagram_ancestor
ON tree_path(diagram_id, ancestor_id);

CREATE INDEX idx_tree_path_diagram_descendant
ON tree_path(diagram_id, descendant_id);

CREATE INDEX idx_tree_path_diagram_depth
ON tree_path(diagram_id, depth);
```

## 7.2 `relationship` に diagram_entity 複合 FK を追加

```sql
ALTER TABLE relationship
ADD CONSTRAINT fk_relationship_source_diagram_entity
FOREIGN KEY (diagram_id, source_entity_id)
REFERENCES diagram_entity(diagram_id, entity_id);

ALTER TABLE relationship
ADD CONSTRAINT fk_relationship_target_diagram_entity
FOREIGN KEY (diagram_id, target_entity_id)
REFERENCES diagram_entity(diagram_id, entity_id);
```

## 7.3 active duplicate 制約は現状維持しつつ補完

現在の active directed duplicate / symmetric duplicate の unique index は維持する。

追加で、同一 pair の parent 系 kind 排他を検討する。

```sql
CREATE UNIQUE INDEX uq_relationship_parent_family_pair_active
ON relationship (
    diagram_id,
    source_entity_id,
    target_entity_id
)
WHERE deleted_at IS NULL
  AND kind IN ('parent', 'adoptive_parent', 'step_parent');
```

ただし、実親と養親を同一 pair に両方付けるユースケースを許可したい場合、この制約は入れない。

---

## 8. 推奨する service 分離

## 8.1 RelationshipService

責務:

```text
- stored relationship の create / update / delete
- write payload validation
- date range validation
- duplicate validation
- cycle validation
- diagram scope validation
- transaction 境界内で TreePathMaintenanceService を呼ぶ orchestration
```

責務ではない:

```text
- sibling / cousin / in-law の導出
- localized label の決定
- read-side graph expansion
```

## 8.2 TreePathMaintenanceService

責務:

```text
- relationship から tree_path を再構築する
- self path を作る
- DAG / closure integrity を検証する
- tree_path out-of-sync diagnostics を提供する
```

## 8.3 KinshipDerivationService

責務:

```text
- explicit relationship を canonical read graph に変換する
- inverse / symmetric relation を read-side に展開する
- tree_path を使って ancestor / descendant を導出する
- parent map から sibling を導出する
- higher-order kinship を導出する
- Explicit / Derived / Suggested を区別する
- Unknown / Ambiguous / MultiplePaths を qualifier として返す
```

責務ではない:

```text
- relationship row の保存
- tree_path の mutation
- HTTP DTO への mapping
- localized label の確定
```

## 8.4 FamilyTreeUsecase

責務:

```text
- diagram を load する
- family_tree diagram か検証する
- active person を load する
- explicit relationship を load する
- tree_path を load する
- KinshipDerivationService を呼ぶ
- endpoint response に assemble する
```

---

## 9. 推奨アルゴリズム

## 9.1 relationship 更新時

```text
begin transaction
  advisory lock by diagram_id

  validate diagram exists and active
  validate source/target are active entities in diagram
  validate no self relation
  validate date range
  canonicalize relationship orientation
  validate duplicate / overlap / parent constraints
  validate no lineage cycle

  insert/update/delete relationship

  delete tree_path where diagram_id = ?
  rebuild tree_path from active lineage relationships
  validate tree_path integrity

commit
```

## 9.2 tree_path 再構築

```text
input:
  active persons in diagram
  active lineage relationships in diagram

steps:
  1. insert self path for all active persons
  2. build adjacency from parent/adoptive_parent only
  3. assert graph is DAG
  4. for each node, BFS/DFS to descendants
  5. insert shortest depth per ancestor/descendant
```

`step_parent` を closure に含めるかは仕様で決める。初期推奨は含めない。

## 9.3 kinship derivation

```text
input:
  active_person_ids
  explicit_relationships
  lineage_paths

steps:
  1. canonicalize explicit relationships
  2. emit explicit relations
  3. emit inverse child relations
  4. emit symmetric spouse/partner/cohabitant relations
  5. emit ancestor/descendant from tree_path
  6. derive siblings from parent map
  7. derive uncle/aunt, nephew/niece, cousin from parent + sibling
  8. derive in-law from spouse/partner/cohabitant + parent
  9. attach qualifiers
  10. return relations + diagnostics
```

---

## 10. テスト観点

## 10.1 unit tests

- self relationship は拒否される
- directed duplicate は拒否される
- symmetric duplicate は向き違いでも拒否される
- parent cycle は拒否される
- start_date > end_date は拒否される
- diagram scope 外 entity は拒否される
- parent 追加で tree_path が再構築される
- parent 削除で tree_path が再構築される
- sibling は parent map から導出される
- derived-only kind は保存 API で拒否される

## 10.2 property-based tests

生成する graph:

```text
- finite persons
- DAG parent edges
- symmetric spouse edges
- optional missing gender / birth_date
- optional deleted relationship
```

確認する property:

```text
- tree_path は self path を持つ
- tree_path.depth は非負
- ancestor_id = descendant_id iff depth = 0
- tree_path は relationship graph の closure と一致する
- relationship graph に cycle がない
- derive_kinship は panic しない
- derive_kinship は finite result を返す
- derived relation は explicit fact または closure に根拠を持つ
```

## 10.3 integration tests

- 複数 diagram で同じ entity を使っても tree_path が混ざらない
- relationship 更新後、同じ transaction 終了後に tree_path が正しい
- deleted entity / relationship が read model に出ない
- concurrent update で cycle が作れない
- as_of 指定時に tree_path cache ではなく active relationship から計算される

## 10.4 diagnostics tests

- tree_path に余分な path があると `TreePathOutOfSync`
- self path が欠けると `MissingSelfPath`
- relationship が scope 外 entity を参照すると `RelationshipOutsideActiveScope`
- parent graph に cycle があると `CycleDetected`

---

## 11. 実装優先度

## Priority 1: 必須

1. `tree_path` に `diagram_id` を追加する
2. `relationship` の source / target が diagram 内 entity であることを保証する
3. `tree_path` を relationship から再構築可能な cache と明文化する
4. relationship 更新と tree_path 再構築を同一 transaction にする
5. diagram 単位の advisory lock を入れる
6. derived-only kind を storage-facing enum から排除する
7. `KinshipDerivation` を単一ラベルではなく `Vec<KinshipRelation>` にする

## Priority 2: 強く推奨

1. parent 系 kind の同一 pair 排他を決める
2. biological parent 最大 2 人制約を決める
3. overlapping spouse の扱いを決める
4. step_parent を tree_path に含めるか決める
5. sibling classification に Unknown / ParentSetIncomplete を導入する
6. diagnostics を導入する

## Priority 3: 将来対応

1. as-of mode 用の on-the-fly closure
2. tree_path_history / period closure
3. all paths の保持または multiple paths diagnostics 強化
4. genealogy overview 用の world-scoped read model
5. localized kinship label ranking policy

---

## 12. 最終判断

現在の `relationship + tree_path` 構成は、到達性の高速化には有効である。

しかし、`tree_path` を source of truth として扱うと、次の情報を失う。

```text
- diagram scope
- relationship kind
- date range
- multiple paths
- explicit / derived / suggested の区別
- biological / adoptive / step の区別
```

したがって、最終方針は次にするべきである。

```text
relationship:
  正規の保存済み explicit fact

tree_path:
  diagram scoped な lineage reachability cache
  relationship から再構築可能
  direct な親族分類には使わない

KinshipDerivationService:
  relationship + tree_path から read-only に親族関係を導出する
  計算不能を返さず、Unknown / Ambiguous / Multiple / Diagnostics を型で表す

Presentation layer:
  gender / age / locale / ranking policy を使って最終ラベルを選ぶ
```

この方針なら、少なくともアプリケーションで定義した親族関係について、次を主張できる。

> 有効な保存状態に対して、親族導出は常に停止し、計算不能ではなく「関係なし」「関係あり」「情報不足」「複数関係」「データ診断」のいずれかを返す。

ただし、現実世界のすべての親族呼称を唯一に決めることはできない。唯一ラベルが必要な場合は、親族導出とは別に presentation ranking として扱う。
