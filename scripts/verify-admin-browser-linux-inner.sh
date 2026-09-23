#!/usr/bin/env bash
set -euo pipefail

expected_chromium_version=${RUSTZEN_VERIFY_CHROMIUM_VERSION:?missing pinned Chromium version}
expected_verifier_provenance_sha=${RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256:?missing verifier provenance hash}
test "$(dpkg-query -W -f='${Version}' chromium)" = "$expected_chromium_version"
test "$(sha256sum /usr/local/share/rustzen-browser-verifier.provenance | awk '{print $1}')" = "$expected_verifier_provenance_sha"
for command in chromium curl jq file setpriv ss python3 fc-list; do command -v "$command" >/dev/null; done
fc-list :lang=zh | grep -qi 'Noto'

service_ports=(19801 19802 19803 19804)
proxy_port=19805
for port in "${service_ports[@]}" "$proxy_port"; do
  if ss -H -ltn "sport = :$port" | grep -q .; then
    echo "verification port is already occupied: $port" >&2
    exit 1
  fi
done

groupadd --system rustzen
useradd --system --gid rustzen --home-dir /opt/rz --shell /usr/sbin/nologin rustzen
install -d -m 0750 -o rustzen -g rustzen /opt/rz /opt/rz/data/db /opt/rz/data/reports/db /opt/rz/logs /opt/rz/logs/reports /opt/rz/output
for name in rz-admin rz-monitor rz-insights rz-reports; do
  install -m 0755 "/verify/bin/$name" "/opt/rz/$name"
done
chown -R rustzen:rustzen /opt/rz

export HOME=/opt/rz
export XDG_CONFIG_HOME=/opt/rz/.config
export XDG_CACHE_HOME=/opt/rz/.cache
export RUSTZEN_ENV=development
export RUSTZEN_RUNTIME_ROOT=/opt/rz
export RUSTZEN_ADMIN_HOST=127.0.0.1
export RUSTZEN_ADMIN_PORT=19801
export RUSTZEN_INTERNAL_HOST=127.0.0.1
export RUSTZEN_MONITOR_PORT=19802
export RUSTZEN_INSIGHTS_PORT=19803
export RUSTZEN_REPORTS_PORT=19804
export RUSTZEN_ADMIN_SQLITE_PATH=./data/db/admin.db
export RUSTZEN_MONITOR_SQLITE_PATH=./data/db/monitor.db
export RUSTZEN_INSIGHTS_SQLITE_PATH=./data/db/insights.db
export RUSTZEN_REPORTS_SQLITE_PATH=./data/reports/db/reports.db
export RUSTZEN_JWT_SECRET=ui-browser-verification-jwt-secret
export RUSTZEN_IPC_TOKEN=ui-browser-verification-ipc-secret
export RUSTZEN_MONITOR_AGENT_TOKEN=ui-browser-verification-agent-secret
export RUSTZEN_REPORTS_CREDENTIAL_KEY=ui-browser-verification-credential-key
export RUSTZEN_REPORTS_BROWSER_PATH=/usr/bin/chromium
export RUSTZEN_REPORTS_MAX_CONCURRENCY=1
export RUSTZEN_TIMEZONE=UTC
export RUST_LOG=warn
# The controller only starts against a bound fresh schema. These fixture-only
# identities are stable within this isolated container and are never used by
# the fault proxy or by the target-backed browser lifecycle.
export RUSTZEN_BUILD_ID=$(printf 'a%.0s' {1..64})
export RUSTZEN_COMPOSITION_ID=$(printf 'b%.0s' {1..64})
export RUSTZEN_MONITOR_SCHEMA_FINGERPRINT=$(printf 'c%.0s' {1..64})
export RUSTZEN_MONITOR_DATA_CONTRACT_ID=$(printf 'd%.0s' {1..64})

pids=()
cleanup() {
  result=$?
  for pid in "${pids[@]}"; do kill -TERM "$pid" 2>/dev/null || true; done
  for pid in "${pids[@]}"; do
    for _ in $(seq 1 50); do kill -0 "$pid" 2>/dev/null || break; sleep .1; done
    kill -KILL "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  done
  browser_processes() {
    ps -eo uid=,pid=,stat=,args= | awk -v uid="$(id -u rustzen)" \
      '$1 == uid && $3 !~ /^Z/ && ($0 ~ /chromium/ || $0 ~ /browser-/) { print }'
  }
  for _ in $(seq 1 50); do
    [ -n "$(browser_processes)" ] || break
    sleep .1
  done
  if [ -n "$(browser_processes)" ]; then
    browser_processes >&2
    pkill -KILL -u rustzen -f 'chromium|browser-' 2>/dev/null || true
    result=1
  fi
  if [ "$result" -ne 0 ]; then
    for log in /opt/rz/logs/verify-*.log; do [ -s "$log" ] && { echo "== $log ==" >&2; tail -n 80 "$log" >&2; }; done
  fi
  exit "$result"
}
trap cleanup EXIT INT TERM

start() {
  name=$1
  shift
  setpriv --reuid=rustzen --regid=rustzen --init-groups --no-new-privs -- env \
    HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" XDG_CACHE_HOME="$XDG_CACHE_HOME" \
    RUSTZEN_ENV="$RUSTZEN_ENV" RUSTZEN_RUNTIME_ROOT="$RUSTZEN_RUNTIME_ROOT" \
    RUSTZEN_ADMIN_HOST="$RUSTZEN_ADMIN_HOST" RUSTZEN_ADMIN_PORT="$RUSTZEN_ADMIN_PORT" \
    RUSTZEN_INTERNAL_HOST="$RUSTZEN_INTERNAL_HOST" RUSTZEN_MONITOR_PORT="$RUSTZEN_MONITOR_PORT" \
    RUSTZEN_INSIGHTS_PORT="$RUSTZEN_INSIGHTS_PORT" RUSTZEN_REPORTS_PORT="$RUSTZEN_REPORTS_PORT" \
    RUSTZEN_ADMIN_SQLITE_PATH="$RUSTZEN_ADMIN_SQLITE_PATH" RUSTZEN_MONITOR_SQLITE_PATH="$RUSTZEN_MONITOR_SQLITE_PATH" \
    RUSTZEN_INSIGHTS_SQLITE_PATH="$RUSTZEN_INSIGHTS_SQLITE_PATH" RUSTZEN_REPORTS_SQLITE_PATH="$RUSTZEN_REPORTS_SQLITE_PATH" \
    RUSTZEN_JWT_SECRET="$RUSTZEN_JWT_SECRET" RUSTZEN_IPC_TOKEN="$RUSTZEN_IPC_TOKEN" \
    RUSTZEN_MONITOR_AGENT_TOKEN="$RUSTZEN_MONITOR_AGENT_TOKEN" RUSTZEN_REPORTS_CREDENTIAL_KEY="$RUSTZEN_REPORTS_CREDENTIAL_KEY" \
    RUSTZEN_REPORTS_BROWSER_PATH="$RUSTZEN_REPORTS_BROWSER_PATH" RUSTZEN_REPORTS_MAX_CONCURRENCY=1 \
    RUSTZEN_BUILD_ID="$RUSTZEN_BUILD_ID" RUSTZEN_COMPOSITION_ID="$RUSTZEN_COMPOSITION_ID" \
    RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_MONITOR_SCHEMA_FINGERPRINT" \
    RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_MONITOR_DATA_CONTRACT_ID" \
    RUSTZEN_TIMEZONE=UTC RUST_LOG=warn "$@" >"/opt/rz/logs/verify-$name.log" 2>&1 &
  pids+=("$!")
}

initialize_monitor_database() {
  setpriv --reuid=rustzen --regid=rustzen --init-groups --no-new-privs -- env \
    HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" XDG_CACHE_HOME="$XDG_CACHE_HOME" \
    RUSTZEN_ENV="$RUSTZEN_ENV" RUSTZEN_RUNTIME_ROOT="$RUSTZEN_RUNTIME_ROOT" \
    RUSTZEN_MONITOR_SQLITE_PATH="$RUSTZEN_MONITOR_SQLITE_PATH" \
    RUSTZEN_BUILD_ID="$RUSTZEN_BUILD_ID" RUSTZEN_COMPOSITION_ID="$RUSTZEN_COMPOSITION_ID" \
    RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_MONITOR_SCHEMA_FINGERPRINT" \
    RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_MONITOR_DATA_CONTRACT_ID" \
    RUSTZEN_TIMEZONE=UTC /opt/rz/rz-monitor init-db
  setpriv --reuid=rustzen --regid=rustzen --init-groups --no-new-privs -- env \
    HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" XDG_CACHE_HOME="$XDG_CACHE_HOME" \
    RUSTZEN_ENV="$RUSTZEN_ENV" RUSTZEN_RUNTIME_ROOT="$RUSTZEN_RUNTIME_ROOT" \
    RUSTZEN_MONITOR_SQLITE_PATH="$RUSTZEN_MONITOR_SQLITE_PATH" \
    RUSTZEN_BUILD_ID="$RUSTZEN_BUILD_ID" RUSTZEN_COMPOSITION_ID="$RUSTZEN_COMPOSITION_ID" \
    RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_MONITOR_SCHEMA_FINGERPRINT" \
    RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_MONITOR_DATA_CONTRACT_ID" \
    RUSTZEN_TIMEZONE=UTC /opt/rz/rz-monitor bind-database
}

