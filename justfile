# justfile - Project unified command entry

# check
check:
    cargo check &
    cd web && pnpm lint

# Development mode: start backend + web together
dev:
    just dev-web &
    just dev-backend

# Start Rust backend (with hot reload)
dev-backend:
    cargo watch -x run -w src

# Start web (Vite dev mode)
dev-web:
    cd web && pnpm dev

# Build all (production)
build:
    just build-web
    just build-backend

# Build Rust backend release
build-backend:
    cargo build --release

# Build web production bundle
build-web:
    cd web && pnpm build

# Clean build outputs
clean:
    rm -rf /target web/dist
