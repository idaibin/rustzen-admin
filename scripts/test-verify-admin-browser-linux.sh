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
manifest_verifier="$root/scripts/verify-admin-browser-linux-manifest.sh"
ensure="$root/scripts/ensure-admin-browser-verifier-image.sh"
verifier_dockerfile="$root/scripts/admin-browser-verifier.Dockerfile"

for file in "$outer" "$inner" "$docs" "$session" "$identity" "$provenance" "$build" "$manifest_verifier" "$ensure" "$verifier_dockerfile"; do test -s "$file"; done
grep -Fq 'if ! mkdir "$lock_dir"' "$outer"
grep -Fq 'rm -rf "$current" "$candidate"' "$outer"
grep -Fq 'mv "$candidate" "$current"' "$outer"
grep -Fq 'if ! docker rm "$container"' "$outer"
grep -Fq 'ensure-admin-browser-verifier-image.sh' "$outer"
grep -Fq 'run_bounded "$run_timeout" docker run' "$outer"
grep -Fq 'run_bounded_capture "$architecture_timeout" docker info' "$outer"
grep -Fq 'RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256' "$outer"
grep -Fq 'RUSTZEN_VERIFY_CHROMIUM_VERSION' "$outer"
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
grep -Fq 'browser initialization viewport validation failed: expected 1440x900' "$session"
grep -Fq '.launch_timeout(BROWSER_INIT_TIMEOUT)' "$session"
grep -Fq 'shutdown.wait_for(|stopped| *stopped)' "$session"
grep -Fq 'launch_phase(Browser::launch(config))' "$session"
grep -Fq 'browser.new_page("about:blank")' "$session"
grep -Fq '"initial target navigation", page.goto(&system.base_url)' "$session"
grep -Fq 'admin-browser-fault-proxy.py' "$outer"
grep -Fq 'schemaVersion:2' "$inner"
grep -Fq 'setViewport' "$inner"
grep -Fq 'setUiPreferences' "$inner"
grep -Fq 'assertNoHorizontalOverflow' "$inner"
grep -Fq 'schedule-create-daily' "$inner"
grep -Fq 'schedule-view-only-mobile' "$inner"
grep -Fq 'schedule-desktop-dark-en.png' "$outer"
grep -Fq 'schedule-mobile-light-zh.png' "$outer"
grep -Fq 'run-retry-desktop-dark-en.png' "$outer"
grep -Fq 'run-retry-mobile-light-zh.png' "$outer"
grep -Fq 'module-log-desktop-dark-zh.png' "$outer"
grep -Fq 'module-log-mobile-light-en.png' "$outer"
grep -Fq 'successCases:$successCases' "$inner"
grep -Fq 'successCases | length) == 13' "$outer"
grep -Fq 'screenshotViewport' "$inner"
grep -Fq 'schedule-mobile-full' "$inner"
grep -Fq 'run-retry-list-trigger' "$inner"
grep -Fq 'run-retry-list-child-selected' "$inner"
grep -Fq 'run-retry-audit' "$inner"
grep -Fq 'run-retry-list-\($source)' "$inner"
grep -Fq 'run-retry-audit-\($source)' "$inner"
grep -Fq 'run-retry-view-only-mobile' "$inner"
grep -Fq 'run-retry-terminal-hidden' "$inner"
grep -Fq 'module-log-owner-desktop' "$inner"
grep -Fq 'module-log-owner-mobile' "$inner"
grep -Fq 'ADMIN_LOG_CURRENT_MARKER' "$inner"
grep -Fq 'MONITOR_LOG_EXPIRED_MARKER' "$inner"
grep -Fq 'module-log cleanup did not remove only the expired fixture' "$inner"
grep -Fq 'module-log-current-before-cleanup.json' "$inner"
grep -Fq 'module-log-current-after-cleanup.json' "$inner"
grep -Fq 'cmp -s /verify/evidence/module-log-current-before-cleanup.json /verify/evidence/module-log-current-after-cleanup.json' "$inner"
grep -Fq 'retry-source-run.before.json' "$inner"
grep -Fq "data-run-id='\\''\\(\$child)" "$inner"
grep -Fq 'run_viewer_retry_status' "$inner"
grep -Fq 'assertAbsent",selector:".ant-message-error"' "$inner"
grep -Fq 'screenshot",name:"schedule-mobile-full"},{action:"screenshotViewport",name:"schedule-mobile-light-zh"' "$inner"
grep -Fq '1440 x 900' "$outer"
grep -Fq '390 x 844' "$outer"
grep -Fq 'assertValue' "$inner"
grep -Fq 'assertAbsent' "$inner"
grep -Fq 'faultCases:$faultCases' "$inner"
grep -Fq 'monitor-node-reset-network' "$inner"
grep -Fq 'RUSTZEN_VERIFY_FAULT_METHOD' "$inner"
grep -Fq 'hitCount == 1' "$inner"
grep -Fq 'service_ports=(19801 19802 19803 19804)' "$inner"
grep -Fq 'proxy_port=19805' "$inner"
grep -Fq 'initialize_monitor_database' "$inner"
grep -Fq 'rz-monitor init-db' "$inner"
grep -Fq 'rz-monitor bind-database' "$inner"
grep -Fq 'verify_manifest_screenshots' "$outer"
grep -Fq 'io.rustzen.browser-verifier.key' "$ensure"
grep -Fq 'RUSTZEN_UI_VERIFIER_BUILD_TIMEOUT' "$ensure"
grep -Fq 'RUSTZEN_UI_VERIFIER_IMAGE_CHECK_TIMEOUT' "$ensure"
grep -Fq 'run_bounded_capture 10 "$docker_bin" info' "$ensure"
grep -Fq 'io.rustzen.browser-verifier.chromium-version' "$ensure"
grep -Fq 'probe_container' "$ensure"
grep -Fq 'ARG CHROMIUM_VERSION=120.0.6099.224-1~deb11u1' "$verifier_dockerfile"
grep -Fq 'rustzen-browser-verifier.provenance' "$verifier_dockerfile"
test "$(grep -Fc 'jq -nc --argjson login' "$inner")" -ge 8
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

source "$manifest_verifier"
tmp=
manifest_tmp=$(mktemp -d "${TMPDIR:-/tmp}/rz-ui-manifest-test.XXXXXX")
trap 'rm -rf "$tmp" "$manifest_tmp"' EXIT
python3 - "$manifest_tmp" <<'PY'
from pathlib import Path
import hashlib, json, sys
root = Path(sys.argv[1])
png = bytes.fromhex('89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000a49444154789c6360000002000154a24f9d0000000049454e44ae426082')
for name in ('fault.png', 'success.png'):
    (root / name).write_bytes(png)
digest = hashlib.sha256(png).hexdigest()
artifact = lambda name: {'file': name, 'sha256': digest, 'dimensions': '1 x 1'}
(root / 'manifest.json').write_text(json.dumps({'faultCases': [{'artifact': artifact('fault.png')}], 'successCases': [{'artifact': artifact('success.png')}] }))
PY
verify_manifest_screenshots "$manifest_tmp"
printf tamper >>"$manifest_tmp/success.png"
if verify_manifest_screenshots "$manifest_tmp"; then
  echo 'manifest verifier accepted a tampered screenshot' >&2
  exit 1
fi

tmp=$(mktemp -d "${TMPDIR:-/tmp}/rz-ui-gate-test.XXXXXX")
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