initialize_monitor_database
start monitor /opt/rz/rz-monitor controller
start insights /opt/rz/rz-insights serve
start reports /opt/rz/rz-reports serve
start admin /opt/rz/rz-admin serve

curl_json() {
  curl --fail --silent --show-error --connect-timeout 3 --max-time 15 "$@"
}
curl_json_with_timeout() {
  max_time=$1
  shift
  curl --fail --silent --show-error --connect-timeout 3 --max-time "$max_time" "$@"
}
# Round a wall-clock instant to the next schedule minute while retaining at
# least 15 seconds for API creation before that minute becomes due.
next_schedule_due_epoch() {
  now_epoch=$1
  printf '%s\n' "$(( ((now_epoch + 75) / 60) * 60 ))"
}
for boundary_epoch in 0 44 45 59; do
  boundary_due_epoch=$(next_schedule_due_epoch "$boundary_epoch")
  boundary_margin=$((boundary_due_epoch - boundary_epoch))
  [ "$boundary_margin" -ge 15 ] && [ "$boundary_margin" -le 75 ] || {
    echo "next schedule admission margin is outside 15..75 seconds" >&2
    exit 1
  }
done
case_diagnostics() {
  diagnostic_run_id=$1
  echo "== browser case diagnostics: $diagnostic_run_id ==" >&2
  curl_json "${auth[@]}" "$admin/api/reports/runs/$diagnostic_run_id" >&2 || true
  curl_json "${auth[@]}" "$admin/api/reports/runs/$diagnostic_run_id/steps" >&2 || true
  curl_json "${auth[@]}" "$admin/api/reports/runs/$diagnostic_run_id/artifacts" >&2 || true
  ps -eo uid=,pid=,stat=,args= | awk -v uid="$(id -u rustzen)" '$1 == uid && $3 !~ /^Z/ && /chromium|browser-/' >&2 || true
  find /opt/rz/output -maxdepth 3 \( -name 'browser-*' -o -name "*$diagnostic_run_id*" \) -print >&2 || true
  tail -n 80 /opt/rz/logs/verify-reports.log >&2 || true
}
for port in "${service_ports[@]}"; do
  ready=0
  for _ in $(seq 1 150); do
    if curl_json "http://127.0.0.1:$port/health" >/dev/null 2>&1; then ready=1; break; fi
    sleep .1
  done
  [ "$ready" = 1 ] || { echo "service on port $port did not become healthy" >&2; exit 1; }
done
for pid in "${pids[@]}"; do kill -0 "$pid"; done

admin=http://127.0.0.1:19801
refresh_api_auth() {
  login=$(curl_json -H 'content-type: application/json' -d '{"username":"owner","password":"rustzen@123"}' "$admin/api/auth/login")
  token=$(jq -er '.data.token | select(length > 20)' <<<"$login")
  auth=(-H "authorization: Bearer $token")
}
refresh_api_auth

# These are the only module-log fixtures. Service stdout goes to verify-*.log,
# which the fixed module-log allowlist never recognizes.
module_log_today=$(date -u +%F)
module_log_expired=2020-01-01
# The running services may already have current-UTC rolling files. The verifier
# writes only these two explicit fixtures and keeps every current-day file intact.
printf 'ADMIN_LOG_CURRENT_MARKER\n' >>"/opt/rz/logs/admin/admin.$module_log_today"
printf 'MONITOR_LOG_EXPIRED_MARKER\n' >"/opt/rz/logs/monitor/monitor.$module_log_expired"
chown rustzen:rustzen "/opt/rz/logs/admin/admin.$module_log_today" "/opt/rz/logs/monitor/monitor.$module_log_expired"
module_log_files=$(curl_json "${auth[@]}" "$admin/api/system/status/module-logs")
printf '%s\n' "$module_log_files" | jq -c --arg today "$module_log_today" '[.data[] | select(.date == $today) | {module: .module, date: .date, fileName: .fileName, active: .active}] | sort_by(.module, .date)' >/verify/evidence/module-log-current-before-cleanup.json
jq -e --arg today "$module_log_today" --arg expired "$module_log_expired" '
  ([.data[] | select(.date == $expired) | {module: .module, date: .date, fileName: .fileName, active: .active}] == [{module:"monitor", date:$expired, fileName:("monitor." + $expired), active:false}]) and
  ([.data[] | select(.date != $expired)] | length > 0) and
  ([.data[] | select(.date != $expired) | (.date == $today and .active == true and (.module | IN("admin", "monitor", "insights", "reports")))] | all)
' <<<"$module_log_files" >/dev/null

# Python is deliberately used only in this disposable Debian verifier: the base
# image has neither Bun nor Node, while Python's standard library is sufficient
# for a route-exact forward proxy. It is mounted from scripts and is not shipped.
start_fault_proxy() {
  method=$1 mode=$2 route=$3 receipt=$4
  RUSTZEN_VERIFY_FAULT_METHOD="$method" RUSTZEN_VERIFY_FAULT_MODE="$mode" RUSTZEN_VERIFY_FAULT_ROUTE="$route" RUSTZEN_VERIFY_FAULT_RECEIPT="$receipt" \
    python3 /verify/fault-proxy.py >/opt/rz/logs/verify-fault-proxy.log 2>&1 &
  proxy_pid=$!
  pids+=("$proxy_pid")
  for _ in $(seq 1 50); do curl_json http://127.0.0.1:19805/__verify_proxy_health >/dev/null 2>&1 && return; sleep .1; done
  echo "fault proxy did not become ready" >&2; exit 1
}

stop_fault_proxy() {
  kill -TERM "$proxy_pid" 2>/dev/null || true
  wait "$proxy_pid" 2>/dev/null || true
  pids=("${pids[@]:0:${#pids[@]}-1}")
}

browser_system=$(jq -nc '{name:"Linux Admin browser fault proxy",baseUrl:"http://127.0.0.1:19805/health",enabled:true}')
browser_system_response=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$browser_system" "$admin/api/reports/systems")
browser_system_id=$(jq -er '.data.id' <<<"$browser_system_response")
browser_success_system=$(jq -nc '{name:"Linux Admin browser target",baseUrl:"http://127.0.0.1:19801/health",enabled:true}')
browser_success_system_response=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$browser_success_system" "$admin/api/reports/systems")
browser_success_system_id=$(jq -er '.data.id' <<<"$browser_success_system_response")
browser_seed_flow=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg system "$browser_system_id" '{systemId:$system,name:"Browser fault seed",steps:[{action:"goto",url:"/health"},{action:"assertText",selector:"body",text:"ok"}]}')" "$admin/api/reports/flows")
browser_seed_flow_id=$(jq -er '.data.id' <<<"$browser_seed_flow")
browser_success_seed_flow=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg system "$browser_success_system_id" '{systemId:$system,name:"Browser schedule lifecycle seed",steps:[{action:"goto",url:"/health"},{action:"assertText",selector:"body",text:"ok"}]}')" "$admin/api/reports/flows")
browser_success_seed_flow_id=$(jq -er '.data.id' <<<"$browser_success_seed_flow")

