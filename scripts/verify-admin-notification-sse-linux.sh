#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
docker_bin=${RUSTZEN_ADMIN_SSE_DOCKER:-docker}
source_identity=${RUSTZEN_ADMIN_SSE_SOURCE_IDENTITY:-"$root/scripts/admin-browser-source-identity.sh"}
verifier_helper=${RUSTZEN_ADMIN_SSE_VERIFIER_HELPER:-"$root/scripts/ensure-admin-browser-verifier-image.sh"}
file_bin=${RUSTZEN_ADMIN_SSE_FILE:-file}
timeout=${RUSTZEN_ADMIN_SSE_TIMEOUT:-900}
cleanup_timeout=${RUSTZEN_ADMIN_SSE_CLEANUP_TIMEOUT:-10}
kill_grace=${RUSTZEN_ADMIN_SSE_KILL_GRACE:-5}
helper_timeout=${RUSTZEN_ADMIN_SSE_HELPER_TIMEOUT:-1020}
helper_cleanup_grace=${RUSTZEN_ADMIN_SSE_HELPER_CLEANUP_GRACE:-20}
evidence_root=${RUSTZEN_ADMIN_SSE_EVIDENCE_ROOT:-"$root/target/rz/admin-notification-sse"}
current=$evidence_root/current
command_pid= watchdog_pid= build_active=0 runtime_active=0 helper_active=0

descendants(){ local parent=$1 child; while IFS= read -r child; do [ -z "$child" ] || { descendants "$child"; printf '%s\n' "$child"; }; done < <(pgrep -P "$parent" 2>/dev/null || true); }
stop_tree(){ local parent=${1:-} tree child; [ -n "$parent" ] || return 0; tree=$(descendants "$parent"); while IFS= read -r child; do [ -z "$child" ] || kill -TERM "$child" 2>/dev/null || true; done <<<"$tree"; kill -TERM "$parent" 2>/dev/null || true; }
validate_seconds(){ value=$1 name=$2 maximum=$3; case "$value" in ''|*[!0-9]*) echo "$name must be a positive integer" >&2; exit 2;; esac; [ "$value" -gt 0 ] && [ "$value" -le "$maximum" ] || { echo "$name must be 1..$maximum seconds" >&2; exit 2; }; }
validate_seconds "$timeout" RUSTZEN_ADMIN_SSE_TIMEOUT 1800
validate_seconds "$cleanup_timeout" RUSTZEN_ADMIN_SSE_CLEANUP_TIMEOUT 30
validate_seconds "$kill_grace" RUSTZEN_ADMIN_SSE_KILL_GRACE 5
validate_seconds "$helper_timeout" RUSTZEN_ADMIN_SSE_HELPER_TIMEOUT 2100
validate_seconds "$helper_cleanup_grace" RUSTZEN_ADMIN_SSE_HELPER_CLEANUP_GRACE 30
run_bounded(){ seconds=$1; shift; marker=$(mktemp "${TMPDIR:-/tmp}/rz-admin-sse-timeout.XXXXXX"); rm -f "$marker"; "$@" & command_pid=$!; (sleep "$seconds"; : >"$marker"; stop_tree "$command_pid"; sleep "$kill_grace"; kill -KILL "$command_pid" 2>/dev/null || true) & watchdog_pid=$!; if wait "$command_pid"; then status=0; else status=$?; fi; stop_tree "$watchdog_pid"; wait "$watchdog_pid" 2>/dev/null || true; [ ! -e "$marker" ] || status=124; rm -f "$marker"; command_pid=; watchdog_pid=; return "$status"; }
stop_helper(){ helper_pid=${1:-}; [ -n "$helper_pid" ] || return 0; kill -TERM "$helper_pid" 2>/dev/null || true; waited=0; while kill -0 "$helper_pid" 2>/dev/null && [ "$waited" -lt "$helper_cleanup_grace" ]; do sleep 1; waited=$((waited + 1)); done; if kill -0 "$helper_pid" 2>/dev/null; then stop_tree "$helper_pid"; kill -KILL "$helper_pid" 2>/dev/null || true; fi; }
run_helper_bounded(){ seconds=$1; shift; marker=$(mktemp "${TMPDIR:-/tmp}/rz-admin-sse-helper-timeout.XXXXXX"); rm -f "$marker"; helper_active=1; "$@" & command_pid=$!; (sleep "$seconds"; : >"$marker"; stop_helper "$command_pid") & watchdog_pid=$!; if wait "$command_pid"; then status=0; else status=$?; fi; helper_active=0; stop_tree "$watchdog_pid"; wait "$watchdog_pid" 2>/dev/null || true; [ ! -e "$marker" ] || status=124; rm -f "$marker"; command_pid=; watchdog_pid=; return "$status"; }
run_capture(){ capture=$(mktemp "${TMPDIR:-/tmp}/rz-admin-sse-command.XXXXXX"); if run_bounded "$@" >"$capture"; then status=0; else status=$?; fi; cat "$capture"; rm -f "$capture"; return "$status"; }
atomic_replace_symlink(){ case "$(uname -s)" in Darwin|FreeBSD) mv -fh "$1" "$2";; *) mv -Tf "$1" "$2";; esac; }
remove_container(){ name=$1; run_bounded "$cleanup_timeout" "$docker_bin" rm -f "$name" >/dev/null 2>&1 || { echo "failed to remove Docker container $name" >&2; return 1; }; if run_bounded "$cleanup_timeout" "$docker_bin" container inspect "$name" >/dev/null 2>&1; then echo "Docker container still exists after removal: $name" >&2; return 1; else inspect_status=$?; fi; [ "$inspect_status" -eq 1 ] || { echo "Docker absence check failed for $name ($inspect_status)" >&2; return 1; }; }
capture_container_log(){ name=$1 output=$2; run_bounded "$cleanup_timeout" "$docker_bin" logs "$name" >"$output" 2>&1 || true; [ -s "$output" ] || echo "no Docker log was available for $name" >"$output"; }

