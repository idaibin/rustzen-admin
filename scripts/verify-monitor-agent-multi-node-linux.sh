#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
docker_bin=${RUSTZEN_MONITOR_MULTI_NODE_DOCKER:-docker}
architecture=${RUSTZEN_UI_LINUX_ARCH:-}
evidence_root="$root/target/rz/monitor-agent-multi-node"
current="$evidence_root/current"
run_timeout=${RUSTZEN_MONITOR_MULTI_NODE_TIMEOUT:-240}
build_timeout=${RUSTZEN_MONITOR_MULTI_NODE_BUILD_TIMEOUT:-900}
info_timeout=${RUSTZEN_MONITOR_MULTI_NODE_DOCKER_INFO_TIMEOUT:-10}
log_timeout=${RUSTZEN_MONITOR_MULTI_NODE_LOG_TIMEOUT:-15}

validate_seconds() {
  value=$1 name=$2 maximum=$3
  case "$value" in ''|*[!0-9]*) echo "$name must be a positive integer" >&2; exit 2 ;; esac
  [ "$value" -gt 0 ] && [ "$value" -le "$maximum" ] || { echo "$name must be 1..$maximum seconds" >&2; exit 2; }
}
validate_seconds "$run_timeout" RUSTZEN_MONITOR_MULTI_NODE_TIMEOUT 600
validate_seconds "$build_timeout" RUSTZEN_MONITOR_MULTI_NODE_BUILD_TIMEOUT 1800
validate_seconds "$info_timeout" RUSTZEN_MONITOR_MULTI_NODE_DOCKER_INFO_TIMEOUT 60
validate_seconds "$log_timeout" RUSTZEN_MONITOR_MULTI_NODE_LOG_TIMEOUT 60

run_bounded() {
  seconds=$1; shift
  "$@" & command_pid=$!
  ( sleep "$seconds"; kill -TERM "$command_pid" 2>/dev/null || true; sleep 10; kill -KILL "$command_pid" 2>/dev/null || true ) >/dev/null 2>&1 & watchdog_pid=$!
  if wait "$command_pid"; then command_status=0; else command_status=$?; fi
  kill "$watchdog_pid" 2>/dev/null || true
  pkill -TERM -P "$watchdog_pid" 2>/dev/null || true
  wait "$watchdog_pid" 2>/dev/null || true
  return "$command_status"
}
run_bounded_capture() {
  capture=$(mktemp "${TMPDIR:-/tmp}/rz-monitor-multi-node.XXXXXX") || return 1
  if run_bounded "$@" >"$capture"; then command_status=0; else command_status=$?; fi
  cat "$capture"
  rm -f "$capture"
  return "$command_status"
}
remove_container() {
  name=$1
  run_bounded 30 "$docker_bin" rm -f "$name" >/dev/null 2>&1 || true
  remaining=$(run_bounded_capture 15 "$docker_bin" ps -a --filter "name=^/${name}$" --format '{{.Names}}') || return 1
  [ -z "$remaining" ]
}
candidate= staged_bin_dir= container= build_container= lock_dir=
cleanup() {
  status=$?
  trap - EXIT INT TERM
  cleanup_failed=0
  [ -z "$build_container" ] || remove_container "$build_container" || cleanup_failed=1
  if [ -n "$candidate" ] && [ -d "$candidate" ] && [ -n "$container" ]; then
    run_bounded "$log_timeout" "$docker_bin" logs "$container" >"$candidate/container.log" 2>&1 || true
  fi
  [ -z "$container" ] || remove_container "$container" || cleanup_failed=1
  [ -z "$staged_bin_dir" ] || rm -rf "$staged_bin_dir"
  [ "$status" -eq 0 ] || { [ -z "$candidate" ] || rm -rf "$candidate"; }
  [ -z "$lock_dir" ] || rm -rf "$lock_dir"
  [ "$cleanup_failed" -eq 0 ] || status=1
  exit "$status"
}
atomic_replace_symlink() {
  source_path=$1 target_path=$2
  case "$(uname -s)" in Darwin|FreeBSD) mv -fh "$source_path" "$target_path" ;; *) mv -Tf "$source_path" "$target_path" ;; esac
}
write_agent_provenance() {
  output=$1 binary=$2
  {
    printf 'schemaVersion\t1\n'
    printf 'gitHead\t%s\n' "$head"
    printf 'sourceTreeState\t%s\n' "$source_tree_state"
    printf 'sourceTreeSha256\t%s\n' "$source_tree_sha256"
    printf 'targetTriple\t%s\n' "$target_triple"
    printf 'cargoPackage\trustzen-monitor\n'
    printf 'cargoFeatures\tagent,no-default-features\n'
    printf 'binarySha256\t%s\n' "$(sha256sum "$binary" | awk '{print $1}')"
  } >"$output"
}
verify_agent_provenance() {
  provenance=$1 binary=$2
  actual=$(sha256sum "$binary" | awk '{print $1}')
  awk -F '\t' -v head="$head" -v state="$source_tree_state" -v digest="$source_tree_sha256" -v target="$target_triple" -v actual="$actual" '
    $1=="schemaVersion" && $2=="1" { schema=1 }
    $1=="gitHead" && $2==head { h=1 }
    $1=="sourceTreeState" && $2==state { s=1 }
    $1=="sourceTreeSha256" && $2==digest { d=1 }
    $1=="targetTriple" && $2==target { t=1 }
    $1=="cargoPackage" && $2=="rustzen-monitor" { p=1 }
    $1=="cargoFeatures" && $2=="agent,no-default-features" { f=1 }
    $1=="binarySha256" && $2==actual { b=1 }
    END { exit !(schema&&h&&s&&d&&t&&p&&f&&b) }
  ' "$provenance"
}
on_interrupt() { exit 130; }
on_terminate() { exit 143; }
if [ -e "$current" ] && [ ! -L "$current" ]; then
  echo "refusing to replace unsupported Monitor multi-node evidence directory: $current" >&2
  exit 1