run_browser_case() {
  case_name=$1 method=$2 mode=$3 route=$4 steps=$5
  echo "running browser fault case: $case_name"
  # Every browser run signs in with a fresh isolated profile. Refresh the API
  # controller session too, so the ten-session production cap cannot evict a
  # long-lived verifier token midway through this intentionally long matrix.
  refresh_api_auth
  receipt=/verify/evidence/"$case_name".receipt.json
  start_fault_proxy "$method" "$mode" "$route" "$receipt"
  steps=$(jq -c --arg name "$case_name" '. + [{action:"screenshot",name:$name}]' <<<"$steps")
  body=$(jq -nc --arg system "$browser_system_id" --arg name "$case_name" --argjson steps "$steps" '{systemId:$system,name:$name,steps:$steps}')
  flow=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$body" "$admin/api/reports/flows")
  case_flow_id=$(jq -er '.data.id' <<<"$flow")
  run=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg flow "$case_flow_id" '{flowId:$flow,input:{}}')" "$admin/api/reports/runs")
  case_run_id=$(jq -er '.data.id' <<<"$run")
  state=queued
  for _ in $(seq 1 900); do
    run=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$case_run_id")
    state=$(jq -er '.data.status' <<<"$run")
    case "$state" in queued|running|cancelling) sleep .1 ;; *) break ;; esac
  done
  stop_fault_proxy
  if [ "$state" != succeeded ]; then
    case_diagnostics "$case_run_id"
    [ ! -s "$receipt" ] || { echo "fault receipt:" >&2; cat "$receipt" >&2; echo >&2; }
    curl_json "${auth[@]}" "$admin/api/reports/runs/$case_run_id/steps" >&2 || true
    echo "fault case $case_name ended with $state" >&2
    exit 1
  fi
  jq -e --arg method "$method" --arg mode "$mode" --arg route "$route" '.method == $method and .mode == $mode and .route == $route and .hitCount == 1' "$receipt" >/dev/null || { echo "fault proxy receipt was not exactly one hit: $case_name" >&2; exit 1; }
  artifacts=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$case_run_id/artifacts")
  artifact_id=$(jq -er --arg prefix "$case_name-" '.data[] | select(.kind == "screenshot" and (.fileName | startswith($prefix))) | .id' <<<"$artifacts")
  image=/verify/evidence/"$case_name".png
  if ! curl --fail --silent --show-error --connect-timeout 3 --max-time 30 -o "$image" "${auth[@]}" "$admin/api/reports/runs/$case_run_id/artifacts/$artifact_id"; then
    echo "artifact download failed for $case_name (run $case_run_id, artifact $artifact_id):" >&2
    echo "$artifacts" >&2
    exit 1
  fi
  file "$image" | grep -Eq 'PNG image data, [1-9][0-9]* x [1-9][0-9]*'
  dimensions=$(file "$image" | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
  jq -nc --arg runId "$case_run_id" --arg method "$method" --arg mode "$mode" --arg route "$route" --slurpfile receipt "$receipt" --arg file "$(basename "$image")" --arg sha "$(sha256sum "$image" | awk '{print $1}')" --arg dimensions "$dimensions" '{runId:$runId,method:$method,mode:$mode,route:$route,receipt:$receipt[0],artifact:{file:$file,sha256:$sha,dimensions:$dimensions}}' >>/verify/evidence/fault-cases.jsonl
  echo "browser fault case passed: $case_name"
}

run_target_browser_case() {
  case_name=$1 steps=$2
  echo "running browser target case: $case_name"
  refresh_api_auth
  body=$(jq -nc --arg system "$browser_success_system_id" --arg name "$case_name" --argjson steps "$steps" '{systemId:$system,name:$name,steps:$steps}')
  flow=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$body" "$admin/api/reports/flows")
  case_flow_id=$(jq -er '.data.id' <<<"$flow")
  run=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg flow "$case_flow_id" '{flowId:$flow,input:{}}')" "$admin/api/reports/runs")
  case_run_id=$(jq -er '.data.id' <<<"$run")
  state=queued
  for _ in $(seq 1 900); do
    run=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$case_run_id")
    state=$(jq -er '.data.status' <<<"$run")
    case "$state" in queued|running|cancelling) sleep .1 ;; *) break ;; esac
  done
  if [ "$state" != succeeded ]; then
    case_diagnostics "$case_run_id"
    curl_json "${auth[@]}" "$admin/api/reports/runs/$case_run_id/steps" >&2 || true
    echo "target case $case_name ended with $state" >&2
    exit 1
  fi
  jq --arg name "$case_name" --arg runId "$case_run_id" \
    '. + [{name:$name,runId:$runId,execution:"target-backed"}]' \
    /verify/evidence/success-cases.json >/verify/evidence/success-cases.json.next
  mv /verify/evidence/success-cases.json.next /verify/evidence/success-cases.json
  echo "browser target case passed: $case_name"
}

download_target_screenshot() {
  case_name=$1 screenshot_name=$2 output=/verify/evidence/$3
  artifacts=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$case_run_id/artifacts")
  artifact_id=$(jq -er --arg prefix "$screenshot_name-" '.data[] | select(.kind == "screenshot" and (.fileName | startswith($prefix))) | .id' <<<"$artifacts")
  curl --fail --silent --show-error --connect-timeout 3 --max-time 30 \
    -o "$output" "${auth[@]}" "$admin/api/reports/runs/$case_run_id/artifacts/$artifact_id"
  file "$output" | grep -Eq 'PNG image data, [1-9][0-9]* x [1-9][0-9]*'
  sha=$(sha256sum "$output" | awk '{print $1}')
  dimensions=$(file "$output" | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
  jq --arg name "$case_name" --arg file "$(basename "$output")" --arg sha "$sha" --arg dimensions "$dimensions" \
    'map(if .name == $name then . + {artifact:{file:$file,sha256:$sha,dimensions:$dimensions}} else . end)' \
    /verify/evidence/success-cases.json >/verify/evidence/success-cases.json.next
  mv /verify/evidence/success-cases.json.next /verify/evidence/success-cases.json
}

login_steps='[{"action":"goto","url":"/login"},{"action":"waitFor","selector":"#login_username"},{"action":"fill","selector":"#login_username","value":"owner"},{"action":"fill","selector":"#login_password","value":"rustzen@123"},{"action":"click","selector":"button[type=submit]"},{"action":"waitFor","selector":".shell-content"}]'
printf '[]\n' >/verify/evidence/success-cases.json
schedule_steps=$(jq -nc --argjson login "$login_steps" '$login + [{action:"goto",url:"/reports/templates"},{action:"waitFor",selector:"[data-testid=schedule-create]"},{action:"click",selector:"[data-testid=schedule-create]"},{action:"waitFor",selector:"[data-testid=schedule-save]"},{action:"fill",selector:"input[type=time]",value:"10:15"},{action:"click",selector:"[data-testid=schedule-save]"},{action:"waitFor",selector:"[data-testid=schedule-save-error]"},{action:"assertText",selector:"[data-testid=schedule-save-error]",text:"计划未保存"},{action:"assertValue",selector:"input[type=time]",value:"10:15"}]')
run_browser_case schedule-create-network POST network /api/reports/schedules "$schedule_steps"
run_browser_case schedule-create-http POST http /api/reports/schedules "$schedule_steps"
seed_schedule=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg flow "$browser_seed_flow_id" '{flowId:$flow,cadence:"daily",dueTime:"09:00",input:{},description:"fault edit",enabled:true}')" "$admin/api/reports/schedules")
seed_schedule_id=$(jq -er '.data.id' <<<"$seed_schedule")
schedule_edit_steps=$(jq -nc --argjson login "$login_steps" '$login + [{action:"goto",url:"/reports/templates"},{action:"waitFor",selector:"[data-testid=schedule-edit]"},{action:"click",selector:"[data-testid=schedule-edit]"},{action:"waitFor",selector:"[data-testid=schedule-save]"},{action:"fill",selector:"input[type=time]",value:"10:16"},{action:"click",selector:"[data-testid=schedule-save]"},{action:"waitFor",selector:"[data-testid=schedule-save-error]"},{action:"assertText",selector:"[data-testid=schedule-save-error]",text:"计划未保存"},{action:"assertValue",selector:"input[type=time]",value:"10:16"}]')
run_browser_case schedule-edit-network PUT network "/api/reports/schedules/$seed_schedule_id" "$schedule_edit_steps"
run_browser_case schedule-edit-http PUT http "/api/reports/schedules/$seed_schedule_id" "$schedule_edit_steps"
curl_json "${auth[@]}" -X DELETE "$admin/api/reports/schedules/$seed_schedule_id" >/dev/null

