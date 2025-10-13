FROM rust:1.85
WORKDIR /usr/src/app
# see: https://github.com/launchbadge/sqlx/blob/v0.7.4/sqlx-cli/README.md#force-building-in-offline-mode
ENV SQLX_OFFLINE=true
COPY ./.sqlx ./.sqlx
# debug tool
RUN cargo install systemfd watchexec-cli
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
COPY xtask/Cargo.toml ./xtask/Cargo.toml
RUN mkdir xtask/src && echo 'fn main() {}' > xtask/src/main.rs
RUN cargo build
RUN rm -rf creation-adapter creation-driver creation-service creation-usecase xtask

# build source
COPY creation-adapter ./creation-adapter
COPY creation-driver ./creation-driver
COPY creation-service ./creation-service
COPY creation-usecase ./creation-usecase
COPY xtask ./xtask
# break the Cargo cache
RUN touch creation-driver/src/bin/main.rs

ENV RUST_LOG=debug
CMD ["systemfd", "--no-pid", "-s", "http::0.0.0.0:8001", "--", "watchexec", "-r", "--", "cargo", "run", "--target-dir", "target_debug", "-p", "creation-driver", "--bin", "main"]
