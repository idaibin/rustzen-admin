ARG BASE_IMAGE=ubuntu:24.04
FROM oven/bun:1.3.14 AS bun-runtime
FROM ${BASE_IMAGE} AS build

ARG TARGET_TRIPLE=x86_64-unknown-linux-musl

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_DIST_SERVER=https://rsproxy.cn \
    RUSTUP_UPDATE_ROOT=https://rsproxy.cn/rustup \
    RUST_VERSION=1.95.0 \
    CARGO_HOME=/root/.cargo \
    RUSTUP_HOME=/root/.rustup \
    PATH=/root/.cargo/bin:${PATH} \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc

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

ARG DISTRIBUTION=full
RUN case "${DISTRIBUTION}" in full|monitor) ;; *) echo "DISTRIBUTION must be full or monitor" >&2; exit 2 ;; esac
ARG SOURCE_IDENTITY=
RUN if [ "${DISTRIBUTION}" = "monitor" ]; then \
      test "${TARGET_TRIPLE}" = "x86_64-unknown-linux-musl" || { echo "Monitor container export requires x86_64-unknown-linux-musl" >&2; exit 2; }; \
      test -n "${SOURCE_IDENTITY}" || { echo "Monitor container export requires SOURCE_IDENTITY" >&2; exit 2; }; \
    fi

COPY --from=bun-runtime /usr/local/bin/bun /usr/local/bin/bun
COPY apps/web/package.json apps/web/bun.lock apps/web/
RUN --mount=type=cache,target=/root/.bun/install/cache \
    cd apps/web && bun install --frozen-lockfile --ignore-scripts
COPY apps/web apps/web
COPY apps/admin apps/admin
COPY distribution distribution
COPY scripts/distribution-build-web.ts scripts/distribution-verify-web.ts scripts/distribution-resolve.ts scripts/distribution-web-inventory-policy.ts scripts/distribution-web-allowed-packages.ts scripts/distribution-produce-contracts.ts scripts/distribution-produce-protocol.ts scripts/distribution-produce-native-layout.ts scripts/distribution-produce-container-export.ts scripts/
RUN if [ "${DISTRIBUTION}" = "monitor" ]; then \
      bun scripts/distribution-build-web.ts --selection distribution/fixtures/monitor.json && \
      bun scripts/distribution-verify-web.ts --selection distribution/fixtures/monitor.json && \
      composition="$(bun scripts/distribution-resolve.ts resolve --selection distribution/fixtures/monitor.json | bun -e 'const data=await Bun.stdin.json(); console.log(data.compositionId)')" && \
      rm -rf "apps/admin/selected-web/${composition}" && \
      mkdir -p "apps/admin/selected-web/${composition}/dist" && \
      cp "target/distributions/${composition}/web/inventory.json" "apps/admin/selected-web/${composition}/inventory.json" && \
      cp -R "target/distributions/${composition}/web/dist/." "apps/admin/selected-web/${composition}/dist" && \
      (cd "target/distributions/${composition}/web/dist" && find . -type f -print0 | sort -z | xargs -0 sha256sum) >/tmp/rz-selected-web.source && \
      (cd "apps/admin/selected-web/${composition}/dist" && find . -type f -print0 | sort -z | xargs -0 sha256sum) >/tmp/rz-selected-web.embedded && \
      cmp -s /tmp/rz-selected-web.source /tmp/rz-selected-web.embedded; \
    else cd apps/web && bun run vp build; fi

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
COPY apps/cli apps/cli
COPY apps/monitor apps/monitor
COPY apps/insights apps/insights
COPY apps/reports apps/reports

