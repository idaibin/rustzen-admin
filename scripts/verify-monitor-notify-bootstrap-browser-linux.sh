#!/usr/bin/env bash
# P8f monitor-notify bootstrap closure. It intentionally does not prepare P8g load work.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
usage() {
  echo "usage: $0 --export-root PATH --release-result FILE --certificate FILE --public-key FILE --expected-source-identity ID --native-output NEW_DIRECTORY --browser-output NEW_DIRECTORY [--chromium PATH]" >&2
  exit 2
}
test "$#" -eq 14 -o "$#" -eq 16 || usage
chromium=chromium
while test "$#" -gt 0; do
  case "$1" in
    --export-root) export_root="$2" ;;
    --release-result) release_result="$2" ;;
    --certificate) certificate="$2" ;;
    --public-key) public_key="$2" ;;
    --expected-source-identity) source_identity="$2" ;;
    --native-output) native_output="$2" ;;
    --browser-output) browser_output="$2" ;;
    --chromium) chromium="$2" ;;
    *) usage ;;
  esac
  test -n "$2" || usage
  shift 2
done
for value in "${export_root:-}" "${release_result:-}" "${certificate:-}" "${public_key:-}" "${source_identity:-}" "${native_output:-}" "${browser_output:-}"; do test -n "$value" || usage; done
test ! -e "$native_output" && test ! -L "$native_output" && test ! -e "$browser_output" && test ! -L "$browser_output" || { echo "browser outputs must be fresh" >&2; exit 2; }
work="$(mktemp -d "$root/target/rz/.p8f-notify.XXXXXX")"
password="$work/owner-password"
context="$work/retained-context.json"
owner_token="$(pnpm dlx bun@1.3.14 -e 'console.log(crypto.randomUUID())')"
cleanup() {
  status=$?
  trap - EXIT INT TERM
  scripts/cleanup-retained-p8e-container.sh --owner-token "$owner_token" || status=1
  rm -rf "$work"
  exit "$status"
}
trap cleanup EXIT INT TERM
umask 077
printf '%s\n' 'p8e-owner-password-4c819a' > "$password"
scripts/verify-monitor-native-runtime-linux-amd64.sh --selection distribution/fixtures/monitor-notify.json --export-root "$export_root" --release-result "$release_result" --certificate "$certificate" --public-key "$public_key" --expected-source-identity "$source_identity" --output "$native_output" --retain-container-output "$context" --retain-owner-token "$owner_token"
container="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).containerName)' "$context")"
host_port="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).hostPort)' "$context")"
admin_bin="$(realpath "$export_root/release/server/bin/rz-admin")"
test -f "$admin_bin" && test ! -L "$admin_bin" || { echo "signed export Admin binary is unsafe" >&2; exit 1; }
scripts/verify-selected-web-bootstrap-browser.py --selection distribution/fixtures/monitor-notify.json --admin-url "http://127.0.0.1:$host_port" --output "$browser_output" --chromium "$chromium" --password-file "$password" --export-root "$export_root" --release-result "$release_result" --certificate "$certificate" --public-key "$public_key" --expected-source-identity "$source_identity" --admin-bin "$admin_bin" --runtime-container "$container"
echo "P8f monitor-notify bootstrap browser closure PASS: $browser_output/manifest.json"