target_desktop_login=$(jq -nc --argjson login "$login_steps" '[{action:"setUiPreferences",theme:"dark",locale:"en-US"},{action:"setViewport",width:1440,height:900}] + $login')
create_daily_steps=$(jq -nc --argjson login "$target_desktop_login" '$login + [{action:"goto",url:"/reports/templates"},{action:"waitFor",selector:"[data-testid=schedule-create]"},{action:"assertText",selector:"[data-testid=schedule-panel]",text:"Scheduled reports"},{action:"click",selector:"[data-testid=schedule-create]"},{action:"waitFor",selector:"[data-testid=schedule-dialog]"},{action:"fill",selector:"[data-testid=schedule-due-time]",value:"10:15"},{action:"fill",selector:"[data-testid=schedule-description]",value:"browser lifecycle daily"},{action:"click",selector:"[data-testid=schedule-save]"},{action:"pause",durationMs:600},{action:"waitFor",selector:"[data-testid=schedule-toggle]"},{action:"assertText",selector:"[data-testid=schedule-panel]",text:"Daily"},{action:"assertNoHorizontalOverflow"},{action:"screenshotViewport",name:"schedule-desktop-dark-en"}]')
run_target_browser_case schedule-create-daily "$create_daily_steps"
download_target_screenshot schedule-create-daily schedule-desktop-dark-en schedule-desktop-dark-en.png
schedule_id=$(curl_json "${auth[@]}" "$admin/api/reports/schedules" | jq -er '.data[] | select(.description == "browser lifecycle daily") | .id')

edit_weekly_steps=$(jq -nc --argjson login "$target_desktop_login" '$login + [{action:"goto",url:"/reports/templates"},{action:"waitFor",selector:"[data-testid=schedule-edit]"},{action:"click",selector:"[data-testid=schedule-edit]"},{action:"waitFor",selector:"[data-testid=schedule-dialog]"},{action:"click",selector:"[data-testid=schedule-cadence]"},{action:"waitFor",selector:"[data-testid=schedule-cadence-weekly]"},{action:"click",selector:"[data-testid=schedule-cadence-weekly]"},{action:"waitFor",selector:"[data-testid=schedule-weekday]"},{action:"click",selector:"[data-testid=schedule-weekday]"},{action:"waitFor",selector:"[data-testid=schedule-weekday-0]"},{action:"click",selector:"[data-testid=schedule-weekday-0]"},{action:"fill",selector:"[data-testid=schedule-due-time]",value:"10:16"},{action:"click",selector:"[data-testid=schedule-save]"},{action:"pause",durationMs:600},{action:"assertText",selector:"[data-testid=schedule-panel]",text:"Weekly Monday"}]')
run_target_browser_case schedule-edit-weekly "$edit_weekly_steps"
curl_json "${auth[@]}" "$admin/api/reports/schedules/$schedule_id" | jq -e '.data.cadence == "weekly" and .data.weekday == 0 and .data.dueTime == "10:16"' >/dev/null

disable_steps=$(jq -nc --argjson login "$target_desktop_login" '$login + [{action:"goto",url:"/reports/templates"},{action:"waitFor",selector:"[data-testid=schedule-toggle]"},{action:"click",selector:"[data-testid=schedule-toggle]"},{action:"pause",durationMs:600},{action:"assertText",selector:"[data-testid=schedule-toggle]",text:"Off"}]')
run_target_browser_case schedule-disable "$disable_steps"
curl_json "${auth[@]}" "$admin/api/reports/schedules/$schedule_id" | jq -e '.data.enabled == false' >/dev/null

enable_steps=$(jq -nc --argjson login "$target_desktop_login" '$login + [{action:"goto",url:"/reports/templates"},{action:"waitFor",selector:"[data-testid=schedule-toggle]"},{action:"click",selector:"[data-testid=schedule-toggle]"},{action:"pause",durationMs:600},{action:"assertText",selector:"[data-testid=schedule-toggle]",text:"On"}]')
run_target_browser_case schedule-enable "$enable_steps"
curl_json "${auth[@]}" "$admin/api/reports/schedules/$schedule_id" | jq -e '.data.enabled == true' >/dev/null

menu_options=$(curl_json "${auth[@]}" "$admin/api/system/menus/options?limit=500")
schedule_view_menu_id=$(jq -er '.data[] | select(.code == "reports:schedule:view") | .value' <<<"$menu_options")
curl_json "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --argjson menu "$schedule_view_menu_id" '{name:"Browser schedule viewer",code:"browser_schedule_viewer",status:1,menuIds:[$menu],description:"Linux browser verifier"}')" "$admin/api/system/roles" >/dev/null
viewer_role_id=$(curl_json "${auth[@]}" "$admin/api/system/roles/options?limit=500" | jq -er '.data[] | select(.code == "browser_schedule_viewer") | .value')
curl_json "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --argjson role "$viewer_role_id" '{username:"schedule_viewer",email:"schedule_viewer@example.test",password:"schedule-viewer-password",realName:"Schedule viewer",status:1,roleIds:[$role]}')" "$admin/api/system/users" >/dev/null
viewer_login='[{"action":"goto","url":"/login"},{"action":"waitFor","selector":"#login_username"},{"action":"fill","selector":"#login_username","value":"schedule_viewer"},{"action":"fill","selector":"#login_password","value":"schedule-viewer-password"},{"action":"click","selector":"button[type=submit]"},{"action":"waitFor","selector":".shell-content"}]'
viewer_mobile_steps=$(jq -nc --argjson login "$viewer_login" '[{action:"setUiPreferences",theme:"light",locale:"zh-CN"},{action:"setViewport",width:390,height:844}] + $login + [{action:"goto",url:"/reports/templates"},{action:"waitFor",selector:"[data-testid=schedule-panel]"},{action:"assertText",selector:"[data-testid=schedule-panel]",text:"定时报表计划"},{action:"assertAbsent",selector:"[data-testid=schedule-create]"},{action:"assertAbsent",selector:"[data-testid=schedule-edit]"},{action:"assertAbsent",selector:"[data-testid=schedule-toggle]"},{action:"assertAbsent",selector:"[data-testid=schedule-delete]"},{action:"assertNoHorizontalOverflow"},{action:"screenshot",name:"schedule-mobile-full"},{action:"screenshotViewport",name:"schedule-mobile-light-zh"}]')
run_target_browser_case schedule-view-only-mobile "$viewer_mobile_steps"
download_target_screenshot schedule-view-only-mobile schedule-mobile-light-zh schedule-mobile-light-zh.png

delete_steps=$(jq -nc --argjson login "$target_desktop_login" '$login + [{action:"goto",url:"/reports/templates"},{action:"waitFor",selector:"[data-testid=schedule-delete]"},{action:"click",selector:"[data-testid=schedule-delete]"},{action:"waitFor",selector:"[data-testid=schedule-delete-confirm]"},{action:"click",selector:"[data-testid=schedule-delete-confirm]"},{action:"pause",durationMs:600},{action:"assertAbsent",selector:"[data-testid=schedule-toggle]"}]')
run_target_browser_case schedule-delete "$delete_steps"
if curl_json "${auth[@]}" "$admin/api/reports/schedules" | jq -e --arg id "$schedule_id" '.data | all(.id != $id)' >/dev/null; then :; else
  echo "deleted lifecycle schedule still appears in the Reports API" >&2
  exit 1
fi

# This target-backed occurrence uses the actual 15-second Reports scheduler.
# Rounding now+75 seconds down to a minute always leaves 16..75 seconds for API
# admission before the due minute, then a wall-clock deadline bounds polling.
scheduled_admission_epoch=$(date -u +%s)
scheduled_due_epoch=$(next_schedule_due_epoch "$scheduled_admission_epoch")
scheduled_due_time=$(date -u -d "@$scheduled_due_epoch" +%H:%M)
scheduled_occurrence=$(curl_json "${auth[@]}" -H 'content-type: application/json' \
  -d "$(jq -nc --arg flow "$browser_success_seed_flow_id" --arg due "$scheduled_due_time" '{flowId:$flow,cadence:"daily",dueTime:$due,input:{},description:"browser scheduler occurrence",enabled:true}')" \
  "$admin/api/reports/schedules")
scheduled_occurrence_id=$(jq -er '.data.id' <<<"$scheduled_occurrence")
scheduled_deadline_epoch=$(( $(date -u +%s) + 90 ))
scheduled_run_id=
while :; do
  scheduled_now_epoch=$(date -u +%s)
  scheduled_remaining_seconds=$((scheduled_deadline_epoch - scheduled_now_epoch))
  [ "$scheduled_remaining_seconds" -gt 0 ] || break
  scheduled_curl_timeout=$scheduled_remaining_seconds
  [ "$scheduled_curl_timeout" -le 15 ] || scheduled_curl_timeout=15
  scheduled_occurrence=$(curl_json_with_timeout "$scheduled_curl_timeout" "${auth[@]}" "$admin/api/reports/schedules/$scheduled_occurrence_id")
  scheduled_run_id=$(jq -r '.data.lastOccurrence | select(.decision == "enqueued") | .runId // empty' <<<"$scheduled_occurrence")
  [ -n "$scheduled_run_id" ] && break
  sleep .25
