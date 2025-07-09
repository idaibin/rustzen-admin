# justfile - Project unified command entry

# Development mode: start backend + web together
dev:
    just backend-dev &
    just web-dev

# Start Rust backend (with hot reload)
backend-dev:
    cargo watch -x run

# Start web (Vite dev mode)
web-dev:
    cd web && pnpm dev

# Build all (production)
build:
    just backend-build
    just web-build

# Build Rust backend release
backend-build:
    cd backend && cargo build --release

# Build web production bundle
web-build:
    cd web && pnpm build

# Clean build outputs
clean:
    rm -rf /target web/dist

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
