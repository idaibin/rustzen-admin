#!/usr/bin/env bash
set -euo pipefail

for command in chromium curl dpkg-query file groupadd jq python3 setpriv sha256sum ss useradd; do command -v "$command" >/dev/null; done
for port in 19801 19802 19804; do ! ss -H -ltn "sport = :$port" | grep -q . || { echo "port occupied: $port" >&2; exit 1; }; done
groupadd -g 2410 rz-central
useradd -u 2410 -g rz-central -M -s /usr/sbin/nologin rz-central
for identity in a b; do
  case "$identity" in a) uid=2411 ;; b) uid=2412 ;; esac
  groupadd -g "$uid" "rz-agent-$identity"
  useradd -u "$uid" -g "rz-agent-$identity" -M -s /usr/sbin/nologin "rz-agent-$identity"
done
install -d -m 0750 -o rz-central -g rz-central /opt/rz/data /opt/rz/data/reports /opt/rz/logs
for name in rz-admin rz-monitor rz-reports; do install -m 0755 "/verify/bin/$name" "/opt/rz/$name"; chown rz-central:rz-central "/opt/rz/$name"; done
for identity in a b; do install -d -m 0750 -o "rz-agent-$identity" -g "rz-agent-$identity" "/var/lib/rz-agent-$identity/logs"; done

export RUSTZEN_ENV=development RUSTZEN_RUNTIME_ROOT=/opt/rz RUSTZEN_INTERNAL_HOST=127.0.0.1
export RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_ADMIN_PORT=19801 RUSTZEN_MONITOR_PORT=19802 RUSTZEN_REPORTS_PORT=19804
export RUSTZEN_ADMIN_SQLITE_PATH=/opt/rz/data/admin.db RUSTZEN_MONITOR_SQLITE_PATH=/opt/rz/data/monitor.db RUSTZEN_REPORTS_SQLITE_PATH=/opt/rz/data/reports/reports.db
export RUSTZEN_JWT_SECRET=monitor-multi-node-ui-jwt RUSTZEN_IPC_TOKEN=monitor-multi-node-ui-ipc RUSTZEN_MONITOR_AGENT_TOKEN=monitor-multi-node-ui-agent
export RUSTZEN_REPORTS_CREDENTIAL_KEY=monitor-multi-node-ui-reports RUSTZEN_REPORTS_BROWSER_PATH=/usr/bin/chromium RUSTZEN_TIMEZONE=UTC RUST_LOG=warn
export RUSTZEN_BUILD_ID=$(printf 'a%.0s' {1..64}) RUSTZEN_COMPOSITION_ID=$(printf 'b%.0s' {1..64}) RUSTZEN_MONITOR_SCHEMA_FINGERPRINT=$(printf 'c%.0s' {1..64}) RUSTZEN_MONITOR_DATA_CONTRACT_ID=$(printf 'd%.0s' {1..64})
pids=()
cleanup() { status=$?; trap - EXIT INT TERM; for pid in "${pids[@]}"; do kill -TERM "$pid" 2>/dev/null || true; done; for pid in "${pids[@]}"; do wait "$pid" 2>/dev/null || true; done; exit "$status"; }
trap cleanup EXIT INT TERM

