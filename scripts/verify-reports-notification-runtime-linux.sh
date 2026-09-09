#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
docker_bin=${RUSTZEN_REPORTS_NOTIFY_DOCKER:-docker}
source_identity=${RUSTZEN_REPORTS_NOTIFY_SOURCE_IDENTITY:-"$root/scripts/admin-browser-source-identity.sh"}
verifier_helper=${RUSTZEN_REPORTS_NOTIFY_VERIFIER_HELPER:-"$root/scripts/ensure-admin-browser-verifier-image.sh"}
file_bin=${RUSTZEN_REPORTS_NOTIFY_FILE:-file}
build_timeout=${RUSTZEN_REPORTS_NOTIFY_BUILD_TIMEOUT:-3600}
runtime_timeout=${RUSTZEN_REPORTS_NOTIFY_RUNTIME_TIMEOUT:-900}
cleanup_timeout=${RUSTZEN_REPORTS_NOTIFY_CLEANUP_TIMEOUT:-10}
kill_grace=${RUSTZEN_REPORTS_NOTIFY_KILL_GRACE:-5}
evidence_root=${RUSTZEN_REPORTS_NOTIFY_EVIDENCE_ROOT:-"$root/target/rz/reports-notification-runtime"}
current="$evidence_root/current"
command_pid=
watchdog_pid=
bounded_timed_out=0
build_active=0
runtime_active=0

descendants() {
  local parent=$1 child
  while IFS= read -r child; do
    [ -z "$child" ] || { descendants "$child"; printf '%s\n' "$child"; }
  done < <(pgrep -P "$parent" 2>/dev/null || true)
}
stop_tree() {
  local parent=${1:-} tree child
  [ -n "$parent" ] || return 0
  tree=$(descendants "$parent")
  while IFS= read -r child; do [ -z "$child" ] || kill -TERM "$child" 2>/dev/null || true; done <<<"$tree"
  kill -TERM "$parent" 2>/dev/null || true
}

validate_seconds() {
  value=$1 name=$2 maximum=$3
  case "$value" in ''|*[!0-9]*) echo "$name must be a positive integer" >&2; exit 2 ;; esac
  [ "$value" -gt 0 ] && [ "$value" -le "$maximum" ] || { echo "$name must be 1..$maximum seconds" >&2; exit 2; }
}
validate_seconds "$build_timeout" RUSTZEN_REPORTS_NOTIFY_BUILD_TIMEOUT 7200
validate_seconds "$runtime_timeout" RUSTZEN_REPORTS_NOTIFY_RUNTIME_TIMEOUT 1800
validate_seconds "$cleanup_timeout" RUSTZEN_REPORTS_NOTIFY_CLEANUP_TIMEOUT 30
validate_seconds "$kill_grace" RUSTZEN_REPORTS_NOTIFY_KILL_GRACE 5

run_bounded() {
  seconds=$1; shift
  bounded_timed_out=0
  marker=$(mktemp "${TMPDIR:-/tmp}/rz-reports-selected-timeout.XXXXXX")
  rm -f "$marker"
  "$@" & command_pid=$!
  ( sleep "$seconds"; : >"$marker"; kill -TERM "$command_pid" 2>/dev/null || true; sleep "$kill_grace"; kill -KILL "$command_pid" 2>/dev/null || true ) & watchdog_pid=$!
  if wait "$command_pid"; then status=0; else status=$?; fi
  stop_tree "$watchdog_pid"
  wait "$watchdog_pid" 2>/dev/null || true
  [ ! -e "$marker" ] || { status=124; bounded_timed_out=1; }
  rm -f "$marker"
  command_pid=
  watchdog_pid=
  return "$status"
}
run_capture() {
  capture=$(mktemp "${TMPDIR:-/tmp}/rz-reports-selected-command.XXXXXX")
  if run_bounded "$@" >"$capture"; then status=0; else status=$?; fi
  cat "$capture"
  rm -f "$capture"
  return "$status"
}
atomic_replace_symlink() {
  case "$(uname -s)" in Darwin|FreeBSD) mv -fh "$1" "$2" ;; *) mv -Tf "$1" "$2" ;; esac
}
remove_container() {
  name=$1
  run_bounded "$cleanup_timeout" "$docker_bin" rm -f "$name" >/dev/null 2>&1 || {
    echo "failed to remove Docker container $name" >&2
    return 1
  }
  if run_bounded "$cleanup_timeout" "$docker_bin" container inspect "$name" >/dev/null 2>&1; then
    echo "Docker container still exists after removal: $name" >&2
    return 1
  else
    inspect_status=$?
  fi
  [ "$inspect_status" -eq 1 ] || {
    echo "Docker container absence check failed for $name (status $inspect_status)" >&2
    return 1
  }
}

