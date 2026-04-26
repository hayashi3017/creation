# RFC 0001: Resource-Oriented API Shape

- 状態: `下書き`
- 最終更新: `2026-03-14`

## 背景

現在の Diagram / Entity endpoint には 2 つの不整合がある。

- update / delete が `POST /.../update` と `POST /.../delete` を使っている
- `GET /api/diagrams` と `GET /api/entities` が JSON body に依存している

これにより、client generation、cache、route discovery が必要以上に難しくなる。

## 提案

次の API revision で resource-oriented path に寄せる。

### Diagram リソース

- `GET /api/diagrams`
- `POST /api/diagrams`
- `PATCH /api/diagrams/:diagram_id`
- `DELETE /api/diagrams/:diagram_id`

### Entity リソース

- `GET /api/diagrams/:diagram_id/entities`
- `POST /api/diagrams/:diagram_id/entities`
- `PATCH /api/entities/:entity_id`
- `DELETE /api/entities/:entity_id`

### リクエスト形状

- `GET` では JSON body を送らない
- `diagram_id` のような親子 ownership は path parameter で表す
- `kind`、pagination、sort order などの filter は query parameter に予約する

## 移行方針

- 既存 action-style endpoint は互換期間だけ残す
- 新 endpoint を OpenAPI に追加する
- client 側の移行が済んだら旧 endpoint を削除する

## 影響

- HTTP method と resource の意味が揃う
- `GET` が cache と client tooling に乗りやすくなる
- path parameter 名は RFC 0010 の explicit id policy に合わせる

## 未解決事項

- 旧 endpoint の廃止時期
- API versioning を入れるか
- `GET /api/entities` のような横断 query を残すか