central() { setpriv --reuid=rz-central --regid=rz-central --init-groups --no-new-privs -- env RUSTZEN_ENV="$RUSTZEN_ENV" RUSTZEN_RUNTIME_ROOT="$RUSTZEN_RUNTIME_ROOT" RUSTZEN_INTERNAL_HOST="$RUSTZEN_INTERNAL_HOST" RUSTZEN_ADMIN_HOST="$RUSTZEN_ADMIN_HOST" RUSTZEN_ADMIN_PORT="$RUSTZEN_ADMIN_PORT" RUSTZEN_MONITOR_PORT="$RUSTZEN_MONITOR_PORT" RUSTZEN_REPORTS_PORT="$RUSTZEN_REPORTS_PORT" RUSTZEN_ADMIN_SQLITE_PATH="$RUSTZEN_ADMIN_SQLITE_PATH" RUSTZEN_MONITOR_SQLITE_PATH="$RUSTZEN_MONITOR_SQLITE_PATH" RUSTZEN_REPORTS_SQLITE_PATH="$RUSTZEN_REPORTS_SQLITE_PATH" RUSTZEN_JWT_SECRET="$RUSTZEN_JWT_SECRET" RUSTZEN_IPC_TOKEN="$RUSTZEN_IPC_TOKEN" RUSTZEN_MONITOR_AGENT_TOKEN="$RUSTZEN_MONITOR_AGENT_TOKEN" RUSTZEN_REPORTS_CREDENTIAL_KEY="$RUSTZEN_REPORTS_CREDENTIAL_KEY" RUSTZEN_REPORTS_BROWSER_PATH="$RUSTZEN_REPORTS_BROWSER_PATH" RUSTZEN_TIMEZONE=UTC RUST_LOG=warn RUSTZEN_BUILD_ID="$RUSTZEN_BUILD_ID" RUSTZEN_COMPOSITION_ID="$RUSTZEN_COMPOSITION_ID" RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_MONITOR_SCHEMA_FINGERPRINT" RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_MONITOR_DATA_CONTRACT_ID" "$@"; }
start() { name=$1; shift; central "$@" >"/opt/rz/logs/$name.log" 2>&1 & pids+=("$!"); }
wait_health() { for url in "$@"; do for _ in $(seq 1 300); do curl --silent --fail --connect-timeout 2 --max-time 3 "$url" >/dev/null 2>&1 && break; sleep .1; done; curl --silent --fail --connect-timeout 2 --max-time 3 "$url" >/dev/null; done; }
central /opt/rz/rz-monitor init-db
central /opt/rz/rz-monitor bind-database
start monitor /opt/rz/rz-monitor controller
start reports /opt/rz/rz-reports serve
start admin /opt/rz/rz-admin serve
wait_health http://127.0.0.1:19801/health http://127.0.0.1:19802/health http://127.0.0.1:19804/health
for identity in a b; do
  setpriv --reuid="rz-agent-$identity" --regid="rz-agent-$identity" --init-groups --no-new-privs -- env RUSTZEN_ENV=development RUSTZEN_RUNTIME_ROOT="/var/lib/rz-agent-$identity" RUSTZEN_ADMIN_PORT=19801 RUSTZEN_MONITOR_AGENT_TOKEN="$RUSTZEN_MONITOR_AGENT_TOKEN" RUSTZEN_MONITOR_NODE_ID="linux-agent-$identity" RUSTZEN_MONITOR_CONTROLLER_URL=http://127.0.0.1:19801 RUST_LOG=warn /verify/bin/rz-monitor-agent >"/var/lib/rz-agent-$identity/logs/launch.log" 2>&1 & pids+=("$!")
done

admin=http://127.0.0.1:19801
login=$(curl --fail --silent -H 'content-type: application/json' -d '{"username":"owner","password":"rustzen@123"}' "$admin/api/auth/login")
token=$(jq -er '.data.token' <<<"$login")
auth=(-H "authorization: Bearer $token")
for _ in $(seq 1 900); do nodes=$(curl --silent "${auth[@]}" "$admin/api/monitor/nodes" || true); jq -e '.data | length == 2 and ([.[].nodeId] | sort) == ["linux-agent-a","linux-agent-b"]' <<<"$nodes" >/dev/null 2>&1 && break; sleep .1; done
jq -e '.data | length == 2 and ([.[].bootId] | unique | length) == 2' <<<"$nodes" >/dev/null
boot_a=$(jq -er '.data[] | select(.nodeId == "linux-agent-a") | .bootId' <<<"$nodes")
boot_b=$(jq -er '.data[] | select(.nodeId == "linux-agent-b") | .bootId' <<<"$nodes")
export RUSTZEN_VERIFY_BOOT_A="$boot_a" RUSTZEN_VERIFY_BOOT_B="$boot_b"
steps=$(python3 /verify/steps.py linux-agent-a linux-agent-b)
printf '%s\n' "$steps" > /verify/evidence/browser-steps.json
system=$(curl --fail --silent "${auth[@]}" -H 'content-type: application/json' -d '{"name":"Dual Agent Nodes Chromium","baseUrl":"http://127.0.0.1:19801/health","enabled":true}' "$admin/api/reports/systems" | jq -er '.data.id')
flow=$(curl --fail --silent "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg system "$system" --argjson steps "$steps" '{systemId:$system,name:"Dual Agent Nodes Chromium",steps:$steps}')" "$admin/api/reports/flows" | jq -er '.data.id')
curl --fail --silent "${auth[@]}" "$admin/api/reports/flows" > /verify/evidence/browser-flow.json
jq -e --arg flow "$flow" '[.data[] | select(.id == $flow)] | length == 1' /verify/evidence/browser-flow.json >/dev/null
run=$(curl --fail --silent "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg flow "$flow" '{flowId:$flow,input:{}}')" "$admin/api/reports/runs" | jq -er '.data.id')
for _ in $(seq 1 900); do status=$(curl --fail --silent "${auth[@]}" "$admin/api/reports/runs/$run" | jq -er '.data.status'); [ "$status" != queued ] && [ "$status" != running ] && break; sleep .1; done
if [ "$status" != succeeded ]; then
  echo "browser run failed with status=$status" >&2
  curl --fail --silent "${auth[@]}" "$admin/api/reports/runs/$run" | jq -c '.data | {id,status,error,startedAt,finishedAt}' >&2 || true
  curl --fail --silent "${auth[@]}" "$admin/api/reports/runs/$run/steps" | jq -c '[.data[] | {stepIndex,action,status,durationMs,message}]' >&2 || true
  exit 1
