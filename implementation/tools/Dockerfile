FROM rust:1.75-slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    default-jre-headless \
    python3 python3-pip \
    && pip3 install ruff \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt

WORKDIR /ueas

COPY . .

RUN cargo build --workspace

CMD ["cargo", "test", "--workspace"]
