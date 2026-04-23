# justfile - monorepo command entry

# check
check:
    cargo check -p server
    cd zen-web && vp lint

# Start Rust backend (with hot reload)
dev-server:
    cargo watch -x 'run -p server'

# Start web (Vite dev mode)
dev-web:
    cd zen-web && pnpm dev

# Build all (production)
build:
    just build-web
    just build-backend

# Build Rust backend release
build-backend:
    cargo build -p server --release

# Build Linux x86_64 backend binary
build-binary:
    rm -rf target/dist/bin
    mkdir -p target/dist/bin
    docker buildx build --platform linux/amd64 --target binary --output type=local,dest=target/dist/bin -f deploy/binary.Dockerfile .

# Build Linux x86_64 release tree and zip
build-release:
    rm -rf target/dist/rustzen-admin target/dist/rustzen-admin.zip
    mkdir -p target/dist
    docker buildx build --platform linux/amd64 --target release --output type=local,dest=target/dist -f deploy/release.Dockerfile .

# Build Linux x86_64 runtime Docker image
build-image:
    docker buildx build --platform linux/amd64 --target runtime --load -t rustzen-admin:runtime -f deploy/runtime.Dockerfile .

# Build web production bundle
build-web:
    cd zen-web && pnpm build

# Clean build outputs
clean:
    rm -rf target zen-web/dist zen-server/target .rustzen-admin