fi
curl --fail --silent "${auth[@]}" "$admin/api/reports/runs/$run/steps" > /verify/evidence/browser-steps-receipt.json
receipt_sha=$(sha256sum /verify/evidence/browser-steps-receipt.json | awk '{print $1}')
receipt_bytes=$(wc -c < /verify/evidence/browser-steps-receipt.json | tr -d ' ')
steps_sha=$(sha256sum /verify/evidence/browser-steps.json | awk '{print $1}')
steps_bytes=$(wc -c < /verify/evidence/browser-steps.json | tr -d ' ')
flow_sha=$(sha256sum /verify/evidence/browser-flow.json | awk '{print $1}')
flow_bytes=$(wc -c < /verify/evidence/browser-flow.json | tr -d ' ')
artifacts=$(curl --fail --silent "${auth[@]}" "$admin/api/reports/runs/$run/artifacts")
pngs='[]'
for identity in a b; do
  artifact=$(jq -er --arg name "monitor-agent-linux-agent-$identity-detail-dark-en" '.data[] | select(.fileName | startswith($name)) | .id' <<<"$artifacts")
  file_name="nodes-linux-agent-$identity.png"
  curl --fail --silent "${auth[@]}" "$admin/api/reports/runs/$run/artifacts/$artifact" > "/verify/evidence/$file_name"
  png_sha=$(sha256sum "/verify/evidence/$file_name" | awk '{print $1}')
  png_bytes=$(wc -c < "/verify/evidence/$file_name" | tr -d ' ')
  dimensions=$(file "/verify/evidence/$file_name" | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
  test "$dimensions" = '1440 x 900'
  pngs=$(jq -nc --argjson prior "$pngs" --arg file "$file_name" --arg sha "$png_sha" --argjson bytes "$png_bytes" --arg dimensions "$dimensions" '$prior + [{file:$file,sha256:$sha,bytes:$bytes,dimensions:$dimensions,viewport:{width:1440,height:900}}]')
done
chromium=$(dpkg-query -W -f='${Version}' chromium)
jq -n --arg head "$RUSTZEN_VERIFY_HEAD" --arg state "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" --arg sha "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" --arg platform "$RUSTZEN_VERIFY_PLATFORM" --arg admin "$RUSTZEN_VERIFY_ADMIN_SHA256" --arg monitor "$RUSTZEN_VERIFY_MONITOR_SHA256" --arg reports "$RUSTZEN_VERIFY_REPORTS_SHA256" --arg agent "$RUSTZEN_VERIFY_AGENT_SHA256" --arg boot_a "$boot_a" --arg boot_b "$boot_b" --arg run "$run" --arg flow "$flow" --arg chromium "$chromium" --arg receipt_sha "$receipt_sha" --arg steps_sha "$steps_sha" --arg flow_sha "$flow_sha" --argjson receipt_bytes "$receipt_bytes" --argjson steps_bytes "$steps_bytes" --argjson flow_bytes "$flow_bytes" --argjson artifacts "$pngs" '{schemaVersion:1,status:"passed",gitHead:$head,sourceTreeState:$state,sourceTreeSha256:$sha,platform:$platform,binaries:{admin:$admin,monitor:$monitor,reports:$reports,agent:$agent},nodeBootIds:[{nodeId:"linux-agent-a",bootId:$boot_a},{nodeId:"linux-agent-b",bootId:$boot_b}],browser:{chromiumVersion:$chromium,runId:$run,steps:{file:"browser-steps.json",sha256:$steps_sha,bytes:$steps_bytes},flow:{id:$flow,file:"browser-flow.json",sha256:$flow_sha,bytes:$flow_bytes},receipts:[{file:"browser-steps-receipt.json",sha256:$receipt_sha,bytes:$receipt_bytes,runId:$run}],artifacts:$artifacts},limits:["shared container kernel","no systemd PID 1","no remote-host or production TLS verification"]}' > /verify/evidence/manifest.json
