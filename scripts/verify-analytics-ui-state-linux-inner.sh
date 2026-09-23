#!/usr/bin/env bash
set -euo pipefail

pids=()
pid_alive() { local state; kill -0 "$1" 2>/dev/null || return 1; state=$(ps -o stat= -p "$1" 2>/dev/null | tr -d ' '); case "$state" in Z*) return 1;; *) return 0;; esac; }
collect_pid_tree() { local pid=$1 child; printf '%s\n' "$pid"; for child in $(pgrep -P "$pid" 2>/dev/null || true); do collect_pid_tree "$child"; done; }
stop_pid() {
  local pid=$1 attempt target any_alive; local targets=( $(collect_pid_tree "$pid") )
  for target in "${targets[@]}"; do kill -TERM "$target" 2>/dev/null || true; done
  for attempt in $(seq 1 5); do any_alive=0; for target in "${targets[@]}"; do if pid_alive "$target"; then any_alive=1; break; fi; done; [ "$any_alive" = 0 ] && break; sleep .05; done
  for target in "${targets[@]}"; do pid_alive "$target" && kill -KILL "$target" 2>/dev/null || true; done
  wait "$pid" 2>/dev/null || true
}
cleanup() {
  result=$?
  trap - EXIT INT TERM
  for pid in "${pids[@]}"; do stop_pid "$pid"; done
  if [ -n "${RUSTZEN_ANALYTICS_UI_STATE_INNER_TEST_STUBBORN:-}" ]; then
    ! pid_alive "$inner_stubborn_pid" && ! pid_alive "$inner_grandchild_pid" && test "$(cat /tmp/rz-analytics-ui-inner-graceful.receipt)" = graceful || exit 99
    rm -f /tmp/rz-analytics-ui-inner-grandchild.pid
    rm -f /tmp/rz-analytics-ui-inner-graceful.receipt
    echo "Analytics UI inner ${RUSTZEN_ANALYTICS_UI_STATE_INNER_TEST_STUBBORN} stubborn cleanup seam passed" >&2
  fi
  if [ "$result" -ne 0 ]; then
    for log in /opt/rz/logs/{admin,monitor,insights,reports,fixture}.log; do
      [ -s "$log" ] || continue
      echo "== $log ==" >&2; tail -n 40 "$log" >&2
    done
  fi
  exit "$result"
}
if [ "${RUSTZEN_ANALYTICS_UI_STATE_INNER_TEST_STUBBORN:-}" = INT ] || [ "${RUSTZEN_ANALYTICS_UI_STATE_INNER_TEST_STUBBORN:-}" = TERM ]; then
  signal=$RUSTZEN_ANALYTICS_UI_STATE_INNER_TEST_STUBBORN
  sh -c 'trap "exit 0" TERM; sh -c '"'"'trap "" TERM; while :; do sleep 60; done'"'"' & echo $! > "$1"; wait' sh /tmp/rz-analytics-ui-inner-grandchild.pid & inner_stubborn_pid=$!; for _ in $(seq 1 20); do [ -s /tmp/rz-analytics-ui-inner-grandchild.pid ] && break; sleep .01; done; inner_grandchild_pid=$(cat /tmp/rz-analytics-ui-inner-grandchild.pid); sh -c 'trap '"'"'sleep .1; printf graceful > "$1"; exit 0'"'"' TERM; while :; do sleep 60; done' sh /tmp/rz-analytics-ui-inner-graceful.receipt & inner_graceful_pid=$!; pids+=("$inner_stubborn_pid" "$inner_graceful_pid")
  trap cleanup EXIT; trap 'exit 130' INT; trap 'exit 143' TERM; kill -"$signal" "$$"
fi

for command in chromium curl jq file setpriv python3 ss; do command -v "$command" >/dev/null; done
test "$(dpkg-query -W -f='${Version}' chromium)" = "${RUSTZEN_VERIFY_CHROMIUM_VERSION:?}"
for port in 19801 19802 19803 19804 19805; do
  ! ss -H -ltn "sport = :$port" | grep -q . || { echo "verification port occupied: $port" >&2; exit 1; }
