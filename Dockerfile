FROM rust:1.70 AS builder
WORKDIR /schnose-api
COPY . .
ENV CARGO_TARGET_DIR="/schnose-api/target"
RUN cargo build --release --locked --bin schnose-api


FROM rust:1.70 AS runtime
WORKDIR /
COPY --from=builder /schnose-api/api/config.toml /etc/schnose-api.toml
COPY --from=builder /schnose-api/target/release/schnose-api /usr/local/bin
ENTRYPOINT ["/usr/local/bin/schnose-api", "--config", "/etc/schnose-api.toml", "--port", "9002", "--public"]

