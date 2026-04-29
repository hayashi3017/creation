# RFC 0021: Auth Abuse and Account Enumeration

- 状態: `下書き`
- 最終更新: `2026-04-29`

## 背景

現在の register / login API には rate limit、lockout、progressive delay がない。login failure は user missing と password mismatch で内部的に区別され、response message も `Invalid email or password` と `Wrong password` で差がある。register は duplicate email を `409` と明示している。

## 問題

- login に rate limit がないため、password spraying / brute force を受けやすい。
- login failure message の差により、email の存在確認に使われる可能性がある。
- register duplicate response により、登録済み email の列挙ができる。
- password policy / breached password check / minimum length が明文化されていない。
- repeated failed login の audit trail がない。

## 目標

- auth endpoint の abuse protection を定義する。
- user enumeration を抑える response policy を決める。
- password policy と audit event の最小要件を定義する。

## 非目標

- CAPTCHA provider の選定。
- account recovery / password reset flow の設計。
- MFA の詳細設計。

## 推奨方針

Login:

- user missing と wrong password は同じ status / message にする。
- IP + email hash をkeyに rate limit をかける。
- failed attempt を audit event として記録する。
- threshold 超過時は `429 TOO MANY REQUESTS` を返す。

Register:

- public signup を続ける場合、duplicate email response の扱いを決める。
- enumeration を避けるなら、登録済みでも同じ success-like response を返し、email verification flow に寄せる。
- 現行の `409` を維持する場合は、公開signupでemail列挙が可能であることを明示的に受け入れる。

Password:

- 最低文字数を設定する。
- 最大長も設定し、hashing cost DoS を避ける。
- breached password check を採用するか検討する。

## Test Plan

- missing user と wrong password が同じ response body / status になる。
- login failure が一定回数を超えると `429` になる。
- rate limit は成功 login または時間経過で解除される。
- register duplicate response policy が documented behavior と一致する。
- password が短すぎる場合に reject される。

## 未解決事項

- public signup を許可するか、招待制にするか。
- CAPTCHA / bot protection を導入するか。
- audit log を DB に保存するか、外部 logging service に送るか。
