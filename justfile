# justfile - monorepo command entry

# check
check:
    cargo check -p server
    cd web && vp lint

# Start Rust backend (with hot reload)
dev-server:
    cargo watch -x 'run -p server'

# Start web (Vite dev mode)
dev-web:
    cd web && pnpm dev

# Build all (production)
build:
    just build-web
    just build-backend

# Build Rust backend release
build-backend:
    cargo build -p server --release

# Build web production bundle
build-web:
    cd web && pnpm build

# Clean build outputs
clean:
    rm -rf target web/dist server/target
