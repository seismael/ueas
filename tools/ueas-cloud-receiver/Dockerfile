FROM ubuntu:24.04
RUN apt-get update && apt-get install -y wget curl unzip dotnet-sdk-8.0 build-essential
ENV DAFNY_VERSION=4.6.0
RUN wget https://github.com/dafny-lang/dafny/releases/download/v${DAFNY_VERSION}/dafny-${DAFNY_VERSION}-x64-ubuntu-20.04.zip \
    && unzip dafny-${DAFNY_VERSION}-x64-ubuntu-20.04.zip -d /opt/dafny \
    && ln -s /opt/dafny/dafny /usr/local/bin/dafny
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /app
COPY . .
RUN cargo build --release --bin ueas-cloud-receiver
CMD ["./target/release/ueas-cloud-receiver"]
