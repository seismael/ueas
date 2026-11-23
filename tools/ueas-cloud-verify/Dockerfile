# Hardened multi-stage Dockerfile for ueas-cloud-verify (Cloud Run)
FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY tools tools
COPY tests tests
COPY xtask xtask
RUN cargo build --release --bin ueas-cloud-verify

FROM debian:bookworm-slim
RUN useradd -m -u 10001 ueas
COPY --from=builder /app/target/release/ueas-cloud-verify /usr/local/bin/ueas-cloud-verify
USER ueas
ENV PORT=8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/ueas-cloud-verify"]
