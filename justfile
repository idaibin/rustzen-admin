# justfile - monorepo command entry

# check
check:
    cargo check --manifest-path server/Cargo.toml
    cd web && vp lint

# Start Rust backend (with hot reload)
dev-server:
    cargo watch --manifest-path server/Cargo.toml -x run -w server/src

# Start web (Vite dev mode)
dev-web:
    cd web && pnpm dev

# Build all (production)
build:
    just build-web
    just build-backend

# Build Rust backend release
build-backend:
    cargo build --manifest-path server/Cargo.toml --release

# Build web production bundle
build-web:
    cd web && pnpm build

# Clean build outputs
clean:
    rm -rf target web/dist server/target
