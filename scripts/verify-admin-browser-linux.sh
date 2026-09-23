#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
source "$root/scripts/verify-admin-browser-linux-manifest.sh"
run_bounded() {
  seconds=$1; shift
  "$@" & command_pid=$!
  ( sleep "$seconds"; kill -TERM "$command_pid" 2>/dev/null || true; sleep 10; kill -KILL "$command_pid" 2>/dev/null || true ) & watchdog_pid=$!
  if wait "$command_pid"; then command_status=0; else command_status=$?; fi
  kill "$watchdog_pid" 2>/dev/null || true
  wait "$watchdog_pid" 2>/dev/null || true
  return "$command_status"
}

run_bounded_capture() {
  capture=$(mktemp "${TMPDIR:-/tmp}/rz-browser-command.XXXXXX") || return 1
  if run_bounded "$@" >"$capture"; then command_status=0; else command_status=$?; fi
  cat "$capture"
  rm -f "$capture"
  return "$command_status"
}

architecture_timeout=10
if [ -n "${RUSTZEN_UI_LINUX_ARCH:-}" ]; then
  architecture=$RUSTZEN_UI_LINUX_ARCH
else
  architecture=$(run_bounded_capture "$architecture_timeout" docker info --format '{{.Architecture}}') || {
    echo "Docker architecture discovery failed or exceeded ${architecture_timeout} seconds" >&2
    exit 1
  }
fi
case "$architecture" in
  aarch64) platform=linux/arm64; target_triple=aarch64-unknown-linux-musl; file_pattern='ELF 64-bit.*ARM aarch64' ;;
  x86_64) platform=linux/amd64; target_triple=x86_64-unknown-linux-musl; file_pattern='ELF 64-bit.*x86-64' ;;
  *) echo "unsupported Colima/Docker architecture: $architecture" >&2; exit 1 ;;
esac
run_timeout=${RUSTZEN_UI_BROWSER_RUN_TIMEOUT:-2400}
case "$run_timeout" in ''|*[!0-9]*) echo 'RUSTZEN_UI_BROWSER_RUN_TIMEOUT must be a positive integer' >&2; exit 2;; esac
[ "$run_timeout" -gt 0 ] && [ "$run_timeout" -le 2400 ] || { echo 'RUSTZEN_UI_BROWSER_RUN_TIMEOUT must be 1..2400 seconds' >&2; exit 2; }
read -r verifier_image verifier_key verifier_provenance_sha < <("$root/scripts/ensure-admin-browser-verifier-image.sh" --platform "$platform")
bin_dir=${RUSTZEN_UI_LINUX_BIN_DIR:-"$root/target/rz/build/$architecture/bin"}
evidence_root="$root/target/rz/ui-browser"
current="$evidence_root/current"
lock_dir="$evidence_root/.verify.lock"
run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$"
candidate="$evidence_root/.candidate-$run_id"
staged_bin_dir="$evidence_root/.binaries-$run_id"
container="rz-admin-browser-$run_id"
status=1

mkdir -p "$evidence_root"
if ! mkdir "$lock_dir" 2>/dev/null; then
  echo "another Admin browser verification owns $lock_dir" >&2
  exit 1
fi
rm -rf "$current" "$candidate" "$staged_bin_dir"
mkdir -p "$candidate" "$staged_bin_dir"

cleanup() {
  status=$?
  trap - EXIT INT TERM
  if [ -d "$candidate" ]; then
    docker logs "$container" >"$candidate/container.log" 2>&1 || true
  fi
  rm -rf "$staged_bin_dir"
  docker rm -f "$container" >/dev/null 2>&1 || true
  if [ "$status" -ne 0 ]; then
    rm -rf "$candidate" "$current"
  fi
  rm -rf "$lock_dir"
  exit "$status"
}
trap cleanup EXIT INT TERM

for name in rz-admin rz-monitor rz-insights rz-reports; do
  binary="$bin_dir/$name"
  test -x "$binary" || { echo "missing $architecture Linux binary: $binary" >&2; exit 1; }
  file "$binary" | grep -q "$file_pattern" || { echo "binary does not match $architecture Linux: $binary" >&2; exit 1; }