done
[ -n "$scheduled_run_id" ] || { echo "scheduler did not enqueue the next-minute occurrence before the 90-second wall-clock deadline" >&2; exit 1; }
scheduled_run=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$scheduled_run_id")
jq -e --arg schedule "$scheduled_occurrence_id" --arg flow "$browser_success_seed_flow_id" --arg run "$scheduled_run_id" '
  .data.id == $schedule and
  .data.flowId == $flow and
  .data.lastOccurrence.decision == "enqueued" and
  .data.lastOccurrence.runId == $run
' <<<"$scheduled_occurrence" >/dev/null
jq -e --arg flow "$browser_success_seed_flow_id" '
  .data.flowId == $flow and (.data.status | IN("queued", "running", "succeeded"))
' <<<"$scheduled_run" >/dev/null

schedule_occurrence_steps=$(jq -nc --argjson login "$target_desktop_login" --arg schedule "$scheduled_occurrence_id" --arg run "$scheduled_run_id" '$login + [
  {action:"goto",url:"/reports/templates"},
  {action:"waitFor",selector:"[data-testid=schedule-occurrence-run-\($schedule)]"},
  {action:"assertText",selector:"[data-testid=schedule-occurrence-\($schedule)]",text:"Enqueued"},
  {action:"click",selector:"[data-testid=schedule-occurrence-run-\($schedule)]"},
  {action:"waitFor",selector:"[data-testid=run-audit][data-run-id='\''\($run)'\'']"},
  {action:"assertText",selector:".ant-modal",text:"Run audit"},
  {action:"assertNoHorizontalOverflow"},
  {action:"screenshotViewport",name:"schedule-occurrence-run-desktop-dark-en"}
]')
run_target_browser_case schedule-occurrence-enqueued-link "$schedule_occurrence_steps"
download_target_screenshot schedule-occurrence-enqueued-link schedule-occurrence-run-desktop-dark-en schedule-occurrence-run-desktop-dark-en.png
jq --arg name schedule-occurrence-enqueued-link --arg schedule "$scheduled_occurrence_id" --arg run "$scheduled_run_id" '
  map(if .name == $name then . + {scheduleId:$schedule,scheduledRunId:$run} else . end)
' /verify/evidence/success-cases.json >/verify/evidence/success-cases.json.next
mv /verify/evidence/success-cases.json.next /verify/evidence/success-cases.json

# Controlled UI fixture only: the API intentionally cannot create a historical
# occurrence before effectiveAt. Keep this schedule disabled so the real
# scheduler cannot touch it, then insert one schema-valid missed decision in
# the disposable verifier database to exercise the no-run-link rendering.
missed_due_at=$(date -u -d '2 minutes ago' +%Y-%m-%dT%H:%M:%SZ)
missed_due_local=$(date -u -d '2 minutes ago' +%Y-%m-%dT%H:%M)
missed_schedule=$(curl_json "${auth[@]}" -H 'content-type: application/json'   -d "$(jq -nc --arg flow "$browser_success_seed_flow_id" --arg due "$(date -u +%H:%M)" '{flowId:$flow,cadence:"daily",dueTime:$due,input:{},description:"browser missed occurrence fixture",enabled:false}')"   "$admin/api/reports/schedules")
missed_schedule_id=$(jq -er '.data.id' <<<"$missed_schedule")
python3 - /opt/rz/data/reports/db/reports.db "$missed_schedule_id" "$missed_due_local" "$missed_due_at" <<'PY2'
import sqlite3
import sys

database, schedule_id, due_local, due_at = sys.argv[1:]
connection = sqlite3.connect(database, timeout=5)
try:
    connection.execute("PRAGMA busy_timeout=5000")
    connection.execute("BEGIN IMMEDIATE")
    connection.execute(
        """INSERT INTO automation_schedule_occurrences
           (schedule_id, occurrence_key, due_local, due_at, decided_at, decision, reason, run_id, run_id_snapshot)
           VALUES (?, ?, ?, ?, ?, 'skipped', 'missed', NULL, NULL)""",
        (schedule_id, due_local, due_local, due_at, due_at),
    )
    connection.commit()
finally:
    connection.close()
PY2
missed_schedule=$(curl_json "${auth[@]}" "$admin/api/reports/schedules/$missed_schedule_id")
jq -e '
  .data.enabled == false and
  .data.lastOccurrence.decision == "skipped" and
  .data.lastOccurrence.reason == "missed" and
  (.data.lastOccurrence.dueAt | type == "string") and
  .data.lastOccurrence.runId == null
' <<<"$missed_schedule" >/dev/null

schedule_missed_steps=$(jq -nc --argjson login "$target_desktop_login" --arg schedule "$missed_schedule_id" --arg due_at "$missed_due_at" '$login + [
  {action:"goto",url:"/reports/templates"},
  {action:"waitFor",selector:"[data-testid=schedule-occurrence-skipped-\($schedule)]"},
  {action:"waitFor",selector:"[data-testid=schedule-occurrence-due-\($schedule)][data-due-at='\''\($due_at)'\'']"},
  {action:"assertText",selector:"[data-testid=schedule-occurrence-\($schedule)]",text:"Skipped"},
  {action:"assertText",selector:"[data-testid=schedule-occurrence-due-\($schedule)]",text:"202"},
  {action:"assertText",selector:"[data-testid=schedule-occurrence-skipped-\($schedule)]",text:"missed"},
  {action:"assertAbsent",selector:"[data-testid=schedule-occurrence-run-\($schedule)]"},
  {action:"assertNoHorizontalOverflow"}
]')
run_target_browser_case schedule-occurrence-missed-no-link "$schedule_missed_steps"
jq --arg name schedule-occurrence-missed-no-link --arg schedule "$missed_schedule_id" '
  map(if .name == $name then . + {scheduleId:$schedule,fixture:"controlled-sqlite-missed-occurrence"} else . end)
' /verify/evidence/success-cases.json >/verify/evidence/success-cases.json.next
mv /verify/evidence/success-cases.json.next /verify/evidence/success-cases.json

# Module-log diagnostics uses the real owner UI and only the two isolated fixtures above.
module_log_desktop_login=$(jq -nc --argjson login "$login_steps" '[{action:"setUiPreferences",theme:"dark",locale:"zh-CN"},{action:"setViewport",width:1440,height:900}] + $login')
module_log_desktop_steps=$(jq -nc --argjson login "$module_log_desktop_login" --arg today "$module_log_today" --arg expired "$module_log_expired" '$login + [
  {action:"goto",url:"/system/module-log"},
  {action:"waitFor",selector:"[data-testid=module-log-panel]"},
  {action:"assertText",selector:"[data-testid=module-log-panel]",text:"模块日志诊断"},
  {action:"waitFor",selector:"[data-testid=module-log-tail-admin-\($today)]"},
  {action:"click",selector:"[data-testid=module-log-tail-admin-\($today)]"},
  {action:"waitFor",selector:"[data-testid=module-log-tail-drawer]"},
  {action:"waitFor",selector:"[data-testid=module-log-tail-content]"},
  {action:"assertText",selector:"[data-testid=module-log-tail-drawer]",text:"受限日志尾部"},
  {action:"assertText",selector:"[data-testid=module-log-tail-content]",text:"ADMIN_LOG_CURRENT_MARKER"},
  {action:"pressKey",key:"Escape"},{action:"pause",durationMs:100},
  {action:"click",selector:"[data-testid=module-log-select-admin-\($today)] input[type=checkbox]"},
  {action:"click",selector:"[data-testid=module-log-backup]"},
  {action:"waitFor",selector:"[data-testid=module-log-backup-summary]"},
  {action:"assertText",selector:"[data-testid=module-log-backup-summary]",text:"1 个文件"},
  {action:"assertText",selector:"[data-testid=module-log-backup-summary]",text:"SHA-256"},
  {action:"click",selector:"[data-testid=module-log-cleanup-preview]"},
  {action:"waitFor",selector:"[data-testid=module-log-cleanup-candidates]"},
  {action:"assertText",selector:"[data-testid=module-log-cleanup-candidates]",text:"monitor.\($expired)"},
  {action:"click",selector:"[data-testid=module-log-cleanup-confirm-trigger]"},
  {action:"waitFor",selector:"[data-testid=module-log-cleanup-confirm]"},
  {action:"click",selector:"[data-testid=module-log-cleanup-confirm]"},
  {action:"waitFor",selector:"[data-testid=module-log-cleanup-result]"},
  {action:"assertText",selector:"[data-testid=module-log-cleanup-result]",text:"已删除 1 个文件"},
  {action:"assertNoHorizontalOverflow"},
  {action:"screenshotViewport",name:"module-log-desktop-dark-zh"}
]')
run_target_browser_case module-log-owner-desktop "$module_log_desktop_steps"
download_target_screenshot module-log-owner-desktop module-log-desktop-dark-zh module-log-desktop-dark-zh.png
module_log_after_cleanup=$(curl_json "${auth[@]}" "$admin/api/system/status/module-logs")
printf '%s\n' "$module_log_after_cleanup" | jq -c --arg today "$module_log_today" '[.data[] | select(.date == $today) | {module: .module, date: .date, fileName: .fileName, active: .active}] | sort_by(.module, .date)' >/verify/evidence/module-log-current-after-cleanup.json
cmp -s /verify/evidence/module-log-current-before-cleanup.json /verify/evidence/module-log-current-after-cleanup.json
if printf '%s\n' "$module_log_after_cleanup" | jq -e --arg today "$module_log_today" --arg expired "$module_log_expired" '([.data[] | select(.date == $expired)] | length == 0) and ([.data[] | (.date == $today and .active == true)] | all)' >/dev/null; then :; else
  echo "module-log cleanup did not remove only the expired fixture" >&2
  exit 1
