# Workspace Directory Structure

Last updated: 2026-02-18

This document is the first-pass directory map for the repository.

## Source and application layers

```text
.
├── creation-driver/              # HTTP API layer (Axum)
│   ├── src/
│   │   ├── bin/                  # entrypoints
│   │   ├── handler/              # request handlers
│   │   ├── middleware/           # middleware setup
│   │   └── route/                # router composition
│   └── tests/
│       ├── common/               # shared test helpers
│       └── fixtures/             # SQL fixtures
├── creation-usecase/             # use case layer
│   └── src/
│       └── usecase/
├── creation-service/             # domain service layer
│   └── src/
│       ├── model/
│       ├── repository/
│       └── service/
├── creation-adapter/             # DB adapter/repository layer
│   ├── migrations/               # sqlx migrations
│   ├── src/
│   │   ├── model/
│   │   ├── persistence/
│   │   └── repository/
│   └── tests/
│       └── fixtures/
└── xtask/                        # developer tooling commands
    └── src/
```

## Workspace and infrastructure

```text
.
├── Cargo.toml
├── Cargo.lock
├── compose.yml
├── docker/
│   ├── backend/
│   ├── backend_debug/
│   └── pgAdmin/
├── .github/
│   └── workflows/
├── .cargo/
├── .sqlx/
└── .vscode/
```

## Generated directories

```text
target/
target_debug/
```

Notes:
- `target/` and `target_debug/` are build artifacts.
- `.git/` and other VCS internals are intentionally omitted.