architecture=$(run_capture 10 "$docker_bin" info --format '{{.Architecture}}') || {
  echo 'Docker architecture discovery failed or timed out' >&2
  exit 1
}
case "$architecture" in
  aarch64) platform=linux/arm64; target_triple=aarch64-unknown-linux-musl; file_pattern='ELF 64-bit.*ARM aarch64' ;;
  x86_64) platform=linux/amd64; target_triple=x86_64-unknown-linux-musl; file_pattern='ELF 64-bit.*x86-64' ;;
  *) echo "unsupported Colima/Docker architecture: $architecture" >&2; exit 1 ;;
esac

mkdir -p "$evidence_root/runs" "$evidence_root/failed-runs"
[ ! -e "$current" ] || [ -L "$current" ] || {
  echo "refusing to replace non-symlink evidence path: $current" >&2
  exit 1
}
run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$"
target_cache_schema=v1
target_cache="rustzen-reports-notify-target-${target_cache_schema}-${architecture}-${target_triple}-rust195-crtstatic"
lock="$evidence_root/.verify.lock"
candidate="$evidence_root/.candidate-$run_id"
staged="$evidence_root/.binaries-$run_id"
published="$evidence_root/runs/$run_id"
failed="$evidence_root/failed-runs/$run_id"
publish_link="$evidence_root/.current-$run_id"
build_container="rz-reports-selected-build-$run_id"
runtime_container="rz-reports-selected-runtime-$run_id"
mkdir "$lock" 2>/dev/null || { echo "another notification runtime gate owns $lock" >&2; exit 1; }
mkdir "$candidate" "$staged"

