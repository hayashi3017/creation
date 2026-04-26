# ADR 0001: Entity-Person 書き込みモデル

- 状態: `下書き`
- 最終更新: `2026-03-15`

## 背景

`entity` の CRUD は実装済みだが、`person` テーブル固有の項目はまだ API から書き込めない。

現在の schema は次の構造になっている。

- `entity`: 汎用 node row
- `person`: `entity(kind='person')` の特化 row

この判断を先送りしすぎると、将来 API を追加するときに request / response の破壊的変更が必要になる可能性がある。

## 選択肢

### Option A: Entity と Person を別々に書き込む

- 汎用 entity endpoint は `entity` だけを管理する
- 別の person endpoint が `person` を管理する

利点:

- table ownership が明確
- 汎用 entity payload が単純

欠点:

- client が複数 write を調整する必要がある
- server 側で orchestration を入れない限り partial write のリスクがある

### Option B: kind 固有 payload を含む単一 transaction write model

- 汎用 entity create/update が共通項目を受け取る
- `kind = person` の場合、payload に nested `person` 詳細を含める
- server が両テーブルを 1 transaction で書き込む

利点:

- API 呼び出しが 1 回で済む
- `entity` と `person` の整合性を保ちやすい
- 将来の `entity_kind` 追加にも同じ形を使える

欠点:

- 汎用 endpoint が kind 固有 payload を知る
- validation と error mapping が少し複雑になる

### Option C: Person を aggregate resource として扱う

- public API は `/api/persons` を中心にする
- `person` create/update が `entity` 共通項目と `person` 詳細をまとめて受け取る
- 汎用 `entity` API は内部用または別用途として残す

利点:

- client から見た resource が明確
- `person` という business object と request shape が一致する
- aggregate write を 1 transaction に閉じ込めやすい

欠点:

- 汎用 `entity` API との責務境界を決める必要がある
- entity kind が増えると kind ごとの endpoint が増える

## 決定

public API では Option C を採用する。

`person` は `entity` と `person` specialization をまとめた aggregate resource として扱い、create/update/delete は usecase 層で orchestration する。

## 影響

- `person` API は `entity` 共通項目と `person` 詳細を同じ request で扱う
- repository は table-focused に保ち、transaction と手順は usecase/service boundary で管理する
- 将来別の specialization が増える場合も、同じ aggregate endpoint pattern を検討する