fi
if [ "${RUSTZEN_MONITOR_MULTI_NODE_TEST_PUBLISH_FAILURE:-}" = 1 ]; then
  test_root=${RUSTZEN_MONITOR_MULTI_NODE_TEST_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/rz-monitor-multi-publish.XXXXXX")}
  evidence_root="$test_root"; current="$evidence_root/current"; run_id=test; candidate="$evidence_root/.candidate"
  publish_candidate() { mv "$candidate" "$evidence_root/runs/$run_id" && ln -s "runs/$run_id" "$evidence_root/.current" && atomic_replace_symlink "$evidence_root/.current" "$current"; }
  mkdir -p "$evidence_root/runs/old" "$candidate"
  printf old >"$evidence_root/runs/old/manifest.json"; ln -s runs/old "$current"; printf new >"$candidate/manifest.json"
  publish_candidate
  if ! test "$(readlink "$current")" = runs/test; then echo "publish success did not update current" >&2; exit 1; fi
  if ! test "$(cat "$current/manifest.json")" = new; then echo "publish success manifest is wrong" >&2; exit 1; fi
  run_id=failed; candidate="$evidence_root/.missing"
  if publish_candidate; then echo "forced publication failure unexpectedly succeeded" >&2; exit 1; fi
  if ! test "$(readlink "$current")" = runs/test; then echo "failed publish changed current" >&2; exit 1; fi
  if ! test "$(cat "$current/manifest.json")" = new; then echo "failed publish changed manifest" >&2; exit 1; fi
  echo "Monitor multi-node publication seam passed"
  [ -n "${RUSTZEN_MONITOR_MULTI_NODE_TEST_ROOT:-}" ] || rm -rf "$test_root"
  exit 0
fi
if [ "${RUSTZEN_MONITOR_MULTI_NODE_TEST_SETUP_FAILURE:-}" = 1 ]; then
  test_root=${RUSTZEN_MONITOR_MULTI_NODE_TEST_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/rz-monitor-multi-setup.XXXXXX")}
  evidence_root="$test_root"; current="$evidence_root/current"; lock_dir="$evidence_root/.verify.lock"; candidate="$evidence_root/.candidate"
  cleanup_setup() {
    result=$?; trap - EXIT INT TERM
    rm -rf "$candidate" "$lock_dir"
    assertion_failure=0
    if test "$result" -eq 0; then echo "setup seam unexpectedly succeeded" >&2; assertion_failure=1; fi
    if ! test "$(readlink "$current")" = runs/old; then echo "setup seam changed current" >&2; assertion_failure=1; fi
    if test -e "$lock_dir"; then echo "setup seam leaked lock" >&2; assertion_failure=1; fi
    [ -n "${RUSTZEN_MONITOR_MULTI_NODE_TEST_ROOT:-}" ] || rm -rf "$test_root"
    if test "$assertion_failure" -ne 0; then exit 99; fi
    echo "Monitor multi-node setup failure seam passed" >&2; exit "$result"
  }
  mkdir -p "$evidence_root/runs/old" "$candidate"; ln -s runs/old "$current"; mkdir "$lock_dir"
  trap cleanup_setup EXIT; trap on_interrupt INT; trap on_terminate TERM
  exit 1
fi
if [ "${RUSTZEN_MONITOR_MULTI_NODE_TEST_SIGNAL:-}" = INT ] || [ "${RUSTZEN_MONITOR_MULTI_NODE_TEST_SIGNAL:-}" = TERM ]; then
  test_signal=${RUSTZEN_MONITOR_MULTI_NODE_TEST_SIGNAL}
  test_root=${RUSTZEN_MONITOR_MULTI_NODE_TEST_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/rz-monitor-multi-signal.XXXXXX")}
  evidence_root="$test_root"; current="$evidence_root/current"; lock_dir="$evidence_root/.verify.lock"; candidate="$evidence_root/.candidate"
  cleanup_signal() {
    result=$?; trap - EXIT INT TERM
    rm -rf "$candidate" "$lock_dir"
    expected=130; [ "$test_signal" = TERM ] && expected=143
    assertion_failure=0
    if ! test "$result" = "$expected"; then echo "signal seam exit code is wrong" >&2; assertion_failure=1; fi
    if ! test "$(readlink "$current")" = runs/old; then echo "signal seam changed current" >&2; assertion_failure=1; fi
    if test -e "$lock_dir"; then echo "signal seam leaked lock" >&2; assertion_failure=1; fi
    [ -n "${RUSTZEN_MONITOR_MULTI_NODE_TEST_ROOT:-}" ] || rm -rf "$test_root"
    if test "$assertion_failure" -ne 0; then exit 99; fi
    echo "Monitor multi-node $test_signal signal seam passed" >&2; exit "$result"
  }
  mkdir -p "$evidence_root/runs/old" "$candidate"; ln -s runs/old "$current"; mkdir "$lock_dir"
  trap cleanup_signal EXIT; trap on_interrupt INT; trap on_terminate TERM
  kill -"$test_signal" "$$"
fi
if [ "${RUSTZEN_MONITOR_MULTI_NODE_TEST_EXIT_CLEANUP:-}" = 1 ]; then
  evidence_root=${RUSTZEN_MONITOR_MULTI_NODE_TEST_ROOT:?}
  current="$evidence_root/current"; candidate="$evidence_root/.candidate"; staged_bin_dir="$evidence_root/.binaries"; lock_dir="$evidence_root/.verify.lock"
  build_container=rz-monitor-multi-node-build-exit-test; container=rz-monitor-multi-node-exit-test
  mkdir -p "$evidence_root/runs/old" "$candidate" "$staged_bin_dir" "$lock_dir"
  printf old >"$evidence_root/runs/old/manifest.json"; ln -s runs/old "$current"
  trap cleanup EXIT; trap on_interrupt INT; trap on_terminate TERM
  exit 9
fi
if [ "${RUSTZEN_MONITOR_MULTI_NODE_TEST_BUILD_CLEANUP:-}" = 1 ]; then
  build_container=rz-monitor-multi-node-build-test
  build_status=0
  run_bounded 5 "$docker_bin" run --name "$build_container" || build_status=$?
  [ "$build_status" -ne 0 ] || { echo "build cleanup seam unexpectedly succeeded" >&2; exit 1; }
  remove_container "$build_container" || { echo "build cleanup seam leaked its container" >&2; exit 1; }
  echo "Monitor multi-node build cleanup seam passed"
  exit 0
fi

if [ -z "$architecture" ]; then
  architecture=$(run_bounded_capture "$info_timeout" "$docker_bin" info --format '{{.Architecture}}') || { echo "Docker architecture discovery failed or exceeded its timeout" >&2; exit 1; }
fi
case "$architecture" in
  aarch64) platform=linux/arm64; target_triple=aarch64-unknown-linux-musl; file_pattern='ELF 64-bit.*ARM aarch64' ;;
  x86_64) platform=linux/amd64; target_triple=x86_64-unknown-linux-musl; file_pattern='ELF 64-bit.*x86-64' ;;
  *) echo "unsupported Colima/Docker architecture: $architecture" >&2; exit 1 ;;