RUN if [ "${DISTRIBUTION}" = "monitor" ]; then mkdir -p /out/release/server/bin /out/witness/bin /tmp/rz-monitor-producers; else mkdir -p /out/bin; fi
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/app/target \
    if [ "${DISTRIBUTION}" = "monitor" ]; then \
        env RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target "${TARGET_TRIPLE}" -p rustzen-admin --no-default-features --features monitor-distribution && \
        env RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target "${TARGET_TRIPLE}" -p rustzen-monitor --no-default-features --features controller --bin rz-monitor && \
        env RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target "${TARGET_TRIPLE}" -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent && \
        install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-admin" "/out/release/server/bin/rz-admin" && \
        install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-monitor" "/out/release/server/bin/rz-monitor" && \
        install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-monitor-agent" "/out/witness/bin/rz-monitor-agent" && \
        install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-admin" "/tmp/rz-monitor-producers/rz-admin" && \
        install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-monitor" "/tmp/rz-monitor-producers/rz-monitor" && \
        install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-monitor-agent" "/tmp/rz-monitor-producers/rz-monitor-agent" && \
        composition="$(bun scripts/distribution-resolve.ts resolve --selection distribution/fixtures/monitor.json | bun -e 'const data=await Bun.stdin.json(); console.log(data.compositionId)')" && \
        mkdir -p /out/release/web && \
        cp apps/admin/selected-web/"${composition}"/inventory.json /out/release/web/inventory.json && \
        cp -R apps/admin/selected-web/"${composition}"/dist /out/release/web/dist && \
        RUSTZEN_CONTRACT_OUTPUT_ROOT=/out/release/contracts bun scripts/distribution-produce-contracts.ts --selection distribution/fixtures/monitor.json --binary-root /tmp/rz-monitor-producers && \
        bun scripts/distribution-produce-protocol.ts --selection distribution/fixtures/monitor.json --binary-root /tmp/rz-monitor-producers --output-root /out/release/contracts/protocol && \
        bun scripts/distribution-produce-native-layout.ts --selection distribution/fixtures/monitor.json --output-root /out/release/contracts/native && \
        RUSTZEN_CONTAINER_TARGET_TRIPLE="${TARGET_TRIPLE}" RUSTZEN_CONTAINER_SOURCE_IDENTITY="${SOURCE_IDENTITY}" RUSTZEN_CONTAINER_BUILD_COMMANDS='[["env","RUSTFLAGS=-C target-feature=+crt-static","cargo","build","--release","--target","x86_64-unknown-linux-musl","-p","rustzen-admin","--no-default-features","--features","monitor-distribution"],["env","RUSTFLAGS=-C target-feature=+crt-static","cargo","build","--release","--target","x86_64-unknown-linux-musl","-p","rustzen-monitor","--no-default-features","--features","controller","--bin","rz-monitor"],["env","RUSTFLAGS=-C target-feature=+crt-static","cargo","build","--release","--target","x86_64-unknown-linux-musl","-p","rustzen-monitor","--no-default-features","--features","agent","--bin","rz-monitor-agent"]]' bun scripts/distribution-produce-container-export.ts --selection distribution/fixtures/monitor.json --output-root /out; \
    elif [ "${TARGET_TRIPLE}" = "aarch64-unknown-linux-gnu" ]; then \
        cargo build --release --target "${TARGET_TRIPLE}" -p rustzen-cli -p rustzen-admin -p rustzen-monitor -p rustzen-insights -p rustzen-reports && \
        install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz" "/out/bin/rz" && install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-admin" "/out/bin/rz-admin" && install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-monitor" "/out/bin/rz-monitor" && install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-insights" "/out/bin/rz-insights" && install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-reports" "/out/bin/rz-reports"; \
    else \
        RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target "${TARGET_TRIPLE}" -p rustzen-cli -p rustzen-admin -p rustzen-monitor -p rustzen-insights -p rustzen-reports && \
        install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz" "/out/bin/rz" && install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-admin" "/out/bin/rz-admin" && install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-monitor" "/out/bin/rz-monitor" && install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-insights" "/out/bin/rz-insights" && install -m 0755 "/app/target/${TARGET_TRIPLE}/release/rz-reports" "/out/bin/rz-reports"; \
    fi

FROM scratch AS export
COPY --from=build /out /
