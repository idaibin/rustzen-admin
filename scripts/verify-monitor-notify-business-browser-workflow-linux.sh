#!/usr/bin/env bash
# One fresh P8e retained PID1 runtime, P8f-A bootstrap, P8f-B business journey, then owner cleanup.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$root"
usage(){ echo "usage: $0 --export-root PATH --release-result FILE --certificate FILE --public-key FILE --expected-source-identity ID --native-output NEW_DIRECTORY --bootstrap-output NEW_DIRECTORY --output NEW_DIRECTORY [--chromium PATH]" >&2; exit 2; }
test "$#" = 16 -o "$#" = 18 || usage; chromium=chromium
while test "$#" -gt 0; do case "$1" in
  --export-root) test -z "${export_root+x}" || usage; export_root="$2";; --release-result) test -z "${release_result+x}" || usage; release_result="$2";;
  --certificate) test -z "${certificate+x}" || usage; certificate="$2";; --public-key) test -z "${public_key+x}" || usage; public_key="$2";;
  --expected-source-identity) test -z "${source+x}" || usage; source="$2";; --native-output) test -z "${native_output+x}" || usage; native_output="$2";;
  --bootstrap-output) test -z "${bootstrap_output+x}" || usage; bootstrap_output="$2";; --output) test -z "${output+x}" || usage; output="$2";;
  --chromium) test "$chromium" = chromium || usage; chromium="$2";; *) usage;;
esac; shift 2; done
for required in export_root release_result certificate public_key source native_output bootstrap_output output; do test -n "${!required:-}" || usage; done
test ! -e "$native_output" -a ! -e "$bootstrap_output" -a ! -e "$output" || { echo "all outputs must be fresh" >&2; exit 2; }
umask 077; work="$(mktemp -d "$root/target/rz/.p8fb.XXXXXX")"; context="$work/context.json"; native_context="$work/native-context.json"; password="$work/owner-password"; agent="$work/agent-token"; owner="$(pnpm dlx bun@1.3.14 -e 'console.log(crypto.randomUUID())')"
cleanup(){ status=$?; scripts/cleanup-retained-p8e-container.sh --owner-token "$owner" || status=1; rm -rf "$work"; exit "$status"; }; trap cleanup EXIT INT TERM
printf '%s\n' 'p8e-owner-password-4c819a' >"$password"; printf '%s\n' 'p8e-agent-token-1b73ef' >"$agent"
scripts/verify-monitor-native-runtime-linux-amd64.sh --selection distribution/fixtures/monitor-notify.json --export-root "$export_root" --release-result "$release_result" --certificate "$certificate" --public-key "$public_key" --expected-source-identity "$source" --output "$native_output" --retain-container-output "$native_context" --retain-owner-token "$owner"
container="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).containerName)' "$native_context")"; port="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).hostPort)' "$native_context")"; evidence="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).nativeEvidence)' "$native_context")"
admin_bin="$(realpath "$export_root/release/server/bin/rz-admin")"; scripts/verify-selected-web-bootstrap-browser.py --selection distribution/fixtures/monitor-notify.json --admin-url "http://127.0.0.1:$port" --output "$bootstrap_output" --chromium "$chromium" --password-file "$password" --export-root "$export_root" --release-result "$release_result" --certificate "$certificate" --public-key "$public_key" --expected-source-identity "$source" --admin-bin "$admin_bin" --runtime-container "$container"
pnpm dlx bun@1.3.14 -e 'import {canonicalJson} from "./distribution/release-manifest-core.ts";const [out,native,bootstrap,password,agent,exportRoot,release,certificate,publicKey,source]=process.argv.slice(1),n=await Bun.file(native).json();await Bun.write(out,canonicalJson({adminUrl:`http://127.0.0.1:${n.hostPort}`,agentTokenFile:agent,bootstrap,certificate,containerName:n.containerName,expectedSourceIdentity:source,exportRoot,hostPort:n.hostPort,nativeEvidence:n.nativeEvidence,ownerToken:n.ownerToken,passwordFile:password,publicKey,releaseResult:release}));' "$context" "$native_context" "$bootstrap_output/manifest.json" "$password" "$agent" "$(realpath "$export_root")" "$(realpath "$release_result")" "$(realpath "$certificate")" "$(realpath "$public_key")" "$source"
scripts/verify-monitor-notify-business-browser-linux.sh --context "$context" --bootstrap "$bootstrap_output/manifest.json" --output "$output" --chromium "$chromium"
