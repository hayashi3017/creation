# RFC 0008: Genealogy Aggregate、Diagram Merge、Entity Visibility

- 状態: `下書き`
- 最終更新: `2026-03-28`

## 背景

現在の model では `diagram` が `entity`、`person`、`relationship`、`tree_path` の top-level container である。

これは単一 diagram の編集には十分だが、次の要件はまだ定義していない。

- 複数 family-tree diagram を 1 つの mergeable unit として管理する
- source row を新しい diagram に deep-copy せず merged family-tree view を描画する
- merge 設定後も source diagram の更新を merged view に反映する
- 共有対象 container の publication scope を表現する
- public output から人物を隠しつつ、visibility ownership を `entity` に置く
- 同じ実人物が diagram ごとに visible/hidden で食い違う状態を避ける
- query parameter ではなく JSON request body で centered view を要求する

既存の単一 diagram family-tree read API は維持するが、multi-diagram 要件には不足している。

## 目標

- 既存 CRUD の write unit として `diagram` を維持する
- diagram を merge / publication 用に group 化する上位 aggregate を追加する
- source update が merged read に自然に反映されるよう、deep-copy merge を避ける
- hidden state の ownership を `entity` に置く
- 同じ人物を表す linked entity の hidden state を一貫させる
- centered read request を JSON body で扱えるようにする

## 対象外

- 同一人物の自動 matching heuristic
- 最終的な ACL / collaborator 実装
- 既存 `GET /api/family-trees/{diagram_id}` の置き換え

## 命名案

候補:

- `genealogy`
- `family_tree_collection`
- `family_graph`
- `lineage_project`

推奨は `genealogy`。理由は diagram より上位の家系情報 container を表しやすく、merged read や publication scope を持たせやすいため。

## 提案 model

追加概念:

- `genealogy`: 複数 diagram を束ねる上位 aggregate
- `genealogy_diagram`: genealogy と source diagram の関連
- `entity_identity_link`: 同じ実人物を表す entity の link
- entity-level visibility: `entity` が public output に出るかを表す状態

`diagram` は引き続き編集単位であり、`genealogy` は merge / publication / visibility consistency の単位である。

## Merge の意味

deep-copy ではなく、source diagram を参照する read-time merge を採用する。

利点:

- source diagram の更新が merged view に反映される
- copy 後の drift を避けられる
- lineage / relationship の source provenance を保持しやすい

欠点:

- read-time assembly が複雑になる
- identity link と conflict resolution が必要になる

## 可視性ポリシー

hidden state は `entity` が所有する。

同じ実人物を表す entity が複数 diagram に存在する場合、link group 内で visibility rule を一貫させる。

例:

- linked entity のどれかが hidden なら public merged view では全て hidden にする
- または genealogy-level policy で override を明示する

初期方針では、安全側に倒して linked group の hidden を public output 全体に伝播させる。

## 中心人物指定の読み取り

centered view は JSON request body を使う。

例:

```http
POST /api/genealogies/{genealogy_id}/family-tree
```

```json
{
  "center_entity_id": 10,
  "ancestor_depth": 3,
  "descendant_depth": 2,
  "include_hidden": false
}
```

query parameter ではなく body にする理由:

- request option が増えても構造化しやすい
- depth、filter、visibility、layout hint をまとめて扱える
- cache key は後で policy として定義できる

## レイヤリング

- write は既存 diagram/person/relationship API を維持する
- genealogy usecase は source diagram の read と identity link を orchestrate する
- derivation service は merged graph の kinship semantics を扱う
- adapter は genealogy 関連 table の repository を追加する

## Migration / rollout

1. 単一 diagram family-tree API を維持する
2. `genealogy` と `genealogy_diagram` を追加する
3. read-only merged view を追加する
4. entity identity link を追加する
5. visibility propagation を実装する
6. publication scope / ACL を追加する

## 未解決事項

- genealogy の naming を最終決定するか
- linked entity の conflict resolution policy
- hidden state の override を許可するか
- merged view の sorting / layout policy
- publication scope と ACL の詳細
