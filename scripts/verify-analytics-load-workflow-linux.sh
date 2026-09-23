#!/usr/bin/env bash
# One fresh retained P8e analytics PID1 runtime, then the P8g load certification.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$root"
usage(){ echo "usage: $0 --export-root PATH --release-result FILE --certificate FILE --public-key FILE --expected-source-identity ID --runtime-evidence FILE --browser-receipt FILE --native-output NEW_DIRECTORY --output NEW_DIRECTORY" >&2; exit 2; }
test "$#" = 18 || usage
while test "$#" -gt 0; do case "$1" in
  --export-root) test -z "${export_root+x}" || usage; export_root="$2";; --release-result) test -z "${release_result+x}" || usage; release_result="$2";;
  --certificate) test -z "${certificate+x}" || usage; certificate="$2";; --public-key) test -z "${public_key+x}" || usage; public_key="$2";;
  --expected-source-identity) test -z "${source+x}" || usage; source="$2";; --native-output) test -z "${native_output+x}" || usage; native_output="$2";;
  --output) test -z "${output+x}" || usage; output="$2";;
  --runtime-evidence) test -z "${runtime_evidence+x}" || usage; runtime_evidence="$2";;
  --browser-receipt) test -z "${browser_receipt+x}" || usage; browser_receipt="$2";;
  *) usage;;
esac; shift 2; done
for required in export_root release_result certificate public_key source native_output output runtime_evidence browser_receipt; do test -n "${!required:-}" || usage; done
test -f "$runtime_evidence" -a -f "$browser_receipt"
test ! -e "$native_output" -a ! -e "$output" || { echo "all outputs must be fresh" >&2; exit 2; }
umask 077; work="$(mktemp -d "$root/target/rz/.p8g-a.XXXXXX")"; context="$work/context.json"; password="$work/owner-password"; project_key="$work/project-key"; owner="$(pnpm dlx bun@1.3.14 -e 'console.log(crypto.randomUUID())')"
cleanup(){ status=$?; scripts/cleanup-retained-p8e-container.sh --owner-token "$owner" || status=1; rm -rf "$work"; exit "$status"; }; trap cleanup EXIT INT TERM
printf '%s\n' 'p8e-owner-password-4c819a' >"$password"
pnpm dlx bun@1.3.14 -e 'console.log(crypto.randomUUID())' >"$project_key"
scripts/verify-analytics-native-runtime-linux-amd64.sh --selection distribution/fixtures/analytics.json --export-root "$export_root" --release-result "$release_result" --certificate "$certificate" --public-key "$public_key" --expected-source-identity "$source" --output "$native_output" --retain-container-output "$work/native-context.json" --retain-owner-token "$owner"
container="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).containerName)' "$work/native-context.json")"; port="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).hostPort)' "$work/native-context.json")"; native_evidence="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).nativeEvidence)' "$work/native-context.json")"
runtime_evidence="$(realpath "$runtime_evidence")"; runtime_sha="$(shasum -a 256 "$runtime_evidence" | cut -d " " -f1)"
browser_receipt="$(realpath "$browser_receipt")"; browser_sha="$(shasum -a 256 "$browser_receipt" | cut -d " " -f1)"
pnpm dlx bun@1.3.14 -e 'import { canonicalJson } from "./distribution/release-manifest-core.ts"; const [out, adminUrl, container, release, certificate, publicKey, exportRoot, password, key, native, runtime, runtimeSha, browser, browserSha]=process.argv.slice(1); await Bun.write(out, canonicalJson({ adminUrl, containerName: container, releaseResult: release, certificate, publicKey, exportRoot, passwordFile: password, projectKeyFile: key, nativeEvidence: native, runtimeEvidence: runtime, runtimeEvidenceSha256: runtimeSha, browserReceipt: browser, browserReceiptSha256: browserSha }));' "$context" "http://127.0.0.1:$port" "$container" "$(realpath "$release_result")" "$(realpath "$certificate")" "$(realpath "$public_key")" "$(realpath "$export_root")" "$password" "$project_key" "$native_evidence" "$runtime_evidence" "$runtime_sha" "$browser_receipt" "$browser_sha"
pnpm dlx bun@1.3.14 scripts/analytics-load-certification.ts --context "$context" --output "$output"
pnpm dlx bun@1.3.14 scripts/analytics-load-receipt-schema.ts "$output/receipt.json"
test "$(find "$output" -mindepth 1 | wc -l | tr -d ' ')" = 1
echo "P8g analytics load certification PASS: $output/receipt.json"
