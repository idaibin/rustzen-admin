#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
docker_bin=${RUSTZEN_MONITOR_NOTIFY_DOCKER:-docker}
source_identity=${RUSTZEN_MONITOR_NOTIFY_SOURCE_IDENTITY:-"$root/scripts/admin-browser-source-identity.sh"}
verifier_helper=${RUSTZEN_MONITOR_NOTIFY_VERIFIER_HELPER:-"$root/scripts/ensure-admin-browser-verifier-image.sh"}
file_bin=${RUSTZEN_MONITOR_NOTIFY_FILE:-file}
timeout=${RUSTZEN_MONITOR_NOTIFY_TIMEOUT:-900}
cleanup_timeout=${RUSTZEN_MONITOR_NOTIFY_CLEANUP_TIMEOUT:-10}
kill_grace=${RUSTZEN_MONITOR_NOTIFY_KILL_GRACE:-5}
evidence_root=${RUSTZEN_MONITOR_NOTIFY_EVIDENCE_ROOT:-"$root/target/rz/monitor-notification-runtime"}
current="$evidence_root/current"
command_pid=
watchdog_pid=
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
validate_seconds "$timeout" RUSTZEN_MONITOR_NOTIFY_TIMEOUT 1800
validate_seconds "$cleanup_timeout" RUSTZEN_MONITOR_NOTIFY_CLEANUP_TIMEOUT 30
validate_seconds "$kill_grace" RUSTZEN_MONITOR_NOTIFY_KILL_GRACE 5

