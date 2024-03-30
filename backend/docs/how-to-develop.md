## How To Develop

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

### Docker

```bash
docker build -t creation_backend .
docker run --rm -p 8000:8000 creation_backend
```
[Compose file versions and upgrading | Docker Docs](https://docs.docker.com/compose/compose-file/compose-versioning/)
[docs/rust at master · docker-library/docs](https://github.com/docker-library/docs/tree/master/rust)
