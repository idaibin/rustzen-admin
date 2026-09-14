#!/usr/bin/env bash
# P8e owns installation; this only retains its capped container for P8f/P8g.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$root"
usage() { echo "usage: $0 --export-root PATH --release-result FILE --certificate FILE --public-key FILE --expected-source-identity ID --native-output NEW_DIRECTORY --browser-output NEW_DIRECTORY --context-output NEW_FILE [--chromium PATH] [--selection FILE]" >&2; exit 2; }
test "$#" -eq 16 -o "$#" -eq 18 -o "$#" -eq 20 || usage
chromium=chromium; selection=distribution/fixtures/monitor.json
while test "$#" -gt 0; do case "$1" in
  --export-root) export_root="$2"; shift 2 ;; --release-result) release_result="$2"; shift 2 ;;
  --certificate) certificate="$2"; shift 2 ;; --public-key) public_key="$2"; shift 2 ;;
  --expected-source-identity) source="$2"; shift 2 ;; --native-output) native_output="$2"; shift 2 ;;
  --browser-output) browser_output="$2"; shift 2 ;; --context-output) context_output="$2"; shift 2 ;;
  --chromium) chromium="$2"; shift 2 ;; --selection) selection="$2"; shift 2 ;; *) usage ;;
esac; done
for value in "${export_root:-}" "${release_result:-}" "${certificate:-}" "${public_key:-}" "${source:-}" "${native_output:-}" "${browser_output:-}" "${context_output:-}"; do test -n "$value" || usage; done
context_output="$(pnpm dlx bun@1.3.14 -e 'import { resolve } from "node:path"; console.log(resolve(process.argv[1]))' "$context_output")"
test ! -e "$context_output" && test ! -L "$context_output" && test ! -e "$browser_output" && test ! -L "$browser_output" || { echo "context and browser outputs must be fresh" >&2; exit 2; }
umask 077; work="$(mktemp -d "$root/target/rz/.p8g-prepare.XXXXXX")"; native_context="$work/native-context.json"; password="$context_output.password"; agent="$context_output.agent-token"; container=""; owner_token="$(pnpm dlx bun@1.3.14 -e 'console.log(crypto.randomUUID())')"
test ! -e "$password" && test ! -e "$agent" || { echo "context credential path exists" >&2; exit 2; }
cleanup() { status=$?; if test "$status" -ne 0; then ids="$(docker ps -aq --filter "label=io.rustzen.p8g-owner=$owner_token" 2>/dev/null || true)"; test "$(printf '%s\n' "$ids" | sed '/^$/d' | wc -l | tr -d ' ')" -le 1 || status=1; test -z "$ids" || docker rm -f "$ids" >/dev/null 2>&1 || true; rm -f "$password" "$agent" "$context_output"; rm -rf "$browser_output"; fi; rm -rf "$work"; exit "$status"; }
trap cleanup EXIT
printf '%s\n' 'p8e-owner-password-4c819a' > "$password"; printf '%s\n' 'p8e-agent-token-1b73ef' > "$agent"
scripts/verify-monitor-native-runtime-linux-amd64.sh --selection "$selection" --export-root "$export_root" --release-result "$release_result" --certificate "$certificate" --public-key "$public_key" --expected-source-identity "$source" --output "$native_output" --retain-container-output "$native_context" --retain-owner-token "$owner_token"
container="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).containerName)' "$native_context")"
host_port="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).hostPort)' "$native_context")"
native_evidence="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).nativeEvidence)' "$native_context")"
admin_bin="$(realpath "$export_root/release/server/bin/rz-admin")"; test -f "$admin_bin" && test ! -L "$admin_bin" || { echo "signed export Admin binary is unsafe" >&2; exit 1; }
admin_url="http://127.0.0.1:$host_port"
curl --fail --silent --show-error "$admin_url/health" > "$work/health.json"
pnpm dlx bun@1.3.14 -e 'const [health,native]=await Promise.all(process.argv.slice(1).map(p=>Bun.file(p).json())); const s=native.selection; if(health?.status!=="ok"||health?.selectedBinding?.buildId!==s?.buildId||health?.selectedBinding?.compositionId!==s?.compositionId) throw Error("retained host health binding differs");' "$work/health.json" "$native_evidence"
scripts/verify-selected-web-bootstrap-browser.py --selection "$selection" --admin-url "$admin_url" --output "$browser_output" --chromium "$chromium" --password-file "$password" --export-root "$export_root" --release-result "$release_result" --certificate "$certificate" --public-key "$public_key" --expected-source-identity "$source" --admin-bin "$admin_bin" --runtime-container "$container"
pnpm dlx bun@1.3.14 -e 'import { canonicalJson } from "./distribution/release-manifest-core.ts"; const [out,native,browser,admin,password,agent,release,exportRoot,certificate,publicKey,source,adminUrl]=process.argv.slice(1); const value=await Bun.file(native).json(); await Bun.write(out,canonicalJson({...value,adminBin:admin,adminUrl,agentTokenFile:agent,browserReceipt:browser,certificate,expectedSourceIdentity:source,exportRoot,nativeEvidence:value.nativeEvidence,passwordFile:password,publicKey,releaseResult:release}));' "$context_output" "$native_context" "$browser_output/manifest.json" "$admin_bin" "$password" "$agent" "$(realpath "$release_result")" "$(realpath "$export_root")" "$(realpath "$certificate")" "$(realpath "$public_key")" "$source" "$admin_url"
trap - EXIT
rm -rf "$work"
echo "P8g prepared retained container: $context_output"
