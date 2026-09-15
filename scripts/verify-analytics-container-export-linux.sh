#!/usr/bin/env bash
# Produces one Analytics linux/amd64 BuildKit container export and closes the
# restricted six-command contract receipt for it. Every output path must be new.
set -euo pipefail

repository="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
selection="distribution/fixtures/analytics.json"
output=""
verifier_image="debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171"

usage() {
  cat >&2 <<'EOF'
usage: scripts/verify-analytics-container-export-linux.sh --output <new-evidence-dir>
          [--selection <selection.json>] [--verifier-image <image@sha256:digest>]
EOF
  exit 2
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --output) [ "$#" -ge 2 ] || usage; output=$2; shift 2 ;;
    --selection) [ "$#" -ge 2 ] || usage; selection=$2; shift 2 ;;
    --verifier-image) [ "$#" -ge 2 ] || usage; verifier_image=$2; shift 2 ;;
    *) usage ;;
  esac
done
[ -n "$output" ] || usage
case "$output" in
  /*) ;;
  *) output="$repository/$output" ;;
esac
[ ! -e "$output" ] || { echo "output path must not exist: $output" >&2; exit 2; }
[[ "$verifier_image" =~ @sha256:[0-9a-f]{64}$ ]] || {
  echo "verifier image must be a sha256 digest reference: $verifier_image" >&2
  exit 2
}
digest="${verifier_image##*@}"

identity_line() {
  (cd "$repository" && scripts/admin-browser-source-identity.sh)
}

run_bun() {
  (cd "$repository" && pnpm dlx bun@1.3.14 "$@")
}

run_restricted() {
  local name=$1
  shift
  docker run --rm --platform linux/amd64 \
    --network none --read-only --cap-drop ALL --security-opt no-new-privileges \
    --pids-limit 32 --memory 256m --tmpfs /tmp:rw,noexec,nosuid,nodev \
    -v "$output/export/release/server/bin":/export:ro \
    "$verifier_image" "$@" > "$output/contract-stdout/$name.stdout"
}

mkdir -p "$output/contract-stdout"

read -r source_head source_state source_tree < <(identity_line)
source_identity="git:$source_head tree:$source_tree state:$source_state"
printf '%s\n' "$source_identity" > "$output/source-identity-before.txt"

docker_context="$(docker context show)"

echo "== Building Analytics linux/amd64 container export" >&2
# The renamed arg is required: this Docker/BuildKit client drops any build arg
# named exactly DISTRIBUTION (space and equals forms, both drivers).
docker buildx build --platform linux/amd64 \
  "--build-arg=TARGET_TRIPLE=x86_64-unknown-linux-musl" \
  "--build-arg=RZ_DISTRIBUTION=analytics" \
  "--build-arg=SOURCE_IDENTITY=$source_identity" \
  --target export --output "type=local,dest=$output/export" \
  "$repository" 2>&1 | tee "$output/build.log"

echo "== Host-validating the export before the restricted pass" >&2
run_bun scripts/distribution-verify-container-export.ts \
  --selection "$selection" --export-root "$output/export" \
  --expected-source-identity "$source_identity" \
  --evidence linux-amd64-buildkit > "$output/verify-before.json"

echo "== Running the six restricted contract commands" >&2
run_restricted rz-admin-contract-selected-admin /export/rz-admin contract selected admin
run_restricted rz-insights-contract-selected /export/rz-insights contract selected
run_restricted rz-admin-contract-config-selected-access /export/rz-admin contract config selected access
run_restricted rz-insights-contract-config-selected /export/rz-insights contract config selected
run_restricted rz-admin-contract-protocol /export/rz-admin contract protocol
run_restricted rz-insights-contract-protocol /export/rz-insights contract protocol

read -r after_head after_state after_tree < <(identity_line)
source_identity_after="git:$after_head tree:$after_tree state:$after_state"
printf '%s\n' "$source_identity_after" > "$output/source-identity-after.txt"
[ "$source_identity_after" = "$source_identity" ] || {
  echo "repository source identity changed during the run" >&2
  exit 1
}

echo "== Host-validating the export after the restricted pass" >&2
run_bun scripts/distribution-verify-container-export.ts \
  --selection "$selection" --export-root "$output/export" \
  --expected-source-identity "$source_identity" \
  --evidence linux-amd64-buildkit > "$output/verify-after.json"

echo "== Producing and re-verifying the contract receipt" >&2
run_bun scripts/distribution-verify-analytics-contract-receipt.ts \
  --selection "$selection" --export-root "$output/export" \
  --expected-source-identity "$source_identity" \
  --source-identity-after "$source_identity_after" \
  --stdout-dir "$output/contract-stdout" \
  --verifier-image-digest "$digest" \
  --docker-context "$docker_context" \
  --receipt-out "$output/receipt.json" > "$output/receipt-produce.json"
run_bun scripts/distribution-verify-analytics-contract-receipt.ts \
  --selection "$selection" --export-root "$output/export" \
  --expected-source-identity "$source_identity" \
  --stdout-dir "$output/contract-stdout" \
  --receipt "$output/receipt.json" > "$output/receipt-verify.json"

echo "Analytics container export evidence closed: $output/receipt.json" >&2
