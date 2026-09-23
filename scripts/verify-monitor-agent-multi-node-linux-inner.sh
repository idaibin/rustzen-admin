#!/usr/bin/env bash
set -euo pipefail

for command in curl jq setpriv ss python3; do command -v "$command" >/dev/null; done
for port in 19801 19802; do
  if ss -H -ltn "sport = :$port" | grep -q .; then
    echo "Monitor multi-node verification port is already occupied: $port" >&2
    exit 1
  fi
done

groupadd -g 2410 rz-central
useradd -u 2410 -g rz-central -M -s /usr/sbin/nologin rz-central
groupadd -g 2411 rz-agent-a
useradd -u 2411 -g rz-agent-a -M -s /usr/sbin/nologin rz-agent-a
groupadd -g 2412 rz-agent-b
useradd -u 2412 -g rz-agent-b -M -s /usr/sbin/nologin rz-agent-b

install -d -m 0750 -o rz-central -g rz-central /opt/rz-central /opt/rz-central/data /opt/rz-central/logs
install -m 0755 /verify/bin/rz-admin /opt/rz-central/rz-admin
install -m 0755 /verify/bin/rz-monitor /opt/rz-central/rz-monitor
chown rz-central:rz-central /opt/rz-central/rz-admin /opt/rz-central/rz-monitor
for identity in a b; do
  install -d -m 0750 -o "rz-agent-$identity" -g "rz-agent-$identity" "/var/lib/rz-agent-$identity"
  install -d -m 0750 -o "rz-agent-$identity" -g "rz-agent-$identity" "/var/lib/rz-agent-$identity/logs"
done

export RUSTZEN_ENV=development RUSTZEN_INTERNAL_HOST=127.0.0.1
export RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_ADMIN_PORT=19801 RUSTZEN_MONITOR_PORT=19802
export RUSTZEN_ADMIN_SQLITE_PATH=/opt/rz-central/data/admin.db RUSTZEN_MONITOR_SQLITE_PATH=/opt/rz-central/data/monitor.db
export RUSTZEN_JWT_SECRET=monitor-multi-node-verification-jwt RUSTZEN_IPC_TOKEN=monitor-multi-node-verification-ipc
export RUSTZEN_MONITOR_AGENT_TOKEN=monitor-multi-node-verification-agent RUSTZEN_TIMEZONE=UTC RUST_LOG=warn
export RUSTZEN_BUILD_ID=$(printf 'a%.0s' {1..64}) RUSTZEN_COMPOSITION_ID=$(printf 'b%.0s' {1..64})
export RUSTZEN_MONITOR_SCHEMA_FINGERPRINT=$(printf 'c%.0s' {1..64}) RUSTZEN_MONITOR_DATA_CONTRACT_ID=$(printf 'd%.0s' {1..64})

