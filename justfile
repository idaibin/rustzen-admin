# @formatter:off
# prettier-ignore
# justfile - Project unified command entry

# development
dev-server:
    cargo watch -x 'run -p rustzen-admin -- serve'

dev-monitor:
    cargo run -p rustzen-monitor -- controller

dev-monitor-agent:
    cargo run -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent

verify-monitor-agent:
    cargo test -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent
    cargo clippy -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent -- -D warnings
    pnpm dlx bun@1.3.14 scripts/distribution-verify-agent.ts

prepare-monitor-embed:
    pnpm dlx bun@1.3.14 scripts/distribution-build-web.ts --selection distribution/fixtures/monitor.json
    pnpm dlx bun@1.3.14 scripts/distribution-verify-web.ts --selection distribution/fixtures/monitor.json
    composition=$(pnpm dlx bun@1.3.14 scripts/distribution-resolve.ts resolve --selection distribution/fixtures/monitor.json | pnpm dlx bun@1.3.14 -e 'const data=await Bun.stdin.json(); console.log(data.compositionId)'); rm -rf "apps/admin/selected-web/$composition"; mkdir -p "apps/admin/selected-web/$composition/dist"; cp "target/distributions/$composition/web/inventory.json" "apps/admin/selected-web/$composition/inventory.json"; cp -R "target/distributions/$composition/web/dist/." "apps/admin/selected-web/$composition/dist"

verify-monitor-admin:
    pnpm dlx bun@1.3.14 test scripts/distribution-verify-docker.test.ts
    just prepare-monitor-embed
    cargo test -p rustzen-config --no-default-features --features admin-monitor
    cargo test -p rustzen-admin --no-default-features --features monitor-distribution -- --test-threads=1
    cargo clippy -p rustzen-admin --no-default-features --features monitor-distribution -- -D warnings
    cargo build -p rustzen-admin --no-default-features --features monitor-distribution
    pnpm dlx bun@1.3.14 scripts/distribution-verify-admin.ts

dev-insights:
    cargo run -p rustzen-insights -- serve

dev-reports:
    cargo run -p rustzen-reports -- serve

dev-web:
    cd apps/web && bun run dev

# check
# Composable selection only. These commands do not compile or certify a release.
# Bun follows the existing apps/web runtime pin and is provisioned through pnpm.
distribution-validate selection="distribution/fixtures/monitor.json":
    pnpm dlx bun@1.3.14 scripts/distribution-resolve.ts validate --selection "{{selection}}"

distribution-plan selection="distribution/fixtures/monitor.json":
    pnpm dlx bun@1.3.14 scripts/distribution-resolve.ts resolve --selection "{{selection}}"

distribution-release-gate selection="distribution/fixtures/monitor.json":
    pnpm dlx bun@1.3.14 scripts/distribution-resolve.ts release-gate --selection "{{selection}}"

verify-distribution-selection:
    pnpm dlx bun@1.3.14 test distribution scripts/distribution-resolve.test.ts

# Builds a composition-qualified Monitor Web artifact and rejects excluded emitted routes/API/assets.
verify-distribution-web:
    pnpm dlx bun@1.3.14 test scripts/distribution-verify-web.test.ts

verify-service-wiring:
    scripts/test-verify-services.sh

check:
    just verify-service-wiring
    cd apps/web && bun install --frozen-lockfile
    cd apps/web && bun run vp fmt --check
    cd apps/web && bun run vp lint
    cd apps/web && bun x tsc --noEmit
    cd apps/web && bun run test
    bun test apps/insights/src/features/tracking/tracker.test.mjs
    cd apps/web && bun run vp build
    cargo fmt --all -- --check
    cargo check --workspace
    cargo clippy --workspace --all-targets -- -D warnings
    # Admin fixtures share a process-global permission cache across independent databases.
    # Serialize test cases; each concurrency test still runs its own parallel tasks.
    cargo test --workspace -- --test-threads=1

verify-services:
    cargo test -p rustzen-admin changed_manifest_swaps_after_commit_and_invalid_change_rolls_back
    cargo test -p rustzen-admin warm_gateway_streams_with_memory_auth_and_a_closed_database
    cargo build --release -p rustzen-cli -p rustzen-admin -p rustzen-monitor -p rustzen-insights -p rustzen-reports
    cargo build --release -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent
    RUSTZEN_VERIFY_BUILD_PROFILE=release scripts/verify-services.sh target/release/rz-admin target/release/rz-monitor target/release/rz-insights target/release/rz-reports target/release/rz target/release/rz-monitor-agent

