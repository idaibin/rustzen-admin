#!/usr/bin/env bash
set -euo pipefail

for command in curl jq chromium file setpriv ss; do command -v "$command" >/dev/null; done
groupadd --system rustzen
useradd --system --gid rustzen --home-dir /opt/rz --shell /usr/sbin/nologin rustzen
install -d -m 0750 -o rustzen -g rustzen /opt/rz /opt/rz/data/db /opt/rz/data/reports/db /opt/rz/logs /opt/rz/output
for name in rz-admin rz-monitor rz-insights rz-reports; do install -m 0755 "/verify/bin/$name" "/opt/rz/$name"; done
chown -R rustzen:rustzen /opt/rz
export HOME=/opt/rz XDG_CONFIG_HOME=/opt/rz/.config XDG_CACHE_HOME=/opt/rz/.cache RUSTZEN_ENV=development RUSTZEN_RUNTIME_ROOT=/opt/rz RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_ADMIN_PORT=19801 RUSTZEN_INTERNAL_HOST=127.0.0.1 RUSTZEN_MONITOR_PORT=19802 RUSTZEN_INSIGHTS_PORT=19803 RUSTZEN_REPORTS_PORT=19804 RUSTZEN_ADMIN_SQLITE_PATH=./data/db/admin.db RUSTZEN_MONITOR_SQLITE_PATH=./data/db/monitor.db RUSTZEN_INSIGHTS_SQLITE_PATH=./data/db/insights.db RUSTZEN_REPORTS_SQLITE_PATH=./data/reports/db/reports.db RUSTZEN_JWT_SECRET=task-console-jwt-secret RUSTZEN_IPC_TOKEN=task-console-ipc-secret RUSTZEN_MONITOR_AGENT_TOKEN=task-console-agent-secret RUSTZEN_REPORTS_CREDENTIAL_KEY=task-console-key RUSTZEN_REPORTS_BROWSER_PATH=/usr/bin/chromium RUSTZEN_REPORTS_MAX_CONCURRENCY=1 RUSTZEN_TIMEZONE=UTC RUSTZEN_BUILD_ID=$(printf 'a%.0s' {1..64}) RUSTZEN_COMPOSITION_ID=$(printf 'b%.0s' {1..64}) RUSTZEN_MONITOR_SCHEMA_FINGERPRINT=$(printf 'c%.0s' {1..64}) RUSTZEN_MONITOR_DATA_CONTRACT_ID=$(printf 'd%.0s' {1..64})
pids=()
cleanup() { status=$?; trap - EXIT; for pid in "${pids[@]}"; do kill -TERM "$pid" 2>/dev/null || true; done; wait "${pids[@]}" 2>/dev/null || true; exit "$status"; }
trap cleanup EXIT
as_rustzen() { setpriv --reuid=rustzen --regid=rustzen --init-groups --no-new-privs -- "$@"; }
as_rustzen /opt/rz/rz-monitor init-db
as_rustzen /opt/rz/rz-monitor bind-database
for pair in 'monitor:/opt/rz/rz-monitor controller' 'insights:/opt/rz/rz-insights serve' 'reports:/opt/rz/rz-reports serve' 'admin:/opt/rz/rz-admin serve'; do as_rustzen ${pair#*:} >"/opt/rz/logs/${pair%%:*}.log" 2>&1 & pids+=("$!"); done
for port in 19801 19802 19803 19804; do for _ in $(seq 1 150); do curl -fsS "http://127.0.0.1:$port/health" >/dev/null && break; sleep .1; done; curl -fsS "http://127.0.0.1:$port/health" >/dev/null; done
admin=http://127.0.0.1:19801
owner=$(curl -fsS -H 'content-type: application/json' -d '{"username":"owner","password":"rustzen@123"}' "$admin/api/auth/login" | jq -er '.data.token')
auth=(-H "authorization: Bearer $owner")
curl -fsS "${auth[@]}" "$admin/api/manage/tasks" > /verify/evidence/tasks.json
jq -e '.data | length == 3' /verify/evidence/tasks.json >/dev/null
key=cleanup-operation-logs-retention
menu=$(curl -fsS "${auth[@]}" "$admin/api/system/menus/options?limit=500" | jq -er '.data[] | select(.code=="manage:task:list") | .value')
curl -fsS "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --argjson menu "$menu" '{name:"Task console viewer",code:"task_console_viewer",status:1,menuIds:[$menu]}')" "$admin/api/system/roles" >/dev/null
role=$(curl -fsS "${auth[@]}" "$admin/api/system/roles/options?limit=500" | jq -er '.data[] | select(.code=="task_console_viewer") | .value')
curl -fsS "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --argjson role "$role" '{username:"task_viewer",email:"task-viewer@example.test",password:"task-viewer-password",realName:"task viewer",status:1,roleIds:[$role]}')" "$admin/api/system/users" >/dev/null
viewer=$(curl -fsS -H 'content-type: application/json' -d '{"username":"task_viewer","password":"task-viewer-password"}' "$admin/api/auth/login" | jq -er '.data.token')
viewer_auth=(-H "authorization: Bearer $viewer")
curl -fsS "${viewer_auth[@]}" "$admin/api/manage/tasks" > /verify/evidence/list-only-tasks.json
jq -e '.data | length == 3' /verify/evidence/list-only-tasks.json >/dev/null
curl -fsS "${viewer_auth[@]}" "$admin/api/manage/tasks/$key/runs?current=1&pageSize=50" > /verify/evidence/list-only-runs-before.json
curl -sS "${viewer_auth[@]}" -o /verify/evidence/list-only-post.json -w '%{http_code}' -X POST "$admin/api/manage/tasks/$key/run" > /verify/evidence/list-only-status.txt
test "$(cat /verify/evidence/list-only-status.txt)" = 403
curl -fsS "${viewer_auth[@]}" "$admin/api/manage/tasks/$key/runs?current=1&pageSize=50" > /verify/evidence/list-only-runs-after.json
jq -e '(.total == $after[0].total) and ([.data[] | {id,status}] == [$after[0].data[] | {id,status}])' /verify/evidence/list-only-runs-before.json --slurpfile after /verify/evidence/list-only-runs-after.json >/dev/null

create_system() { curl -fsS "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg name "$1" --arg base "${2:-http://127.0.0.1:19801/health}" '{name:$name,baseUrl:$base,enabled:true}')" "$admin/api/reports/systems" | jq -er '.data.id'; }
run_browser() {
  local name=$1 steps=$2 flow run status
  flow=$(curl -fsS "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg system "$browser_system" --arg name "$name" --argjson steps "$steps" '{systemId:$system,name:$name,steps:$steps}')" "$admin/api/reports/flows" | jq -er '.data.id')
  run=$(curl -fsS "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg flow "$flow" '{flowId:$flow,input:{}}')" "$admin/api/reports/runs" | jq -er '.data.id')
  for _ in $(seq 1 900); do status=$(curl -fsS "${auth[@]}" "$admin/api/reports/runs/$run" | jq -er '.data.status'); [ "$status" = succeeded ] && break; case "$status" in failed|cancelled) exit 1;; esac; sleep .1; done
  test "$status" = succeeded
  curl -fsS "${auth[@]}" "$admin/api/reports/runs/$run" > "/verify/evidence/browser-$name-run.json"
  curl -fsS "${auth[@]}" "$admin/api/reports/runs/$run/steps" > "/verify/evidence/browser-$name-steps.json"
  curl -fsS "${auth[@]}" "$admin/api/reports/runs/$run/artifacts" > "/verify/evidence/browser-$name-artifacts.json"
  jq -e --arg run "$run" '.data.id == $run and .data.status == "succeeded"' "/verify/evidence/browser-$name-run.json" >/dev/null
  jq -e --arg run "$run" '(.data | length > 0) and all(.data[]; .runId == $run and .status == "succeeded")' "/verify/evidence/browser-$name-steps.json" >/dev/null
  printf '%s\n' "$run"
}
download_png() {
  local run=$1 prefix=$2 output=$3 dimensions hash artifact
  artifact=$(curl -fsS "${auth[@]}" "$admin/api/reports/runs/$run/artifacts" | jq -er --arg prefix "$prefix" '.data[] | select(.fileName | startswith($prefix)) | .id')
  curl -fsS "${auth[@]}" "$admin/api/reports/runs/$run/artifacts/$artifact" > "/verify/evidence/$output"
  test "$(head -c 8 "/verify/evidence/$output" | od -An -tx1 | tr -d ' \n')" = 89504e470d0a1a0a
  dimensions=$(file "/verify/evidence/$output" | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
  test "$dimensions" = "$4"; hash=$(sha256sum "/verify/evidence/$output" | awk '{print $1}')
  jq -nc --arg file "$output" --arg sha "$hash" --arg dimensions "$dimensions" --argjson width "$5" --argjson height "$6" '{file:$file,sha256:$sha,dimensions:$dimensions,viewport:{width:$width,height:$height}}'
}
owner_steps=$(jq -nc --arg key "$key" '[{action:"setUiPreferences",theme:"dark",locale:"en-US"},{action:"setViewport",width:1440,height:900},{action:"goto",url:"/login"},{action:"waitFor",selector:"#login_username"},{action:"fill",selector:"#login_username",value:"owner"},{action:"fill",selector:"#login_password",value:"rustzen@123"},{action:"click",selector:"button[type=submit]"},{action:"waitFor",selector:".shell-content"},{action:"goto",url:"/manage/task"},{action:"waitFor",selector:"[data-testid=maintenance-task-table]"},{action:"assertText",selector:"[data-testid=maintenance-task-table]",text:"Clean operation logs"},{action:"assertText",selector:"[data-testid=maintenance-task-table]",text:"Clean task run records"},{action:"assertText",selector:"[data-testid=maintenance-task-table]",text:"SQLite storage maintenance"},{action:"click",selector:"[data-testid=maintenance-task-run-\($key)]"},{action:"waitFor",selector:"[data-testid=maintenance-task-run-dialog-\($key)]"},{action:"click",selector:"[data-testid=maintenance-task-run-dialog-\($key)] .ant-btn-primary"},{action:"waitFor",selector:"[data-testid=maintenance-task-status-\($key)][data-status=running]"},{action:"waitFor",selector:"[data-testid=maintenance-task-status-\($key)][data-status=success]"},{action:"assertText",selector:"[data-testid=maintenance-task-status-\($key)][data-status=success]",text:"Success"},{action:"assertNoHorizontalOverflow"},{action:"screenshotViewport",name:"task-console-desktop"}]')
viewer_steps=$(jq -nc '[{action:"setUiPreferences",theme:"light",locale:"zh-CN"},{action:"setViewport",width:390,height:844},{action:"goto",url:"/login"},{action:"waitFor",selector:"#login_username"},{action:"fill",selector:"#login_username",value:"task_viewer"},{action:"fill",selector:"#login_password",value:"task-viewer-password"},{action:"click",selector:"button[type=submit]"},{action:"waitFor",selector:".shell-content"},{action:"goto",url:"/manage/task"},{action:"waitFor",selector:"[data-testid=maintenance-task-table]"},{action:"assertAbsent",selector:"[data-testid=maintenance-task-run-cleanup-operation-logs-retention]"},{action:"assertNoHorizontalOverflow"},{action:"screenshotViewport",name:"task-console-mobile"}]')
RUSTZEN_VERIFY_FAULT_METHOD=GET RUSTZEN_VERIFY_FAULT_MODE=task-transition RUSTZEN_VERIFY_FAULT_ROUTE=/api/manage/tasks RUSTZEN_VERIFY_FAULT_RECEIPT=/verify/evidence/task-transition-receipt.json python3 /verify/fault-proxy.py >/opt/rz/logs/task-transition-proxy.log 2>&1 &
transition_proxy=$!; pids+=("$transition_proxy")
for _ in $(seq 1 50); do curl -fsS http://127.0.0.1:19805/__verify_proxy_health >/dev/null && break; sleep .1; done
curl -fsS http://127.0.0.1:19805/__verify_proxy_health >/dev/null
browser_system=$(create_system 'Task console transition' http://127.0.0.1:19805/health)
owner_run=$(run_browser owner "$owner_steps")
curl -fsS "${auth[@]}" "$admin/api/manage/tasks/$key/runs?current=1&pageSize=50" > /verify/evidence/owner-runs-after.json
manual_run=$(jq -er '.data[] | select(.triggerType == "manual" and .status == "success") | .id' /verify/evidence/owner-runs-after.json | head -1)
record_steps=$(jq -nc --arg key "$key" --arg run "$manual_run" '[{action:"setUiPreferences",theme:"dark",locale:"en-US"},{action:"setViewport",width:1440,height:900},{action:"goto",url:"/login"},{action:"waitFor",selector:"#login_username"},{action:"fill",selector:"#login_username",value:"owner"},{action:"fill",selector:"#login_password",value:"rustzen@123"},{action:"click",selector:"button[type=submit]"},{action:"waitFor",selector:".shell-content"},{action:"goto",url:"/manage/task"},{action:"waitFor",selector:"[data-testid=maintenance-task-records-\($key)]"},{action:"click",selector:"[data-testid=maintenance-task-records-\($key)]"},{action:"waitFor",selector:"[data-testid=maintenance-task-records-dialog-\($key)]"},{action:"waitFor",selector:"[data-testid=maintenance-task-run-status-\($run)]"},{action:"assertText",selector:"[data-testid=maintenance-task-run-status-\($run)]",text:"Success"},{action:"assertNoHorizontalOverflow"}]')
record_run=$(run_browser owner-record "$record_steps")
kill -TERM "$transition_proxy"; wait "$transition_proxy" || true; unset 'pids[${#pids[@]}-1]'
jq -e --arg run "$manual_run" '.mode == "task-transition" and (.transitionRunId | tostring) == $run and .transitionReads >= 2 and .hitCount >= 3' /verify/evidence/task-transition-receipt.json >/dev/null
browser_system=$(create_system 'Task console viewer')
viewer_run=$(run_browser viewer "$viewer_steps")
run_fault_browser() {
  local mode=$1 expected=$2 name=$3 steps run proxy
  RUSTZEN_VERIFY_FAULT_METHOD=GET RUSTZEN_VERIFY_FAULT_MODE="$mode" RUSTZEN_VERIFY_FAULT_ROUTE=/api/manage/tasks RUSTZEN_VERIFY_FAULT_RECEIPT="/verify/evidence/$name-receipt.json" python3 /verify/fault-proxy.py >/opt/rz/logs/$name-proxy.log 2>&1 &
  proxy=$!; pids+=("$proxy")
  for _ in $(seq 1 50); do curl -fsS http://127.0.0.1:19805/__verify_proxy_health >/dev/null && break; sleep .1; done
  curl -fsS http://127.0.0.1:19805/__verify_proxy_health >/dev/null
  browser_system=$(curl -fsS "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg name "$name" '{name:$name,baseUrl:"http://127.0.0.1:19805/health",enabled:true}')" "$admin/api/reports/systems" | jq -er '.data.id')
  steps=$(jq -nc --arg expected "$expected" '[{action:"setUiPreferences",theme:"light",locale:"en-US"},{action:"setViewport",width:1440,height:900},{action:"goto",url:"/login"},{action:"waitFor",selector:"#login_username"},{action:"fill",selector:"#login_username",value:"owner"},{action:"fill",selector:"#login_password",value:"rustzen@123"},{action:"click",selector:"button[type=submit]"},{action:"waitFor",selector:".shell-content"},{action:"goto",url:"/manage/task"},{action:"waitFor",selector:".shell-content"},{action:"assertText",selector:".shell-content",text:$expected},{action:"assertNoHorizontalOverflow"}]')
  run=$(run_browser "$name" "$steps")
  if [ "$mode" = empty ]; then
    curl -fsS http://127.0.0.1:19805/api/manage/tasks > "/verify/evidence/$name-response.json"
    jq -e '.code == 0 and .message == "Success" and .data == [] and .total == 0' "/verify/evidence/$name-response.json" >/dev/null
  fi
  kill -TERM "$proxy"; wait "$proxy" || true; unset 'pids[${#pids[@]}-1]'
  jq -e '.hitCount >= 1' "/verify/evidence/$name-receipt.json" >/dev/null
  printf '%s\n' "$run"
}
empty_run=$(run_fault_browser empty 'No scheduled tasks' empty)
error_run=$(run_fault_browser http 'Failed to load tasks' error)
desktop=$(download_png "$owner_run" task-console-desktop task-console-desktop.png '1440 x 900' 1440 900)
mobile=$(download_png "$viewer_run" task-console-mobile task-console-mobile.png '390 x 844' 390 844)
binaries=$(for name in rz-admin rz-monitor rz-insights rz-reports; do jq -nc --arg name "$name" --arg sha "$(sha256sum "/opt/rz/$name" | awk '{print $1}')" '{name:$name,sha256:$sha}'; done | jq -s .)
receipts=$(for file in tasks.json list-only-tasks.json list-only-runs-before.json list-only-post.json list-only-status.txt list-only-runs-after.json owner-runs-after.json task-transition-receipt.json empty-receipt.json empty-response.json error-receipt.json browser-owner-run.json browser-owner-steps.json browser-owner-artifacts.json browser-owner-record-run.json browser-owner-record-steps.json browser-owner-record-artifacts.json browser-viewer-run.json browser-viewer-steps.json browser-viewer-artifacts.json browser-empty-run.json browser-empty-steps.json browser-empty-artifacts.json browser-error-run.json browser-error-steps.json browser-error-artifacts.json; do jq -nc --arg file "$file" --arg sha "$(sha256sum "/verify/evidence/$file" | awk '{print $1}')" --argjson bytes "$(wc -c < "/verify/evidence/$file")" '{file:$file,sha256:$sha,bytes:$bytes}'; done | jq -s .)
jq -nc --arg head "$RUSTZEN_VERIFY_HEAD" --arg state "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" --arg sha "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" --arg platform "$RUSTZEN_VERIFY_PLATFORM" --arg provenance "$RUSTZEN_VERIFY_BUILD_PROVENANCE_SHA256" --arg owner "$owner_run" --arg record "$record_run" --arg manual "$manual_run" --arg viewer "$viewer_run" --arg empty "$empty_run" --arg error "$error_run" --argjson binaries "$binaries" --argjson desktop "$desktop" --argjson mobile "$mobile" --argjson receipts "$receipts" '{schemaVersion:2,status:"passed",gitHead:$head,sourceTreeState:$state,sourceTreeSha256:$sha,platform:$platform,buildProvenanceSha256:$provenance,binaries:$binaries,api:{taskCount:3,manualRun:{id:$manual,status:"success"},listOnlyPost:403},browser:{ownerRun:$owner,recordRun:$record,viewerRun:$viewer,emptyRun:$empty,errorRun:$error},receipts:$receipts,screenshots:[$desktop,$mobile]}' > /verify/evidence/manifest.json