pids=()
admin_pid=
monitor_pid=
cleanup() {
  status=$?
  trap - EXIT INT TERM
  for pid in "$admin_pid" "$monitor_pid" "${pids[@]}"; do [ -n "$pid" ] && kill -TERM "$pid" 2>/dev/null || true; done
  for pid in "$admin_pid" "$monitor_pid" "${pids[@]}"; do [ -n "$pid" ] && wait "$pid" 2>/dev/null || true; done
  if [ "$status" -ne 0 ]; then
    for log in /opt/rz-central/logs/*.log /var/lib/rz-agent-*/logs/*.log; do
      [ -s "$log" ] && { echo "== $log ==" >&2; tail -n 80 "$log" >&2; }
    done
  fi
  exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

central_env() {
  setpriv --reuid=rz-central --regid=rz-central --init-groups --no-new-privs -- env \
    RUSTZEN_ENV="$RUSTZEN_ENV" RUSTZEN_RUNTIME_ROOT=/opt/rz-central RUSTZEN_INTERNAL_HOST="$RUSTZEN_INTERNAL_HOST" \
    RUSTZEN_ADMIN_HOST="$RUSTZEN_ADMIN_HOST" RUSTZEN_ADMIN_PORT="$RUSTZEN_ADMIN_PORT" RUSTZEN_MONITOR_PORT="$RUSTZEN_MONITOR_PORT" \
    RUSTZEN_ADMIN_SQLITE_PATH="$RUSTZEN_ADMIN_SQLITE_PATH" RUSTZEN_MONITOR_SQLITE_PATH="$RUSTZEN_MONITOR_SQLITE_PATH" \
    RUSTZEN_JWT_SECRET="$RUSTZEN_JWT_SECRET" RUSTZEN_IPC_TOKEN="$RUSTZEN_IPC_TOKEN" RUSTZEN_MONITOR_AGENT_TOKEN="$RUSTZEN_MONITOR_AGENT_TOKEN" \
    RUSTZEN_TIMEZONE=UTC RUST_LOG=warn RUSTZEN_BUILD_ID="$RUSTZEN_BUILD_ID" RUSTZEN_COMPOSITION_ID="$RUSTZEN_COMPOSITION_ID" \
    RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_MONITOR_SCHEMA_FINGERPRINT" RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_MONITOR_DATA_CONTRACT_ID" "$@"
}
start_central() {
  ( exec setpriv --reuid=rz-central --regid=rz-central --init-groups --no-new-privs -- env RUSTZEN_ENV="$RUSTZEN_ENV" RUSTZEN_RUNTIME_ROOT=/opt/rz-central RUSTZEN_INTERNAL_HOST="$RUSTZEN_INTERNAL_HOST" RUSTZEN_ADMIN_HOST="$RUSTZEN_ADMIN_HOST" RUSTZEN_ADMIN_PORT="$RUSTZEN_ADMIN_PORT" RUSTZEN_MONITOR_PORT="$RUSTZEN_MONITOR_PORT" RUSTZEN_ADMIN_SQLITE_PATH="$RUSTZEN_ADMIN_SQLITE_PATH" RUSTZEN_MONITOR_SQLITE_PATH="$RUSTZEN_MONITOR_SQLITE_PATH" RUSTZEN_JWT_SECRET="$RUSTZEN_JWT_SECRET" RUSTZEN_IPC_TOKEN="$RUSTZEN_IPC_TOKEN" RUSTZEN_MONITOR_AGENT_TOKEN="$RUSTZEN_MONITOR_AGENT_TOKEN" RUSTZEN_TIMEZONE=UTC RUST_LOG=warn RUSTZEN_BUILD_ID="$RUSTZEN_BUILD_ID" RUSTZEN_COMPOSITION_ID="$RUSTZEN_COMPOSITION_ID" RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_MONITOR_SCHEMA_FINGERPRINT" RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_MONITOR_DATA_CONTRACT_ID" /opt/rz-central/rz-monitor controller ) >/opt/rz-central/logs/monitor.log 2>&1 & monitor_pid=$!
  ( exec setpriv --reuid=rz-central --regid=rz-central --init-groups --no-new-privs -- env RUSTZEN_ENV="$RUSTZEN_ENV" RUSTZEN_RUNTIME_ROOT=/opt/rz-central RUSTZEN_INTERNAL_HOST="$RUSTZEN_INTERNAL_HOST" RUSTZEN_ADMIN_HOST="$RUSTZEN_ADMIN_HOST" RUSTZEN_ADMIN_PORT="$RUSTZEN_ADMIN_PORT" RUSTZEN_MONITOR_PORT="$RUSTZEN_MONITOR_PORT" RUSTZEN_ADMIN_SQLITE_PATH="$RUSTZEN_ADMIN_SQLITE_PATH" RUSTZEN_MONITOR_SQLITE_PATH="$RUSTZEN_MONITOR_SQLITE_PATH" RUSTZEN_JWT_SECRET="$RUSTZEN_JWT_SECRET" RUSTZEN_IPC_TOKEN="$RUSTZEN_IPC_TOKEN" RUSTZEN_MONITOR_AGENT_TOKEN="$RUSTZEN_MONITOR_AGENT_TOKEN" RUSTZEN_TIMEZONE=UTC RUST_LOG=warn RUSTZEN_BUILD_ID="$RUSTZEN_BUILD_ID" RUSTZEN_COMPOSITION_ID="$RUSTZEN_COMPOSITION_ID" RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_MONITOR_SCHEMA_FINGERPRINT" RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_MONITOR_DATA_CONTRACT_ID" /opt/rz-central/rz-admin serve ) >/opt/rz-central/logs/admin.log 2>&1 & admin_pid=$!
}
stop_central() {
  for pid in "$admin_pid" "$monitor_pid"; do kill -TERM "$pid" 2>/dev/null || true; done
  for _ in $(seq 1 100); do
    kill -0 "$admin_pid" 2>/dev/null || kill -0 "$monitor_pid" 2>/dev/null || break
    sleep .1
  done
  for pid in "$admin_pid" "$monitor_pid"; do
    if kill -0 "$pid" 2>/dev/null; then kill -KILL "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; echo "central process did not stop after TERM: $pid" >&2; return 1; fi
    wait "$pid" 2>/dev/null || true
  done
  for port in 19801 19802; do
    for _ in $(seq 1 100); do ! ss -H -ltn "sport = :$port" | grep -q . && break || sleep .1; done
    if ss -H -ltn "sport = :$port" | grep -q .; then echo "central port did not quiesce: $port" >&2; return 1; fi
  done
  admin_pid=
  monitor_pid=
}
wait_health() {
  for url in http://127.0.0.1:19801/health http://127.0.0.1:19802/health; do
    for _ in $(seq 1 180); do curl --fail --silent --connect-timeout 2 --max-time 3 "$url" >/dev/null 2>&1 && break || sleep .1; done
    curl --fail --silent --connect-timeout 2 --max-time 3 "$url" >/dev/null
  done
}
start_agent() {
  identity=$1
  setpriv --reuid="rz-agent-$identity" --regid="rz-agent-$identity" --init-groups --no-new-privs -- env \
    RUSTZEN_ENV=development RUSTZEN_RUNTIME_ROOT="/var/lib/rz-agent-$identity" RUSTZEN_ADMIN_PORT=19801 \
    RUSTZEN_MONITOR_AGENT_TOKEN="$RUSTZEN_MONITOR_AGENT_TOKEN" RUSTZEN_MONITOR_NODE_ID="linux-agent-$identity" \
    RUSTZEN_MONITOR_CONTROLLER_URL=http://127.0.0.1:19801 NOTIFY_SOCKET="/run/rz-agent-$identity.notify" RUST_LOG=warn \
    /verify/bin/rz-monitor-agent >"/var/lib/rz-agent-$identity/logs/launch.log" 2>&1 &
  pids+=("$!")
}

readiness=/verify/evidence/readiness.jsonl
python3 /verify/readiness.py --socket /run/rz-agent-a.notify --socket /run/rz-agent-b.notify --evidence "$readiness" >/verify/evidence/readiness-listener.log 2>&1 &
pids+=("$!")
start_agent a
start_agent b
sleep 4
test ! -s "$readiness"

central_env /opt/rz-central/rz-monitor init-db
central_env /opt/rz-central/rz-monitor bind-database
start_central
wait_health
python3 /verify/readiness.py --wait --evidence "$readiness" --expected 2 --timeout 90
jq -e 'length == 2 and ([.[].payload] | all(. == "READY=1")) and ([.[].socket] | sort) == ["rz-agent-a.notify", "rz-agent-b.notify"]' <(jq -s . "$readiness") >/dev/null

admin=http://127.0.0.1:19801
login=$(curl --fail --silent --show-error --connect-timeout 2 --max-time 10 -H 'content-type: application/json' -d '{"username":"owner","password":"rustzen@123"}' "$admin/api/auth/login")
token=$(jq -er '.data.token | select(length > 20)' <<<"$login")
auth=(-H "authorization: Bearer $token")
nodes=$(curl --fail --silent --show-error --connect-timeout 2 --max-time 10 "${auth[@]}" "$admin/api/monitor/nodes")
jq -e '.data | length == 2 and ([.[].nodeId] | sort) == ["linux-agent-a", "linux-agent-b"] and ([.[].bootId] | unique | length) == 2 and ([.[] | select((.hostname|type == "string" and length > 0) and (.agentVersion|type == "string" and length > 0) and (.status == "online") and (.lastReportAt|type == "string" and test("T")) and (.lastReceivedAt|type == "string" and test("T")) and (.cpuPercent|type == "number" and . >= 0 and . <= 100) and (.memory.usedBytes|type == "number" and . >= 0) and (.memory.totalBytes|type == "number" and . > 0) and (.disks|type == "array") )] | length) == 2' <<<"$nodes" >/dev/null
boot_a=$(jq -er '.data[] | select(.nodeId == "linux-agent-a") | .bootId' <<<"$nodes")
boot_b=$(jq -er '.data[] | select(.nodeId == "linux-agent-b") | .bootId' <<<"$nodes")
seq_a=$(jq -er '.data[] | select(.nodeId == "linux-agent-a") | .sequence' <<<"$nodes")
seq_b=$(jq -er '.data[] | select(.nodeId == "linux-agent-b") | .sequence' <<<"$nodes")
declare -A points_before
declare -A points_after
for identity in a b; do
  metrics=$(curl --fail --silent --show-error --connect-timeout 2 --max-time 10 "${auth[@]}" "$admin/api/monitor/nodes/linux-agent-$identity/metrics?bucket=raw")
  jq -e '.data.bucket == "raw" and (.data.points | length) >= 1 and ([.data.points[] | select((.collectedAt|type == "string" and test("T")) and (.cpuPercent|type == "number" and . >= 0 and . <= 100) and (.memoryPercent|type == "number" and . >= 0 and . <= 100))] | length) == (.data.points | length)' <<<"$metrics" >/dev/null
  points_before[$identity]=$(jq -er '.data.points | length' <<<"$metrics")
done
test -n "$(find /var/lib/rz-agent-a/logs -type f -size +0c -print -quit)"
test -n "$(find /var/lib/rz-agent-b/logs -type f -size +0c -print -quit)"

stop_central
start_central
wait_health
recovered=0
for _ in $(seq 1 480); do
  nodes_after=$(curl --fail --silent --connect-timeout 2 --max-time 10 "${auth[@]}" "$admin/api/monitor/nodes" 2>/dev/null || true)
  if jq -e --arg boot_a "$boot_a" --arg boot_b "$boot_b" --argjson seq_a "$seq_a" --argjson seq_b "$seq_b" '.data | length == 2 and ([.[].nodeId] | sort) == ["linux-agent-a", "linux-agent-b"] and (.[] | select(.nodeId == "linux-agent-a") | .bootId == $boot_a and .sequence > $seq_a) and (.[] | select(.nodeId == "linux-agent-b") | .bootId == $boot_b and .sequence > $seq_b)' <<<"$nodes_after" >/dev/null 2>&1; then recovered=1; break; fi
  sleep .1
done
test "$recovered" = 1
for identity in a b; do
  recovered_metrics=$(curl --fail --silent --show-error --connect-timeout 2 --max-time 10 "${auth[@]}" "$admin/api/monitor/nodes/linux-agent-$identity/metrics?bucket=raw")
  before_points=${points_before[$identity]}
  jq -e --argjson before "$before_points" '.data.bucket == "raw" and (.data.points | length) > $before' <<<"$recovered_metrics" >/dev/null
  points_after[$identity]=$(jq -er '.data.points | length' <<<"$recovered_metrics")
done

jq -n \
  --arg head "$RUSTZEN_VERIFY_HEAD" --arg sourceTreeState "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" --arg sourceTreeSha256 "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" \
  --arg architecture "$RUSTZEN_VERIFY_ARCHITECTURE" --arg platform "$RUSTZEN_VERIFY_PLATFORM" \
  --arg adminHash "$RUSTZEN_VERIFY_ADMIN_SHA256" --arg monitorHash "$RUSTZEN_VERIFY_MONITOR_SHA256" --arg agentHash "$RUSTZEN_VERIFY_AGENT_SHA256" \
  --arg bootA "$boot_a" --arg bootB "$boot_b" --argjson readiness "$(jq -s . "$readiness")" \
  --argjson uidA "$(id -u rz-agent-a)" --argjson uidB "$(id -u rz-agent-b)" --argjson beforeA "${points_before[a]}" --argjson beforeB "${points_before[b]}" --argjson afterA "${points_after[a]}" --argjson afterB "${points_after[b]}" \
  '{schemaVersion:1,status:"partial",gitHead:$head,sourceTreeState:$sourceTreeState,sourceTreeSha256:$sourceTreeSha256,platform:{architecture:$architecture,platform:$platform},binaries:{admin:$adminHash,monitor:$monitorHash,agent:$agentHash},serviceUsers:[{name:"rz-agent-a",uid:$uidA,runtimeRoot:"/var/lib/rz-agent-a",logDirectory:"/var/lib/rz-agent-a/logs"},{name:"rz-agent-b",uid:$uidB,runtimeRoot:"/var/lib/rz-agent-b",logDirectory:"/var/lib/rz-agent-b/logs"}],readiness:{beforeCentral:"passed",events:$readiness},queries:{nodes:"passed",nodesLatestFields:"passed",rawMetrics:{status:"passed",points:{"linux-agent-a":{before:$beforeA,after:$afterA},"linux-agent-b":{before:$beforeB,after:$afterB}}}},recovery:{centralRestart:"passed",nodeCount:2,thirdNodeRegistered:false,bootIds:[$bootA,$bootB]},limits:["shared container kernel", "no systemd PID 1", "no remote-host or production TLS verification"]}' \
  >/verify/evidence/manifest.json
jq -e '.status == "partial" and (.serviceUsers | length) == 2 and (.readiness.events | length) == 2 and .recovery.thirdNodeRegistered == false and (.recovery.bootIds | unique | length) == 2' /verify/evidence/manifest.json >/dev/null
echo "Monitor dual-Agent Linux runtime gate passed (real processes and service users; systemd not verified)"
