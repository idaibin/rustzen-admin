#!/usr/bin/env bash
# Detached Colima evidence producer: Docker owns the long runtime, while this
# launcher polls state and leaves only the immutable evidence directory.
set -euo pipefail
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd); cd "$root"
docker_bin=${RUSTZEN_FULL_PREVIEW_DOCKER:-docker}; platform=linux/amd64; timeout=${RUSTZEN_FULL_PREVIEW_TIMEOUT:-600}
case "$timeout" in ''|*[!0-9]*) exit 2;; esac
read -r head state digest < <(scripts/admin-browser-source-identity.sh)
read -r image_id verifier_key provenance_sha < <(scripts/ensure-admin-browser-verifier-image.sh --platform "$platform")
bin_dir="$root/target/rz/build/x86_64/bin"; agent_dir="$root/target/rz/monitor-agent-multi-node/build/x86_64"
for name in rz-admin rz-monitor rz-insights rz-reports; do test -x "$bin_dir/$name" || { echo "missing $bin_dir/$name" >&2; exit 1; }; done; test -x "$agent_dir/rz-monitor-agent"
test -s "$bin_dir/build-provenance.txt" || { echo "missing current server build provenance" >&2; exit 1; }
test -s "$agent_dir/rz-monitor-agent.provenance.tsv" || { echo "missing current Agent build provenance" >&2; exit 1; }
awk -F '\t' -v head="$head" -v state="$state" -v digest="$digest" '
  $1=="gitHead" && $2==head { h=1 }
  $1=="sourceTreeState" && $2==state { s=1 }
  $1=="sourceTreeSha256" && $2==digest { d=1 }
  END { exit !(h&&s&&d) }
' "$bin_dir/build-provenance.txt" || { echo "server build provenance does not match the current source tree" >&2; exit 1; }
for name in rz-admin rz-monitor rz-insights rz-reports; do
  binary_sha=$(sha256sum "$bin_dir/$name" | awk '{print $1}')
  awk -F '\t' -v name="$name" -v sha="$binary_sha" '$1==name && $2==sha { found=1 } END { exit !found }' "$bin_dir/build-provenance.txt" \
    || { echo "$name digest differs from server build provenance" >&2; exit 1; }
done
agent_sha=$(sha256sum "$agent_dir/rz-monitor-agent" | awk '{print $1}')
awk -F '\t' -v head="$head" -v state="$state" -v digest="$digest" -v sha="$agent_sha" '
  $1=="gitHead" && $2==head { h=1 }
  $1=="sourceTreeState" && $2==state { s=1 }
  $1=="sourceTreeSha256" && $2==digest { d=1 }
  $1=="binarySha256" && $2==sha { b=1 }
  END { exit !(h&&s&&d&&b) }
' "$agent_dir/rz-monitor-agent.provenance.tsv" || { echo "Agent build provenance does not match the current source tree" >&2; exit 1; }
evidence_root="$root/target/rz/full-release-ui-preview"; run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$"; evidence="$evidence_root/runs/$run_id"; staged="$evidence_root/.bin-$run_id"; container="rz-full-release-ui-preview-$run_id"; started=0
cleanup(){ status=$?; trap - EXIT INT TERM; if [ "$started" = 1 ]; then "$docker_bin" logs "$container" >"$evidence/container.log" 2>&1 || true; "$docker_bin" rm -f "$container" >/dev/null 2>&1 || true; fi; rm -rf "$staged"; exit "$status"; }
trap cleanup EXIT INT TERM
mkdir -p "$evidence_root/runs" "$evidence"
mkdir -p "$staged"; cp "$bin_dir"/rz-admin "$bin_dir"/rz-monitor "$bin_dir"/rz-insights "$bin_dir"/rz-reports "$staged/"; cp "$agent_dir/rz-monitor-agent" "$staged/"
printf '%s\n' "$container" > "$evidence/container-name.txt"; printf '%s\n' "$image_id" > "$evidence/verifier-image-id.txt"; printf '%s\n' "$verifier_key" > "$evidence/verifier-key.txt"; printf '%s\n' "$provenance_sha" > "$evidence/verifier-provenance-sha256.txt"
"$docker_bin" image inspect --format '{{.Id}}' "$image_id" > "$evidence/verifier-image-inspect-id.txt"
test "$(cat "$evidence/verifier-image-inspect-id.txt")" = "$image_id"
"$docker_bin" run -d --name "$container" --platform "$platform" --security-opt seccomp=unconfined --env RUSTZEN_VERIFY_HEAD="$head" --env RUSTZEN_VERIFY_SOURCE_TREE_STATE="$state" --env RUSTZEN_VERIFY_SOURCE_TREE_SHA256="$digest" --env RUSTZEN_VERIFY_PLATFORM="$platform" --mount "type=bind,src=$staged,dst=/verify/bin,readonly" --mount "type=bind,src=$evidence,dst=/verify/evidence" --mount "type=bind,src=$root/scripts/capture-full-release-ui-linux-inner.sh,dst=/verify/run.sh,readonly" --mount "type=bind,src=$root/scripts/full-release-preview-cdp.py,dst=/verify/browser.py,readonly" --mount "type=bind,src=$root/scripts/selected_web_bootstrap_cdp.py,dst=/verify/selected_web_bootstrap_cdp.py,readonly" "$image_id" bash /verify/run.sh > "$evidence/container-id.txt"; started=1
deadline=$((SECONDS + timeout)); status=running
while [ "$status" = running ] && [ "$SECONDS" -lt "$deadline" ]; do sleep 1; status=$("$docker_bin" inspect --format '{{.State.Status}}' "$container") || exit 1; done
[ "$status" = exited ] || { echo "preview container timed out" >&2; exit 1; }
test "$("$docker_bin" inspect --format '{{.State.ExitCode}}' "$container")" = 0
jq -e '.status=="passed" and .viewport=={width:1920,height:1080} and (.runtime.nodes|length)==2 and all(.runtime.nodes[];.status=="online") and .runtime.visitorEventCount>=1 and .runtime.automaticSchedule.enabled==true and (.browserAssertions|length)==3 and (.pages|length)==19 and all(.pages[];.authenticated==true and .loading==false and .error=="" and .pageTitle!="" and .documentTitle=="Rustzen Admin" and .viewport=={width:1920,height:1080,devicePixelRatio:1}) and (.screenshots|length)==19 and all(.screenshots[];.dimensions.width==1920 and .dimensions.height==1080)' "$evidence/manifest.json" >/dev/null
for artifact in dashboard.png profile.png monitoring-overview.png monitoring-nodes.png monitoring-incidents.png monitoring-summaries.png analytics-overview.png analytics-details.png reports-templates.png reports-runs.png system-users.png system-roles.png system-menus.png system-modules.png system-status.png system-module-logs.png management-operation-logs.png management-scheduled-tasks.png management-deployments.png; do file "$evidence/$artifact" | grep -q 'PNG image data, 1920 x 1080'; done
echo "$evidence"
