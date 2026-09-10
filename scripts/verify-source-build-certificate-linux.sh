#!/usr/bin/env bash
set -euo pipefail
repository="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
docker run --rm --platform linux/amd64 -v "$repository:/source:ro" -w /tmp/rustzen \
  oven/bun:1.3.14-debian bash -euo pipefail -c '
    mkdir -p /tmp/rustzen
    tar -C /source --exclude=target --exclude=apps/web/node_modules -cf - . | tar --no-same-owner -C /tmp/rustzen -xf -
    mkdir -p target
    bun test \
      distribution/source-identity.test.ts \
      distribution/source-build-certificate.test.ts \
      distribution/source-build-issuer.test.ts \
      distribution/source-build-publisher.test.ts \
      scripts/distribution-issue-source-build-certificate.integration.test.ts
  '