verify-modules-mvp:
    cargo build --workspace
    cargo build -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent
    RUSTZEN_VERIFY_BUILD_PROFILE=debug scripts/verify-services.sh target/debug/rz-admin target/debug/rz-monitor target/debug/rz-insights target/debug/rz-reports target/debug/rz target/debug/rz-monitor-agent

# Admin-native route contract. Rust registration is the authority; this artifact
# is a derived input for client generation and compatibility checks. Module
# service routes remain outside this OpenAPI document until their shared IPC
# contract carries operation and schema metadata.
contract-generate:
    mkdir -p openapi
    cargo run --quiet -p rustzen-admin -- openapi > openapi/admin-contract.json

contract-verify:
    bash scripts/verify-contract.sh
    cargo test -p rustzen-admin infra::contract
    cargo test -p rustzen-admin documented_nested_routes_match
    cargo test -p rustzen-admin generated_document_matches
    cd apps/web && bun x tsc --noEmit
    cd apps/web && bun test src/api/request.contract.test.ts

contract-compat:
    cmp -s openapi/baselines/contract-admin-native-all-refact-modules-mvp.json openapi/admin-contract.json

contract-client:
    cd apps/web && bun run contract:generate && bun scripts/normalize-contract-client.mjs && bun run vp fmt "${CONTRACT_CLIENT_OUTPUT:-src/api/generated/admin-contract.ts}"

contract-bench:
    cargo test --release -p rustzen-admin route_contract_registration_and_hot_request_benchmark -- --ignored --nocapture

verify-cli:
    cargo test -p rustzen-cli
    cargo clippy -p rustzen-cli --all-targets -- -D warnings
    cargo build -p rustzen-cli
    tmp_dir=$(mktemp -d); trap 'rmdir "$tmp_dir"' EXIT; cd "$tmp_dir"; "{{justfile_directory()}}/target/debug/rz" --help >/dev/null; "{{justfile_directory()}}/target/debug/rz" --json doctor; "{{justfile_directory()}}/target/debug/rz" --json version; "{{justfile_directory()}}/target/debug/rz" --json status all

verify-automation-browser browser_path:
    cargo build -p rustzen-reports
    scripts/verify-automation-browser.sh target/debug/rz-reports "{{browser_path}}"

verify-reports-linux:
    scripts/verify-reports-linux.sh

e2e-modules browser_path:
    just verify-modules-mvp
    just verify-automation-browser "{{browser_path}}"

# Reset local sqlite database and let migrations re-run on next startup.
reset-db:
    runtime_root="${RUSTZEN_RUNTIME_ROOT:-.rustzen-admin}"; for db in admin monitor insights; do rm -f "${runtime_root}/data/db/${db}.db" "${runtime_root}/data/db/${db}.db-shm" "${runtime_root}/data/db/${db}.db-wal"; done; rm -f "${runtime_root}/data/reports/db/reports.db" "${runtime_root}/data/reports/db/reports.db-shm" "${runtime_root}/data/reports/db/reports.db-wal" "${runtime_root}/data/rustzen.db" "${runtime_root}/data/rustzen.db-shm" "${runtime_root}/data/rustzen.db-wal"

# Build all (production)
build:
    just build-config
    just build-release

# Build one signed x86_64 Linux bundle containing all four services.
build-release:
    just _build-binaries x86_64 x86_64-unknown-linux-musl linux/amd64
    VERSION=$(awk -F '"' '/^version = / { print $2; exit }' Cargo.toml); BUNDLE=$(scripts/package-release-bundle.sh "$VERSION" x86_64 target/rz/build/x86_64/bin target/rz); bun scripts/deploy-sign.mjs sign-bundle --file "$BUNDLE" --version "$VERSION" --arch x86_64; bun scripts/deploy-sign.mjs verify-bundle --file "$BUNDLE" --version "$VERSION" --arch x86_64

build-native:
    cd apps/web && bun run vp build
    cargo build --release -p rustzen-cli -p rustzen-admin -p rustzen-monitor -p rustzen-insights -p rustzen-reports

# Build web production bundle
build-web:
    cd apps/web && bun run vp build

# Build minimal deployment configuration files
build-config:
    mkdir -p target/rz/config target/rz/systemd
    cp .env.example target/rz/config/rz.env
    cp .env.reports.example target/rz/config/rz-reports.env
    VERIFY_KEY=$(bun scripts/deploy-sign.mjs public-key) && perl -pi -e "s#^RUSTZEN_DEPLOY_VERIFY_KEY=.*#RUSTZEN_DEPLOY_VERIFY_KEY=$VERIFY_KEY#" target/rz/config/rz.env
    cp deploy/rz.target deploy/rz-recovery.service deploy/rz-admin.service deploy/rz-monitor.service deploy/rz-insights.service deploy/rz-reports.service target/rz/systemd/
    cp deploy/setup-layout.sh target/rz/setup-layout.sh
    chmod +x target/rz/setup-layout.sh

