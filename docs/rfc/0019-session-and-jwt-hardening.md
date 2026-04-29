# RFC 0019: Session and JWT Hardening

- 状態: `下書き`
- 最終更新: `2026-04-29`

## 背景

現在の login API は JWT を発行し、同じ token を response body と `HttpOnly` cookie の両方で返している。cookie は `SameSite=Lax` と `HttpOnly` を持つが、`Secure` は設定されていない。また、JWT expiration は handler 内で 60 分に固定され、`JWT_EXPIRED_IN` / `JWT_MAXAGE` configuration と実際の発行処理が揃っていない。

logout は cookie を expire するだけで、server-side revocation はない。そのため、response body や local storage 等に保存された bearer token は expiration まで有効である。

## 問題

- production HTTPS でも cookie に `Secure` が付かないと、誤った HTTP 経路で cookie が送信され得る。
- token を response body に返すと、frontend が local storage 等へ保存しやすくなり、XSS 時の被害が大きくなる。
- cookie と bearer token の二重運用は、どちらを正とするかが曖昧になり、logout / rotation / revocation policy が複雑になる。
- JWT が stateless なため、logout 後も bearer token が expiration まで使える。
- `JWT_EXPIRED_IN` / `JWT_MAXAGE` が設定値として存在するが、発行処理では 1 hour 固定になっている。
- `JWT_SECRET` の長さやentropyを検証していない。

## 目標

- cookie / bearer token のどちらを主要 session transport にするか決める。
- production cookie に `Secure`, `HttpOnly`, `SameSite` policy を明示する。
- JWT expiration と cookie max-age を configuration に揃える。
- logout / token rotation / revocation の方針を決める。
- weak secret configuration を起動時に検出する。

## 非目標

- OAuth / OIDC provider の選定。
- refresh token flow の詳細実装。
- multi-device session management UI。

## 推奨方針

初期方針:

- Browser frontend では `HttpOnly` cookie を primary transport にする。
- Login response body に access token を返す必要がない場合は削除する。
- Mobile / CLI など bearer token が必要な client は別 endpoint または明示的な mode で扱う。
- `RUNTIME_MODE=release` では cookie に `Secure` を必須で付与する。
- `JWT_EXPIRED_IN` / `JWT_MAXAGE` を実際の `exp` と cookie `max_age` に使う。
- `JWT_SECRET` は最低長とentropy要件を起動時 validation に含める。

cookie policy:

```text
debug:   HttpOnly; SameSite=Lax
release: HttpOnly; Secure; SameSite=Lax
```

cross-site frontend を許可する場合は、`SameSite=None; Secure` と CSRF 対策を同時に導入する。

## Revocation

短期では access token lifetime を短くし、logout は cookie clear のみでもよい。ただし bearer token を response body に返す場合、logout の意味が弱くなることを API contract に明記する。

中期では次のどちらかを採用する。

- server-side session table を導入し、session id を cookie に入れる。
- JWT に `jti` を入れ、revoked token / session version を検証する。

## Test Plan

- release mode の login cookie に `Secure` が付く。
- debug mode の login cookie policy が documented behavior と一致する。
- configured expiration が JWT `exp` と cookie max-age に反映される。
- weak `JWT_SECRET` で起動 validation が失敗する。
- token body return を廃止する場合、login response schema が token を含まない。

## 未解決事項

- Browser API で bearer token support を継続するか。
- server-side session table を導入するか、JWT revocation list を導入するか。
- refresh token を導入するか。
