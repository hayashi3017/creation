# Repository Guidelines

## Project Structure & Module Organization
- Workspace crates live at the repo root: `creation-driver/` (HTTP API + Axum handlers), `creation-service/` (domain services), `creation-usecase/` (use cases), `creation-adapter/` (DB/persistence + repositories), and `xtask/` (dev tooling).
- Database schema changes live in `creation-adapter/migrations/` (sqlx migrations).
- Integration-style tests are under `creation-driver/tests/` with fixtures in `creation-driver/tests/fixtures/`.
- Container tooling and images are defined in `compose.yml` and `docker/`.

## Build, Test, and Development Commands
- `cargo build` — builds the full workspace.
- `cargo run -p creation-driver` — runs the API server (reads config from `.env`).
- `cargo test` — runs all tests; for DB-backed tests, ensure `DATABASE_URL` is set.
- `cargo run -p xtask -- migrate` — drops/creates the database and runs migrations (requires `sqlx` CLI).
- `cargo run -p xtask -- migrate-info` — shows migration status.
- `cargo run -p xtask -- docker` — `docker compose up -d` with `.env.docker`.

## Coding Style & Naming Conventions
- Rust standard style: `snake_case` modules/functions, `CamelCase` types, `SCREAMING_SNAKE_CASE` constants.
- Use `cargo fmt` (rustfmt defaults) before committing. Prefer explicit error contexts with `anyhow::Context` or `thiserror`.
- Keep module boundaries aligned with the layered structure: driver → usecase → service → adapter.

## Testing Guidelines
- Tests use `sqlx::test` with fixtures, e.g. `creation-driver/tests/user.rs` and `creation-driver/tests/fixtures/user.sql`.
- Prefer focused API-level tests in `creation-driver/tests/` and keep fixtures minimal.
- Run: `cargo test -p creation-driver` for API tests; `cargo test` for full workspace.

## Commit & Pull Request Guidelines
- Commit messages follow a concise `type: summary` pattern (examples: `add: ...`, `fix: ...`, `chore: ...`).
- PRs should include: a short description, related issues, migration notes (if DB changes), and API/behavior changes. Add screenshots or curl examples when touching HTTP responses.

## Configuration Tips
- Local config is loaded via `.env` (see `dotenvy` usage in tests). Docker uses `.env.docker` (see `xtask` and `compose.yml`).
- Required env vars include `DATABASE_URL`, JWT settings, and Postgres credentials.