_build-binaries ARCH TARGET_TRIPLE PLATFORM:
    rm -rf target/rz/build/{{ARCH}}
    mkdir -p target/rz/build/{{ARCH}}
    docker buildx build --platform {{PLATFORM}} --build-arg TARGET_TRIPLE={{TARGET_TRIPLE}} --target export --output type=local,dest=target/rz/build/{{ARCH}} .

# Update project version.
bump-version VERSION:
    @perl -0pi -e 's/(\[workspace\.package\]\nversion = ")[^"]+/\1{{VERSION}}/' Cargo.toml
    @perl -pi -e 's/"version": ".*"/"version": "{{VERSION}}"/' apps/web/package.json

# Clean build outputs
clean:
    rm -rf target apps/web/dist .rustzen-admin

verify-distribution-manifest:
    apps/web/node_modules/typescript/bin/tsc -p distribution/tsconfig.json --noEmit --pretty false
    pnpm dlx bun@1.3.14 test distribution/resolver.test.ts distribution/release-manifest.test.ts distribution/release-manifest-api.test.ts distribution/release-manifest-protocol.test.ts distribution/selected-contract.test.ts distribution/schema-contract.test.ts distribution/selected-config.test.ts distribution/selected-protocol.test.ts distribution/native-layout.test.ts

verify-distribution-native-layout:
    apps/web/node_modules/typescript/bin/tsc -p distribution/tsconfig.json --noEmit --pretty false
    pnpm dlx bun@1.3.14 test distribution/native-layout.test.ts
    pnpm dlx bun@1.3.14 scripts/distribution-produce-native-layout.ts --selection distribution/fixtures/monitor.json
    pnpm dlx bun@1.3.14 scripts/distribution-produce-native-layout.ts --selection distribution/fixtures/node-agent.json

verify-monitor-protocol:
    cargo build -p rustzen-monitor --no-default-features --features controller --bin rz-monitor
    cargo build -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent
    bash -c 'cmp <(target/debug/rz-monitor contract protocol) <(target/debug/rz-monitor-agent contract protocol)'

verify-monitor-selected-protocol:
    cargo build -p rustzen-monitor --no-default-features --features controller --bin rz-monitor
    cargo build -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent
    apps/web/node_modules/typescript/bin/tsc -p distribution/tsconfig.json --noEmit --pretty false
    pnpm dlx bun@1.3.14 test distribution/selected-protocol.test.ts distribution/release-manifest-protocol.test.ts
    pnpm dlx bun@1.3.14 scripts/distribution-produce-protocol.ts --selection distribution/fixtures/monitor.json
    pnpm dlx bun@1.3.14 scripts/distribution-produce-protocol.ts --selection distribution/fixtures/node-agent.json

verify-monitor-config-descriptors:
    cargo test -p rustzen-config --no-default-features --features admin-monitor
    cargo test -p rustzen-config --no-default-features --features monitor-controller
    cargo test -p rustzen-config --no-default-features --features monitor-agent
    cargo build -p rustzen-admin --no-default-features --features monitor-distribution --bin rz-admin
    cargo build -p rustzen-monitor --no-default-features --features controller --bin rz-monitor
    cargo build -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent
    bash -c 'cd /tmp && env -i PATH="$$PATH" "{{justfile_directory()}}/target/debug/rz-admin" contract config selected >/dev/null && env -i PATH="$$PATH" "{{justfile_directory()}}/target/debug/rz-monitor" contract config selected >/dev/null && env -i PATH="$$PATH" "{{justfile_directory()}}/target/debug/rz-monitor-agent" contract config selected >/dev/null'

verify-monitor-selected-contract:
    cargo build -p rustzen-admin --no-default-features --features monitor-distribution --bin rz-admin
    cargo build -p rustzen-monitor --no-default-features --features controller --bin rz-monitor
    cargo build -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent
    apps/web/node_modules/typescript/bin/tsc -p distribution/tsconfig.json --noEmit --pretty false
    pnpm dlx bun@1.3.14 test distribution/release-manifest.test.ts distribution/release-manifest-api.test.ts distribution/selected-contract.test.ts distribution/schema-contract.test.ts distribution/selected-config.test.ts
    pnpm dlx bun@1.3.14 scripts/distribution-produce-contracts.ts --selection distribution/fixtures/monitor.json
    pnpm dlx bun@1.3.14 scripts/distribution-produce-contracts.ts --selection distribution/fixtures/node-agent.json