done

groupadd --system rustzen
useradd --system --gid rustzen --home-dir /opt/rz --shell /usr/sbin/nologin rustzen
install -d -m 0750 -o rustzen -g rustzen /opt/rz/data/db /opt/rz/data/reports/db /opt/rz/logs /opt/rz/output
for name in rz-admin rz-monitor rz-insights rz-reports; do install -m 0755 "/verify/bin/$name" "/opt/rz/$name"; done
chown -R rustzen:rustzen /opt/rz

export HOME=/opt/rz XDG_CONFIG_HOME=/opt/rz/.config XDG_CACHE_HOME=/opt/rz/.cache
export RUSTZEN_ENV=development RUSTZEN_RUNTIME_ROOT=/opt/rz RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_ADMIN_PORT=19801 RUSTZEN_INTERNAL_HOST=127.0.0.1 RUSTZEN_MONITOR_PORT=19802 RUSTZEN_INSIGHTS_PORT=19803 RUSTZEN_REPORTS_PORT=19804
export RUSTZEN_ADMIN_SQLITE_PATH=./data/db/admin.db RUSTZEN_MONITOR_SQLITE_PATH=./data/db/monitor.db RUSTZEN_INSIGHTS_SQLITE_PATH=./data/db/insights.db RUSTZEN_REPORTS_SQLITE_PATH=./data/reports/db/reports.db
export RUSTZEN_JWT_SECRET=analytics-ui-jwt-secret RUSTZEN_IPC_TOKEN=analytics-ui-ipc-secret RUSTZEN_MONITOR_AGENT_TOKEN=analytics-ui-agent-secret RUSTZEN_REPORTS_CREDENTIAL_KEY=analytics-ui-key RUSTZEN_REPORTS_BROWSER_PATH=/usr/bin/chromium RUSTZEN_REPORTS_MAX_CONCURRENCY=1 RUSTZEN_TIMEZONE=UTC
export RUSTZEN_BUILD_ID=$(printf 'a%.0s' {1..64}) RUSTZEN_COMPOSITION_ID=$(printf 'b%.0s' {1..64}) RUSTZEN_MONITOR_SCHEMA_FINGERPRINT=$(printf 'c%.0s' {1..64}) RUSTZEN_MONITOR_DATA_CONTRACT_ID=$(printf 'd%.0s' {1..64}) RUST_LOG=warn

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

