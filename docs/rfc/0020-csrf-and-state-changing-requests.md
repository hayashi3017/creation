# RFC 0020: CSRF and State-Changing Requests

- 状態: `下書き`
- 最終更新: `2026-04-29`

## 背景

現在の API は cookie authentication をサポートしている。cookie は browser が自動送信するため、state-changing request は CSRF の影響を受ける可能性がある。

現在は `SameSite=Lax` により多くの cross-site subrequest では cookie が送信されにくいが、CSRF boundary として十分な設計を明文化していない。また、logout は `GET /api/auth/logout` であり、GET request が state を変更している。

## 問題

- cookie auth を使う state-changing API に CSRF token / origin validation がない。
- `GET /api/auth/logout` は state-changing operation であり、link / image / top-level navigation などで意図せず実行され得る。
- 将来 cross-site frontend のために `SameSite=None` を使うと、CSRF risk が大きくなる。
- bearer token client と cookie client のCSRF要件が混在している。

## 目標

- state-changing API の CSRF boundary を定義する。
- logout を safe method から外す。
- cookie auth と bearer auth の違いを明確にする。
- CORS configuration と CSRF protection を混同しない。

## 非目標

- frontend form implementation の詳細。
- external WAF / CDN rules の選定。

## 推奨方針

- logout は `POST /api/auth/logout` に変更する。
- `POST` / `PATCH` / `DELETE` は cookie auth の場合に CSRF protection を要求する。
- CSRF token は double-submit cookie または server-side session token 方式を採用する。
- 少なくとも `Origin` / `Referer` validation を state-changing request に適用する。
- bearer token のみで認証された non-browser client には CSRF token を要求しない。

CSRF check は auth middleware の後、handler の前で共通 middleware として実装する。

## CORS との関係

CORS は browser が response を読める origin を制御する仕組みであり、CSRFを完全には防がない。`allow_credentials(true)` を使う場合は、allowed origins を production config で明示し、wildcard origin を使わない。

## Test Plan

- `GET /api/auth/logout` を廃止または 405 にする。
- `POST /api/auth/logout` が有効な CSRF token で成功する。
- cookie auth の `POST` / `PATCH` / `DELETE` が CSRF token なしで reject される。
- bearer token request は CSRF token なしでも許可される。
- invalid `Origin` の cookie request が reject される。

## 未解決事項

- double-submit cookie と server-side session token のどちらを採用するか。
- API-only clients と browser clients を header で明示的に分けるか。
- cross-site frontend を正式に許可するか。
