#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
docker_bin=${RUSTZEN_UI_VERIFIER_DOCKER:-docker}
dockerfile=${RUSTZEN_UI_VERIFIER_DOCKERFILE:-"$root/scripts/admin-browser-verifier.Dockerfile"}
schema=1
base_image='debian@sha256:e5b6442dd2e9684cf5e87d8338b5968f3b348636fc0be6d7850a381e3731a2bd'
snapshot_timestamp=20240131T000000Z
chromium_version=120.0.6099.224-1~deb11u1
platform=
mode=image
command_pid=
watchdog_pid=
probe_container=

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
  while IFS= read -r child; do
    [ -z "$child" ] || kill -TERM "$child" 2>/dev/null || true
  done <<<"$tree"
  kill -TERM "$parent" 2>/dev/null || true
}

run_bounded() {
  seconds=$1; shift
  "$@" & command_pid=$!
  ( sleep "$seconds"; stop_tree "$command_pid"; sleep 10; kill -KILL "$command_pid" 2>/dev/null || true ) >/dev/null 2>&1 & watchdog_pid=$!
  if wait "$command_pid"; then command_status=0; else command_status=$?; fi
  stop_tree "$watchdog_pid"
  wait "$watchdog_pid" 2>/dev/null || true
  command_pid=
  watchdog_pid=
  return "$command_status"
}

run_bounded_capture() {
  capture=$(mktemp "${TMPDIR:-/tmp}/rz-browser-command.XXXXXX") || return 1
  if run_bounded "$@" >"$capture"; then command_status=0; else command_status=$?; fi
  cat "$capture"
  rm -f "$capture"
  return "$command_status"
}

usage() { echo "usage: $0 [--platform linux/arm64|linux/amd64] [--print-key]" >&2; exit 2; }
while [ "$#" -gt 0 ]; do
  case "$1" in
    --platform) platform=${2:-}; shift 2 ;;
    --print-key) mode=key; shift ;;
    *) usage ;;
  esac
done
if [ -z "$platform" ]; then
  architecture=$(run_bounded_capture 10 "$docker_bin" info --format '{{.Architecture}}') || {
    echo "Docker architecture discovery failed or exceeded 10 seconds" >&2
    exit 1
  }
  case "$architecture" in
    aarch64) platform=linux/arm64 ;;
    x86_64) platform=linux/amd64 ;;
    *) usage ;;
  esac
fi
case "$platform" in linux/arm64|linux/amd64) ;; *) usage ;; esac
validate_timeout() {
  value=$1 name=$2 limit=$3
  case "$value" in ''|*[!0-9]*) echo "$name must be a positive integer" >&2; exit 2;; esac
  [ "$value" -gt 0 ] && [ "$value" -le "$limit" ] || { echo "$name must be 1..$limit seconds" >&2; exit 2; }
}
build_timeout=${RUSTZEN_UI_VERIFIER_BUILD_TIMEOUT:-900}
image_check_timeout=${RUSTZEN_UI_VERIFIER_IMAGE_CHECK_TIMEOUT:-45}
validate_timeout "$build_timeout" RUSTZEN_UI_VERIFIER_BUILD_TIMEOUT 1800
validate_timeout "$image_check_timeout" RUSTZEN_UI_VERIFIER_IMAGE_CHECK_TIMEOUT 120
key_input=$(printf 'schema=%s\nplatform=%s\nbase=%s\nsnapshot=%s\nchromium=%s\n' "$schema" "$platform" "$base_image" "$snapshot_timestamp" "$chromium_version")
key=$( { printf '%s' "$key_input"; cat "$dockerfile"; } | shasum -a 256 | awk '{print $1}')
[ "$mode" = key ] && { printf '%s\n' "$key"; exit 0; }
tag="rustzen-admin-browser-verifier:${key:0:24}"
provenance=$(printf 'schemaVersion=%s\nkey=%s\nplatform=%s\nbaseImage=%s\nsnapshot=%s\nchromiumVersion=%s\n' "$schema" "$key" "$platform" "$base_image" "$snapshot_timestamp" "$chromium_version")
provenance_sha=$(printf '%s\n' "$provenance" | shasum -a 256 | awk '{print $1}')
image_docker() {
  remaining=$((image_check_deadline - SECONDS))
  [ "$remaining" -gt 0 ] || return 1
  run_bounded_capture "$remaining" "$docker_bin" "$@"
}