done
read -r head initial_source_tree_state initial_source_tree_sha256 < <("$root/scripts/admin-browser-source-identity.sh")
test -f "$bin_dir/build-provenance.txt" || { echo "missing build provenance: $bin_dir/build-provenance.txt" >&2; exit 1; }
expected_provenance="$candidate/expected-build-provenance.txt"
{
  printf 'schemaVersion\t1\n'
  printf 'gitHead\t%s\n' "$head"
  printf 'sourceTreeState\t%s\n' "$initial_source_tree_state"
  printf 'sourceTreeSha256\t%s\n' "$initial_source_tree_sha256"
  printf 'architecture\t%s\n' "$architecture"
  printf 'targetTriple\t%s\n' "$target_triple"
  printf 'platform\t%s\n' "$platform"
  printf 'distribution\tfull\n'
  for name in rz-admin rz-monitor rz-insights rz-reports; do
    printf '%s\t%s\n' "$name" "$(shasum -a 256 "$bin_dir/$name" | awk '{print $1}')"
  done
} >"$expected_provenance"
cmp -s "$expected_provenance" "$bin_dir/build-provenance.txt" || {
  echo "Linux binaries do not match the current source-tree build provenance" >&2
  exit 1
}
mv "$expected_provenance" "$candidate/build-provenance.txt"
for name in rz-admin rz-monitor rz-insights rz-reports; do
  cp "$bin_dir/$name" "$staged_bin_dir/$name"
  expected_hash=$(awk -F '\t' -v name="$name" '$1 == name { print $2 }' "$candidate/build-provenance.txt")
  actual_hash=$(shasum -a 256 "$staged_bin_dir/$name" | awk '{print $1}')
  test "$actual_hash" = "$expected_hash" || {
    echo "Linux binary changed while staging: $name" >&2
    exit 1
  }
done
binary_hashes=$(for name in rz-admin rz-monitor rz-insights rz-reports; do
  printf '%s ' "$name"
  shasum -a 256 "$staged_bin_dir/$name" | awk '{print $1}'
done)

run_bounded "$run_timeout" docker run --name "$container" --platform "$platform" --security-opt seccomp=unconfined \
  --env RUSTZEN_VERIFY_HEAD="$head" \
  --env RUSTZEN_VERIFY_SOURCE_TREE_STATE="$initial_source_tree_state" \
  --env RUSTZEN_VERIFY_SOURCE_TREE_SHA256="$initial_source_tree_sha256" \
  --env RUSTZEN_VERIFY_ARCHITECTURE="$architecture" \
  --env RUSTZEN_VERIFY_CHROMIUM_VERSION=120.0.6099.224-1~deb11u1 \
  --env RUSTZEN_VERIFY_VERIFIER_IMAGE_ID="$verifier_image" \
  --env RUSTZEN_VERIFY_VERIFIER_KEY="$verifier_key" \
  --env RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256="$verifier_provenance_sha" \
  --env RUSTZEN_VERIFY_BINARY_HASHES="$binary_hashes" \
  --mount "type=bind,src=$staged_bin_dir,dst=/verify/bin,readonly" \
  --mount "type=bind,src=$candidate,dst=/verify/evidence" \
  --mount "type=bind,src=$root/scripts/verify-admin-browser-linux-inner.sh,dst=/verify/run.sh,readonly" \
  --mount "type=bind,src=$root/scripts/admin-browser-fault-proxy.py,dst=/verify/fault-proxy.py,readonly" \
  "$verifier_image" bash /verify/run.sh

