# RFC 0023: Request Size and Public Docs Hardening

- 状態: `下書き`
- 最終更新: `2026-04-29`

## 背景

現在の API は JSON request body size limit を明示していない。また、`/swagger-ui` と `/api-docs/openapi.json` は公開 route として提供され、Swagger UI は `https://unpkg.com/swagger-ui-dist@5/...` から script / stylesheet を読み込む。

## 問題

- request body size limit がないと、大きな JSON body による memory / CPU DoS を受けやすい。
- password hashing endpoint など CPU cost の高い処理は、body limit / rate limit と組み合わせる必要がある。
- public OpenAPI は未公開 endpoint や schema detail を広く公開する可能性がある。
- Swagger UI が外部 CDN script に依存しており、supply-chain / availability / CSP の観点でproduction exposureを検討する必要がある。
- CDN asset が major version range (`@5`) で読み込まれており、厳密なpinningではない。

## 目標

- API request body size limit を定義する。
- public docs exposure policy を定義する。
- Swagger UI asset の supply-chain boundary を明確にする。

## 非目標

- API gateway / WAF provider の選定。
- OpenAPI の内容整理全体。

## 推奨方針

Request size:

- global JSON body limit を設定する。
- endpoint ごとに必要なら個別 limit を設定する。
- oversized request は `413 PAYLOAD TOO LARGE` を返す。
- auth endpoints は rate limit と併用する。

Docs:

- production で `/swagger-ui` を公開するかを config で制御する。
- `/api-docs/openapi.json` は公開してよいか、認証必須にするかを決める。
- production公開する場合、OpenAPI にinternal-only情報が含まれないことをreviewする。

Assets:

- Swagger UI assets は self-host するか、versionを厳密にpinningする。
- 外部 CDN を使う場合は SRI / CSP を検討する。
- CSP header を導入し、script / style の許可元を明示する。

## Test Plan

- configured size を超える JSON request が `413` になる。
- normal request は既存通り処理される。
- production config で `/swagger-ui` が無効化できる。
- docs endpoint を認証必須にする場合、未認証 request が `401` になる。
- Swagger UI assets がself-hostまたは厳密pinningされている。

## 未解決事項

- production で OpenAPI を公開するか。
- docs endpoint を管理者のみ閲覧可能にするか。
- body limit の初期値をいくつにするか。
