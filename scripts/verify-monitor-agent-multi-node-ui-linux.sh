#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
docker_bin=${RUSTZEN_MONITOR_MULTI_NODE_UI_DOCKER:-docker}
architecture=${RUSTZEN_UI_LINUX_ARCH:-}
timeout=${RUSTZEN_MONITOR_MULTI_NODE_UI_TIMEOUT:-300}
cleanup_timeout=${RUSTZEN_MONITOR_MULTI_NODE_UI_CLEANUP_TIMEOUT:-10}
evidence_root="$root/target/rz/monitor-agent-multi-node-ui"
current="$evidence_root/current"; lock_dir="$evidence_root/.verify.lock"

validate_seconds() {
  case "$1" in ''|*[!0-9]*) echo "$2 must be a positive integer" >&2; exit 2;; esac
  [ "$1" -gt 0 ] && [ "$1" -le "$3" ] || { echo "$2 must be 1..$3 seconds" >&2; exit 2; }
}
run_bounded() {
  seconds=$1; shift; "$@" & command_pid=$!
  ( sleep "$seconds"; kill -TERM "$command_pid" 2>/dev/null || true; sleep 1; kill -KILL "$command_pid" 2>/dev/null || true ) >/dev/null 2>&1 & watchdog_pid=$!
  if wait "$command_pid"; then code=0; else code=$?; fi
  kill "$watchdog_pid" 2>/dev/null || true; pkill -TERM -P "$watchdog_pid" 2>/dev/null || true; wait "$watchdog_pid" 2>/dev/null || true
  return "$code"
}
run_bounded_capture() {
  capture=$(mktemp "${TMPDIR:-/tmp}/rz-dual-agent-ui.XXXXXX")
  if run_bounded "$@" >"$capture"; then code=0; else code=$?; fi
  cat "$capture"; rm -f "$capture"; return "$code"
}
atomic_replace_symlink() { case "$(uname -s)" in Darwin|FreeBSD) mv -fh "$1" "$2";; *) mv -Tf "$1" "$2";; esac; }
verify_agent_provenance() { awk -F '\t' -v head="$head" -v state="$state" -v digest="$source_sha" -v target="$target" -v binary="$1" '$1=="schemaVersion"&&$2=="1"{a=1}$1=="gitHead"&&$2==head{b=1}$1=="sourceTreeState"&&$2==state{c=1}$1=="sourceTreeSha256"&&$2==digest{d=1}$1=="targetTriple"&&$2==target{e=1}$1=="cargoPackage"&&$2=="rustzen-monitor"{f=1}$1=="cargoFeatures"&&$2=="agent,no-default-features"{g=1}$1=="binarySha256"&&$2==binary{h=1}END{exit !(a&&b&&c&&d&&e&&f&&g&&h)}' "$2"; }
publish_candidate() { mv "$candidate" "$evidence_root/runs/$run_id" && ln -s "runs/$run_id" "$evidence_root/.current-$run_id" && atomic_replace_symlink "$evidence_root/.current-$run_id" "$current"; }
cleanup() {
  status=$?; trap - EXIT INT TERM
  if [ -d "$candidate" ]; then run_bounded_capture "$cleanup_timeout" "$docker_bin" logs "$container" >"$candidate/container.log" 2>&1 || true; fi
  run_bounded "$cleanup_timeout" "$docker_bin" rm -f "$container" >/dev/null 2>&1 || true
  rm -rf "$staged" "$lock_dir"; [ "$status" -eq 0 ] || rm -rf "$candidate"; exit "$status"
}

validate_seconds "$timeout" RUSTZEN_MONITOR_MULTI_NODE_UI_TIMEOUT 900
validate_seconds "$cleanup_timeout" RUSTZEN_MONITOR_MULTI_NODE_UI_CLEANUP_TIMEOUT 60
if [ -e "$current" ] && [ ! -L "$current" ]; then echo "refusing to replace legacy dual-Agent Nodes evidence directory: $current" >&2; exit 1; fi
if [ "${RUSTZEN_MONITOR_MULTI_NODE_UI_TEST_PUBLISH_FAILURE:-}" = 1 ]; then
  evidence_root=${RUSTZEN_MONITOR_MULTI_NODE_UI_TEST_ROOT:?missing test root}; current="$evidence_root/current"; run_id=test; candidate="$evidence_root/.candidate"; mkdir -p "$evidence_root/runs/old" "$candidate"; printf old >"$evidence_root/runs/old/manifest.json"; ln -s runs/old "$current"; printf new >"$candidate/manifest.json"; publish_candidate; test "$(readlink "$current")" = runs/test && test "$(cat "$current/manifest.json")" = new; run_id=failed; candidate="$evidence_root/.missing"; ! publish_candidate; test "$(readlink "$current")" = runs/test; echo 'dual-Agent Nodes publication seam passed'; exit 0
fi
if [ "${RUSTZEN_MONITOR_MULTI_NODE_UI_TEST_CLEANUP:-}" = 1 ]; then
  evidence_root=${RUSTZEN_MONITOR_MULTI_NODE_UI_TEST_ROOT:?missing test root}; current="$evidence_root/current"; lock_dir="$evidence_root/.verify.lock"; run_id=test; candidate="$evidence_root/.candidate"; staged="$evidence_root/.binaries"; container=stubborn; mkdir -p "$evidence_root/runs/old" "$candidate" "$staged" "$lock_dir"; ln -s runs/old "$current"; trap cleanup EXIT; exit 1
