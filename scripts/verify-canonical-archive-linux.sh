#!/usr/bin/env bash
set -euo pipefail
repository="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
host="$(cd "$repository" && pnpm dlx bun@1.3.14 scripts/distribution-canonical-archive-digest.ts)"
linux="$(docker run --rm --platform linux/arm64 -v "$repository:/work:ro" -w /work oven/bun:1.3.14-debian bun scripts/distribution-canonical-archive-digest.ts)"
test "$host" = "$linux"
printf 'canonical archive SHA-256: %s\n' "$host"
