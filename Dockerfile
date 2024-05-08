FROM rust:bookworm as builder
WORKDIR /usr/src/app
# see: https://github.com/launchbadge/sqlx/blob/v0.7.4/sqlx-cli/README.md#force-building-in-offline-mode
ENV SQLX_OFFLINE=true
COPY ./.sqlx ./.sqlx
# build dependencies
COPY Cargo.toml Cargo.lock ./
COPY creation-adapter/Cargo.toml ./creation-adapter/Cargo.toml
RUN mkdir creation-adapter/src && echo 'fn main() {}' > creation-adapter/src/main.rs
COPY creation-driver/Cargo.toml ./creation-driver/Cargo.toml
RUN mkdir creation-driver/src && echo 'fn main() {}' > creation-driver/src/main.rs
COPY creation-service/Cargo.toml ./creation-service/Cargo.toml
RUN mkdir creation-service/src && echo 'fn main() {}' > creation-service/src/main.rs
COPY creation-usecase/Cargo.toml ./creation-usecase/Cargo.toml
RUN mkdir creation-usecase/src && echo 'fn main() {}' > creation-usecase/src/main.rs
RUN cargo build --release
RUN rm -rf creation-adapter creation-driver creation-service creation-usecase

# build source
COPY creation-adapter ./creation-adapter
COPY creation-driver ./creation-driver
COPY creation-service ./creation-service
COPY creation-usecase ./creation-usecase
# break the Cargo cache
RUN touch creation-driver/src/bin/main.rs
RUN cargo build --release

FROM debian:bookworm-slim as run
RUN apt-get update && apt install -y openssl && rm -rf /var/lib/apt/lists/* && apt-get clean
COPY --from=builder /usr/src/app/target/release/main /usr/local/bin

ENTRYPOINT [ "/usr/local/bin/main" ]