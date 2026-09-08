#!/usr/bin/env bash
set -euo pipefail
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
python3 "$root/scripts/test-message-center-web-proxy.py"
cd "$root/apps/web"
pnpm dlx bun@1.3.14 test \
  src/notifications \
  src/distribution/monitor-auth-store.test.ts \
  src/lib/auth-query-cache.test.ts \
  tests/message-center.seam.test.mjs
