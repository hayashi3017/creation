# RFC 0022: Error Response and Observability Boundary

- 状態: `下書き`
- 最終更新: `2026-04-29`

## 背景

現在の handler は repository / DB error を `format!("Database error: {}", err)` や `format!("Internal error: {}", err)` として client response に含める箇所がある。

これは開発時には便利だが、production では SQL、constraint 名、接続情報、内部型名、実装詳細を漏らす可能性がある。

## 問題

- internal error detail が API response に出る。
- error message が endpoint ごとにばらつき、client contract と security policy が不明瞭になる。
- DB constraint や query detail が攻撃者の入力調整に使われる可能性がある。
- request ID / trace ID が response と log に紐づいていないため、詳細を隠すと調査が難しくなる。

## 目標

- client-facing error と server log の境界を定義する。
- production response から internal detail を除外する。
- observability のための request ID / trace ID 方針を定義する。

## 非目標

- logging backend の選定。
- full audit log schema の設計。

## 推奨方針

Client response:

```json
{
  "status": "fail",
  "message": "Internal Server Error",
  "request_id": "..."
}
```

Server log:

- full error chain
- request ID
- actor user ID if authenticated
- endpoint / method
- resource IDs
- status code

production では internal error detail を response に含めない。debug mode でdetailを返す場合も、明示的な config にする。

## Error Mapping

- validation error: `400`, generic validation message
- unauthenticated: `401`
- unauthorized or invisible resource: `404` or `403` according to RFC 0018
- conflict: `409`, known business code only
- rate limit: `429`
- unexpected error: `500`, generic message

DB error の constraint 名を client に返す場合は、allowlist された business error code に変換する。

## Test Plan

- repository DB error が response body に raw DB message を含まない。
- response に request ID が含まれる。
- server log には request ID と error detail が出る。
- debug / release mode で error detail policy が切り替わる場合、その挙動をtestする。

## 未解決事項

- request ID を middleware で生成するか、upstream header を信頼するか。
- error code catalog を導入するか。
- structured logging のfield名をどう標準化するか。