run_bounded() {
  seconds=$1; shift
  marker=$(mktemp "${TMPDIR:-/tmp}/rz-monitor-notify-timeout.XXXXXX")
  rm -f "$marker"
  "$@" & command_pid=$!
  ( sleep "$seconds"; : >"$marker"; kill -TERM "$command_pid" 2>/dev/null || true; sleep "$kill_grace"; kill -KILL "$command_pid" 2>/dev/null || true ) & watchdog_pid=$!
  if wait "$command_pid"; then status=0; else status=$?; fi
  stop_tree "$watchdog_pid"
  wait "$watchdog_pid" 2>/dev/null || true
  [ ! -e "$marker" ] || status=124
  rm -f "$marker"
  command_pid=
  watchdog_pid=
  return "$status"
}
run_capture() {
  capture=$(mktemp "${TMPDIR:-/tmp}/rz-monitor-notify-command.XXXXXX")
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
lock="$evidence_root/.verify.lock"
candidate="$evidence_root/.candidate-$run_id"
staged="$evidence_root/.binaries-$run_id"
published="$evidence_root/runs/$run_id"
failed="$evidence_root/failed-runs/$run_id"
publish_link="$evidence_root/.current-$run_id"
build_container="rz-monitor-notify-build-$run_id"
runtime_container="rz-monitor-notify-runtime-$run_id"
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
build_root="$evidence_root/build/$architecture"
rm -rf "$build_root"
mkdir -p "$build_root/bin"
build_active=1
run_bounded "$timeout" "$docker_bin" run --name "$build_container" --platform "$platform" \
  --mount "type=bind,src=$root,dst=/work" \
  --mount type=volume,src=rustzen-monitor-notify-cargo,dst=/usr/local/cargo/registry \
  -w /work rust:1.95-bookworm bash -euo pipefail -c "
    apt-get update >/dev/null
    apt-get install -y --no-install-recommends musl-tools >/dev/null
    rustup target add $target_triple >/dev/null
    export RUSTFLAGS='-C target-feature=+crt-static'
    out=target/rz/monitor-notification-runtime/build/$architecture/bin
    target=target/monitor-notification-runtime/cargo
    cargo build --release --target $target_triple --target-dir \"\$target\" -p rustzen-admin --no-default-features --features monitor-distribution,notifications --bin rz-admin
    install -m 0755 \"\$target/$target_triple/release/rz-admin\" \"\$out/rz-admin-notify\"
    cargo build --release --target $target_triple --target-dir \"\$target\" -p rustzen-monitor --no-default-features --features notifications --bin rz-monitor
    install -m 0755 \"\$target/$target_triple/release/rz-monitor\" \"\$out/rz-monitor-notify\"
    cargo build --release --target $target_triple --target-dir \"\$target\" -p rustzen-admin --no-default-features --features monitor-distribution --bin rz-admin
    install -m 0755 \"\$target/$target_triple/release/rz-admin\" \"\$out/rz-admin-pure\"
    cargo build --release --target $target_triple --target-dir \"\$target\" -p rustzen-monitor --no-default-features --features controller --bin rz-monitor
    install -m 0755 \"\$target/$target_triple/release/rz-monitor\" \"\$out/rz-monitor-pure\"
  "
remove_container "$build_container"
build_active=0

binary_hashes='{}'
for name in rz-admin-notify rz-monitor-notify rz-admin-pure rz-monitor-pure; do
  binary="$build_root/bin/$name"
  test -x "$binary"
  "$file_bin" "$binary" | grep -q "$file_pattern"
  cp "$binary" "$staged/$name"
  hash=$(shasum -a 256 "$staged/$name" | awk '{print $1}')
  binary_hashes=$(jq -nc --argjson current "$binary_hashes" --arg name "$name" --arg hash "$hash" '$current + {($name):$hash}')
done
[ "$(jq -r '."rz-admin-notify"' <<<"$binary_hashes")" != "$(jq -r '."rz-admin-pure"' <<<"$binary_hashes")" ]
[ "$(jq -r '."rz-monitor-notify"' <<<"$binary_hashes")" != "$(jq -r '."rz-monitor-pure"' <<<"$binary_hashes")" ]
{
  printf 'schemaVersion\t1\nhead\t%s\nsourceTreeState\t%s\nsourceTreeSha256\t%s\narchitecture\t%s\ntargetTriple\t%s\n' \
    "$head" "$source_state" "$source_sha" "$architecture" "$target_triple"
  printf 'rz-admin-notify\tmonitor-distribution,notifications\t%s\n' "$(jq -r '."rz-admin-notify"' <<<"$binary_hashes")"
  printf 'rz-monitor-notify\tnotifications,no-default-features\t%s\n' "$(jq -r '."rz-monitor-notify"' <<<"$binary_hashes")"
  printf 'rz-admin-pure\tmonitor-distribution,no-default-features\t%s\n' "$(jq -r '."rz-admin-pure"' <<<"$binary_hashes")"
  printf 'rz-monitor-pure\tcontroller,no-default-features\t%s\n' "$(jq -r '."rz-monitor-pure"' <<<"$binary_hashes")"
} >"$candidate/build-provenance.tsv"
build_provenance_sha=$(shasum -a 256 "$candidate/build-provenance.tsv" | awk '{print $1}')

read -r verifier_image verifier_key verifier_sha < <(
  RUSTZEN_UI_VERIFIER_DOCKER="$docker_bin" "$verifier_helper" --platform "$platform"
)
runtime_active=1
run_bounded "$timeout" "$docker_bin" run --name "$runtime_container" --platform "$platform" \
  --security-opt seccomp=unconfined \
  --env RUSTZEN_VERIFY_HEAD="$head" --env RUSTZEN_VERIFY_SOURCE_TREE_STATE="$source_state" \
  --env RUSTZEN_VERIFY_SOURCE_TREE_SHA256="$source_sha" --env RUSTZEN_VERIFY_ARCHITECTURE="$architecture" \
  --env RUSTZEN_VERIFY_PLATFORM="$platform" --env RUSTZEN_VERIFY_BINARY_HASHES="$binary_hashes" \
  --env RUSTZEN_VERIFY_BUILD_PROVENANCE_SHA256="$build_provenance_sha" \
  --env RUSTZEN_VERIFY_VERIFIER_IMAGE_ID="$verifier_image" --env RUSTZEN_VERIFY_VERIFIER_KEY="$verifier_key" \
  --env RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256="$verifier_sha" \
  --mount "type=bind,src=$staged,dst=/verify/bin,readonly" \
  --mount "type=bind,src=$candidate,dst=/verify/evidence" \
  --mount "type=bind,src=$root/scripts/verify-monitor-notification-runtime-linux-inner.sh,dst=/verify/run.sh,readonly" \
  --mount "type=bind,src=$root/scripts/monitor-notification-runtime-client.py,dst=/verify/client.py,readonly" \
  "$verifier_image" bash /verify/run.sh
remove_container "$runtime_container"
runtime_active=0

test -f "$candidate/manifest.json"
receipt_allowlist=$(jq -nc '[
  "steps.log", "open-event.json", "open-event.json.identity", "selected-ingress-listener.txt",
  "inbox-open.json", "duplicate.json", "bad-signature.json", "unsigned.json",
  "public-internal.json", "inbox-final.json", "selected-state.json", "plain-admin-api.json",
  "plain-admin-config.json", "plain-monitor-api.json", "plain-monitor-config.json",
  "plain-absence.json", "plain-notification-route.json", "plain-listeners.txt"
]')
jq -e --arg head "$head" --arg state "$source_state" --arg sourceSha "$source_sha" \
  --arg architecture "$architecture" --arg platform "$platform" --argjson binaries "$binary_hashes" \
  --arg provenanceSha "$build_provenance_sha" --arg verifierImage "$verifier_image" \
  --arg verifierKey "$verifier_key" --arg verifierSha "$verifier_sha" \
  --argjson allowlist "$receipt_allowlist" --rawfile identity "$candidate/open-event.json.identity" \
  --slurpfile open "$candidate/open-event.json" --slurpfile inboxOpen "$candidate/inbox-open.json" \
  --slurpfile inbox "$candidate/inbox-final.json" --rawfile selectedListener "$candidate/selected-ingress-listener.txt" \
  --slurpfile selected "$candidate/selected-state.json" --slurpfile duplicate "$candidate/duplicate.json" \
  --slurpfile bad "$candidate/bad-signature.json" --slurpfile unsigned "$candidate/unsigned.json" \
  --slurpfile public "$candidate/public-internal.json" --slurpfile plainAdminApi "$candidate/plain-admin-api.json" \
  --slurpfile plainAdminConfig "$candidate/plain-admin-config.json" --slurpfile plainMonitorApi "$candidate/plain-monitor-api.json" \
  --slurpfile plainMonitorConfig "$candidate/plain-monitor-config.json" --slurpfile plainAbsence "$candidate/plain-absence.json" \
  --slurpfile plainRoute "$candidate/plain-notification-route.json" --rawfile plainListeners "$candidate/plain-listeners.txt" \
  -f "$root/scripts/verify-monitor-notification-runtime-evidence.jq" "$candidate/manifest.json" >/dev/null
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
echo "Monitor notification Linux evidence published: $current/manifest.json"
