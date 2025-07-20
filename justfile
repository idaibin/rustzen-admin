# justfile - Project unified command entry

# Development mode: start backend + web together
dev:
    just dev-backend &
    just dev-web

# Start Rust backend (with hot reload)
dev-backend:
    cargo watch -x run -w src

# Start web (Vite dev mode)
dev-web:
    cd web && pnpm dev

# Build all (production)
build:
    just build-backend
    just build-web

# Build Rust backend release
build-backend:
    cargo build --release

# Build web production bundle
build-web:
    cd web && pnpm build

# Clean build outputs
clean:
    rm -rf /target web/dist
