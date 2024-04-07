FROM rust:bookworm as builder
# see: https://github.com/launchbadge/sqlx/blob/v0.7.4/sqlx-cli/README.md#force-building-in-offline-mode
ENV SQLX_OFFLINE=true
COPY ./.sqlx ./.sqlx
WORKDIR /usr/src/app
# build dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir ./src && echo 'fn main() {}' > ./src/main.rs
RUN cargo build --release
RUN rm -rf ./src
COPY ./src ./src
# break the Cargo cache
RUN touch ./src/main.rs
RUN cargo build --release

FROM debian:bookworm-slim as run
RUN apt-get update && apt install -y openssl && rm -rf /var/lib/apt/lists/* && apt-get clean
# `backend`はcrate_name
COPY --from=builder /usr/src/app/target/release/backend /usr/local/bin

ENTRYPOINT [ "/usr/local/bin/backend" ]