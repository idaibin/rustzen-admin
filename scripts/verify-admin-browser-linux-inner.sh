#!/usr/bin/env bash
set -euo pipefail

export DEBIAN_FRONTEND=noninteractive
apt-get update >/dev/null
apt-get install -y --no-install-recommends ca-certificates >/dev/null
if [ "$RUSTZEN_VERIFY_BROWSER_CHANNEL" = snapshot120 ]; then
  cat >/etc/apt/sources.list <<'SOURCES'
deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/20240131T000000Z bullseye main
deb [check-valid-until=no] https://snapshot.debian.org/archive/debian-security/20240131T000000Z bullseye-security main
SOURCES
  rm -f /etc/apt/sources.list.d/*
  apt-get update >/dev/null
  chromium_version=120.0.6099.224-1~deb11u1
  apt-get install -y --no-install-recommends \
    "chromium=$chromium_version" "chromium-common=$chromium_version" "chromium-sandbox=$chromium_version" \
    fonts-noto-cjk curl jq file procps util-linux iproute2 >/dev/null
else
  apt-get install -y --no-install-recommends chromium chromium-sandbox fonts-noto-cjk curl jq file procps util-linux iproute2 >/dev/null
fi

ports=(19801 19802 19803 19804)
for port in "${ports[@]}"; do
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
    RUSTZEN_TIMEZONE=UTC RUST_LOG=warn "$@" >"/opt/rz/logs/verify-$name.log" 2>&1 &
  pids+=("$!")
}

start monitor /opt/rz/rz-monitor controller
start insights /opt/rz/rz-insights serve
start reports /opt/rz/rz-reports serve
start admin /opt/rz/rz-admin serve

curl_json() {
  curl --fail --silent --show-error --connect-timeout 3 --max-time 15 "$@"
}
for port in "${ports[@]}"; do
  ready=0
  for _ in $(seq 1 150); do
    if curl_json "http://127.0.0.1:$port/health" >/dev/null 2>&1; then ready=1; break; fi
    sleep .1
  done
  [ "$ready" = 1 ] || { echo "service on port $port did not become healthy" >&2; exit 1; }
done
for pid in "${pids[@]}"; do kill -0 "$pid"; done

admin=http://127.0.0.1:19801
login=$(curl_json -H 'content-type: application/json' -d '{"username":"owner","password":"rustzen@123"}' "$admin/api/auth/login")
token=$(jq -er '.data.token | select(length > 20)' <<<"$login")
auth=(-H "authorization: Bearer $token")

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
  --arg runId "$run_id" \
  --arg browser "$chromium_version" \
  --arg binaries "$RUSTZEN_VERIFY_BINARY_HASHES" \
  --arg dashboardSha "$(sha256sum /verify/evidence/dashboard.png | awk '{print $1}')" \
  --argjson dashboardBytes "$(wc -c </verify/evidence/dashboard.png)" \
  --arg dashboardDimensions "$(sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/' <<<"$dashboard_file")" \
  --arg analyticsSha "$(sha256sum /verify/evidence/analytics-details.png | awk '{print $1}')" \
  --argjson analyticsBytes "$(wc -c </verify/evidence/analytics-details.png)" \
  --arg analyticsDimensions "$(sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/' <<<"$analytics_file")" \
  '{schemaVersion:1,gitHead:$head,sourceTreeState:$sourceTreeState,sourceTreeSha256:$sourceTreeSha256,architecture:$architecture,reportsRunId:$runId,browser:$browser,binaryHashes:$binaries,artifacts:{dashboard:{file:"dashboard.png",sha256:$dashboardSha,bytes:$dashboardBytes,dimensions:$dashboardDimensions},analyticsDetails:{file:"analytics-details.png",sha256:$analyticsSha,bytes:$analyticsBytes,dimensions:$analyticsDimensions}}}' \
  >/verify/evidence/manifest.json

test "$(jq -r .gitHead /verify/evidence/manifest.json)" = "$RUSTZEN_VERIFY_HEAD"
echo "Linux Admin UI browser run passed ($chromium_version)"
