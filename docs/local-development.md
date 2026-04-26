# Local Development

Last updated: 2026-04-26

## Goal

This guide documents the local bootstrap and test flow that CI follows as closely as possible.

## Prerequisites

- Rust stable toolchain
- Docker with `docker compose`
- `sqlx-cli` with PostgreSQL support:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

- A local `.env` with at least:
  - `DATABASE_URL`
  - `JWT_SECRET`
  - `JWT_EXPIRED_IN`
  - `JWT_MAXAGE`
  - `RUNTIME_MODE`

Notes:

- `cargo run -p xtask -- docker` uses `.env.docker` for the container-side settings.
- `cargo run -p xtask -- migrate`, `cargo run -p xtask -- test`, and `cargo run -p xtask -- sqlx-prepare` read `.env`.
- Host-side commands should point `.env` `DATABASE_URL` at `localhost` and disable local PostgreSQL SSL (for example `postgres://...@localhost:5432/creation?sslmode=disable`). `host.docker.internal` is a container-oriented hostname and can fail from local `cargo run` / `cargo test`.
- `xtask` normalizes `host.docker.internal` to `localhost` and appends `sslmode=disable` for host-side commands, but direct `sqlx` commands should use the explicit local URL.

## Bootstrap

1. Start PostgreSQL:

```bash
cargo run -p xtask -- docker
```

2. Apply migrations:

```bash
cargo run -p xtask -- migrate
```

## Database Reset

このプロジェクトはまだリリース前のため、migration を読みやすい単一の初期 schema に統合することがある。その場合は既存 DB に差分 migration を当てるのではなく、DB を削除して作り直してから migration を適用する。

通常は次のコマンドを使う。

```bash
cargo run -p xtask -- migrate
```

`xtask migrate` は `.env` の `DATABASE_URL` を読み込み、次を順に実行する。

1. `sqlx database drop -y`
2. `sqlx database create`
3. `sqlx migrate run --source creation-adapter/migrations`

Docker volume も含めて PostgreSQL の永続データを完全に消したい場合だけ、次を使う。

```bash
docker compose down -v
cargo run -p xtask -- docker
cargo run -p xtask -- migrate
```

`docker compose down -v` は `creation_db` volume を削除する破壊的操作であり、ローカルの DB データは復元できない。共有環境や必要な検証データがある環境では実行しない。

## Test Commands

Full workspace test run:

```bash
cargo run -p xtask -- test
```

`xtask test` passes `--no-fail-fast`, so one failing test binary does not hide failures in the rest of the workspace.

Scoped examples:

```bash
cargo run -p xtask -- test -p creation-driver
cargo run -p xtask -- test -p creation-driver --test auth
cargo run -p xtask -- test -p creation-driver --test auth -- --nocapture
```

Convention-based aliases for common scopes:

```bash
cargo run -p xtask -- test-scope diagram
cargo run -p xtask -- test-scope entity auth
cargo run -p xtask -- test-scope package:xtask
cargo run -p xtask -- test-scope full
```

CI uses the same full-suite command after database bootstrap and migrations:

```bash
cargo run -p xtask -- test
```

## SQLx Cache

Refresh the workspace `.sqlx` cache after changing `query!` / `query_as!` usage:

```bash
cargo run -p xtask -- sqlx-prepare
```

Verify that the checked-in cache is up to date without rewriting it:

```bash
cargo run -p xtask -- sqlx-prepare --check
```

## Common Flow

Typical local cycle:

```bash
cargo run -p xtask -- docker
cargo run -p xtask -- migrate
cargo run -p xtask -- test
```
