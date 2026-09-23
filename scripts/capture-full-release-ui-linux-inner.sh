#!/usr/bin/env bash
# Disposable, detached-container evidence producer for the README preview.
set -euo pipefail
trap 'echo "full preview inner failed at line $LINENO" >&2' ERR

for command in chromium curl jq file setpriv sha256sum ss groupadd useradd; do command -v "$command" >/dev/null; done
for port in 19801 19802 19803 19804; do ! ss -H -ltn "sport = :$port" | grep -q . || { echo "port occupied: $port" >&2; exit 1; }; done

groupadd -g 2510 rz-preview
useradd -u 2510 -g rz-preview -M -s /usr/sbin/nologin rz-preview
for identity in a b; do case "$identity" in a) uid=2511;; b) uid=2512;; esac; groupadd -g "$uid" "rz-preview-agent-$identity"; useradd -u "$uid" -g "rz-preview-agent-$identity" -M -s /usr/sbin/nologin "rz-preview-agent-$identity"; done
install -d -m 0750 -o rz-preview -g rz-preview /opt/rz/data/db /opt/rz/data/db/admin /opt/rz/data/reports/db /opt/rz/logs /opt/rz/output
chown rz-preview:rz-preview /opt/rz /opt/rz/data /opt/rz/data/db /opt/rz/data/db/admin /opt/rz/data/reports /opt/rz/data/reports/db /opt/rz/logs /opt/rz/output
owner_password=full-preview-owner-password
printf '%s\n' "$owner_password" > /opt/rz/data/db/admin/bootstrap-owner-password
chown rz-preview:rz-preview /opt/rz/data/db/admin/bootstrap-owner-password
for name in rz-admin rz-monitor rz-insights rz-reports; do install -m 0755 "/verify/bin/$name" "/opt/rz/$name"; chown rz-preview:rz-preview "/opt/rz/$name"; done
for identity in a b; do install -d -m 0750 -o "rz-preview-agent-$identity" -g "rz-preview-agent-$identity" "/var/lib/rz-preview-agent-$identity/logs"; done

export HOME=/opt/rz XDG_CONFIG_HOME=/opt/rz/.config XDG_CACHE_HOME=/opt/rz/.cache
export RUSTZEN_ENV=development RUSTZEN_RUNTIME_ROOT=/opt/rz RUSTZEN_INTERNAL_HOST=127.0.0.1 RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_ADMIN_PORT=19801 RUSTZEN_MONITOR_PORT=19802 RUSTZEN_INSIGHTS_PORT=19803 RUSTZEN_REPORTS_PORT=19804
export RUSTZEN_ADMIN_SQLITE_PATH=./data/db/admin.db RUSTZEN_MONITOR_SQLITE_PATH=./data/db/monitor.db RUSTZEN_INSIGHTS_SQLITE_PATH=./data/db/insights.db RUSTZEN_REPORTS_SQLITE_PATH=./data/reports/db/reports.db
export RUSTZEN_JWT_SECRET=full-preview-jwt-secret RUSTZEN_IPC_TOKEN=full-preview-ipc-secret RUSTZEN_MONITOR_AGENT_TOKEN=full-preview-agent-secret RUSTZEN_REPORTS_CREDENTIAL_KEY=full-preview-reports-secret RUSTZEN_REPORTS_BROWSER_PATH=/usr/bin/chromium RUSTZEN_REPORTS_MAX_CONCURRENCY=1 RUSTZEN_TIMEZONE=UTC RUST_LOG=warn
export RUSTZEN_BUILD_ID=$(printf 'a%.0s' {1..64}) RUSTZEN_COMPOSITION_ID=$(printf 'b%.0s' {1..64})

