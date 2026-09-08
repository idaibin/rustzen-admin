#!/usr/bin/env bash
set -euo pipefail
repository="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
docker run --rm --platform linux/amd64 -v "$repository:/work:ro" -v "$repository/target:/work/target" -w /work \
  oven/bun:1.3.14-debian bun test \
  distribution/source-identity.test.ts \
  distribution/source-build-certificate.test.ts \
  distribution/source-build-issuer.test.ts \
  distribution/source-build-publisher.test.ts \
  scripts/distribution-issue-source-build-certificate.integration.test.ts
