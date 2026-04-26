# RFC 0004: SQLx Workflow と Test Environment

- 状態: `採用`
- 最終更新: `2026-03-15`

## 背景

この repository には SQLx と DB-backed test に関する recurring workflow issue がある。

- `SQLX_OFFLINE=true` のため、新しい `query!` / `query_as!` を追加すると query cache refresh が必要
- `sqlx::test` は接続可能な setup database に依存するが、bootstrap 手順が十分にまとまっていない

## 提案

SQLx と DB-backed test workflow の入口を `xtask` に統一する。

### 推奨 command

- local DB 起動: `cargo run -p xtask -- docker`
- migration 実行: `cargo run -p xtask -- migrate`
- SQLx cache 更新: `cargo run -p xtask -- sqlx-prepare`
- scoped test 実行: `cargo run -p xtask -- test -p creation-driver --test entity`
- convention-based scope 実行: `cargo run -p xtask -- test-scope entity`

### CI 前提

- 接続可能な PostgreSQL instance を用意する
- DB-backed test 前に migration を実行する
- SQLx cache verification を専用 step として実行する
- `.env` / `DATABASE_URL` がない場合は早期に失敗させる

### ドキュメント更新

- required environment variables を 1 箇所にまとめる
- contributor 向けの最低限の local bootstrap order を明記する

## 理由

`xtask` は cross-device link failure を避ける SQLx prepare workaround など、project-specific behavior を既に含んでいる。raw cargo/sqlx command を複数箇所に書くと drift が起きやすい。

## 影響

- local / CI の test workflow が揃う
- SQLx offline cache の更新漏れを検知しやすくなる
- 新規 contributor の setup failure を減らせる