as_rustzen() { setpriv --reuid=rustzen --regid=rustzen --init-groups --no-new-privs -- env HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" XDG_CACHE_HOME="$XDG_CACHE_HOME" RUSTZEN_ENV="$RUSTZEN_ENV" RUSTZEN_RUNTIME_ROOT="$RUSTZEN_RUNTIME_ROOT" RUSTZEN_ADMIN_HOST="$RUSTZEN_ADMIN_HOST" RUSTZEN_ADMIN_PORT="$RUSTZEN_ADMIN_PORT" RUSTZEN_INTERNAL_HOST="$RUSTZEN_INTERNAL_HOST" RUSTZEN_MONITOR_PORT="$RUSTZEN_MONITOR_PORT" RUSTZEN_INSIGHTS_PORT="$RUSTZEN_INSIGHTS_PORT" RUSTZEN_REPORTS_PORT="$RUSTZEN_REPORTS_PORT" RUSTZEN_ADMIN_SQLITE_PATH="$RUSTZEN_ADMIN_SQLITE_PATH" RUSTZEN_MONITOR_SQLITE_PATH="$RUSTZEN_MONITOR_SQLITE_PATH" RUSTZEN_INSIGHTS_SQLITE_PATH="$RUSTZEN_INSIGHTS_SQLITE_PATH" RUSTZEN_REPORTS_SQLITE_PATH="$RUSTZEN_REPORTS_SQLITE_PATH" RUSTZEN_JWT_SECRET="$RUSTZEN_JWT_SECRET" RUSTZEN_IPC_TOKEN="$RUSTZEN_IPC_TOKEN" RUSTZEN_MONITOR_AGENT_TOKEN="$RUSTZEN_MONITOR_AGENT_TOKEN" RUSTZEN_REPORTS_CREDENTIAL_KEY="$RUSTZEN_REPORTS_CREDENTIAL_KEY" RUSTZEN_REPORTS_BROWSER_PATH="$RUSTZEN_REPORTS_BROWSER_PATH" RUSTZEN_REPORTS_MAX_CONCURRENCY=1 RUSTZEN_BUILD_ID="$RUSTZEN_BUILD_ID" RUSTZEN_COMPOSITION_ID="$RUSTZEN_COMPOSITION_ID" RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_MONITOR_SCHEMA_FINGERPRINT" RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_MONITOR_DATA_CONTRACT_ID" RUSTZEN_TIMEZONE=UTC RUST_LOG=warn "$@"; }

as_rustzen /opt/rz/rz-monitor init-db
as_rustzen /opt/rz/rz-monitor bind-database
for pair in 'monitor:/opt/rz/rz-monitor controller' 'insights:/opt/rz/rz-insights serve' 'reports:/opt/rz/rz-reports serve' 'admin:/opt/rz/rz-admin serve'; do
  name=${pair%%:*}; command=${pair#*:}
  as_rustzen $command >"/opt/rz/logs/$name.log" 2>&1 & pids+=("$!")
done
curl_json() { curl --fail --silent --show-error --connect-timeout 3 --max-time 15 "$@"; }
for port in 19801 19802 19803 19804; do
  for retry in $(seq 1 150); do curl_json "http://127.0.0.1:$port/health" >/dev/null 2>&1 && break; sleep .1; done
  curl_json "http://127.0.0.1:$port/health" >/dev/null
done
python3 /verify/fixture.py >/opt/rz/logs/fixture.log 2>&1 & pids+=("$!")
for retry in $(seq 1 50); do curl_json http://127.0.0.1:19805/__analytics_fixture/health >/dev/null 2>&1 && break; sleep .1; done
curl_json http://127.0.0.1:19805/__analytics_fixture/health >/dev/null

admin=http://127.0.0.1:19801
refresh_auth() {
  token=$(curl_json -H 'content-type: application/json' -d '{"username":"owner","password":"rustzen@123"}' "$admin/api/auth/login" | jq -er '.data.token')
  auth=(-H "authorization: Bearer $token")
}
refresh_auth
system=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d '{"name":"Analytics UI fixture","baseUrl":"http://127.0.0.1:19805/health","enabled":true}' "$admin/api/reports/systems" | jq -er '.data.id')
steps=$(cat /verify/evidence/browser-steps.json)

set_mode() { curl_json -X PATCH -H 'content-type: application/json' -d "$1" http://127.0.0.1:19805/__analytics_fixture/mode >/dev/null; }
diagnostics() {
  local name=$1 run=$2
  echo "== Analytics UI browser case failed: $name ($run) ==" >&2
  curl_json "${auth[@]}" "$admin/api/reports/runs/$run" | jq -c '.data|{id,status,error}' >&2 || true
  curl_json "${auth[@]}" "$admin/api/reports/runs/$run/steps" | jq -c '.data[]|{stepIndex,action,status,message}' >&2 || true
  curl_json "${auth[@]}" "$admin/api/reports/runs/$run/artifacts" >"/verify/evidence/failure-run-artifacts.json" 2>/dev/null || true
  curl_json http://127.0.0.1:19805/__analytics_fixture/receipt >"/verify/evidence/fixture-receipt-on-failure.json" 2>/dev/null || true
}
run_case() {
  local name=$1 case_steps=$2 body flow run status receipt_tmp
  refresh_auth
  body=$(jq -nc --arg system "$system" --arg name "Analytics UI state" --argjson steps "$case_steps" '{systemId:$system,name:$name,steps:$steps}')
  flow=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$body" "$admin/api/reports/flows" | jq -er '.data.id')
  run=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg flow "$flow" '{flowId:$flow,input:{}}')" "$admin/api/reports/runs" | jq -er '.data.id')
  for _ in $(seq 1 900); do status=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$run" | jq -er '.data.status'); case "$status" in queued|running) sleep .1;; *) break;; esac; done
  [ "$status" = succeeded ] || { diagnostics "$name" "$run"; exit 1; }
  mkdir -p /verify/evidence/run-steps
  receipt_tmp="/verify/evidence/run-steps/.${name}.json.tmp"
  curl_json "${auth[@]}" "$admin/api/reports/runs/$run/steps" | jq -e '.data | map({action,status,message})' >"$receipt_tmp"
  mv -f "$receipt_tmp" "/verify/evidence/run-steps/$name.json"
  printf '%s\n' "$run"
}
artifact() {
  local run=$1 prefix=$2 output=$3 id listing tmp
  refresh_auth
  listing=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$run/artifacts") || return
  id=$(jq -er --arg prefix "$prefix" '.data[]|select(.fileName|startswith($prefix))|.id' <<<"$listing") || return
  tmp="/verify/evidence/.${output}.tmp"
  curl_json "${auth[@]}" "$admin/api/reports/runs/$run/artifacts/$id" >"$tmp" || { rm -f "$tmp"; return 1; }
  test -s "$tmp" || { rm -f "$tmp"; return 1; }
  mv -f "$tmp" "/verify/evidence/$output"
  sha256sum "/verify/evidence/$output" | awk '{print $1}'
}

set_mode '{"overview":"slow","events":"success"}'
overview_run=$(run_case overview-loading "$(jq -c '.overviewLoading' <<<"$steps")")
set_mode '{"overview":"success","events":"empty"}'
empty_run=$(run_case details-empty "$(jq -c '.detailsEmpty' <<<"$steps")")
set_mode '{"overview":"403","events":"success"}'
forbidden_run=$(run_case overview-403 "$(jq -c '.overview403' <<<"$steps")")
set_mode '{"overview":"success","events":"403"}'
details_forbidden_run=$(run_case details-403 "$(jq -c '.details403' <<<"$steps")")
set_mode '{"overview":"500","events":"success"}'
overview_error_run=$(run_case overview-500 "$(jq -c '.overview500' <<<"$steps")")
set_mode '{"overview":"success","events":"500"}'
error_run=$(run_case details-500 "$(jq -c '.details500' <<<"$steps")")
set_mode '{"overview":"success","events":"success"}'
filter_run=$(run_case details-filter-resets-page "$(jq -c '.detailsFilterResetsPage' <<<"$steps")")
set_mode '{"overview":"success","events":"success","eventsFailAfterFirstStatus":"500"}'
background_run=$(run_case details-background-refresh "$(jq -c '.detailsBackgroundRefresh' <<<"$steps")")
set_mode '{"overview":"success","events":"success","overviewFailAfterFirstStatus":"403"}'
overview_background_forbidden_run=$(run_case overview-background-403 "$(jq -c '.overviewBackground403' <<<"$steps")")
set_mode '{"overview":"success","events":"success","eventsFailAfterFirstStatus":"403"}'
details_background_forbidden_run=$(run_case details-background-403 "$(jq -c '.detailsBackground403' <<<"$steps")")
receipt=$(curl_json http://127.0.0.1:19805/__analytics_fixture/receipt)
receipt_tmp=/verify/evidence/.fixture-receipt.json.tmp
printf '%s\n' "$receipt" >"$receipt_tmp"
mv -f "$receipt_tmp" /verify/evidence/fixture-receipt.json
desktop_sha=$(artifact "$overview_run" analytics-overview-desktop-dark-en analytics-overview-desktop-dark-en.png)
mobile_sha=$(artifact "$empty_run" analytics-details-mobile-light-zh analytics-details-mobile-light-zh.png)
desktop_dimensions=$(file /verify/evidence/analytics-overview-desktop-dark-en.png | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
mobile_dimensions=$(file /verify/evidence/analytics-details-mobile-light-zh.png | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
[ "$desktop_dimensions" = '1440 x 900' ] && [ "$mobile_dimensions" = '390 x 844' ]
jq -e '([.requests[] | select(.route == "/api/insights/overview") | .mode]) as $overview | ([.requests[] | select(.route == "/api/insights/events") | .mode]) as $events | ([.requests[] | select(.route == "/api/insights/overview" and (.mode == "slow" or .mode == "403" or .mode == "500"))] | length >= 3) and ([.requests[] | select(.route == "/api/insights/events" and (.mode == "empty" or .mode == "403" or .mode == "500"))] | length >= 4) and any(range(0; ($overview | length) - 1); . as $i | $overview[$i] == "success" and $overview[$i + 1] == "403") and any(range(0; ($events | length) - 1); . as $i | $events[$i] == "success" and $events[$i + 1] == "500") and any(range(0; ($events | length) - 1); . as $i | $events[$i] == "success" and $events[$i + 1] == "403") and ([.requests[] | select(.route == "/api/insights/events" and .query.current[0] == "2")] | length >= 1) and ([.requests[] | select(.route == "/api/insights/events" and .query.path[0] == "/fixture-filter" and .query.current[0] == "1")] | length >= 1)' <<<"$receipt" >/dev/null
jq -nc --arg head "$RUSTZEN_VERIFY_HEAD" --arg state "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" --arg sha "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" --arg platform "$RUSTZEN_VERIFY_PLATFORM" --arg overview "$overview_run" --arg empty "$empty_run" --arg forbidden "$forbidden_run" --arg detailsForbidden "$details_forbidden_run" --arg overviewFailure "$overview_error_run" --arg failure "$error_run" --arg filter "$filter_run" --arg background "$background_run" --arg overviewBackgroundForbidden "$overview_background_forbidden_run" --arg detailsBackgroundForbidden "$details_background_forbidden_run" --argjson receipt "$receipt" --slurpfile overviewSteps /verify/evidence/run-steps/overview-loading.json --slurpfile emptySteps /verify/evidence/run-steps/details-empty.json --slurpfile forbiddenSteps /verify/evidence/run-steps/overview-403.json --slurpfile detailsForbiddenSteps /verify/evidence/run-steps/details-403.json --slurpfile overviewFailureSteps /verify/evidence/run-steps/overview-500.json --slurpfile failureSteps /verify/evidence/run-steps/details-500.json --slurpfile filterSteps /verify/evidence/run-steps/details-filter-resets-page.json --slurpfile backgroundSteps /verify/evidence/run-steps/details-background-refresh.json --slurpfile overviewBackgroundForbiddenSteps /verify/evidence/run-steps/overview-background-403.json --slurpfile detailsBackgroundForbiddenSteps /verify/evidence/run-steps/details-background-403.json --arg desktop "$desktop_sha" --arg mobile "$mobile_sha" --arg dd "$desktop_dimensions" --arg md "$mobile_dimensions" '{schemaVersion:1,status:"passed",gitHead:$head,sourceTreeState:$state,sourceTreeSha256:$sha,platform:$platform,chromiumVersion:env.RUSTZEN_VERIFY_CHROMIUM_VERSION,runs:{overviewLoading:$overview,detailsEmpty:$empty,overview403:$forbidden,details403:$detailsForbidden,overview500:$overviewFailure,details500:$failure,detailsFilterResetsPage:$filter,detailsBackgroundRefresh:$background,overviewBackground403:$overviewBackgroundForbidden,detailsBackground403:$detailsBackgroundForbidden},runSteps:{overviewLoading:$overviewSteps[0],detailsEmpty:$emptySteps[0],overview403:$forbiddenSteps[0],details403:$detailsForbiddenSteps[0],overview500:$overviewFailureSteps[0],details500:$failureSteps[0],detailsFilterResetsPage:$filterSteps[0],detailsBackgroundRefresh:$backgroundSteps[0],overviewBackground403:$overviewBackgroundForbiddenSteps[0],detailsBackground403:$detailsBackgroundForbiddenSteps[0]},fixtureReceipt:$receipt,artifacts:[{file:"analytics-overview-desktop-dark-en.png",sha256:$desktop,dimensions:$dd},{file:"analytics-details-mobile-light-zh.png",sha256:$mobile,dimensions:$md}]}' >/verify/evidence/manifest.json
