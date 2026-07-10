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
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates python3 curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /ueas/implementation/target/release/ueas-mcp /usr/local/bin/ueas-mcp
COPY implementation/tools/mcp_bridge.py /usr/local/bin/mcp_bridge.py

# Expose the MCP server over HTTP using the Python bridge
# Render's HTTP load balancer will proxy requests to this bridge
ENV PORT=7860
ENV PYTHONUNBUFFERED=1
EXPOSE 7860
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT:-7860}/ || exit 1

# Start the Python HTTP bridge
CMD ["python3", "/usr/local/bin/mcp_bridge.py"]
