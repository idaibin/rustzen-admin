#!/usr/bin/env bash
# One fresh retained P8e analytics PID1 runtime, then the P8f business browser journey.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$root"
usage(){ echo "usage: $0 --export-root PATH --release-result FILE --certificate FILE --public-key FILE --expected-source-identity ID --runtime-evidence FILE --native-output NEW_DIRECTORY --output NEW_DIRECTORY [--chromium PATH]" >&2; exit 2; }
test "$#" = 16 -o "$#" = 18 || usage; chromium="${CHROMIUM_BINARY:-chromium}"
while test "$#" -gt 0; do case "$1" in
  --export-root) test -z "${export_root+x}" || usage; export_root="$2";; --release-result) test -z "${release_result+x}" || usage; release_result="$2";;
  --certificate) test -z "${certificate+x}" || usage; certificate="$2";; --public-key) test -z "${public_key+x}" || usage; public_key="$2";;
  --expected-source-identity) test -z "${source+x}" || usage; source="$2";; --native-output) test -z "${native_output+x}" || usage; native_output="$2";;
  --output) test -z "${output+x}" || usage; output="$2";;
  --runtime-evidence) test -z "${runtime_evidence+x}" || usage; runtime_evidence="$2";;
  --chromium) chromium="$2";; *) usage;;
esac; shift 2; done
for required in export_root release_result certificate public_key source native_output output runtime_evidence; do test -n "${!required:-}" || usage; done
test -f "$runtime_evidence"
test ! -e "$native_output" -a ! -e "$output" || { echo "all outputs must be fresh" >&2; exit 2; }
umask 077; work="$(mktemp -d "$root/target/rz/.p8f.XXXXXX")"; context="$work/context.json"; facts="$work/container-facts.json"; password="$work/owner-password"; signals="$work/signals"; owner="$(pnpm dlx bun@1.3.14 -e 'console.log(crypto.randomUUID())')"
mkdir "$signals"
cleanup(){ status=$?; test -z "${watcher:-}" || { kill "$watcher" 2>/dev/null || true; wait "$watcher" 2>/dev/null || true; }; scripts/cleanup-retained-p8e-container.sh --owner-token "$owner" || status=1; rm -rf "$work"; exit "$status"; }; trap cleanup EXIT INT TERM
printf '%s\n' 'p8e-owner-password-4c819a' >"$password"
scripts/verify-analytics-native-runtime-linux-amd64.sh --selection distribution/fixtures/analytics.json --export-root "$export_root" --release-result "$release_result" --certificate "$certificate" --public-key "$public_key" --expected-source-identity "$source" --output "$native_output" --retain-container-output "$work/native-context.json" --retain-owner-token "$owner"
container="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).containerName)' "$work/native-context.json")"; port="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).hostPort)' "$work/native-context.json")"; native_evidence="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).nativeEvidence)' "$work/native-context.json")"
read -r head _ < <(cd "$root" && git rev-parse HEAD)
read -r harness_head harness_state harness_tree < <(cd "$root" && scripts/admin-browser-source-identity.sh)
harness_identity="git:$harness_head tree:$harness_tree state:$harness_state"
insights_health="$(docker exec "$container" curl --fail --silent http://127.0.0.1:19802/health)"
current_target="$(docker exec "$container" readlink /opt/rz/current)"
composition_id="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).selection.compositionId)' "$native_evidence")"
container_id="$(docker inspect --format '{{.Id}}' "$container")"; container_image="$(docker inspect --format '{{.Image}}' "$container")"
pnpm dlx bun@1.3.14 -e 'import { canonicalJson } from "./distribution/release-manifest-core.ts"; const [out, insights, target, composition, id, image]=process.argv.slice(1); await Bun.write(out, canonicalJson({ insightsHealth: JSON.parse(insights), currentTarget: target, compositionId: composition, containerId: id, containerImage: image, units: ["rz-admin.service", "rz-insights.service", "rz-full.service"] }));' "$facts" "$insights_health" "$current_target" "$composition_id" "$container_id" "$container_image"
runtime_evidence="$(realpath "$runtime_evidence")"
runtime_evidence_sha="$(shasum -a 256 "$runtime_evidence" | cut -d " " -f1)"
pnpm dlx bun@1.3.14 -e 'import { canonicalJson } from "./distribution/release-manifest-core.ts"; const [out, adminUrl, name, id, image, release, certificate, exportRoot, passwordFile, native, identity, head, facts, runtimeEvidence, runtimeSha]=process.argv.slice(1); await Bun.write(out, canonicalJson({ adminUrl, containerName: name, containerId: id, containerImage: image, releaseResult: release, certificate, exportRoot, passwordFile, nativeEvidence: native, harnessSourceIdentity: identity, head, containerFacts: facts, runtimeEvidence, runtimeEvidenceSha256: runtimeSha }));' "$context" "http://127.0.0.1:$port" "$container" "$container_id" "$container_image" "$(realpath "$release_result")" "$(realpath "$certificate")" "$(realpath "$export_root")" "$password" "$native_evidence" "$harness_identity" "$head" "$facts" "$runtime_evidence" "$runtime_evidence_sha"
(
  # A failed lifecycle command still publishes a *-failed marker so the driver's
  # bounded wait fails fast instead of timing out against a marker that promises
  # an event which never happened.
  while :; do
    if test -e "$signals/stop-insights-requested"; then
      rm -f "$signals/stop-insights-requested"
      echo "watcher: stopping rz-insights $(date -u +%H:%M:%S)" >&2
      if docker exec "$container" systemctl stop rz-insights.service; then
        touch "$signals/insights-stopped"
      else
        echo "watcher: stop command failed" >&2
        touch "$signals/insights-stopped-failed"
      fi
    fi
    if test -e "$signals/restart-requested"; then
      rm -f "$signals/restart-requested"
      echo "watcher: restarting both services $(date -u +%H:%M:%S)" >&2
      if docker exec "$container" systemctl restart rz-insights.service rz-admin.service; then
        for _ in $(seq 1 90); do docker exec "$container" sh -c 'curl --fail --silent http://127.0.0.1:19801/health >/dev/null && curl --fail --silent http://127.0.0.1:19802/health >/dev/null' && break; sleep 1; done
        touch "$signals/restart-done"
      else
        echo "watcher: restart command failed" >&2
        touch "$signals/restart-done-failed"
      fi
    fi
    if test -e "$signals/restart-2-requested"; then
      rm -f "$signals/restart-2-requested"
      echo "watcher: second restart $(date -u +%H:%M:%S)" >&2
      if docker exec "$container" systemctl restart rz-insights.service rz-admin.service; then
        for _ in $(seq 1 90); do docker exec "$container" sh -c 'curl --fail --silent http://127.0.0.1:19801/health >/dev/null && curl --fail --silent http://127.0.0.1:19802/health >/dev/null' && break; sleep 1; done
        touch "$signals/restart-2-done"
        echo "watcher: second restart complete $(date -u +%H:%M:%S)" >&2
        break
      else
        echo "watcher: second restart command failed" >&2
        touch "$signals/restart-2-done-failed"
        break
      fi
    fi
    sleep 1
  done
) & watcher=$!
scripts/verify-analytics-business-browser-linux.sh --context "$context" --container-facts "$facts" --output "$output" --signals "$signals" --chromium "$chromium"
kill "$watcher" 2>/dev/null || true; wait "$watcher" 2>/dev/null || true; watcher=
echo "P8f analytics business browser workflow PASS: $output/receipt.json"
