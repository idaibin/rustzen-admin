# justfile - Project unified command entry

# Development mode: start backend + frontend together
dev:
    just backend-dev &
    just frontend-dev

# Start Rust backend (with hot reload)
backend-dev:
    cd backend && cargo watch -x run


# Start frontend (Vite dev mode)
frontend-dev:
    cd frontend && pnpm dev

# Build all (production)
build:
    just backend-build
    just frontend-build

# Build Rust backend release
backend-build:
    cd backend && cargo build --release

# Build frontend production bundle
frontend-build:
    cd frontend && pnpm build

# Clean build outputs
clean:
    rm -rf backend/target frontend/dist desktop/src-tauri/target

# 📋 Changelog Management
# Preview unreleased changes
changelog-preview:
    git-cliff --unreleased

# Update CHANGELOG.md Unreleased section
changelog-update:
    git-cliff --unreleased --prepend CHANGELOG.md

# Generate complete changelog
changelog-full:
    git-cliff --output CHANGELOG.md

# Generate changelog for specific version range
changelog-range FROM TO:
    git-cliff {{FROM}}..{{TO}}

# Release new version (generate changelog + create tag)
release VERSION:
    echo "Release {{VERSION}} prepared! Run 'git push origin {{VERSION}}' to publish."
    git-cliff --tag {{VERSION}} --prepend CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "chore(release): bump version to {{VERSION}}"
    git tag {{VERSION}}
    echo "Release {{VERSION}} prepared! Run 'git push origin {{VERSION}}' to publish."