fi
if [ -z "$architecture" ]; then architecture=$(run_bounded_capture "$cleanup_timeout" "$docker_bin" info --format '{{.Architecture}}') || { echo 'Docker architecture discovery failed or timed out' >&2; exit 1; }; fi
case "$architecture" in aarch64) platform=linux/arm64; target=aarch64-unknown-linux-musl; pattern='ELF 64-bit.*ARM aarch64';; x86_64) platform=linux/amd64; target=x86_64-unknown-linux-musl; pattern='ELF 64-bit.*x86-64';; *) echo "unsupported Docker architecture: $architecture" >&2; exit 1;; esac
read -r head state source_sha < <("$root/scripts/admin-browser-source-identity.sh"); read -r image _ _ < <("$root/scripts/ensure-admin-browser-verifier-image.sh" --platform "$platform")
bin_dir=${RUSTZEN_UI_LINUX_BIN_DIR:-"$root/target/rz/build/$architecture/bin"}; agent_dir="$root/target/rz/monitor-agent-multi-node/build/$architecture"; agent_bin="$agent_dir/rz-monitor-agent"; agent_provenance="$agent_dir/rz-monitor-agent.provenance.tsv"
for name in rz-admin rz-monitor rz-reports; do test -x "$bin_dir/$name" || { echo "missing Linux binary: $bin_dir/$name; run just build-admin-browser-linux" >&2; exit 1; }; file "$bin_dir/$name" | grep -q "$pattern"; done
test -x "$agent_bin" && test -s "$agent_provenance" || { echo 'missing current dual-Agent binary/provenance; run just verify-monitor-agent-multi-node-linux first' >&2; exit 1; }; file "$agent_bin" | grep -q "$pattern"; test -s "$bin_dir/build-provenance.txt"
awk -F '\t' -v head="$head" -v state="$state" -v digest="$source_sha" '$1=="gitHead"&&$2==head{h=1}$1=="sourceTreeState"&&$2==state{s=1}$1=="sourceTreeSha256"&&$2==digest{d=1}END{exit !(h&&s&&d)}' "$bin_dir/build-provenance.txt" || { echo 'server provenance does not match current source identity' >&2; exit 1; }
agent_sha=$(sha256sum "$agent_bin" | awk '{print $1}'); verify_agent_provenance "$agent_sha" "$agent_provenance" || { echo 'Agent provenance does not match current source identity' >&2; exit 1; }
mkdir -p "$evidence_root/runs"; if ! mkdir "$lock_dir" 2>/dev/null; then echo "another dual-Agent Nodes verification owns $lock_dir" >&2; exit 1; fi
run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$"; candidate="$evidence_root/.candidate-$run_id"; staged="$evidence_root/.binaries-$run_id"; container="rz-monitor-agent-multi-node-ui-$run_id"; status=1
trap cleanup EXIT; trap 'exit 130' INT; trap 'exit 143' TERM
mkdir -p "$candidate" "$staged"; for name in rz-admin rz-monitor rz-reports; do cp "$bin_dir/$name" "$staged/$name"; done; cp "$agent_bin" "$staged/rz-monitor-agent"; cp "$bin_dir/build-provenance.txt" "$candidate/build-provenance.txt"; cp "$agent_provenance" "$candidate/rz-monitor-agent.provenance.tsv"
admin_sha=$(sha256sum "$staged/rz-admin" | awk '{print $1}'); monitor_sha=$(sha256sum "$staged/rz-monitor" | awk '{print $1}'); reports_sha=$(sha256sum "$staged/rz-reports" | awk '{print $1}'); agent_sha=$(sha256sum "$staged/rz-monitor-agent" | awk '{print $1}')
awk -F '\t' -v admin="$admin_sha" -v monitor="$monitor_sha" -v reports="$reports_sha" '$1=="rz-admin"&&$2==admin{a=1}$1=="rz-monitor"&&$2==monitor{b=1}$1=="rz-reports"&&$2==reports{c=1}END{exit !(a&&b&&c)}' "$candidate/build-provenance.txt" || { echo 'staged server binary differs from provenance' >&2; exit 1; }; verify_agent_provenance "$agent_sha" "$candidate/rz-monitor-agent.provenance.tsv"
run_bounded "$timeout" "$docker_bin" run --name "$container" --platform "$platform" --security-opt seccomp=unconfined --env RUSTZEN_VERIFY_HEAD="$head" --env RUSTZEN_VERIFY_SOURCE_TREE_STATE="$state" --env RUSTZEN_VERIFY_SOURCE_TREE_SHA256="$source_sha" --env RUSTZEN_VERIFY_PLATFORM="$platform" --env RUSTZEN_VERIFY_ADMIN_SHA256="$admin_sha" --env RUSTZEN_VERIFY_MONITOR_SHA256="$monitor_sha" --env RUSTZEN_VERIFY_REPORTS_SHA256="$reports_sha" --env RUSTZEN_VERIFY_AGENT_SHA256="$agent_sha" --mount "type=bind,src=$staged,dst=/verify/bin,readonly" --mount "type=bind,src=$candidate,dst=/verify/evidence" --mount "type=bind,src=$root/scripts/verify-monitor-agent-multi-node-ui-linux-inner.sh,dst=/verify/run.sh,readonly" --mount "type=bind,src=$root/scripts/monitor-agent-multi-node-ui-browser-steps.py,dst=/verify/steps.py,readonly" "$image" bash /verify/run.sh
pnpm dlx bun@1.3.14 "$root/scripts/verify-monitor-agent-multi-node-ui-evidence.mjs" "$candidate" "$source_sha" "$staged"
read -r final_head final_state final_sha < <("$root/scripts/admin-browser-source-identity.sh"); test "$final_head" = "$head" && test "$final_state" = "$state" && test "$final_sha" = "$source_sha"
publish_candidate; status=0; rm -rf "$staged" "$lock_dir"; trap - EXIT INT TERM; echo "Monitor dual-Agent Nodes Chromium evidence published: $current/manifest.json"