architecture=$(run_capture 10 "$docker_bin" info --format '{{.Architecture}}') || { echo 'Docker architecture discovery failed or timed out' >&2; exit 1; }
case "$architecture" in aarch64) platform=linux/arm64; target_triple=aarch64-unknown-linux-musl; file_pattern='ELF 64-bit.*ARM aarch64';; x86_64) platform=linux/amd64; target_triple=x86_64-unknown-linux-musl; file_pattern='ELF 64-bit.*x86-64';; *) echo "unsupported Docker architecture: $architecture" >&2; exit 1;; esac
mkdir -p "$evidence_root/runs" "$evidence_root/failed-runs"
[ ! -e "$current" ] || [ -L "$current" ] || { echo "refusing non-symlink evidence path: $current" >&2; exit 1; }
run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$"; lock=$evidence_root/.verify.lock; candidate=$evidence_root/.candidate-$run_id; staged=$evidence_root/.binaries-$run_id; published=$evidence_root/runs/$run_id; failed=$evidence_root/failed-runs/$run_id; publish_link=$evidence_root/.current-$run_id
build_container=rz-admin-sse-build-$run_id; runtime_container=rz-admin-sse-runtime-$run_id
mkdir "$lock" 2>/dev/null || { echo 'another Admin SSE gate owns the lock' >&2; exit 1; }; mkdir "$candidate" "$staged"
: >"$candidate/phase-status.tsv"
cleanup(){ result=$?; trap - EXIT INT TERM; stop_tree "$watchdog_pid"; if [ "$helper_active" -eq 1 ]; then stop_helper "$command_pid"; [ -s "$candidate/verifier-helper.log" ] || echo 'verifier helper was interrupted without diagnostic output' >"$candidate/verifier-helper.log"; printf 'verifier-helper\t%s\n' "$result" >>"$candidate/phase-status.tsv"; else stop_tree "$command_pid"; fi; helper_active=0; if [ "$result" -ne 0 ] && [ -d "$candidate" ]; then [ "$build_active" -eq 0 ] || capture_container_log "$build_container" "$candidate/build-container.log"; [ "$runtime_active" -eq 0 ] || capture_container_log "$runtime_container" "$candidate/runtime-container.log"; fi; [ "$build_active" -eq 0 ] || run_bounded "$cleanup_timeout" "$docker_bin" rm -f "$build_container" >/dev/null 2>&1 || true; [ "$runtime_active" -eq 0 ] || run_bounded "$cleanup_timeout" "$docker_bin" rm -f "$runtime_container" >/dev/null 2>&1 || true; if [ "$result" -ne 0 ] && [ -d "$candidate" ]; then mv "$candidate" "$failed"; fi; rm -rf "$staged" "$publish_link" "$lock"; exit "$result"; }
trap cleanup EXIT; trap 'exit 130' INT; trap 'exit 143' TERM