cleanup() {
  result=$?
  trap - EXIT INT TERM
  stop_tree "$watchdog_pid"
  stop_tree "$command_pid"
  [ "$build_active" -eq 0 ] || run_bounded "$cleanup_timeout" "$docker_bin" rm -f "$build_container" >/dev/null 2>&1 || true
  if [ "$result" -ne 0 ] && [ -d "$candidate" ]; then
    [ "$runtime_active" -eq 0 ] || run_bounded "$cleanup_timeout" "$docker_bin" logs "$runtime_container" >"$candidate/container.log" 2>&1 || true
    mv "$candidate" "$failed"
  fi
  [ "$runtime_active" -eq 0 ] || run_bounded "$cleanup_timeout" "$docker_bin" rm -f "$runtime_container" >/dev/null 2>&1 || true
  rm -rf "$staged" "$publish_link" "$lock"
  exit "$result"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

read -r head source_state source_sha < <("$source_identity")
build_active=1
if run_bounded "$build_timeout" "$docker_bin" run --name "$build_container" --platform "$platform" \
  --mount "type=bind,src=$root,dst=/work" \
  --mount type=volume,src=rustzen-reports-notify-cargo,dst=/usr/local/cargo/registry \
  --mount "type=volume,src=$target_cache,dst=/cargo-target" \
  --mount "type=bind,src=$staged,dst=/out" \
  -w /work rust:1.95-bookworm bash -euo pipefail -c "
    apt-get update >/dev/null
    apt-get install -y --no-install-recommends musl-tools util-linux >/dev/null
    rustup target add $target_triple >/dev/null
    export RUSTFLAGS='-C target-feature=+crt-static'
    out=/out
    selected_target=/cargo-target
    pure_target=/cargo-target/pure-v1
    command -v flock >/dev/null
    exec 9>/cargo-target/.gate.lock
    flock -x 9
    cargo build --release --target $target_triple --target-dir \"\$selected_target\" -p rustzen-admin --bin rz-admin
    install -m 0755 \"\$selected_target/$target_triple/release/rz-admin\" \"\$out/rz-admin-selected\"
    cargo build --release --target $target_triple --target-dir \"\$selected_target\" -p rustzen-reports --bin rz-reports
    install -m 0755 \"\$selected_target/$target_triple/release/rz-reports\" \"\$out/rz-reports-selected\"
    cargo build --release --target $target_triple --target-dir \"\$pure_target\" -p rustzen-admin --no-default-features --features monitor-distribution --bin rz-admin
    install -m 0755 \"\$pure_target/$target_triple/release/rz-admin\" \"\$out/rz-admin-pure\"
    cargo build --release --target $target_triple --target-dir \"\$pure_target\" -p rustzen-reports --no-default-features --bin rz-reports
    install -m 0755 \"\$pure_target/$target_triple/release/rz-reports\" \"\$out/rz-reports-pure\"
  "
then :; else
  status=$?
  if [ "$bounded_timed_out" -eq 1 ]; then
    echo "build stage timed out after ${build_timeout}s" >&2
  else
    echo "build stage failed (status $status)" >&2
  fi
  exit "$status"
fi
remove_container "$build_container"
build_active=0

binary_hashes='{}'
for name in rz-admin-selected rz-reports-selected rz-admin-pure rz-reports-pure; do
  binary="$staged/$name"
  test -x "$binary"
  "$file_bin" "$binary" | grep -q "$file_pattern"
  hash=$(shasum -a 256 "$staged/$name" | awk '{print $1}')
  binary_hashes=$(jq -nc --argjson current "$binary_hashes" --arg name "$name" --arg hash "$hash" '$current + {($name):$hash}')
done
[ "$(jq -r '."rz-admin-selected"' <<<"$binary_hashes")" != "$(jq -r '."rz-admin-pure"' <<<"$binary_hashes")" ]
[ "$(jq -r '."rz-reports-selected"' <<<"$binary_hashes")" != "$(jq -r '."rz-reports-pure"' <<<"$binary_hashes")" ]
{
  printf 'schemaVersion\t1\nhead\t%s\nsourceTreeState\t%s\nsourceTreeSha256\t%s\narchitecture\t%s\ntargetTriple\t%s\n' \
    "$head" "$source_state" "$source_sha" "$architecture" "$target_triple"
  printf 'rz-admin-selected\tfull(default)\t%s\n' "$(jq -r '."rz-admin-selected"' <<<"$binary_hashes")"
  printf 'rz-reports-selected\tnotifications(default)\t%s\n' "$(jq -r '."rz-reports-selected"' <<<"$binary_hashes")"
  printf 'rz-admin-pure\tmonitor-distribution,no-default-features\t%s\n' "$(jq -r '."rz-admin-pure"' <<<"$binary_hashes")"
  printf 'rz-reports-pure\tno-default-features\t%s\n' "$(jq -r '."rz-reports-pure"' <<<"$binary_hashes")"
} >"$candidate/build-provenance.tsv"
build_provenance_sha=$(shasum -a 256 "$candidate/build-provenance.tsv" | awk '{print $1}')

read -r verifier_image verifier_key verifier_sha < <(
  RUSTZEN_UI_VERIFIER_DOCKER="$docker_bin" "$verifier_helper" --platform "$platform"
)
runtime_active=1
if run_bounded "$runtime_timeout" "$docker_bin" run --name "$runtime_container" --platform "$platform" \
  --security-opt seccomp=unconfined \
  --env RUSTZEN_VERIFY_HEAD="$head" --env RUSTZEN_VERIFY_SOURCE_TREE_STATE="$source_state" \
  --env RUSTZEN_VERIFY_SOURCE_TREE_SHA256="$source_sha" --env RUSTZEN_VERIFY_ARCHITECTURE="$architecture" \
  --env RUSTZEN_VERIFY_PLATFORM="$platform" --env RUSTZEN_VERIFY_BINARY_HASHES="$binary_hashes" \
  --env RUSTZEN_VERIFY_BUILD_PROVENANCE_SHA256="$build_provenance_sha" \
  --env RUSTZEN_VERIFY_VERIFIER_IMAGE_ID="$verifier_image" --env RUSTZEN_VERIFY_VERIFIER_KEY="$verifier_key" \
  --env RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256="$verifier_sha" \
  --mount "type=bind,src=$staged,dst=/verify/bin,readonly" \
  --mount "type=bind,src=$candidate,dst=/verify/evidence" \
  --mount "type=bind,src=$root/scripts/verify-reports-notification-runtime-linux-inner.sh,dst=/verify/run.sh,readonly" \
  --mount "type=bind,src=$root/scripts/reports-notification-runtime-client.py,dst=/verify/client.py,readonly" \
  "$verifier_image" bash /verify/run.sh
then :; else
  status=$?
  if [ "$bounded_timed_out" -eq 1 ]; then
    echo "runtime stage timed out after ${runtime_timeout}s" >&2
  else
    echo "runtime stage failed (status $status)" >&2
  fi
  exit "$status"
fi
remove_container "$runtime_container"
runtime_active=0

test -f "$candidate/manifest.json"
receipt_allowlist=$(jq -nc '[
  "steps.log","lifecycle.json","outage-state.json","retry-state.json","scheduled-state.json",
  "revoked-state.json","drop-event.json","drop-proxy.json","duplicate.json","bad-signature.json",
  "unsigned.json","public-internal.json","selected-ingress-listener.txt","selected-reports-config.json",
  "selected-reports-api.json","inbox-final.json","selected-state.json","reports-runtime-identity.json","pure-admin-api.json",
  "pure-admin-config.json","pure-reports-config.json","pure-reports-api.json",
  "pure-absence.json","pure-notification-route.json","pure-listeners.txt"
]')
jq -e --arg head "$head" --arg state "$source_state" --arg sourceSha "$source_sha" \
  --arg architecture "$architecture" --arg platform "$platform" --argjson binaries "$binary_hashes" \
  --arg provenanceSha "$build_provenance_sha" --arg verifierImage "$verifier_image" \
  --arg verifierKey "$verifier_key" --arg verifierSha "$verifier_sha" --argjson allowlist "$receipt_allowlist" \
  --slurpfile lifecycle "$candidate/lifecycle.json" --slurpfile outage "$candidate/outage-state.json" \
  --slurpfile retry "$candidate/retry-state.json" --slurpfile scheduled "$candidate/scheduled-state.json" \
  --slurpfile revoked "$candidate/revoked-state.json" --slurpfile proxy "$candidate/drop-proxy.json" \
  --slurpfile inbox "$candidate/inbox-final.json" --rawfile selectedListener "$candidate/selected-ingress-listener.txt" \
  --slurpfile runtimeIdentity "$candidate/reports-runtime-identity.json" \
  --slurpfile selectedState "$candidate/selected-state.json" --slurpfile selectedConfig "$candidate/selected-reports-config.json" \
  --slurpfile selectedApi "$candidate/selected-reports-api.json" --slurpfile duplicate "$candidate/duplicate.json" \
  --slurpfile bad "$candidate/bad-signature.json" --slurpfile unsigned "$candidate/unsigned.json" --slurpfile public "$candidate/public-internal.json" \
  --slurpfile pureAdminApi "$candidate/pure-admin-api.json" --slurpfile pureAdminConfig "$candidate/pure-admin-config.json" \
  --slurpfile pureReportsConfig "$candidate/pure-reports-config.json" --slurpfile pureReportsApi "$candidate/pure-reports-api.json" \
  --slurpfile pureAbsence "$candidate/pure-absence.json" --slurpfile pureRoute "$candidate/pure-notification-route.json" --rawfile pureListeners "$candidate/pure-listeners.txt" \
  -f "$root/scripts/verify-reports-notification-runtime-evidence.jq" "$candidate/manifest.json" >/dev/null
test "$(shasum -a 256 "$candidate/build-provenance.tsv" | awk '{print $1}')" = "$build_provenance_sha"
while IFS=$'\t' read -r file sha bytes; do
  test -f "$candidate/$file" && test ! -L "$candidate/$file"
  test "$(shasum -a 256 "$candidate/$file" | awk '{print $1}')" = "$sha"
  test "$(stat -f %z "$candidate/$file" 2>/dev/null || stat -c %s "$candidate/$file")" = "$bytes"
done < <(jq -r '.receipts[] | [.file,.sha256,.bytes] | @tsv' "$candidate/manifest.json")
read -r final_head final_state final_sha < <("$source_identity")
[ "$final_head:$final_state:$final_sha" = "$head:$source_state:$source_sha" ] || {
  echo 'source tree changed during notification runtime gate' >&2
  exit 1
}
mv "$candidate" "$published"
ln -s "runs/$run_id" "$publish_link"
atomic_replace_symlink "$publish_link" "$current"
echo "Reports notification Linux evidence published: $current/manifest.json"