fi

module_log_mobile_login=$(jq -nc --argjson login "$login_steps" '[{action:"setUiPreferences",theme:"light",locale:"en-US"},{action:"setViewport",width:390,height:844}] + $login')
module_log_mobile_steps=$(jq -nc --argjson login "$module_log_mobile_login" --arg today "$module_log_today" '$login + [
  {action:"goto",url:"/system/module-log"},
  {action:"waitFor",selector:"[data-testid=module-log-panel]"},
  {action:"assertText",selector:"[data-testid=module-log-panel]",text:"Module log diagnostics"},
  {action:"waitFor",selector:"[data-testid=module-log-tail-admin-\($today)]"},
  {action:"click",selector:"[data-testid=module-log-tail-admin-\($today)]"},
  {action:"waitFor",selector:"[data-testid=module-log-tail-content]"},
  {action:"assertText",selector:"[data-testid=module-log-tail-content]",text:"ADMIN_LOG_CURRENT_MARKER"},
  {action:"assertNoHorizontalOverflow"},
  {action:"screenshotViewport",name:"module-log-mobile-light-en"}
]')
run_target_browser_case module-log-owner-mobile "$module_log_mobile_steps"
download_target_screenshot module-log-owner-mobile module-log-mobile-light-en module-log-mobile-light-en.png

# Runs retry acceptance deliberately uses terminal and active executions from the
# real Reports service. The child is identified by the retry endpoint after the
# list action, then its exact ID is checked when retrying from the source audit.
wait_for_run_status() {
  expected_run_id=$1 expected_status=$2
  for _ in $(seq 1 900); do
    expected_run=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$expected_run_id")
    actual_status=$(jq -er '.data.status' <<<"$expected_run")
    [ "$actual_status" = "$expected_status" ] && return
    case "$actual_status" in queued|running|cancelling) sleep .1 ;; *) break ;; esac
  done
  echo "run $expected_run_id did not reach $expected_status (got ${actual_status:-unknown})" >&2
  exit 1
}

retry_succeeded_run=$(curl_json "${auth[@]}" -H 'content-type: application/json' \
  -d "$(jq -nc --arg flow "$browser_success_seed_flow_id" '{flowId:$flow,input:{}}')" \
  "$admin/api/reports/runs")
retry_succeeded_run_id=$(jq -er '.data.id' <<<"$retry_succeeded_run")
wait_for_run_status "$retry_succeeded_run_id" succeeded

# The current browser flow is itself running while it renders this list, so the
# broad absence check proves neither it nor the known succeeded row exposes Retry.
retry_terminal_hidden_steps=$(jq -nc --argjson login "$target_desktop_login" --arg succeeded "$retry_succeeded_run_id" '$login + [{action:"goto",url:"/reports/runs"},{action:"waitFor",selector:"[data-testid=run-view-\($succeeded)]"},{action:"assertAbsent",selector:"[data-testid=run-retry-list-\($succeeded)]"},{action:"assertAbsent",selector:"[data-testid^=run-retry-]"},{action:"assertNoHorizontalOverflow"}]')
run_target_browser_case run-retry-terminal-hidden "$retry_terminal_hidden_steps"

retry_source_flow=$(curl_json "${auth[@]}" -H 'content-type: application/json' \
  -d "$(jq -nc --arg system "$browser_success_system_id" '{systemId:$system,name:"Browser retry failed source",steps:[{action:"goto",url:"/health"},{action:"assertText",selector:"body",text:"browser retry source must fail"}]}')" \
  "$admin/api/reports/flows")
retry_source_flow_id=$(jq -er '.data.id' <<<"$retry_source_flow")
retry_source_run=$(curl_json "${auth[@]}" -H 'content-type: application/json' \
  -d "$(jq -nc --arg flow "$retry_source_flow_id" '{flowId:$flow,input:{}}')" \
  "$admin/api/reports/runs")
retry_source_run_id=$(jq -er '.data.id' <<<"$retry_source_run")
wait_for_run_status "$retry_source_run_id" failed
curl_json "${auth[@]}" "$admin/api/reports/runs/$retry_source_run_id" >/verify/evidence/retry-source-run.before.json
curl_json "${auth[@]}" "$admin/api/reports/runs/$retry_source_run_id/steps" >/verify/evidence/retry-source-steps.before.json
curl_json "${auth[@]}" "$admin/api/reports/runs/$retry_source_run_id/artifacts" >/verify/evidence/retry-source-artifacts.before.json

retry_list_steps=$(jq -nc --argjson login "$target_desktop_login" --arg source "$retry_source_run_id" '$login + [{action:"goto",url:"/reports/runs"},{action:"waitFor",selector:"[data-testid=run-retry-list-\($source)]"},{action:"click",selector:"[data-testid=run-retry-list-\($source)]"},{action:"waitFor",selector:"[data-testid=run-audit]"},{action:"assertText",selector:".ant-modal",text:"Run audit"},{action:"assertNoHorizontalOverflow"},{action:"screenshotViewport",name:"run-retry-desktop-dark-en"}]')
run_target_browser_case run-retry-list-trigger "$retry_list_steps"

# The list action already made the first request. A second request can only
# return that retained direct child, allowing an exact audit-ID assertion.
retry_child=$(curl_json "${auth[@]}" -X POST "$admin/api/reports/runs/$retry_source_run_id/retry")
retry_child_id=$(jq -er '.data.id' <<<"$retry_child")
curl_json "${auth[@]}" "$admin/api/reports/runs/$retry_source_run_id" >/verify/evidence/retry-source-run.after-list.json
curl_json "${auth[@]}" "$admin/api/reports/runs/$retry_source_run_id/steps" >/verify/evidence/retry-source-steps.after-list.json
curl_json "${auth[@]}" "$admin/api/reports/runs/$retry_source_run_id/artifacts" >/verify/evidence/retry-source-artifacts.after-list.json
cmp -s /verify/evidence/retry-source-run.before.json /verify/evidence/retry-source-run.after-list.json
cmp -s /verify/evidence/retry-source-steps.before.json /verify/evidence/retry-source-steps.after-list.json
cmp -s /verify/evidence/retry-source-artifacts.before.json /verify/evidence/retry-source-artifacts.after-list.json

retry_list_child_steps=$(jq -nc --argjson login "$target_desktop_login" --arg source "$retry_source_run_id" --arg child "$retry_child_id" '$login + [{action:"goto",url:"/reports/runs"},{action:"waitFor",selector:"[data-testid=run-retry-list-\($source)]"},{action:"click",selector:"[data-testid=run-retry-list-\($source)]"},{action:"waitFor",selector:"[data-testid=run-audit][data-run-id='\''\($child)'\'']"},{action:"assertText",selector:".ant-modal",text:"Run audit"},{action:"assertNoHorizontalOverflow"},{action:"screenshotViewport",name:"run-retry-desktop-dark-en"}]')
run_target_browser_case run-retry-list-child-selected "$retry_list_child_steps"
download_target_screenshot run-retry-list-child-selected run-retry-desktop-dark-en run-retry-desktop-dark-en.png

