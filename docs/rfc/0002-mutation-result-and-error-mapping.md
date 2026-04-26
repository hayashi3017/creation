# RFC 0002: Mutation Result と Error Mapping

- 状態: `採用`
- 最終更新: `2026-03-14`

## 背景

soft-delete を使う update/delete handler には現在 2 つの問題がある。

- 対象 ID に一致する active row がなくても `200 OK` を返す
- 一部の database failure が `400 Bad Request` に mapping される

これでは client が invalid input、missing resource、server failure を区別しにくい。

## 提案

Diagram / Entity の mutation 結果を次のように標準化する。

### HTTP ステータス対応

- `400 Bad Request`: request 形式不正または validation failure
- `404 Not Found`: 対象 row が存在しない、または既に soft-delete 済み
- `409 Conflict`: unsupported state transition などの business rule violation
- `500 Internal Server Error`: database または infrastructure failure

### Repository / service behavior

- update/delete は `rows_affected()` を確認する
- `rows_affected() == 0` の場合、repository または service は typed `NotFound` error を返す
- validation error と database error は分離する

## 対象範囲

- `update_diagram`
- `delete_diagram`
- `update_entity`
- `delete_entity`

## 移行方針

- repository に `NotFound` error variant を追加する
- service/usecase/handler で typed error を HTTP status に mapping する
- 既存 test を `404` expectation に更新する

## 影響

- client は missing resource を正しく扱える
- `400` と `500` の意味が明確になる
- 同じ方針を Person / Relationship / User API にも適用しやすくなる
