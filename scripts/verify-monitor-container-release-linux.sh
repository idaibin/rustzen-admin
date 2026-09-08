#!/usr/bin/env bash
set -euo pipefail
repository="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
docker run --rm --platform linux/amd64 -v "$repository:/work:ro" -v "$repository/target:/work/target" -w /work \
  oven/bun:1.3.14-debian bun test \
  distribution/release-key-file.test.ts \
  distribution/container-export-release.test.ts \
  scripts/distribution-publish-selected-release.integration.test.ts
