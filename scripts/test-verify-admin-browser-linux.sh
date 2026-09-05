#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
outer="$root/scripts/verify-admin-browser-linux.sh"
inner="$root/scripts/verify-admin-browser-linux-inner.sh"
docs="$root/docs/ui/runtime-browser-validation.md"
session="$root/apps/reports/src/features/automation/browser/session.rs"
identity="$root/scripts/admin-browser-source-identity.sh"
provenance="$root/scripts/write-admin-browser-build-provenance.sh"
build="$root/scripts/build-admin-browser-linux.sh"

for file in "$outer" "$inner" "$docs" "$session" "$identity" "$provenance" "$build"; do test -s "$file"; done
grep -Fq 'if ! mkdir "$lock_dir"' "$outer"
grep -Fq 'rm -rf "$current" "$candidate"' "$outer"
grep -Fq 'mv "$candidate" "$current"' "$outer"
grep -Fq 'if ! docker rm "$container"' "$outer"
grep -Fq 'timeout --signal=TERM --kill-after=10s 300s' "$outer"
grep -Fq 'verification port is already occupied' "$inner"
grep -Fq -- '--connect-timeout 3 --max-time' "$inner"
grep -Fq 'content-type:[[:space:]]*image/png' "$inner"
grep -Fq '89504e470d0a1a0a' "$inner"
grep -Fq 'gitHead:$head' "$inner"
grep -Fq 'sourceTreeState:$sourceTreeState' "$inner"
grep -Fq 'sourceTreeSha256:$sourceTreeSha256' "$inner"
grep -Fq 'source tree content changed during verification' "$outer"
grep -Fq 'Linux binaries do not match the current source-tree build provenance' "$outer"
grep -Fq 'Linux binary changed while staging' "$outer"
grep -Fq 'source tree changed during Admin browser binary build' "$build"
grep -Fq 'initial_head initial_state initial_digest' "$build"
grep -Fq 'final_head final_state final_digest' "$build"
grep -Fq '.viewport(viewport)' "$session"
grep -Fq 'browser viewport differs from 1440x900' "$session"
grep -Fq 'admin-browser-fault-proxy.py' "$outer"
grep -Fq 'schemaVersion:2' "$inner"
grep -Fq 'assertValue' "$inner"
grep -Fq 'assertAbsent' "$inner"
grep -Fq 'faultCases:$faultCases' "$inner"
grep -Fq 'monitor-node-reset-network' "$inner"
grep -Fq 'RUSTZEN_VERIFY_FAULT_METHOD' "$inner"
grep -Fq 'hitCount == 1' "$inner"
grep -Fq 'service_ports=(19801 19802 19803 19804)' "$inner"
grep -Fq 'proxy_port=19805' "$inner"
test "$(grep -Fc 'jq -nc --argjson login' "$inner")" -eq 8
if grep -Fq 'jq -c --argjson login' "$inner"; then
  echo 'browser step composition must use jq null input mode' >&2
  exit 1
fi
grep -Fq 'faultCases | length) == 10' "$outer"
grep -Fq 'receipt.hitCount != 1' "$outer"
grep -Fq 'RUSTZEN_VERIFY_FAULT_METHOD' "$root/scripts/admin-browser-fault-proxy.py"
grep -Fq 'self.command == METHOD and path == ROUTE' "$root/scripts/admin-browser-fault-proxy.py"
grep -Fq 'HITS += 1' "$root/scripts/admin-browser-fault-proxy.py"
python3 "$root/scripts/test-admin-browser-fault-proxy.py"

tmp=$(mktemp -d "${TMPDIR:-/tmp}/rz-ui-gate-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/current" "$tmp/candidate"
printf stale >"$tmp/current/manifest.json"
mkdir "$tmp/.verify.lock"
if mkdir "$tmp/.verify.lock" 2>/dev/null; then
  echo 'concurrent evidence lock unexpectedly succeeded' >&2
  exit 1
fi
test -f "$tmp/current/manifest.json"
rm -rf "$tmp/.verify.lock"
rm -rf "$tmp/current" "$tmp/candidate"
test ! -e "$tmp/current"
test ! -e "$tmp/candidate"
echo 'Admin browser Linux verifier seams passed'
