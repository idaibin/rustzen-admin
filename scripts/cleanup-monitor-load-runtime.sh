#!/usr/bin/env bash
set -euo pipefail
test "$#" -eq 1 || { echo "usage: $0 CONTEXT" >&2; exit 2; }
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$root"
context="$1"; test -f "$context" && test ! -L "$context" || { echo "context must be a regular file" >&2; exit 2; }
pnpm dlx bun@1.3.14 scripts/prepared-monitor-load-context.ts "$context" >/dev/null
container="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).containerName)' "$context")"
container_id="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).containerId)' "$context")"
password="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).passwordFile)' "$context")"
agent="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).agentTokenFile)' "$context")"
owner_token="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).ownerToken)' "$context")"
actual="$(docker inspect --format '{{.Id}}' "$container" 2>/dev/null || true)"
actual_owner=""; test -z "$actual" || actual_owner="$(docker inspect --format '{{index .Config.Labels "io.rustzen.p8g-owner"}}' "$container" 2>/dev/null || true)"
test -z "$actual" -o \( "$actual" = "$container_id" -a "$actual_owner" = "$owner_token" \) || { rm -f "$password" "$agent" "$context"; echo "container identity differs" >&2; exit 1; }
test -z "$actual" || docker rm -f "$container" >/dev/null 2>&1 || true
rm -f "$password" "$agent" "$context"
