#!/usr/bin/env bash
set -euo pipefail
for command in chromium curl jq file setpriv ss python3
do command -v "$command" >/dev/null
done
test "$(dpkg-query -W -f='${Version}' chromium)" = "${RUSTZEN_VERIFY_CHROMIUM_VERSION:?}"
for port in 19801 19802 19803 19804 19805
do ! ss -H -ltn "sport = :$port" | grep -q . || { echo "verification port occupied: $port" >&2
exit 1
}
done
groupadd --system rustzen
useradd --system --gid rustzen --home-dir /opt/rz --shell /usr/sbin/nologin rustzen
install -d -m 0750 -o rustzen -g rustzen /opt/rz/data/db /opt/rz/data/reports/db /opt/rz/logs /opt/rz/output
for name in rz-admin rz-monitor rz-insights rz-reports
do install -m 0755 "/verify/bin/$name" "/opt/rz/$name"
done
chown -R rustzen:rustzen /opt/rz
export HOME=/opt/rz XDG_CONFIG_HOME=/opt/rz/.config XDG_CACHE_HOME=/opt/rz/.cache RUSTZEN_ENV=development RUSTZEN_RUNTIME_ROOT=/opt/rz RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_ADMIN_PORT=19801 RUSTZEN_INTERNAL_HOST=127.0.0.1 RUSTZEN_MONITOR_PORT=19802 RUSTZEN_INSIGHTS_PORT=19803 RUSTZEN_REPORTS_PORT=19804 RUSTZEN_ADMIN_SQLITE_PATH=./data/db/admin.db RUSTZEN_MONITOR_SQLITE_PATH=./data/db/monitor.db RUSTZEN_INSIGHTS_SQLITE_PATH=./data/db/insights.db RUSTZEN_REPORTS_SQLITE_PATH=./data/reports/db/reports.db RUSTZEN_JWT_SECRET=schedule-form-jwt-secret RUSTZEN_IPC_TOKEN=schedule-form-ipc-secret RUSTZEN_MONITOR_AGENT_TOKEN=schedule-form-agent-secret RUSTZEN_REPORTS_CREDENTIAL_KEY=schedule-form-key RUSTZEN_REPORTS_BROWSER_PATH=/usr/bin/chromium RUSTZEN_REPORTS_MAX_CONCURRENCY=1 RUSTZEN_TIMEZONE=UTC RUSTZEN_BUILD_ID=$(printf 'a%.0s' {1..64}) RUSTZEN_COMPOSITION_ID=$(printf 'b%.0s' {1..64}) RUSTZEN_MONITOR_SCHEMA_FINGERPRINT=$(printf 'c%.0s' {1..64}) RUSTZEN_MONITOR_DATA_CONTRACT_ID=$(printf 'd%.0s' {1..64}) RUST_LOG=warn
pids=()
cleanup() { result=$?
trap - EXIT INT TERM
for pid in "${pids[@]}"
do kill -TERM "$pid" 2>/dev/null || true
done
wait "${pids[@]}" 2>/dev/null || true
if [ "$result" -ne 0 ]; then
  for log in /opt/rz/logs/{admin,monitor,insights,reports,proxy}.log; do
    [ -s "$log" ] || continue
    echo "== $log ==" >&2
    tail -n 40 "$log" >&2
  done
fi
exit "$result"
}
on_interrupt() { exit 130
}
on_terminate() { exit 143
}
trap cleanup EXIT
trap on_interrupt INT
trap on_terminate TERM
as_rustzen() { setpriv --reuid=rustzen --regid=rustzen --init-groups --no-new-privs -- env HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" XDG_CACHE_HOME="$XDG_CACHE_HOME" RUSTZEN_ENV="$RUSTZEN_ENV" RUSTZEN_RUNTIME_ROOT="$RUSTZEN_RUNTIME_ROOT" RUSTZEN_ADMIN_HOST="$RUSTZEN_ADMIN_HOST" RUSTZEN_ADMIN_PORT="$RUSTZEN_ADMIN_PORT" RUSTZEN_INTERNAL_HOST="$RUSTZEN_INTERNAL_HOST" RUSTZEN_MONITOR_PORT="$RUSTZEN_MONITOR_PORT" RUSTZEN_INSIGHTS_PORT="$RUSTZEN_INSIGHTS_PORT" RUSTZEN_REPORTS_PORT="$RUSTZEN_REPORTS_PORT" RUSTZEN_ADMIN_SQLITE_PATH="$RUSTZEN_ADMIN_SQLITE_PATH" RUSTZEN_MONITOR_SQLITE_PATH="$RUSTZEN_MONITOR_SQLITE_PATH" RUSTZEN_INSIGHTS_SQLITE_PATH="$RUSTZEN_INSIGHTS_SQLITE_PATH" RUSTZEN_REPORTS_SQLITE_PATH="$RUSTZEN_REPORTS_SQLITE_PATH" RUSTZEN_JWT_SECRET="$RUSTZEN_JWT_SECRET" RUSTZEN_IPC_TOKEN="$RUSTZEN_IPC_TOKEN" RUSTZEN_MONITOR_AGENT_TOKEN="$RUSTZEN_MONITOR_AGENT_TOKEN" RUSTZEN_REPORTS_CREDENTIAL_KEY="$RUSTZEN_REPORTS_CREDENTIAL_KEY" RUSTZEN_REPORTS_BROWSER_PATH="$RUSTZEN_REPORTS_BROWSER_PATH" RUSTZEN_REPORTS_MAX_CONCURRENCY=1 RUSTZEN_BUILD_ID="$RUSTZEN_BUILD_ID" RUSTZEN_COMPOSITION_ID="$RUSTZEN_COMPOSITION_ID" RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_MONITOR_SCHEMA_FINGERPRINT" RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_MONITOR_DATA_CONTRACT_ID" RUSTZEN_TIMEZONE=UTC RUST_LOG=warn "$@"
}
as_rustzen /opt/rz/rz-monitor init-db
as_rustzen /opt/rz/rz-monitor bind-database
for pair in 'monitor:/opt/rz/rz-monitor controller' 'insights:/opt/rz/rz-insights serve' 'reports:/opt/rz/rz-reports serve' 'admin:/opt/rz/rz-admin serve'
do name=${pair%%:*}
command=${pair#*:}
as_rustzen $command >"/opt/rz/logs/$name.log" 2>&1 & pids+=("$!")
done
curl_json() { curl --fail --silent --show-error --connect-timeout 3 --max-time 15 "$@"
}
for port in 19801 19802 19803 19804
do for retry in $(seq 1 150)
do curl_json "http://127.0.0.1:$port/health" >/dev/null 2>&1 && break
sleep .1
done
curl_json "http://127.0.0.1:$port/health" >/dev/null
done
admin=http://127.0.0.1:19801
login=$(curl_json -H 'content-type: application/json' -d '{"username":"owner","password":"rustzen@123"}' "$admin/api/auth/login")
token=$(jq -er '.data.token' <<<"$login")
auth=(-H "authorization: Bearer $token")
start_proxy() { receipt=$1
RUSTZEN_VERIFY_FAULT_METHOD=POST RUSTZEN_VERIFY_FAULT_MODE=count RUSTZEN_VERIFY_FAULT_ROUTE=/api/reports/schedules RUSTZEN_VERIFY_FAULT_RECEIPT="$receipt" python3 /verify/fault-proxy.py >/opt/rz/logs/proxy.log 2>&1 & proxy_pid=$!
pids+=("$proxy_pid")
for _ in $(seq 1 50)
do curl_json http://127.0.0.1:19805/__verify_proxy_health >/dev/null 2>&1 && return
sleep .1
done
exit 1
}
stop_proxy() { kill -TERM "$proxy_pid"
wait "$proxy_pid" || true
unset 'pids[${#pids[@]}-1]'
}
create_system() { curl_json "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg name "$1" --arg url "$2" '{name:$name,baseUrl:$url,enabled:true}')" "$admin/api/reports/systems" | jq -er '.data.id'
}
proxy_system=$(create_system 'SR UI proxy' http://127.0.0.1:19805/health)
direct_system=$(create_system 'SR UI target' http://127.0.0.1:19801/health)
case_diagnostics() {
  local case_name=$1 run_id=$2
  echo "== schedule form browser case failed: $case_name ($run_id) ==" >&2
  curl_json "${auth[@]}" "$admin/api/reports/runs/$run_id" | jq -c '.data | {id,status,error,createdAt,startedAt,finishedAt}' >&2 || true
  curl_json "${auth[@]}" "$admin/api/reports/runs/$run_id/steps" | jq -c '.data[] | {stepIndex,action,status,message,durationMs}' >&2 || true
  curl_json "${auth[@]}" "$admin/api/reports/runs/$run_id/artifacts" | jq -c '.data[] | {id,kind,fileName,createdAt}' >&2 || true
}

run_case() {
  local case_name=$1 system_id=$2 case_steps=$3 body flow_id run_id case_status
  body=$(jq -nc --arg system "$system_id" --arg name "SR UI browser" --argjson steps "$case_steps" '{systemId:$system,name:$name,steps:$steps}')
  flow_id=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$body" "$admin/api/reports/flows" | jq -er '.data.id')
  run_id=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg flow "$flow_id" '{flowId:$flow,input:{}}')" "$admin/api/reports/runs" | jq -er '.data.id')
  for _ in $(seq 1 900); do
    case_status=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$run_id" | jq -er '.data.status')
    case "$case_status" in
      queued|running) sleep .1 ;;
      *) break ;;
    esac
  done
  [ "$case_status" = succeeded ] || { case_diagnostics "$case_name" "$run_id"; exit 1; }
  echo "$run_id"
}
steps=$(cat /verify/evidence/browser-steps.json)
count_rows() { curl_json "${auth[@]}" "$admin/api/reports/schedules" | jq '.data|length'
}
start_proxy /verify/evidence/local.receipt.json
before=$(count_rows)
run_case malformed-input "$proxy_system" "$(jq -c '.malformedInput' <<<"$steps")" >/dev/null
run_case missing-time "$proxy_system" "$(jq -c '.missingTime' <<<"$steps")" >/dev/null
after=$(count_rows)
stop_proxy
local_posts=$(jq -er '.hitCount' /verify/evidence/local.receipt.json)
[ "$before" = "$after" ] && [ "$local_posts" = 0 ]
start_proxy /verify/evidence/secret.receipt.json
secret_before=$(count_rows)
run_case secret-rejected "$proxy_system" "$(jq -c '.secretRejected' <<<"$steps")" >/dev/null
secret_after=$(count_rows)
stop_proxy
secret_posts=$(jq -er '.hitCount' /verify/evidence/secret.receipt.json)
[ "$secret_before" = "$secret_after" ] && [ "$secret_posts" = 1 ]
cancel_run=$(run_case cancel-create "$direct_system" "$(jq -c '.cancelCreate' <<<"$steps")")
daily_run=$(run_case create-daily "$direct_system" "$(jq -c '.createDaily' <<<"$steps")")
weekly_run=$(run_case edit-weekly "$direct_system" "$(jq -c '.editWeekly' <<<"$steps")")
schedule=$(curl_json "${auth[@]}" "$admin/api/reports/schedules" | jq -c '.data[] | select(.description == "sr-ui-002 daily")')
jq -e '.cadence == "weekly" and .weekday == 0 and .dueTime == "10:16" and .timezone == "UTC"' <<<"$schedule" >/dev/null
menu=$(curl_json "${auth[@]}" "$admin/api/system/menus/options?limit=500" | jq -er '.data[]|select(.code=="reports:schedule:view")|.value')
curl_json "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --argjson menu "$menu" '{name:"SR UI viewer",code:"sr_ui_viewer",status:1,menuIds:[$menu]}')" "$admin/api/system/roles" >/dev/null
role=$(curl_json "${auth[@]}" "$admin/api/system/roles/options?limit=500" | jq -er '.data[]|select(.code=="sr_ui_viewer")|.value')
curl_json "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --argjson role "$role" '{username:"schedule_viewer",email:"viewer@example.test",password:"schedule-viewer-password",realName:"viewer",status:1,roleIds:[$role]}')" "$admin/api/system/users" >/dev/null
viewer_run=$(run_case view-only-mobile "$direct_system" "$(jq -c '.viewer' <<<"$steps")")
artifact() { run=$1
prefix=$2
output=$3
id=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$run/artifacts" | jq -er --arg prefix "$prefix" '.data[]|select(.fileName|startswith($prefix))|.id')
curl_json "${auth[@]}" "$admin/api/reports/runs/$run/artifacts/$id" >"/verify/evidence/$output"
sha256sum "/verify/evidence/$output"|awk '{print $1}'
}
desktop_sha=$(artifact "$cancel_run" schedule-form-desktop-dark-en schedule-form-desktop-dark-en.png)
mobile_sha=$(artifact "$viewer_run" schedule-form-mobile-light-zh schedule-form-mobile-light-zh.png)
desktop_dimensions=$(file /verify/evidence/schedule-form-desktop-dark-en.png | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
mobile_dimensions=$(file /verify/evidence/schedule-form-mobile-light-zh.png | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
[ "$desktop_dimensions" = '1440 x 900' ] && [ "$mobile_dimensions" = '390 x 844' ]
jq -nc --arg head "$RUSTZEN_VERIFY_HEAD" --arg state "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" --arg sha "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" --arg platform "$RUSTZEN_VERIFY_PLATFORM" --argjson before "$before" --argjson after "$after" --argjson local "$local_posts" --argjson sb "$secret_before" --argjson sa "$secret_after" --argjson secret "$secret_posts" --arg daily "$daily_run" --arg weekly "$weekly_run" --arg viewer "$viewer_run" --arg desktop "$desktop_sha" --arg mobile "$mobile_sha" --arg dd "$desktop_dimensions" --arg md "$mobile_dimensions" '{schemaVersion:1,status:"passed",gitHead:$head,sourceTreeState:$state,sourceTreeSha256:$sha,platform:$platform,chromiumVersion:env.RUSTZEN_VERIFY_CHROMIUM_VERSION,runs:{daily:$daily,weekly:$weekly,viewer:$viewer},localValidation:{rowsBefore:$before,rowsAfter:$after,proxyPostCount:$local},secretPolicy:{rowsBefore:$sb,rowsAfter:$sa,rowDelta:($sa-$sb),proxyPostCount:$secret},schedule:{cadence:"weekly",weekday:0,dueTime:"10:16",timezone:"UTC"},viewOnly:{managementVisible:false,dueTime:"10:16 · UTC"},artifacts:[{file:"schedule-form-desktop-dark-en.png",sha256:$desktop,dimensions:$dd},{file:"schedule-form-mobile-light-zh.png",sha256:$mobile,dimensions:$md}]}' >/verify/evidence/manifest.json