esac
read -r head source_tree_state source_tree_sha256 < <("$root/scripts/admin-browser-source-identity.sh")
read -r verifier_image verifier_key verifier_provenance_sha < <("$root/scripts/ensure-admin-browser-verifier-image.sh" --platform "$platform")
bin_dir=${RUSTZEN_UI_LINUX_BIN_DIR:-"$root/target/rz/build/$architecture/bin"}
for name in rz-admin rz-monitor; do
  test -x "$bin_dir/$name" || { echo "missing Linux binary: $bin_dir/$name; run just build-admin-browser-linux" >&2; exit 1; }
  file "$bin_dir/$name" | grep -q "$file_pattern"
done
agent_build_dir="$root/target/rz/monitor-agent-multi-node/build/$architecture"
agent_bin="$agent_build_dir/rz-monitor-agent"
mkdir -p "$agent_build_dir"
agent_provenance="$agent_build_dir/rz-monitor-agent.provenance.tsv"
test -s "$bin_dir/build-provenance.txt"
awk -F '\t' -v head="$head" -v state="$source_tree_state" -v digest="$source_tree_sha256" '
  $1 == "gitHead" && $2 == head { found_head = 1 }
  $1 == "sourceTreeState" && $2 == state { found_state = 1 }
  $1 == "sourceTreeSha256" && $2 == digest { found_digest = 1 }
  END { exit !(found_head && found_state && found_digest) }