cleanup_probe() {
  mode=${1:-best-effort}
  [ -n "${probe_container:-}" ] || return 0
  if ! run_bounded 15 "$docker_bin" rm -f "$probe_container" >/dev/null 2>&1; then
    [ "$mode" = strict ] && return 1
    return 0
  fi
  probe_container=
}

cleanup() {
  result=$?
  trap - EXIT INT TERM
  stop_tree "$watchdog_pid"
  stop_tree "$command_pid"
  [ -z "$watchdog_pid" ] || wait "$watchdog_pid" 2>/dev/null || true
  [ -z "$command_pid" ] || wait "$command_pid" 2>/dev/null || true
  command_pid=
  watchdog_pid=
  cleanup_probe best-effort || true
  exit "$result"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

image_matches() {
  image_check_deadline=$((SECONDS + image_check_timeout))
  probe_container=
  inspect=$(image_docker image inspect "$tag" --format '{{json .}}' 2>/dev/null) || return 1
  id=$(jq -er '.Id' <<<"$inspect") || return 1
  [ "$(jq -er '.Os' <<<"$inspect")" = linux ] || return 1
  [ "$(jq -er '.Architecture' <<<"$inspect")" = "${platform#linux/}" ] || return 1
  label() { jq -er --arg key "$1" '.Config.Labels[$key]' <<<"$inspect"; }
  [ "$(label io.rustzen.browser-verifier.schema)" = "$schema" ] || return 1
  [ "$(label io.rustzen.browser-verifier.key)" = "$key" ] || return 1
  [ "$(label io.rustzen.browser-verifier.platform)" = "$platform" ] || return 1
  [ "$(label io.rustzen.browser-verifier.snapshot)" = "$snapshot_timestamp" ] || return 1
  [ "$(label io.rustzen.browser-verifier.chromium-version)" = "$chromium_version" ] || return 1
  [ "$(label io.rustzen.browser-verifier.base-image)" = "$base_image" ] || return 1
  probe_container="rz-admin-browser-probe-${key:0:12}-$$"
  actual=$(image_docker run --name "$probe_container" --platform "$platform" --entrypoint cat "$id" /usr/local/share/rustzen-browser-verifier.provenance 2>/dev/null) || {
    cleanup_probe
    return 1
  }
  cleanup_probe strict || return 2
  [ "$actual" = "$provenance" ] || return 1
  image_id=$id
}

if image_matches; then
  :
else
  image_match_status=$?
  [ "$image_match_status" -eq 2 ] && { echo "verifier probe cleanup failed" >&2; exit 1; }
  echo "building pinned Linux browser verifier image for $platform" >&2
  run_bounded "$build_timeout" "$docker_bin" buildx build --platform "$platform" --load \
    --build-arg "BASE_IMAGE=$base_image" --build-arg "VERIFIER_KEY=$key" --build-arg "VERIFIER_PLATFORM=$platform" \
    --build-arg "VERIFIER_SCHEMA=$schema" --build-arg "SNAPSHOT_TIMESTAMP=$snapshot_timestamp" --build-arg "CHROMIUM_VERSION=$chromium_version" \
    --tag "$tag" --file "$dockerfile" "$root/scripts" || { echo "pinned verifier image build failed or exceeded ${build_timeout}s" >&2; exit 1; }
  image_matches || { echo "built verifier image did not match its labels or provenance" >&2; exit 1; }
fi
printf '%s\t%s\t%s\n' "$image_id" "$key" "$provenance_sha"
