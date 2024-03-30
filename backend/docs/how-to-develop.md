## How To Develop

### Prerequirements

- Install rustup, cargo, rustc


### Option: sqlx-cli

1. Install sqlx-cli

```bash
cargo install sqlx-cli
```

2. Using sqlx-cli

Initialize migration files.
```bash
sqlx migrate add -r init
```

Establish a connection to the DB using `DATABASE_URL` provided by `.env` file.
Executing the `up` migration file.
```bash
sqlx migrate run
```

Executing the `down` migration file.
```bash
sqlx migrate revert
```

[sqlx/sqlx-cli at main · launchbadge/sqlx](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)

### Option: cargo-watch

1. Install cargo-watch

```bash
cargo install cargo-watch
```
[cargo-watch - crates.io: Rust Package Registry](https://crates.io/crates/cargo-watch)
[watchexec/cargo-watch: Watches over your Cargo project's source.](https://github.com/watchexec/cargo-watch)

2. Execute cargo-watch

```bash
cargo watch -q -c -w src/ -x run
```
This command sets up cargo-watch to automatically build and run whenever changes are detected in the src directory.

### Option: Docker

Start and Stop docker containers
```bash
docker compose up -d
docker compose stop
```


FIXME: Running just Rust contianer
```bash
docker build -t creation_backend .
docker run --rm -p 8000:8000 creation_backend
```
[Compose file versions and upgrading | Docker Docs](https://docs.docker.com/compose/compose-file/compose-versioning/)
[docs/rust at master · docker-library/docs](https://github.com/docker-library/docs/tree/master/rust)
