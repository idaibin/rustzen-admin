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

FROM ubuntu:24.04 AS release-builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        zip \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /out

COPY --from=server-builder /app/target/x86_64-unknown-linux-musl/release/rustzen-admin /out/rustzen-admin/bin/rustzen-admin
COPY --from=web-builder /app/zen-web/dist /out/rustzen-admin/web/dist
COPY .env.example /out/rustzen-admin/config/app.env
COPY deploy/rustzen-admin.service /out/rustzen-admin/systemd/rustzen-admin.service

RUN mkdir -p /out/rustzen-admin/data/uploads /out/rustzen-admin/data/avatars /out/rustzen-admin/logs \
    && chmod +x /out/rustzen-admin/bin/rustzen-admin \
    && sed -i 's#^RUSTZEN_RUNTIME_ROOT=.*#RUSTZEN_RUNTIME_ROOT=.#' /out/rustzen-admin/config/app.env \
    && cd /out \
    && zip -qr rustzen-admin.zip rustzen-admin

FROM scratch AS release

COPY --from=release-builder /out/rustzen-admin /rustzen-admin
COPY --from=release-builder /out/rustzen-admin.zip /rustzen-admin.zip
