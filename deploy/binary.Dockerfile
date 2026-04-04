FROM ubuntu:24.04 AS server-builder

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/root/.rustup
ENV CARGO_HOME=/root/.cargo
ENV PATH=/root/.cargo/bin:/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        build-essential \
        curl \
        musl-tools \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml server/Cargo.toml
COPY server/build.rs server/build.rs
COPY server/src server/src
COPY server/migrations server/migrations

RUN cargo build -p server --release --target x86_64-unknown-linux-musl

FROM scratch AS binary

COPY --from=server-builder /app/target/x86_64-unknown-linux-musl/release/rustzen-admin /rustzen-admin
