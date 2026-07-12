FROM ubuntu:24.04

# Install .NET SDK from Microsoft's official repo
RUN apt-get update && apt-get install -y wget curl unzip build-essential
RUN wget https://packages.microsoft.com/config/ubuntu/24.04/packages-microsoft-prod.deb -O /tmp/packages-microsoft-prod.deb \
    && dpkg -i /tmp/packages-microsoft-prod.deb \
    && rm /tmp/packages-microsoft-prod.deb \
    && apt-get update \
    && apt-get install -y dotnet-sdk-8.0

# Install Dafny
ENV DAFNY_VERSION=4.6.0
RUN wget https://github.com/dafny-lang/dafny/releases/download/v${DAFNY_VERSION}/dafny-${DAFNY_VERSION}-x64-ubuntu-20.04.zip \
    && unzip dafny-${DAFNY_VERSION}-x64-ubuntu-20.04.zip -d /opt/dafny \
    && ln -s /opt/dafny/dafny /usr/local/bin/dafny

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
ENV DOTNET_ROOT=/usr/share/dotnet

# Build and run
WORKDIR /app
COPY . .
RUN cargo build --release --manifest-path tools/ueas-cloud-receiver/Cargo.toml --target-dir /app/target
CMD ["./target/release/ueas-cloud-receiver"]
