FROM rust:bookworm as builder
WORKDIR /usr/src/app
# see: https://github.com/launchbadge/sqlx/blob/v0.7.4/sqlx-cli/README.md#force-building-in-offline-mode
ENV SQLX_OFFLINE=true
COPY ./.sqlx ./.sqlx
# TODO: build only dependencies

COPY Cargo.toml Cargo.lock ./
COPY creation-adapter ./creation-adapter
COPY creation-driver ./creation-driver
COPY creation-service ./creation-service
COPY creation-usecase ./creation-usecase
# break the Cargo cache
RUN touch creation-driver/src/bin/main.rs
RUN cargo build --release

FROM debian:bookworm-slim as run
RUN apt-get update && apt install -y openssl && rm -rf /var/lib/apt/lists/* && apt-get clean
# `backend`はcrate_name
COPY --from=builder /usr/src/app/target/release/main /usr/local/bin

ENTRYPOINT [ "/usr/local/bin/main" ]