#!/usr/bin/env bash
set -euo pipefail
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ensure="$root/scripts/ensure-admin-browser-verifier-image.sh"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/rz-verifier-image-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
cat >"$tmp/docker" <<'DOCKER'
#!/usr/bin/env bash
set -euo pipefail
state=${FAKE_STATE:?}; log=${FAKE_LOG:?}; mode=$(cat "$state" 2>/dev/null || true)
case "$1 ${2:-}" in
  info\ *)
    if [ "$mode" = hang-info ]; then sleep 30; fi
    printf 'aarch64\n' ;;
  'image inspect')
    case "$mode" in '' ) exit 1;; hang-inspect) sleep 30;; esac
    schema=$FAKE_SCHEMA key=$FAKE_KEY platform=$FAKE_PLATFORM snapshot=$FAKE_SNAPSHOT chromium=$FAKE_CHROMIUM base=$FAKE_BASE os=linux architecture=arm64
    case "$mode" in
      bad-schema) schema=wrong;; bad-key) key=wrong;; bad-platform) platform=linux/amd64;;
      bad-snapshot) snapshot=wrong;; bad-chromium) chromium=wrong;; bad-base) base=wrong;;
      bad-os) os=windows;; bad-architecture) architecture=amd64;;
    esac
    jq -nc --arg id sha256:fixture --arg os "$os" --arg architecture "$architecture" --arg schema "$schema" --arg key "$key" --arg platform "$platform" --arg snapshot "$snapshot" --arg chromium "$chromium" --arg base "$base" \
      '{Id:$id,Os:$os,Architecture:$architecture,Config:{Labels:{"io.rustzen.browser-verifier.schema":$schema,"io.rustzen.browser-verifier.key":$key,"io.rustzen.browser-verifier.platform":$platform,"io.rustzen.browser-verifier.snapshot":$snapshot,"io.rustzen.browser-verifier.chromium-version":$chromium,"io.rustzen.browser-verifier.base-image":$base}}}' ;;
  run\ *)
    if [ "$mode" = hang-run ]; then sleep 30; fi
    if [ "$mode" = bad-provenance ]; then printf 'wrong\n'; exit 0; fi
    printf '%s' "$FAKE_PROVENANCE" ;;
  rm\ *)
    echo cleanup >>"$log"
    if [ "$mode" = hang-rm ]; then sleep 30; fi
    if [ "$mode" = fail-rm ]; then exit 1; fi ;;
  'buildx build') echo build >>"$log"; printf good >"$state" ;;
  *) echo "unexpected fake docker invocation: $*" >&2; exit 2;;
esac
DOCKER
chmod +x "$tmp/docker"
platform=linux/arm64
dockerfile="$tmp/verifier.Dockerfile"
cp "$root/scripts/admin-browser-verifier.Dockerfile" "$dockerfile"
key=$(RUSTZEN_UI_VERIFIER_DOCKERFILE="$dockerfile" "$ensure" --platform "$platform" --print-key)
provenance=$(printf 'schemaVersion=1\nkey=%s\nplatform=%s\nbaseImage=%s\nsnapshot=%s\nchromiumVersion=%s\n' "$key" "$platform" 'debian@sha256:e5b6442dd2e9684cf5e87d8338b5968f3b348636fc0be6d7850a381e3731a2bd' 20240131T000000Z 120.0.6099.224-1~deb11u1)
export FAKE_STATE="$tmp/state" FAKE_LOG="$tmp/log" FAKE_SCHEMA=1 FAKE_KEY="$key" FAKE_PLATFORM="$platform" FAKE_SNAPSHOT=20240131T000000Z FAKE_CHROMIUM=120.0.6099.224-1~deb11u1 FAKE_BASE='debian@sha256:e5b6442dd2e9684cf5e87d8338b5968f3b348636fc0be6d7850a381e3731a2bd' FAKE_PROVENANCE="$provenance"
run_ensure() {
  RUSTZEN_UI_VERIFIER_DOCKERFILE="$dockerfile" RUSTZEN_UI_VERIFIER_DOCKER="$tmp/docker" RUSTZEN_UI_VERIFIER_IMAGE_CHECK_TIMEOUT=5 "$ensure" --platform "$platform"
}
printf hang-info >"$tmp/state"
started=$(date +%s)
if RUSTZEN_UI_VERIFIER_DOCKERFILE="$dockerfile" RUSTZEN_UI_VERIFIER_DOCKER="$tmp/docker" "$ensure" >"$tmp/identity" 2>"$tmp/error"; then
  echo 'unbounded Docker info was unexpectedly accepted' >&2
  exit 1
fi
elapsed=$(( $(date +%s) - started ))
[ "$elapsed" -le 15 ] || { echo "Docker info timeout exceeded bound: ${elapsed}s" >&2; exit 1; }
test ! -s "$tmp/identity"
: >"$tmp/state"
run_ensure
test "$(grep -c '^build$' "$tmp/log")" -eq 1
test "$(grep -c '^cleanup$' "$tmp/log")" -eq 1
run_ensure
test "$(grep -c '^build$' "$tmp/log")" -eq 1
test "$(grep -c '^cleanup$' "$tmp/log")" -eq 2
for mismatch in bad-schema bad-key bad-platform bad-snapshot bad-chromium bad-base bad-os bad-architecture bad-provenance hang-inspect hang-run; do
  printf '%s' "$mismatch" >"$tmp/state"
  run_ensure
done
# one cold build plus every mismatch; this also proves a warm check never builds.
test "$(grep -c '^build$' "$tmp/log")" -eq 12
test "$(grep -c '^cleanup$' "$tmp/log")" -ge 12
for cleanup_failure in fail-rm hang-rm; do
  printf '%s' "$cleanup_failure" >"$tmp/state"
  if run_ensure >"$tmp/identity" 2>"$tmp/error"; then
    echo "cleanup failure unexpectedly accepted: $cleanup_failure" >&2
    exit 1
  fi
  test ! -s "$tmp/identity"
done
test "$(grep -c '^build$' "$tmp/log")" -eq 12
printf '\n# key invalidation fixture\n' >>"$dockerfile"
changed_key=$(RUSTZEN_UI_VERIFIER_DOCKERFILE="$dockerfile" "$ensure" --platform "$platform" --print-key)
test "$changed_key" != "$key"
echo 'Admin browser verifier image ensure seams passed'
