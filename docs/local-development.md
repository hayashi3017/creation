# Local Development

Last updated: 2026-03-15

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
- Host-side commands should point `.env` `DATABASE_URL` at `localhost` (for example `postgres://...@localhost:5432/creation`). `host.docker.internal` is a container-oriented hostname and can fail from local `cargo run` / `cargo test`.

## Bootstrap

1. Start PostgreSQL:

```bash
cargo run -p xtask -- docker
```

2. Apply migrations:

```bash
cargo run -p xtask -- migrate
```

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