retry_audit_steps=$(jq -nc --argjson login "$target_desktop_login" --arg source "$retry_source_run_id" --arg child "$retry_child_id" '$login + [{action:"goto",url:"/reports/runs"},{action:"waitFor",selector:"[data-testid=run-view-\($source)]"},{action:"click",selector:"[data-testid=run-view-\($source)]"},{action:"waitFor",selector:"[data-testid=run-audit][data-run-id='\''\($source)'\'']"},{action:"waitFor",selector:"[data-testid=run-retry-audit-\($source)]"},{action:"click",selector:"[data-testid=run-retry-audit-\($source)]"},{action:"waitFor",selector:"[data-testid=run-audit][data-run-id='\''\($child)'\'']"},{action:"assertText",selector:".ant-modal",text:"Run audit"},{action:"assertNoHorizontalOverflow"}]')
run_target_browser_case run-retry-audit "$retry_audit_steps"
curl_json "${auth[@]}" "$admin/api/reports/runs/$retry_source_run_id" >/verify/evidence/retry-source-run.after-audit.json
curl_json "${auth[@]}" "$admin/api/reports/runs/$retry_source_run_id/steps" >/verify/evidence/retry-source-steps.after-audit.json
curl_json "${auth[@]}" "$admin/api/reports/runs/$retry_source_run_id/artifacts" >/verify/evidence/retry-source-artifacts.after-audit.json
cmp -s /verify/evidence/retry-source-run.before.json /verify/evidence/retry-source-run.after-audit.json
cmp -s /verify/evidence/retry-source-steps.before.json /verify/evidence/retry-source-steps.after-audit.json
cmp -s /verify/evidence/retry-source-artifacts.before.json /verify/evidence/retry-source-artifacts.after-audit.json

run_view_menu_id=$(jq -er '.data[] | select(.code == "reports:run:view") | .value' <<<"$menu_options")
curl_json "${auth[@]}" -H 'content-type: application/json' \
  -d "$(jq -nc --argjson menu "$run_view_menu_id" '{name:"Browser runs viewer",code:"browser_runs_viewer",status:1,menuIds:[$menu],description:"Linux browser verifier"}')" \
  "$admin/api/system/roles" >/dev/null
run_viewer_role_id=$(curl_json "${auth[@]}" "$admin/api/system/roles/options?limit=500" | jq -er '.data[] | select(.code == "browser_runs_viewer") | .value')
curl_json "${auth[@]}" -H 'content-type: application/json' \
  -d "$(jq -nc --argjson role "$run_viewer_role_id" '{username:"run_viewer",email:"run_viewer@example.test",password:"run-viewer-password",realName:"Run viewer",status:1,roleIds:[$role]}')" \
  "$admin/api/system/users" >/dev/null
run_viewer_login=$(curl_json -H 'content-type: application/json' -d '{"username":"run_viewer","password":"run-viewer-password"}' "$admin/api/auth/login")
run_viewer_token=$(jq -er '.data.token | select(length > 20)' <<<"$run_viewer_login")
run_viewer_retry_status=$(curl --silent --show-error --output /verify/evidence/run-viewer-retry.json --write-out '%{http_code}' \
  --connect-timeout 3 --max-time 15 -H "authorization: Bearer $run_viewer_token" -X POST \
  "$admin/api/reports/runs/$retry_source_run_id/retry")
[ "$run_viewer_retry_status" = 403 ] || { echo "run-view-only retry endpoint returned $run_viewer_retry_status" >&2; exit 1; }
run_viewer_mobile_login='[{"action":"goto","url":"/login"},{"action":"waitFor","selector":"#login_username"},{"action":"fill","selector":"#login_username","value":"run_viewer"},{"action":"fill","selector":"#login_password","value":"run-viewer-password"},{"action":"click","selector":"button[type=submit]"},{"action":"waitFor","selector":".shell-content"}]'
retry_viewer_steps=$(jq -nc --argjson login "$run_viewer_mobile_login" --arg source "$retry_source_run_id" '[{action:"setUiPreferences",theme:"light",locale:"zh-CN"},{action:"setViewport",width:390,height:844}] + $login + [{action:"goto",url:"/reports/runs"},{action:"waitFor",selector:"[data-testid=run-view-\($source)]"},{action:"assertText",selector:".shell-content",text:"填报执行"},{action:"assertAbsent",selector:"[data-testid=run-retry-list-\($source)]"},{action:"assertAbsent",selector:".ant-message-error"},{action:"assertNoHorizontalOverflow"},{action:"screenshotViewport",name:"run-retry-mobile-light-zh"}]')
run_target_browser_case run-retry-view-only-mobile "$retry_viewer_steps"
download_target_screenshot run-retry-view-only-mobile run-retry-mobile-light-zh run-retry-mobile-light-zh.png

node_report=$(jq -nc --arg boot "11111111-1111-4111-8111-111111111111" --arg collected "$(date -u +%Y-%m-%dT%H:%M:%SZ)" '{nodeId:"browser-fault-node",bootId:$boot,sequence:1,hostname:"browser-fault-node",agentVersion:"verify",collectedAt:$collected,cpuPercent:10,memory:{usedBytes:10,totalBytes:100},disks:[{mountPoint:"/",usedBytes:10,totalBytes:100}]}')
if ! curl_json -H 'x-rustzen-monitor-agent-token: ui-browser-verification-agent-secret' -H 'content-type: application/json' -d "$node_report" "$admin/api/monitor/agent-reports" >/dev/null; then
  echo "browser fixture node registration failed" >&2
  exit 1
fi
node_settings='{"cpu":{"enabled":true,"thresholdPercent":80},"memory":{"enabled":true,"thresholdPercent":80},"disk":{"enabled":true,"thresholdPercent":80},"offline":{"enabled":true,"afterSeconds":60}}'
if ! curl_json "${auth[@]}" -H 'content-type: application/json' -X PUT -d "$node_settings" "$admin/api/monitor/nodes/browser-fault-node/alert-settings" >/dev/null; then
  echo "browser fixture node policy setup failed" >&2
  exit 1
