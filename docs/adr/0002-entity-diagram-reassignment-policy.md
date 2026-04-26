# ADR 0002: Entity の Diagram 再割り当て方針

- 状態: `下書き`
- 最終更新: `2026-03-14`

## 背景

`update_entity` は現在 `diagram_id` を受け取る。これは汎用 update によって entity を diagram 間で移動できることを意味する。

この選択は中立ではない。次の table は diagram scope を持つ。

- `relationship.diagram_id`
- `tree_path`
- 将来追加される diagram 単位の制約

明示的な方針なしに再割り当てを許可すると、静かな data inconsistency を起こしやすい。

## 選択肢

### Option A: 汎用 update で diagram 間移動を許可する

利点:

- endpoint が少なくて済む
- client の操作は単純

欠点:

- relationship と tree path の cascade policy が不明確
- 誤操作で data を移動しやすい

### Option B: 汎用 update では再割り当てを禁止する

- `diagram_id` は create 後に固定する
- entity の移動は将来専用 workflow を追加する場合だけ扱う

利点:

- 安全な default
- 汎用 update の意味が単純
- relationship / `tree_path` の整合性を壊しにくい

欠点:

- 本当に移動が必要な場合は別設計が必要

### Option C: 条件付きで再割り当てを許可する

- relationship がない場合だけ移動を許可する
- または server が関連 relationship / tree_path を一括更新する

利点:

- 柔軟

欠点:

- ルールが複雑
- API 利用者から結果を予測しにくい

## 決定

Option B を採用する。

汎用 entity update では `diagram_id` の変更を許可しない。移動が必要な場合は、relationship と tree_path を含む専用 workflow を別途設計する。

## 影響

- update payload に `diagram_id` が残る場合でも、既存所属 diagram と一致することを validation する
- diagram 間移動の cascade はこの API では扱わない
- relationship と `tree_path` の diagram-scoped invariant を守りやすくなる