pids=()
cleanup(){ status=$?; for pid in "${pids[@]}"; do kill -TERM "$pid" 2>/dev/null || true; done; for pid in "${pids[@]}"; do wait "$pid" 2>/dev/null || true; done; exit "$status"; }
trap cleanup EXIT INT TERM
as_preview(){ setpriv --reuid=rz-preview --regid=rz-preview --init-groups --no-new-privs -- env HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" XDG_CACHE_HOME="$XDG_CACHE_HOME" RUSTZEN_ENV="$RUSTZEN_ENV" RUSTZEN_RUNTIME_ROOT="$RUSTZEN_RUNTIME_ROOT" RUSTZEN_INTERNAL_HOST="$RUSTZEN_INTERNAL_HOST" RUSTZEN_ADMIN_HOST="$RUSTZEN_ADMIN_HOST" RUSTZEN_ADMIN_PORT="$RUSTZEN_ADMIN_PORT" RUSTZEN_MONITOR_PORT="$RUSTZEN_MONITOR_PORT" RUSTZEN_INSIGHTS_PORT="$RUSTZEN_INSIGHTS_PORT" RUSTZEN_REPORTS_PORT="$RUSTZEN_REPORTS_PORT" RUSTZEN_ADMIN_SQLITE_PATH="$RUSTZEN_ADMIN_SQLITE_PATH" RUSTZEN_MONITOR_SQLITE_PATH="$RUSTZEN_MONITOR_SQLITE_PATH" RUSTZEN_INSIGHTS_SQLITE_PATH="$RUSTZEN_INSIGHTS_SQLITE_PATH" RUSTZEN_REPORTS_SQLITE_PATH="$RUSTZEN_REPORTS_SQLITE_PATH" RUSTZEN_JWT_SECRET="$RUSTZEN_JWT_SECRET" RUSTZEN_IPC_TOKEN="$RUSTZEN_IPC_TOKEN" RUSTZEN_MONITOR_AGENT_TOKEN="$RUSTZEN_MONITOR_AGENT_TOKEN" RUSTZEN_REPORTS_CREDENTIAL_KEY="$RUSTZEN_REPORTS_CREDENTIAL_KEY" RUSTZEN_REPORTS_BROWSER_PATH="$RUSTZEN_REPORTS_BROWSER_PATH" RUSTZEN_REPORTS_MAX_CONCURRENCY="$RUSTZEN_REPORTS_MAX_CONCURRENCY" RUSTZEN_BUILD_ID="$RUSTZEN_BUILD_ID" RUSTZEN_COMPOSITION_ID="$RUSTZEN_COMPOSITION_ID" RUSTZEN_TIMEZONE=UTC RUST_LOG=warn "$@"; }
for pair in 'monitor:/opt/rz/rz-monitor controller' 'insights:/opt/rz/rz-insights serve' 'reports:/opt/rz/rz-reports serve' 'admin:/opt/rz/rz-admin serve'; do name=${pair%%:*}; command=${pair#*:}; as_preview $command >"/opt/rz/logs/$name.log" 2>&1 & pids+=("$!"); done
wait_health(){ for url in "$@"; do for _ in $(seq 1 300); do curl --silent --fail --connect-timeout 2 --max-time 3 "$url" >/dev/null 2>&1 && break; sleep .1; done; curl --silent --fail --connect-timeout 2 --max-time 3 "$url" >/dev/null; done; }
wait_health http://127.0.0.1:19801/health http://127.0.0.1:19802/health http://127.0.0.1:19803/health http://127.0.0.1:19804/health
for identity in a b; do setpriv --reuid="rz-preview-agent-$identity" --regid="rz-preview-agent-$identity" --init-groups --no-new-privs -- env RUSTZEN_ENV=development RUSTZEN_RUNTIME_ROOT="/var/lib/rz-preview-agent-$identity" RUSTZEN_ADMIN_PORT=19801 RUSTZEN_MONITOR_AGENT_TOKEN="$RUSTZEN_MONITOR_AGENT_TOKEN" RUSTZEN_MONITOR_NODE_ID="preview-node-$identity" RUSTZEN_MONITOR_CONTROLLER_URL=http://127.0.0.1:19801 RUST_LOG=warn /verify/bin/rz-monitor-agent >"/var/lib/rz-preview-agent-$identity/logs/launch.log" 2>&1 & pids+=("$!"); done

admin=http://127.0.0.1:19801
login=$(curl --fail --silent -H 'content-type: application/json' -d "$(jq -nc --arg password "$owner_password" '{username:"owner",password:$password}')" "$admin/api/auth/login")
token=$(jq -er '.data.token' <<<"$login"); auth=(-H "authorization: Bearer $token")
for _ in $(seq 1 600); do nodes=$(curl --silent "${auth[@]}" "$admin/api/monitor/nodes" || true); jq -e '.data|length==2 and ([.[].nodeId]|sort)==["preview-node-a","preview-node-b"]' <<<"$nodes" >/dev/null 2>&1 && break; sleep .1; done
jq -e '(.data|length)==2 and ([.data[] | .status] | all(. == "online"))' <<<"$nodes" >/dev/null
curl --fail --silent -X PUT "${auth[@]}" -H 'content-type: application/json' -d '{"collectionEnabled":true,"projectKey":"preview-project-key","allowedOrigins":["https://preview.example"]}' "$admin/api/insights/collection-policy" > /verify/evidence/insights-policy.json
curl --fail --silent -H 'origin: https://preview.example' -H 'x-rustzen-project-key: preview-project-key' -H 'content-type: application/json' -d '[{"eventName":"page_view","visitorId":"preview-visitor-001","pagePath":"/release-preview"}]' "$admin/api/insights/track" > /verify/evidence/insights-track.json
events=$(curl --fail --silent "${auth[@]}" "$admin/api/insights/events")
jq -e '.data.data | any(.visitorId=="preview-visitor-001" and .pagePath=="/release-preview")' <<<"$events" >/dev/null
printf '%s\n' "$events" > /verify/evidence/insights-events.json

reports_system_status=$(curl --silent --show-error -D /verify/evidence/reports-system.headers -o /verify/evidence/reports-system.json -w '%{http_code}' "${auth[@]}" -H 'content-type: application/json' -d '{"name":"Full release preview","baseUrl":"http://127.0.0.1:19801/health","enabled":true}' "$admin/api/reports/systems")
test "$reports_system_status" = 200
system=$(jq -er '.data.id' /verify/evidence/reports-system.json)
reports_flow=$(curl --fail --silent "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg system "$system" '{systemId:$system,name:"Release preview automatic task",steps:[{action:"goto",url:"/health"},{action:"assertText",selector:"body",text:"ok"}]}')" "$admin/api/reports/flows" | jq -er '.data.id')
schedule=$(curl --fail --silent "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg flow "$reports_flow" '{flowId:$flow,cadence:"daily",dueTime:"10:15",input:{},description:"Release preview automatic report",enabled:true}')" "$admin/api/reports/schedules")
jq -e '.data.enabled==true and .data.cadence=="daily"' <<<"$schedule" >/dev/null; printf '%s\n' "$schedule" > /verify/evidence/automatic-schedule.json
python3 /verify/browser.py "$admin" "$owner_password" /verify/evidence
shots=$(jq -c '.screenshots' /verify/evidence/browser-direct.json)
browser_assertions=$(jq -c '.domAssertions' /verify/evidence/browser-direct.json)
pages=$(jq -c '.pages' /verify/evidence/browser-direct.json)
curl --fail --silent "${auth[@]}" "$admin/api/monitor/nodes" > /verify/evidence/nodes.json
jq -e '(.data|length)==2 and ([.data[] | .status] | all(. == "online"))' /verify/evidence/nodes.json >/dev/null
for name in rz-admin rz-monitor rz-insights rz-reports rz-monitor-agent; do sha256sum "/verify/bin/$name"; done > /verify/evidence/binary-sha256.txt
health=$(for port in 19801 19802 19803 19804; do curl --fail --silent "http://127.0.0.1:$port/health"; done | jq -sc .)
jq -n --arg head "$RUSTZEN_VERIFY_HEAD" --arg state "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" --arg digest "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" --arg platform "$RUSTZEN_VERIFY_PLATFORM" --argjson nodes "$nodes" --argjson schedule "$schedule" --argjson events "$events" --argjson health "$health" --arg chromium "$(chromium --version)" --argjson screenshots "$shots" --argjson pages "$pages" --argjson browserAssertions "$browser_assertions" '{schemaVersion:1,status:"passed",gitHead:$head,sourceTreeState:$state,sourceTreeSha256:$digest,platform:$platform,chromium:$chromium,viewport:{width:1920,height:1080},runtime:{health:$health,nodes:$nodes.data,automaticSchedule:$schedule.data,visitorEventCount:$events.data.total},pages:$pages,browserAssertions:$browserAssertions,screenshots:$screenshots,binaryHashesFile:"binary-sha256.txt",limits:["Disposable Colima container","No production deployment"]}' > /verify/evidence/manifest.json
