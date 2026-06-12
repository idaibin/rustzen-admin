# @formatter:off
# prettier-ignore
# justfile - Project unified command entry

# development
dev-server:
    cargo watch -x 'run -p server'

dev-web:
    cd apps/web && pnpm dev

# check
check:
    cargo check --workspace
    cd apps/web && pnpm exec vp lint

# Reset local sqlite database and let migrations re-run on next startup.
reset-db:
    runtime_root="${RUSTZEN_RUNTIME_ROOT:-.rustzen-admin}"; rm -f "${runtime_root}/data/rustzen.db"

# Build all (production)
build:
    just build-server
    just build-web
    just build-config

# Build x86_64 Linux musl server deployment binary
build-server:
    just _build-binary server x86_64-unknown-linux-musl linux/amd64

# Build web production bundle
build-web:
    rm -f apps/web/dist-rustzen-admin-*.zip
    cd apps/web && pnpm build
    VERSION=$(awk -F '"' '/^version = / { print $2; exit }' apps/server/Cargo.toml) && mkdir -p target/rustzen-admin && mv apps/web/dist-rustzen-admin-*.zip target/rustzen-admin/dist-$VERSION.zip

# Build minimal deployment configuration files
build-config:
    mkdir -p target/rustzen-admin/config target/rustzen-admin/systemd
    cp .env.example target/rustzen-admin/config/app.env
    sed -i '' 's#^RUSTZEN_RUNTIME_ROOT=.*#RUSTZEN_RUNTIME_ROOT=.#' target/rustzen-admin/config/app.env
    sed -i '' 's#^RUSTZEN_APP_PORT=.*#RUSTZEN_APP_PORT=9880#' target/rustzen-admin/config/app.env
    sed -i '' 's#^RUSTZEN_SQLITE_PATH=.*#RUSTZEN_SQLITE_PATH=./data/db/rustzen.db#' target/rustzen-admin/config/app.env
    cp deploy/rustzen-admin.service target/rustzen-admin/systemd/rustzen-admin.service
    cp deploy/setup-layout.sh target/rustzen-admin/setup-layout.sh
    chmod +x target/rustzen-admin/setup-layout.sh

_build-binary PACKAGE_NAME TARGET_TRIPLE PLATFORM:
    mkdir -p target/rustzen-admin
    docker buildx build --platform {{PLATFORM}} --build-arg PACKAGE_NAME={{PACKAGE_NAME}} --build-arg TARGET_TRIPLE={{TARGET_TRIPLE}} --target export --output type=local,dest=target/rustzen-admin .

# Update project version.
bump-version VERSION:
    @sed -i '' 's/^version = ".*"/version = "{{VERSION}}"/' apps/server/Cargo.toml
    @sed -i '' 's/^version = ".*"/version = "{{VERSION}}"/' crates/auth/Cargo.toml
    @sed -i '' 's/^version = ".*"/version = "{{VERSION}}"/' crates/config/Cargo.toml
    @sed -i '' 's/^version = ".*"/version = "{{VERSION}}"/' crates/runtime/Cargo.toml
    @sed -i '' 's/^version = ".*"/version = "{{VERSION}}"/' crates/storage/Cargo.toml
    @sed -i '' 's/"version": ".*"/"version": "{{VERSION}}"/' apps/web/package.json

# Clean build outputs
clean:
    rm -rf target apps/web/dist apps/server/target .rustzen-admin
