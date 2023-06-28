FROM lukemathwalker/cargo-chef:latest-rust-1.70.0 AS chef
WORKDIR /app


FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin schnose-api


FROM debian:bullseye-slim AS runtime
WORKDIR /
COPY --from=builder /app/configs/api-docker.toml /etc/schnose-api.toml
COPY --from=builder /app/target/release/schnose-api /usr/local/bin
ENTRYPOINT ["/usr/local/bin/schnose-api", "--config", "/etc/schnose-api.toml", "--port", "9002", "--public"]

