ARG BASE_IMAGE=ubuntu:24.04
FROM oven/bun:1.3.14 AS bun-runtime
FROM ${BASE_IMAGE} AS build

ARG TARGET_TRIPLE=x86_64-unknown-linux-musl
ARG RUSTZEN_DEPLOY_VERIFY_KEY

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_DIST_SERVER=https://rsproxy.cn \
    RUSTUP_UPDATE_ROOT=https://rsproxy.cn/rustup \
    RUST_VERSION=1.95.0 \
    CARGO_HOME=/root/.cargo \
    RUSTUP_HOME=/root/.rustup \
    PATH=/root/.cargo/bin:${PATH} \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc \
    RUSTZEN_DEPLOY_VERIFY_KEY=${RUSTZEN_DEPLOY_VERIFY_KEY}

RUN sed -i "s|archive.ubuntu.com|mirrors.aliyun.com|g; s|ports.ubuntu.com|mirrors.aliyun.com|g" /etc/apt/sources.list.d/ubuntu.sources && \
    apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl build-essential musl-tools pkg-config && \
    rm -rf /var/lib/apt/lists/*

RUN curl --retry 5 --retry-all-errors --connect-timeout 15 --max-time 600 https://sh.rustup.rs -sSf | \
    sh -s -- -y --profile minimal --default-toolchain ${RUST_VERSION} && \
    rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-gnu aarch64-unknown-linux-musl

RUN mkdir -p "${CARGO_HOME}" && printf '%s\n' \
    '[source.crates-io]' \
    'replace-with = "ustc"' \
    '' \
    '[source.ustc]' \
    'registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"' \
    > "${CARGO_HOME}/config.toml"

WORKDIR /app

COPY --from=bun-runtime /usr/local/bin/bun /usr/local/bin/bun
COPY apps/web/package.json apps/web/bun.lock apps/web/
RUN --mount=type=cache,target=/root/.bun/install/cache \
    cd apps/web && bun install --frozen-lockfile --ignore-scripts
COPY apps/web apps/web
RUN cd apps/web && bun run vp build

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
COPY apps apps

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/app/target \
    mkdir -p /out/bin && \
    if [ "${TARGET_TRIPLE}" = "aarch64-unknown-linux-gnu" ]; then \
        cargo build --release --target "${TARGET_TRIPLE}" -p rustzen-cli -p rustzen-admin -p rustzen-monitor -p rustzen-insights -p rustzen-reports; \
    else \
        RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target "${TARGET_TRIPLE}" -p rustzen-cli -p rustzen-admin -p rustzen-monitor -p rustzen-insights -p rustzen-reports; \
    fi && \
    install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz" /out/bin/rz && \
    install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-admin" /out/bin/rz-admin && \
    install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-monitor" /out/bin/rz-monitor && \
    if [ "${TARGET_TRIPLE}" = "x86_64-unknown-linux-musl" ]; then /app/target/${TARGET_TRIPLE}/release/rz-monitor contract protocol > /out/bin/controller-protocol.json; fi && \
    install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-insights" /out/bin/rz-insights && \
    install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-reports" /out/bin/rz-reports

FROM scratch AS export
COPY --from=build /out /