read -r head source_state source_sha < <("$source_identity")
build_root=$evidence_root/build/$architecture; rm -rf "$build_root"; mkdir -p "$build_root/bin"
build_active=1
if run_bounded "$timeout" "$docker_bin" run --name "$build_container" --platform "$platform" --mount "type=bind,src=$root,dst=/work" --mount type=volume,src=rustzen-admin-sse-cargo,dst=/usr/local/cargo/registry -w /work rust:1.95-bookworm bash -euo pipefail -c "
 apt-get update >/dev/null; apt-get install -y --no-install-recommends musl-tools >/dev/null; rustup target add $target_triple >/dev/null
 export RUSTFLAGS='-C target-feature=+crt-static'; target=target/admin-notification-sse/cargo; out=target/rz/admin-notification-sse/build/$architecture/bin
 cargo build --release --target $target_triple --target-dir \"\$target\" -p rustzen-admin --no-default-features --features monitor-distribution,notifications --bin rz-admin
 install -m 0755 \"\$target/$target_triple/release/rz-admin\" \"\$out/rz-admin\"
" >"$candidate/build.log" 2>&1; then build_status=0; else build_status=$?; fi
[ -s "$candidate/build.log" ] || echo 'build command completed without output' >"$candidate/build.log"
printf 'build\t%s\n' "$([ "$build_status" -eq 0 ] && echo passed || echo "$build_status")" >>"$candidate/phase-status.tsv"
[ "$build_status" -eq 0 ] || exit "$build_status"
remove_container "$build_container"; build_active=0
binary=$build_root/bin/rz-admin; test -x "$binary"; "$file_bin" "$binary" | grep -q "$file_pattern"; cp "$binary" "$staged/rz-admin"; binary_sha=$(shasum -a 256 "$staged/rz-admin"|awk '{print $1}')
printf 'schemaVersion\t1\nhead\t%s\nsourceTreeState\t%s\nsourceTreeSha256\t%s\narchitecture\t%s\ntargetTriple\t%s\nrz-admin\tmonitor-distribution,notifications\t%s\n' "$head" "$source_state" "$source_sha" "$architecture" "$target_triple" "$binary_sha" >"$candidate/build-provenance.tsv"
provenance_sha=$(shasum -a 256 "$candidate/build-provenance.tsv"|awk '{print $1}')
if run_helper_bounded "$helper_timeout" env RUSTZEN_UI_VERIFIER_DOCKER="$docker_bin" "$verifier_helper" --platform "$platform" >"$candidate/.verifier-output" 2>"$candidate/verifier-helper.log"; then helper_status=0; else helper_status=$?; fi
if [ "$helper_status" -eq 0 ]; then read -r verifier_image verifier_key verifier_sha <"$candidate/.verifier-output"; echo "verifier helper selected $verifier_image" >>"$candidate/verifier-helper.log"; else cat "$candidate/.verifier-output" >>"$candidate/verifier-helper.log"; fi
rm -f "$candidate/.verifier-output"; [ -s "$candidate/verifier-helper.log" ] || echo 'verifier helper produced no diagnostic' >"$candidate/verifier-helper.log"
printf 'verifier-helper\t%s\n' "$([ "$helper_status" -eq 0 ] && echo passed || echo "$helper_status")" >>"$candidate/phase-status.tsv"
[ "$helper_status" -eq 0 ] || exit "$helper_status"
runtime_active=1
if run_bounded "$timeout" "$docker_bin" run --name "$runtime_container" --platform "$platform" --security-opt seccomp=unconfined \
 --env RUSTZEN_VERIFY_HEAD="$head" --env RUSTZEN_VERIFY_SOURCE_TREE_STATE="$source_state" --env RUSTZEN_VERIFY_SOURCE_TREE_SHA256="$source_sha" --env RUSTZEN_VERIFY_ARCHITECTURE="$architecture" --env RUSTZEN_VERIFY_PLATFORM="$platform" --env RUSTZEN_VERIFY_BINARY_SHA256="$binary_sha" --env RUSTZEN_VERIFY_BUILD_PROVENANCE_SHA256="$provenance_sha" --env RUSTZEN_VERIFY_VERIFIER_IMAGE_ID="$verifier_image" --env RUSTZEN_VERIFY_VERIFIER_KEY="$verifier_key" --env RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256="$verifier_sha" \
 --mount "type=bind,src=$staged,dst=/verify/bin,readonly" --mount "type=bind,src=$candidate,dst=/verify/evidence" --mount "type=bind,src=$root/scripts/verify-admin-notification-sse-linux-inner.sh,dst=/verify/run.sh,readonly" --mount "type=bind,src=$root/scripts/monitor-notification-runtime-client.py,dst=/verify/client.py,readonly" "$verifier_image" bash /verify/run.sh >"$candidate/runtime.log" 2>&1; then runtime_status=0; else runtime_status=$?; fi
