FROM node:24-bookworm AS web-builder

WORKDIR /app/zen-web

RUN corepack enable

COPY zen-web/package.json zen-web/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY zen-web ./
RUN pnpm exec vp build

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
COPY zen-core zen-core
COPY zen-server/Cargo.toml zen-server/Cargo.toml
COPY zen-server/build.rs zen-server/build.rs
COPY zen-server/src zen-server/src
COPY zen-server/migrations zen-server/migrations

RUN cargo build -p server --release --target x86_64-unknown-linux-musl

FROM ubuntu:24.04 AS runtime

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTZEN_RUNTIME_ROOT=.
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

WORKDIR /opt/rustzen-admin

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=server-builder /app/target/x86_64-unknown-linux-musl/release/rustzen-admin ./bin/rustzen-admin
COPY --from=web-builder /app/zen-web/dist ./web/dist

RUN mkdir -p ./data/uploads ./data/avatars ./logs \
    && chmod +x ./bin/rustzen-admin

EXPOSE 8007

ENTRYPOINT ["/opt/rustzen-admin/bin/rustzen-admin"]
