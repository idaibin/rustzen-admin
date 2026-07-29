# @formatter:off
# prettier-ignore
# justfile - Project unified command entry

# development
dev-server:
    cargo watch -x 'run -p rustzen-admin -- serve'

dev-monitor:
    cargo run -p rustzen-monitor -- controller

dev-insights:
    cargo run -p rustzen-insights -- serve

dev-reports:
    cargo run -p rustzen-reports -- serve

dev-web:
    cd apps/web && bun run dev

# check
check:
    cd apps/web && bun install --frozen-lockfile
    cd apps/web && bun run vp fmt --check
    cd apps/web && bun run vp lint
    cd apps/web && bun x tsc --noEmit
    cd apps/web && bun run test
    cd apps/web && bun run vp build
    cargo fmt --all -- --check
    cargo check --workspace
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

verify-services:
    cargo test -p rustzen-admin changed_manifest_swaps_after_commit_and_invalid_change_rolls_back
    cargo test -p rustzen-admin warm_gateway_streams_with_memory_auth_and_a_closed_database
    cargo build --release -p rustzen-cli -p rustzen-admin -p rustzen-monitor -p rustzen-insights -p rustzen-reports
    scripts/verify-services.sh target/release/rz-admin target/release/rz-monitor target/release/rz-insights target/release/rz-reports target/release/rz

verify-modules-mvp:
    cargo build --workspace
    scripts/verify-services.sh target/debug/rz-admin target/debug/rz-monitor target/debug/rz-insights target/debug/rz-reports target/debug/rz

verify-cli:
    cargo test -p rustzen-cli
    cargo clippy -p rustzen-cli --all-targets -- -D warnings
    cargo build -p rustzen-cli
    tmp_dir=$(mktemp -d); trap 'rmdir "$tmp_dir"' EXIT; cd "$tmp_dir"; "{{justfile_directory()}}/target/debug/rz" --help >/dev/null; "{{justfile_directory()}}/target/debug/rz" --json doctor; "{{justfile_directory()}}/target/debug/rz" --json version; "{{justfile_directory()}}/target/debug/rz" --json status all

verify-automation-browser browser_path:
    cargo build -p rustzen-reports
    scripts/verify-automation-browser.sh target/debug/rz-reports "{{browser_path}}"

e2e-modules browser_path:
    just verify-modules-mvp
    just verify-automation-browser "{{browser_path}}"

# Reset local sqlite database and let migrations re-run on next startup.
reset-db:
    runtime_root="${RUSTZEN_RUNTIME_ROOT:-.rustzen-admin}"; for db in admin monitor insights reports; do rm -f "${runtime_root}/data/db/${db}.db" "${runtime_root}/data/db/${db}.db-shm" "${runtime_root}/data/db/${db}.db-wal"; done; rm -f "${runtime_root}/data/rustzen.db" "${runtime_root}/data/rustzen.db-shm" "${runtime_root}/data/rustzen.db-wal"

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