' "$bin_dir/build-provenance.txt" || { echo "Linux server binaries do not match the current source-tree build provenance" >&2; exit 1; }
lock_dir="$evidence_root/.verify.lock"; run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$"; candidate="$evidence_root/.candidate-$run_id"; staged_bin_dir="$evidence_root/.binaries-$run_id"; container="rz-monitor-multi-node-$run_id"; build_container="rz-monitor-multi-node-build-$run_id"
status=1
mkdir -p "$evidence_root/runs"
if ! mkdir "$lock_dir" 2>/dev/null; then echo "another Monitor multi-node verification owns $lock_dir" >&2; exit 1; fi
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
mkdir -p "$candidate" "$staged_bin_dir"
build_status=0
run_bounded "$build_timeout" "$docker_bin" run --name "$build_container" --platform "$platform" -v "$root:/work" -w /work -v rustzen-monitor-multi-node-cargo:/usr/local/cargo/registry rust:1.95-bookworm \
  bash -euo pipefail -c "apt-get update >/dev/null && apt-get install -y --no-install-recommends musl-tools >/dev/null && rustup target add $target_triple >/dev/null && cargo build --release --target $target_triple -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent --target-dir target/monitor-agent-multi-node/cargo" || build_status=$?
remove_container "$build_container" || { echo "Agent build container cleanup failed" >&2; exit 1; }
[ "$build_status" -eq 0 ] || { echo "Agent build failed or exceeded ${build_timeout}s" >&2; exit "$build_status"; }
cp "$root/target/monitor-agent-multi-node/cargo/$target_triple/release/rz-monitor-agent" "$agent_bin"
file "$agent_bin" | grep -q "$file_pattern"
write_agent_provenance "$agent_provenance" "$agent_bin"
verify_agent_provenance "$agent_provenance" "$agent_bin" || { echo "Agent build provenance does not match the current source basis" >&2; exit 1; }
for name in rz-admin rz-monitor; do cp "$bin_dir/$name" "$staged_bin_dir/$name"; done
cp "$agent_bin" "$staged_bin_dir/rz-monitor-agent"
admin_hash=$(sha256sum "$staged_bin_dir/rz-admin" | awk '{print $1}')
monitor_hash=$(sha256sum "$staged_bin_dir/rz-monitor" | awk '{print $1}')
agent_hash=$(sha256sum "$staged_bin_dir/rz-monitor-agent" | awk '{print $1}')
awk -F '\t' -v admin="$admin_hash" -v monitor="$monitor_hash" '
  $1 == "rz-admin" && $2 == admin { found_admin = 1 }
  $1 == "rz-monitor" && $2 == monitor { found_monitor = 1 }
  END { exit !(found_admin && found_monitor) }
