# Stage 1: Build
FROM rust:1.80-slim-bookworm AS builder
WORKDIR /ueas

# Copy only workspace manifests and source needed for ueas-mcp
COPY implementation/Cargo.toml implementation/Cargo.lock ./implementation/
COPY implementation/kernel/Cargo.toml ./implementation/kernel/
COPY implementation/backends/Cargo.toml ./implementation/backends/
COPY implementation/tools/ueas-mcp/Cargo.toml ./implementation/tools/ueas-mcp/

# Prefetch dependencies (layer caching)
RUN mkdir -p implementation/kernel/src && touch implementation/kernel/src/lib.rs
RUN mkdir -p implementation/backends/src && touch implementation/backends/src/lib.rs
RUN mkdir -p implementation/tools/ueas-mcp/src && echo "fn main() {}" > implementation/tools/ueas-mcp/src/main.rs
RUN cargo build --manifest-path implementation/Cargo.toml --release -p ueas-mcp 2>/dev/null || true
RUN rm implementation/kernel/src/lib.rs implementation/backends/src/lib.rs
RUN rm -r implementation/tools/ueas-mcp/src

# Copy actual source
COPY implementation/ ./implementation/

RUN cargo build --manifest-path implementation/Cargo.toml --release -p ueas-mcp

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates socat procps \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /ueas/implementation/target/release/ueas-mcp /usr/local/bin/ueas-mcp

# Expose the MCP server over a raw TCP socket using socat.
# Render will detect the bound port and mark the deploy as successful.
ENV PORT=7860
EXPOSE 7860
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD pgrep socat || exit 1

# socat listens on $PORT and forks the ueas-mcp process for every new connection, bridging TCP to STDIO
CMD sh -c "socat TCP-LISTEN:${PORT:-7860},fork,reuseaddr EXEC:/usr/local/bin/ueas-mcp"
