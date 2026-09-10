#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

# The Reports gate uses the proven staging/publication primitives. Its own
# variables avoid coupling an Automation acceptance run to Monitoring settings.
export RUSTZEN_MONITORING_UI_STATE_DOCKER="${RUSTZEN_REPORTS_UI_STATE_DOCKER:-docker}"
export RUSTZEN_MONITORING_UI_STATE_TIMEOUT="${RUSTZEN_REPORTS_UI_STATE_TIMEOUT:-900}"
export RUSTZEN_MONITORING_UI_STATE_DOCKER_INFO_TIMEOUT="${RUSTZEN_REPORTS_UI_STATE_DOCKER_INFO_TIMEOUT:-10}"
export RUSTZEN_MONITORING_UI_STATE_DOCKER_CLEANUP_TIMEOUT="${RUSTZEN_REPORTS_UI_STATE_DOCKER_CLEANUP_TIMEOUT:-3}"
. "$root/scripts/monitoring-ui-state-gate-lib.sh"
. "$root/scripts/reports-ui-state-evidence-lib.sh"

validate_timeout "$timeout" RUSTZEN_REPORTS_UI_STATE_TIMEOUT 1800
validate_timeout "$info_timeout" RUSTZEN_REPORTS_UI_STATE_DOCKER_INFO_TIMEOUT 60
validate_timeout "$cleanup_timeout" RUSTZEN_REPORTS_UI_STATE_DOCKER_CLEANUP_TIMEOUT 30
architecture=$(env_value RUSTZEN_UI_LINUX_ARCH)
if [ -z "$architecture" ]; then
    architecture=$(run_bounded_capture "$info_timeout" "$docker_bin" info --format '{{.Architecture}}') || {
        echo "Docker architecture discovery failed or timed out" >&2
        exit 1
    }
fi
case "$architecture" in
    aarch64)
        platform=linux/arm64
        target_triple=aarch64-unknown-linux-musl
        file_pattern='ELF 64-bit.*ARM aarch64'
        ;;
    x86_64)
        platform=linux/amd64
        target_triple=x86_64-unknown-linux-musl
        file_pattern='ELF 64-bit.*x86-64'
        ;;
    *)
        echo "unsupported Docker architecture: $architecture" >&2
        exit 1
        ;;
esac

read -r verifier_image verifier_key verifier_provenance_sha < <(
    "$root/scripts/ensure-admin-browser-verifier-image.sh" --platform "$platform"
)
bin_dir=$(env_value RUSTZEN_UI_LINUX_BIN_DIR)
[ -n "$bin_dir" ] || bin_dir="$root/target/rz/build/$architecture/bin"
read -r head tree_state tree_sha < <("$root/scripts/admin-browser-source-identity.sh")
evidence_root="$root/target/rz/reports-ui-state"
current="$evidence_root/current"
lock_dir="$evidence_root/.verify.lock"
run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$"
candidate="$evidence_root/.candidate-$run_id"
staged="$evidence_root/.binaries-$run_id"
published_run="$evidence_root/runs/$run_id"
publish_link="$evidence_root/.current-$run_id"
container="rz-reports-ui-state-$run_id"

mkdir -p "$evidence_root/runs"
if [ -e "$current" ] && [ ! -L "$current" ]; then
    echo "refusing to replace legacy Reports UI evidence directory" >&2
    exit 1
fi
if ! mkdir "$lock_dir" 2>/dev/null; then
    echo "another Reports UI state verification owns $lock_dir" >&2
    exit 1
fi
trap outer_cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
mkdir -p "$candidate" "$staged"

test -s "$bin_dir/build-provenance.txt" || {
    echo "missing build provenance" >&2
    exit 1
}
cp "$bin_dir/build-provenance.txt" "$candidate/build-provenance.txt"
verify_build_provenance \
    "$candidate/build-provenance.txt" \
    "$head" "$tree_state" "$tree_sha" "$architecture" "$target_triple" "$platform" || {
    echo "Linux provenance does not match current source identity" >&2
    exit 1
}
for name in rz-admin rz-monitor rz-insights rz-reports; do
    test -x "$bin_dir/$name" || {
        echo "missing Linux binary: $bin_dir/$name" >&2
        echo "run just build-admin-browser-linux first" >&2
        exit 1
    }
    stage_binary "$name" || {
        echo "staged Linux binary differs from immutable provenance: $name" >&2
        exit 1
    }
done
verify_staged_binaries

run_bounded "$timeout" "$docker_bin" run \
    --name "$container" \
    --platform "$platform" \
    --security-opt seccomp=unconfined \
    --env RUSTZEN_VERIFY_HEAD="$head" \
    --env RUSTZEN_VERIFY_SOURCE_TREE_STATE="$tree_state" \
    --env RUSTZEN_VERIFY_SOURCE_TREE_SHA256="$tree_sha" \
    --env RUSTZEN_VERIFY_PLATFORM="$platform" \
    --env RUSTZEN_VERIFY_CHROMIUM_VERSION=120.0.6099.224-1~deb11u1 \
    --env TZ=UTC \
    --mount "type=bind,src=$staged,dst=/verify/bin,readonly" \
    --mount "type=bind,src=$candidate,dst=/verify/evidence" \
    --mount "type=bind,src=$root/scripts/verify-reports-ui-state-linux-inner.sh,dst=/verify/run.sh,readonly" \
    --mount "type=bind,src=$root/scripts/monitoring-ui-state-inner-lib.sh,dst=/verify/inner-lib.sh,readonly" \
    --mount "type=bind,src=$root/scripts/reports-ui-state-delivery-lib.sh,dst=/verify/delivery-lib.sh,readonly" \
    --mount "type=bind,src=$root/scripts/reports-ui-state-browser-lib.sh,dst=/verify/browser-lib.sh,readonly" \
    --mount "type=bind,src=$root/scripts/monitoring-ui-state-fixture.py,dst=/verify/fixture.py,readonly" \
    "$verifier_image" bash /verify/run.sh
run_bounded "$cleanup_timeout" "$docker_bin" rm "$container" >/dev/null

verify_reports_ui_state_manifest "$candidate/manifest.json" "$head" "$tree_state" "$tree_sha" "$platform"
verify_reports_ui_state_artifacts "$candidate" "$candidate/manifest.json"
verify_reports_ui_state_receipts "$candidate" "$candidate/manifest.json"
verify_reports_ui_state_delivery_receipts "$candidate" "$candidate/manifest.json"
verify_reports_ui_state_forbidden_receipts "$candidate" "$candidate/manifest.json"
verify_reports_ui_state_source_evidence "$candidate" "$candidate/manifest.json"
verify_staged_binaries
read -r final_head final_state final_sha < <("$root/scripts/admin-browser-source-identity.sh")
test "$final_head" = "$head"
test "$final_state" = "$tree_state"
test "$final_sha" = "$tree_sha"
publish_candidate
rm -rf "$staged" "$lock_dir"
trap - EXIT INT TERM
echo "Reports UI state Linux evidence published: $current/manifest.json"