' "$bin_dir/build-provenance.txt" || { echo "staged server binary digest differs from build provenance" >&2; exit 1; }
cp "$bin_dir/build-provenance.txt" "$candidate/build-provenance.txt"
cp "$agent_provenance" "$candidate/rz-monitor-agent.provenance.tsv"
verify_agent_provenance "$candidate/rz-monitor-agent.provenance.tsv" "$staged_bin_dir/rz-monitor-agent" || { echo "staged Agent provenance differs from current source basis" >&2; exit 1; }
run_bounded "$run_timeout" "$docker_bin" run --name "$container" --platform "$platform" --security-opt seccomp=unconfined \
  --env RUSTZEN_VERIFY_HEAD="$head" --env RUSTZEN_VERIFY_SOURCE_TREE_STATE="$source_tree_state" --env RUSTZEN_VERIFY_SOURCE_TREE_SHA256="$source_tree_sha256" \
  --env RUSTZEN_VERIFY_ARCHITECTURE="$architecture" --env RUSTZEN_VERIFY_PLATFORM="$platform" \
  --env RUSTZEN_VERIFY_ADMIN_SHA256="$admin_hash" --env RUSTZEN_VERIFY_MONITOR_SHA256="$monitor_hash" --env RUSTZEN_VERIFY_AGENT_SHA256="$agent_hash" \
  --mount "type=bind,src=$staged_bin_dir,dst=/verify/bin,readonly" --mount "type=bind,src=$candidate,dst=/verify/evidence" \
  --mount "type=bind,src=$root/scripts/verify-monitor-agent-multi-node-linux-inner.sh,dst=/verify/run.sh,readonly" \
  --mount "type=bind,src=$root/scripts/monitor-agent-multi-node-readiness.py,dst=/verify/readiness.py,readonly" \
  "$verifier_image" bash /verify/run.sh
remove_container "$container" || { echo "Monitor multi-node runtime container cleanup failed" >&2; exit 1; }
jq -e --arg sha "$source_tree_sha256" '.schemaVersion == 1 and .status == "partial" and .sourceTreeSha256 == $sha and (.serviceUsers | length) == 2 and (.readiness.events | length) == 2 and .recovery.thirdNodeRegistered == false' "$candidate/manifest.json" >/dev/null
read -r final_head final_source_tree_state final_source_tree_sha256 < <("$root/scripts/admin-browser-source-identity.sh")
test "$final_head" = "$head" && test "$final_source_tree_state" = "$source_tree_state" && test "$final_source_tree_sha256" = "$source_tree_sha256"
publish_candidate() { mv "$candidate" "$evidence_root/runs/$run_id" && ln -s "runs/$run_id" "$evidence_root/.current-$run_id" && atomic_replace_symlink "$evidence_root/.current-$run_id" "$current"; }
publish_candidate
status=0
rm -rf "$staged_bin_dir"
rm -rf "$lock_dir"
trap - EXIT INT TERM
echo "Monitor dual-Agent Linux evidence published: $current/manifest.json"
