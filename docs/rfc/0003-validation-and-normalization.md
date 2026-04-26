# RFC 0003: Validation と Normalization

- 状態: `採用`
- 最終更新: `2026-03-14`

## 背景

Diagram / Entity の validation は現在最小限である。

- `id` / `diagram_id` は 0 以外であること
- `name` は空文字列でないこと

whitespace-only name、最大長、description normalization、foreign key check は未定義に近い。

## 提案

write 時の共通 validation rule を定義する。

### 名前

- validation 前に前後の whitespace を trim する
- trim 後に空なら reject する
- DB column size と揃えた明示的な最大長を適用する

### 説明

- 前後の whitespace を trim する
- 空文字列は `NULL` に変換する
- multi-line text をそのまま保持するかを明文化する

### Diagram の検証

- 共通 name / description rule を適用する

### Entity の検証

- 共通 name / description rule を適用する
- create 前に target `diagram_id` が存在し、soft-delete されていないことを検証する
- update は [ADR 0002](../adr/0002-entity-diagram-reassignment-policy.md) に従って検証する

## 影響

- API 入力の揺れを service 層で吸収できる
- DB constraint violation を application-level validation error に寄せられる
- repository は normalize 済み payload を受け取る前提にできる

## 未解決事項

- description の最大長を設けるか
- locale-aware な name validation が必要か
- HTML / markdown / plain text の扱いをどこまで制限するか