test -f "$candidate/manifest.json"
test -f "$candidate/dashboard.png"
test -f "$candidate/analytics-details.png"
test -f "$candidate/schedule-desktop-dark-en.png"
test -f "$candidate/schedule-mobile-light-zh.png"
test -f "$candidate/run-retry-desktop-dark-en.png"
test -f "$candidate/run-retry-mobile-light-zh.png"
test -f "$candidate/module-log-desktop-dark-zh.png"
test -f "$candidate/module-log-mobile-light-en.png"
test -f "$candidate/schedule-occurrence-run-desktop-dark-en.png"
jq -e '
  .schemaVersion == 2 and
  (.verifier.imageId == $verifier_image and .verifier.key == $verifier_key and .verifier.provenanceSha256 == $verifier_provenance_sha) and
  (.successCases | length) == 15 and
  ([.successCases[] | .name, .runId, .execution] | all(. != null)) and
  ([.successCases[] | .name] | sort) == ["module-log-owner-desktop", "module-log-owner-mobile", "run-retry-audit", "run-retry-list-child-selected", "run-retry-list-trigger", "run-retry-terminal-hidden", "run-retry-view-only-mobile", "schedule-create-daily", "schedule-delete", "schedule-disable", "schedule-edit-weekly", "schedule-enable", "schedule-occurrence-enqueued-link", "schedule-occurrence-missed-no-link", "schedule-view-only-mobile"] and
  ([.successCases[] | select(.execution != "target-backed")] | length) == 0 and
  ([.successCases[] | select(.name == "schedule-create-daily") | .artifact.file == "schedule-desktop-dark-en.png" and .artifact.dimensions == "1440 x 900" and (.artifact.sha256 | test("^[0-9a-f]{64}$"))] | all) and
  ([.successCases[] | select(.name == "schedule-view-only-mobile") | .artifact.file == "schedule-mobile-light-zh.png" and .artifact.dimensions == "390 x 844" and (.artifact.sha256 | test("^[0-9a-f]{64}$"))] | all) and
  ([.successCases[] | select(.name == "schedule-occurrence-enqueued-link") | .scheduleId != null and .scheduledRunId != null and .artifact.file == "schedule-occurrence-run-desktop-dark-en.png" and .artifact.dimensions == "1440 x 900" and (.artifact.sha256 | test("^[0-9a-f]{64}$"))] | all) and
  ([.successCases[] | select(.name == "schedule-occurrence-missed-no-link") | .scheduleId != null and .fixture == "controlled-sqlite-missed-occurrence" and (.artifact == null)] | all) and
  ([.successCases[] | select(.name == "run-retry-list-child-selected") | .artifact.file == "run-retry-desktop-dark-en.png" and .artifact.dimensions == "1440 x 900" and (.artifact.sha256 | test("^[0-9a-f]{64}$"))] | all) and
  ([.successCases[] | select(.name == "run-retry-view-only-mobile") | .artifact.file == "run-retry-mobile-light-zh.png" and .artifact.dimensions == "390 x 844" and (.artifact.sha256 | test("^[0-9a-f]{64}$"))] | all) and
  ([.successCases[] | select(.name == "module-log-owner-desktop") | .artifact.file == "module-log-desktop-dark-zh.png" and .artifact.dimensions == "1440 x 900" and (.artifact.sha256 | test("^[0-9a-f]{64}$"))] | all) and
  ([.successCases[] | select(.name == "module-log-owner-mobile") | .artifact.file == "module-log-mobile-light-en.png" and .artifact.dimensions == "390 x 844" and (.artifact.sha256 | test("^[0-9a-f]{64}$"))] | all) and
  (.faultCases | length) == 10 and
  ([.faultCases[] | .runId, .method, .mode, .route, .receipt.method, .receipt.mode, .receipt.route, .receipt.hitCount, .artifact.file, .artifact.sha256, .artifact.dimensions] | all(. != null)) and
  ([.faultCases[] | select(.receipt.hitCount != 1)] | length) == 0 and
  ([.faultCases[] | "\(.method) \(.mode) \((if (.route | startswith("/api/reports/schedules/")) then "/api/reports/schedules/{id}" else .route end))"] | sort) == [
    "DELETE http /api/monitor/nodes/browser-fault-node/alert-settings", "DELETE network /api/monitor/nodes/browser-fault-node/alert-settings",
    "POST http /api/reports/schedules", "POST network /api/reports/schedules",
    "PUT http /api/monitor/alert-settings", "PUT http /api/monitor/nodes/browser-fault-node/alert-settings", "PUT http /api/reports/schedules/{id}",
    "PUT network /api/monitor/alert-settings", "PUT network /api/monitor/nodes/browser-fault-node/alert-settings", "PUT network /api/reports/schedules/{id}"
  ]
' --arg verifier_image "$verifier_image" --arg verifier_key "$verifier_key" --arg verifier_provenance_sha "$verifier_provenance_sha" "$candidate/manifest.json" >/dev/null
verify_manifest_screenshots "$candidate"
for image in "$candidate"/*.png; do
  [ "$(od -An -tx1 -N8 "$image" | tr -d ' \n')" = 89504e470d0a1a0a ]
  file "$image" | grep -Eq 'PNG image data, [1-9][0-9]* x [1-9][0-9]*'
  sha256sum "$image" | grep -Eq '^[0-9a-f]{64}[[:space:]]'
done
read -r final_head final_source_tree_state final_source_tree_sha256 < <("$root/scripts/admin-browser-source-identity.sh")
test "$final_head" = "$head" || {
  echo "Git HEAD changed during verification" >&2
  exit 1
}
test "$final_source_tree_state" = "$initial_source_tree_state" || {
  echo "source tree state changed during verification" >&2
  exit 1
}
test "$final_source_tree_sha256" = "$initial_source_tree_sha256" || {
  echo "source tree content changed during verification" >&2
  exit 1
}
if ! docker rm "$container" >/dev/null; then
  echo "verification container cleanup failed: $container" >&2
  exit 1
fi
rm -rf "$staged_bin_dir"
mv "$candidate" "$current"
status=0
trap - EXIT INT TERM
rm -rf "$lock_dir"
echo "Admin/Reports Linux browser gate passed: $current"