fi
global_steps=$(jq -nc --argjson login "$login_steps" '$login + [{action:"goto",url:"/monitoring/nodes"},{action:"waitFor",selector:"[data-testid=monitor-global-settings]"},{action:"click",selector:"[data-testid=monitor-global-settings]"},{action:"waitFor",selector:"[data-testid=monitor-global-save]"},{action:"fill",selector:"input[aria-label=\"CPU 阈值（%）\"]",value:"73"},{action:"click",selector:"[data-testid=monitor-global-save]"},{action:"waitFor",selector:"[data-testid=monitor-global-save-error]"},{action:"assertText",selector:"[data-testid=monitor-global-save-error]",text:"告警设置未保存"},{action:"assertValue",selector:"input[aria-label=\"CPU 阈值（%）\"]",value:"73"}]')
global_http_steps=$(jq -nc --argjson login "$login_steps" '$login + [{action:"goto",url:"/monitoring/nodes"},{action:"waitFor",selector:"[data-testid=monitor-global-settings]"},{action:"click",selector:"[data-testid=monitor-global-settings]"},{action:"waitFor",selector:"[data-testid=monitor-global-save]"},{action:"fill",selector:"input[aria-label=\"CPU 阈值（%）\"]",value:"74"},{action:"click",selector:"[data-testid=monitor-global-save]"},{action:"waitFor",selector:".ant-message"},{action:"assertText",selector:".ant-message",text:"browser fault injection"},{action:"assertAbsent",selector:"[data-testid=monitor-global-save-error]"},{action:"assertValue",selector:"input[aria-label=\"CPU 阈值（%）\"]",value:"74"}]')
node_steps=$(jq -nc --argjson login "$login_steps" '$login + [{action:"goto",url:"/monitoring/nodes"},{action:"waitFor",selector:"[data-testid=monitor-node-view]"},{action:"click",selector:"[data-testid=monitor-node-view]"},{action:"waitFor",selector:"[data-testid=monitor-node-save]"},{action:"fill",selector:"input[aria-label=\"CPU 阈值（%）\"]",value:"75"},{action:"click",selector:"[data-testid=monitor-node-save]"},{action:"waitFor",selector:"[data-testid=monitor-node-save-error]"},{action:"assertText",selector:"[data-testid=monitor-node-save-error]",text:"节点策略未保存"},{action:"assertValue",selector:"input[aria-label=\"CPU 阈值（%）\"]",value:"75"}]')
node_http_steps=$(jq -nc --argjson login "$login_steps" '$login + [{action:"goto",url:"/monitoring/nodes"},{action:"waitFor",selector:"[data-testid=monitor-node-view]"},{action:"click",selector:"[data-testid=monitor-node-view]"},{action:"waitFor",selector:"[data-testid=monitor-node-save]"},{action:"fill",selector:"input[aria-label=\"CPU 阈值（%）\"]",value:"76"},{action:"click",selector:"[data-testid=monitor-node-save]"},{action:"waitFor",selector:".ant-message"},{action:"assertText",selector:".ant-message",text:"browser fault injection"},{action:"assertAbsent",selector:"[data-testid=monitor-node-save-error]"},{action:"assertValue",selector:"input[aria-label=\"CPU 阈值（%）\"]",value:"76"}]')
node_reset_steps=$(jq -nc --argjson login "$login_steps" '$login + [{action:"goto",url:"/monitoring/nodes"},{action:"waitFor",selector:"[data-testid=monitor-node-view]"},{action:"click",selector:"[data-testid=monitor-node-view]"},{action:"waitFor",selector:"[data-testid=monitor-node-reset]"},{action:"click",selector:"[data-testid=monitor-node-reset]"},{action:"waitFor",selector:"[data-testid=monitor-node-save-error]"},{action:"assertText",selector:"[data-testid=monitor-node-save-error]",text:"节点策略未保存"},{action:"assertText",selector:"[data-testid=monitor-node-policy-source]",text:"节点自定义"},{action:"assertText",selector:"[data-testid=monitor-node-reset]",text:"重置为全局默认"}]')
node_reset_http_steps=$(jq -nc --argjson login "$login_steps" '$login + [{action:"goto",url:"/monitoring/nodes"},{action:"waitFor",selector:"[data-testid=monitor-node-view]"},{action:"click",selector:"[data-testid=monitor-node-view]"},{action:"waitFor",selector:"[data-testid=monitor-node-reset]"},{action:"click",selector:"[data-testid=monitor-node-reset]"},{action:"waitFor",selector:".ant-message"},{action:"assertText",selector:".ant-message",text:"browser fault injection"},{action:"assertAbsent",selector:"[data-testid=monitor-node-save-error]"},{action:"assertText",selector:"[data-testid=monitor-node-policy-source]",text:"节点自定义"},{action:"assertText",selector:"[data-testid=monitor-node-reset]",text:"重置为全局默认"}]')
run_browser_case monitor-global-save-network PUT network /api/monitor/alert-settings "$global_steps"
run_browser_case monitor-global-save-http PUT http /api/monitor/alert-settings "$global_http_steps"
run_browser_case monitor-node-save-network PUT network /api/monitor/nodes/browser-fault-node/alert-settings "$node_steps"
run_browser_case monitor-node-save-http PUT http /api/monitor/nodes/browser-fault-node/alert-settings "$node_http_steps"
run_browser_case monitor-node-reset-network DELETE network /api/monitor/nodes/browser-fault-node/alert-settings "$node_reset_steps"
run_browser_case monitor-node-reset-http DELETE http /api/monitor/nodes/browser-fault-node/alert-settings "$node_reset_http_steps"

# Start Chromium on the lightweight health response. The first audited flow
# step then navigates to /login on the same Admin origin. This separates CDP
# session readiness from the SPA load while preserving the origin guard.
system_body=$(jq -nc '{name:"Linux Admin UI",baseUrl:"http://127.0.0.1:19801/health",enabled:true}')
system=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$system_body" "$admin/api/reports/systems")
system_id=$(jq -er '.data.id' <<<"$system")
flow_body=$(jq -nc --arg system "$system_id" '{systemId:$system,name:"Admin rendered routes",steps:[
  {action:"goto",url:"/login"},
  {action:"waitFor",selector:"#login_username"},
  {action:"fill",selector:"#login_username",value:"owner"},
  {action:"fill",selector:"#login_password",value:"rustzen@123"},
  {action:"click",selector:"button[type=submit]"},
  {action:"waitFor",selector:".shell-content"},
  {action:"assertText",selector:".shell-content",text:"仪表盘"},
  {action:"screenshot",name:"dashboard"},
  {action:"goto",url:"/analytics/details"},
  {action:"waitFor",selector:".shell-content"},
  {action:"assertText",selector:".shell-content",text:"分析明细"},
  {action:"screenshot",name:"analytics-details"}
]}')
flow=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$flow_body" "$admin/api/reports/flows")
flow_id=$(jq -er '.data.id' <<<"$flow")
run_body=$(jq -nc --arg flow "$flow_id" '{flowId:$flow,input:{}}')
run=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$run_body" "$admin/api/reports/runs")
run_id=$(jq -er '.data.id' <<<"$run")

state=queued
for _ in $(seq 1 900); do
  run=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$run_id")
  state=$(jq -er '.data.status' <<<"$run")
  case "$state" in queued|running|cancelling) sleep .1 ;; *) break ;; esac
done
if [ "$state" != succeeded ]; then
  curl_json "${auth[@]}" "$admin/api/reports/runs/$run_id/steps" >&2 || true
  echo "browser run ended with $state" >&2
  exit 1
fi

artifacts=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$run_id/artifacts")
for name in dashboard analytics-details; do
  artifact_id=$(jq -er --arg prefix "$name-" '.data[] | select(.kind == "screenshot" and (.fileName | startswith($prefix))) | .id' <<<"$artifacts")
  headers="/verify/evidence/$name.headers"
  output="/verify/evidence/$name.png"
  curl --fail --silent --show-error --connect-timeout 3 --max-time 30 \
    -D "$headers" -o "$output" "${auth[@]}" "$admin/api/reports/runs/$run_id/artifacts/$artifact_id"
  grep -Eiq '^content-type:[[:space:]]*image/png' "$headers"
  [ "$(od -An -tx1 -N8 "$output" | tr -d ' \n')" = 89504e470d0a1a0a ]
  [ "$(wc -c <"$output")" -gt 0 ]
  file "$output" | grep -Eq 'PNG image data, [1-9][0-9]* x [1-9][0-9]*'
done

chromium_version=$(/usr/bin/chromium --version)
dashboard_file=$(file /verify/evidence/dashboard.png)
analytics_file=$(file /verify/evidence/analytics-details.png)
jq -n \
  --arg head "$RUSTZEN_VERIFY_HEAD" \
  --arg sourceTreeState "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" \
  --arg sourceTreeSha256 "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" \
  --arg architecture "$RUSTZEN_VERIFY_ARCHITECTURE" \
  --arg verifierImageId "$RUSTZEN_VERIFY_VERIFIER_IMAGE_ID" \
  --arg verifierKey "$RUSTZEN_VERIFY_VERIFIER_KEY" \
  --arg verifierProvenanceSha256 "$RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256" \
  --arg runId "$run_id" \
  --arg browser "$chromium_version" \
  --arg binaries "$RUSTZEN_VERIFY_BINARY_HASHES" \
  --arg dashboardSha "$(sha256sum /verify/evidence/dashboard.png | awk '{print $1}')" \
  --argjson dashboardBytes "$(wc -c </verify/evidence/dashboard.png)" \
  --arg dashboardDimensions "$(sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/' <<<"$dashboard_file")" \
  --arg analyticsSha "$(sha256sum /verify/evidence/analytics-details.png | awk '{print $1}')" \
  --argjson analyticsBytes "$(wc -c </verify/evidence/analytics-details.png)" \
  --arg analyticsDimensions "$(sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/' <<<"$analytics_file")" \
  --slurpfile faultCases /verify/evidence/fault-cases.jsonl \
  --argfile successCases /verify/evidence/success-cases.json \
  '{schemaVersion:2,gitHead:$head,sourceTreeState:$sourceTreeState,sourceTreeSha256:$sourceTreeSha256,architecture:$architecture,verifier:{imageId:$verifierImageId,key:$verifierKey,provenanceSha256:$verifierProvenanceSha256},reportsRunId:$runId,browser:$browser,binaryHashes:$binaries,artifacts:{dashboard:{file:"dashboard.png",sha256:$dashboardSha,bytes:$dashboardBytes,dimensions:$dashboardDimensions},analyticsDetails:{file:"analytics-details.png",sha256:$analyticsSha,bytes:$analyticsBytes,dimensions:$analyticsDimensions}},faultCases:$faultCases,successCases:$successCases}' \
  >/verify/evidence/manifest.json

test "$(jq -r .gitHead /verify/evidence/manifest.json)" = "$RUSTZEN_VERIFY_HEAD"
echo "Linux Admin UI browser run passed ($chromium_version)"