[ -s "$candidate/runtime.log" ] || echo 'runtime command completed without output' >"$candidate/runtime.log"
printf 'runtime\t%s\n' "$([ "$runtime_status" -eq 0 ] && echo passed || echo "$runtime_status")" >>"$candidate/phase-status.tsv"
[ "$runtime_status" -eq 0 ] || exit "$runtime_status"
remove_container "$runtime_container"; runtime_active=0

outer_receipts=$(for file in phase-status.tsv build.log verifier-helper.log runtime.log; do jq -nc --arg file "$file" --arg sha "$(shasum -a 256 "$candidate/$file"|awk '{print $1}')" --argjson bytes "$(stat -f %z "$candidate/$file" 2>/dev/null || stat -c %s "$candidate/$file")" '{file:$file,sha256:$sha,bytes:$bytes}'; done|jq -s .)
jq --argjson outer "$outer_receipts" '.receipts += $outer' "$candidate/manifest.json" >"$candidate/.manifest.json"; mv "$candidate/.manifest.json" "$candidate/manifest.json"
allowlist=$(jq -nc '["steps.log","initial.json","event.json","ingest.json","live-change.json","inbox.json","read.json","retention.json","reconnect.json","auth-401.json","auth-403.json","query-rejection.json","redaction.json","quota.json","authority-change.json","expiry.json","authority-outage.json","selected-state.json","phase-status.tsv","build.log","verifier-helper.log","runtime.log"]')
jq -e --arg head "$head" --arg state "$source_state" --arg sourceSha "$source_sha" --arg architecture "$architecture" --arg platform "$platform" --arg binary "$binary_sha" --arg provenance "$provenance_sha" --arg verifierImage "$verifier_image" --arg verifierKey "$verifier_key" --arg verifierSha "$verifier_sha" --argjson allowlist "$allowlist" \
 --slurpfile initial "$candidate/initial.json" --slurpfile event "$candidate/event.json" --slurpfile ingest "$candidate/ingest.json" --slurpfile live "$candidate/live-change.json" --slurpfile inbox "$candidate/inbox.json" --slurpfile read "$candidate/read.json" --slurpfile retention "$candidate/retention.json" --slurpfile reconnect "$candidate/reconnect.json" --slurpfile unauthorized "$candidate/auth-401.json" --slurpfile forbidden "$candidate/auth-403.json" --slurpfile query "$candidate/query-rejection.json" --slurpfile redaction "$candidate/redaction.json" --slurpfile quota "$candidate/quota.json" --slurpfile authorityChange "$candidate/authority-change.json" --slurpfile expiry "$candidate/expiry.json" --slurpfile outage "$candidate/authority-outage.json" --slurpfile selected "$candidate/selected-state.json" --rawfile phaseStatus "$candidate/phase-status.tsv" --rawfile buildLog "$candidate/build.log" --rawfile helperLog "$candidate/verifier-helper.log" --rawfile runtimeLog "$candidate/runtime.log" -f "$root/scripts/verify-admin-notification-sse-evidence.jq" "$candidate/manifest.json" >/dev/null
test "$(shasum -a 256 "$candidate/build-provenance.tsv"|awk '{print $1}')" = "$provenance_sha"
while IFS=$'\t' read -r file sha bytes; do test -f "$candidate/$file" && test ! -L "$candidate/$file"; test "$(shasum -a 256 "$candidate/$file"|awk '{print $1}')" = "$sha"; test "$(stat -f %z "$candidate/$file" 2>/dev/null || stat -c %s "$candidate/$file")" = "$bytes"; done < <(jq -r '.receipts[]|[.file,.sha256,.bytes]|@tsv' "$candidate/manifest.json")
read -r final_head final_state final_sha < <("$source_identity"); [ "$final_head:$final_state:$final_sha" = "$head:$source_state:$source_sha" ] || { echo 'source tree changed during Admin SSE gate' >&2; exit 1; }
mv "$candidate" "$published"; ln -s "runs/$run_id" "$publish_link"; atomic_replace_symlink "$publish_link" "$current"
echo "Admin notification SSE evidence published: $current/manifest.json"
