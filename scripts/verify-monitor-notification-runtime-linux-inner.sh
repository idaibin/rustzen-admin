#!/usr/bin/env bash
set -euo pipefail

selected_root=/verify/evidence/runtime-selected
plain_root=/verify/evidence/runtime-plain
admin_public=http://127.0.0.1:19810
ingress=http://127.0.0.1:19811/internal/v1/notification-events
monitor=http://127.0.0.1:19812
owner_password='runtime-owner-password'
event_key='runtime-notification-secret-0123456789'
agent_token='runtime-agent-token-0123456789012345'
pids=()
current_step=bootstrap

step() {
  current_step=$1
  printf 'STEP %s\n' "$current_step" | tee -a /verify/evidence/steps.log
}

cleanup() {
  result=$?
  trap - EXIT INT TERM
  stop_processes
  if [ "$result" -ne 0 ]; then
    echo "FAILED STEP $current_step (exit $result)" >&2
    for log in "$selected_root"/*.log "$plain_root"/*.log; do
      [ ! -s "$log" ] || { echo "== $log ==" >&2; tail -n 80 "$log" >&2; }
    done
  fi
  exit "$result"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

export RUSTZEN_ENV=development RUSTZEN_TIMEZONE=UTC RUST_LOG=warn
export RUSTZEN_BUILD_ID=$(printf 'a%.0s' {1..64})
export RUSTZEN_COMPOSITION_ID=$(printf 'b%.0s' {1..64})
export RUSTZEN_ADMIN_SCHEMA_FINGERPRINT=$(printf 'c%.0s' {1..64})
export RUSTZEN_ADMIN_DATA_CONTRACT_ID=$(printf 'd%.0s' {1..64})
export RUSTZEN_MONITOR_SCHEMA_FINGERPRINT=$(printf 'e%.0s' {1..64})
export RUSTZEN_MONITOR_DATA_CONTRACT_ID=$(printf 'f%.0s' {1..64})
export RUSTZEN_JWT_SECRET='runtime-jwt-secret-01234567890123456789'
export RUSTZEN_IPC_TOKEN='runtime-ipc-secret-01234567890123456789'
export RUSTZEN_MONITOR_AGENT_TOKEN=$agent_token
export RUSTZEN_NOTIFICATION_EVENT_KEY_ID=runtime-v1
export RUSTZEN_NOTIFICATION_EVENT_KEY=$event_key
export RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_INTERNAL_HOST=127.0.0.1

scalar() {
  python3 - "$1" "$2" <<'PY'
import sqlite3,sys
db=sqlite3.connect(sys.argv[1], timeout=5)
value=db.execute(sys.argv[2]).fetchone()[0]
print(value)
PY
}
wait_scalar() {
  db=$1 sql=$2 expected=$3
  for _ in $(seq 1 200); do
    [ "$(scalar "$db" "$sql" 2>/dev/null || true)" = "$expected" ] && return 0
    sleep .05
  done
  echo "timed out waiting for SQLite value: $sql = $expected" >&2
  return 1
}
wait_health() {
  for _ in $(seq 1 200); do
    curl --fail --silent --connect-timeout 1 --max-time 2 "$1/health" >/dev/null 2>&1 && return 0
    sleep .05
  done
  echo "timed out waiting for $1/health" >&2
  return 1
}
stop_processes() {
  for pid in "${pids[@]}"; do kill -TERM "$pid" 2>/dev/null || true; done
  for _ in $(seq 1 100); do
    live=0
    for pid in "${pids[@]}"; do kill -0 "$pid" 2>/dev/null && live=1 || true; done
    [ "$live" = 0 ] && break
    sleep .05
  done
  for pid in "${pids[@]}"; do kill -KILL "$pid" 2>/dev/null || true; done
  for pid in "${pids[@]}"; do wait "$pid" 2>/dev/null || true; done
  pids=()
}
report() {
  sequence=$1 cpu=$2
  collected=$(python3 -c 'from datetime import datetime,timezone; print(datetime.now(timezone.utc).isoformat().replace("+00:00","Z"))')
  jq -nc --argjson sequence "$sequence" --argjson cpu "$cpu" --arg collected "$collected" \
    '{nodeId:"runtime-node",bootId:"11111111-1111-4111-8111-111111111111",sequence:$sequence,hostname:"runtime-node",agentVersion:"runtime",collectedAt:$collected,cpuPercent:$cpu,memory:{usedBytes:1,totalBytes:2},disks:[]}' \
    >"$selected_root/report.json"
  status=$(curl --silent --show-error --connect-timeout 2 --max-time 5 -o "$selected_root/report-response-$sequence.json" -w '%{http_code}' \
    -H 'content-type: application/json' -H "x-rustzen-monitor-agent-token: $agent_token" \
    --data-binary @"$selected_root/report.json" "$admin_public/api/monitor/agent-reports")
  if [ "$status" != 200 ]; then
    echo "agent report $sequence returned HTTP $status" >&2
    cat "$selected_root/report-response-$sequence.json" >&2
    return 1
  fi
  jq -e '.code == 0 and .data.status == "accepted"' "$selected_root/report-response-$sequence.json" >/dev/null
  sleep .02
}

rm -rf "$selected_root" "$plain_root"
mkdir -p "$selected_root/logs" "$plain_root/logs" /verify/evidence
: > /verify/evidence/steps.log
export RUSTZEN_RUNTIME_ROOT=$selected_root
export RUSTZEN_ADMIN_SQLITE_PATH=$selected_root/admin.db
export RUSTZEN_MONITOR_SQLITE_PATH=$selected_root/monitor.db
export RUSTZEN_ADMIN_PORT=19810 RUSTZEN_MONITOR_PORT=19812
export RUSTZEN_NOTIFICATION_INGRESS_PORT=19811
export RUSTZEN_NOTIFICATION_INGRESS_URL=http://127.0.0.1:19813/internal/v1/notification-events

step selected-database-bootstrap
printf '%s\n' "$owner_password" | /verify/bin/rz-admin-notify bootstrap-owner
/verify/bin/rz-admin-notify bind-database
/verify/bin/rz-admin-notify validate-database
/verify/bin/rz-monitor-notify init-db
/verify/bin/rz-monitor-notify bind-database
/verify/bin/rz-monitor-notify validate-database

step selected-entry-and-monitor-start
/verify/bin/rz-monitor-notify controller >"$selected_root/monitor.log" 2>&1 & pids+=("$!")
wait_health "$monitor"
/verify/bin/rz-admin-notify serve >"$selected_root/admin.log" 2>&1 & pids+=("$!")
wait_health "$admin_public"
step incident-open-reports
report 1 95
report 2 95
report 3 95
wait_scalar "$selected_root/monitor.db" 'SELECT COUNT(*) FROM notification_outbox' 1
python3 - "$selected_root/monitor.db" /verify/evidence/open-event.json <<'PY'
import sqlite3,sys
db=sqlite3.connect(sys.argv[1])
body,event,subject=db.execute("SELECT payload_json,event_id,subject_id FROM notification_outbox").fetchone()
open(sys.argv[2],"w").write(body)
open(sys.argv[2]+".identity","w").write(event+"\t"+subject+"\n")
PY
open_event=$(cut -f1 /verify/evidence/open-event.json.identity)
subject=$(cut -f2 /verify/evidence/open-event.json.identity)

step relay-restart-after-producer-commit
stop_processes
export RUSTZEN_NOTIFICATION_INGRESS_URL=$ingress
/verify/bin/rz-monitor-notify controller >>"$selected_root/monitor.log" 2>&1 & pids+=("$!")
wait_health "$monitor"
/verify/bin/rz-admin-notify serve >>"$selected_root/admin.log" 2>&1 & pids+=("$!")
wait_health "$admin_public"
ss -H -ltn 'sport = :19811' > /verify/evidence/selected-ingress-listener.txt
grep -Eq '127\.0\.0\.1:19811' /verify/evidence/selected-ingress-listener.txt
wait_scalar "$selected_root/admin.db" "SELECT COUNT(*) FROM notification_receipts WHERE event_id='$open_event'" 1
wait_scalar "$selected_root/monitor.db" 'SELECT COUNT(*) FROM notification_outbox' 0
step owner-login
login_status=$(curl --silent --show-error -o "$selected_root/login.json" -w '%{http_code}' \
  -H 'content-type: application/json' -d "{\"username\":\"owner\",\"password\":\"$owner_password\"}" \
  "$admin_public/api/auth/login")
if [ "$login_status" != 200 ]; then
  echo "owner login returned HTTP $login_status" >&2
  cat "$selected_root/login.json" >&2
  exit 1
fi
login=$(cat "$selected_root/login.json")
token=$(jq -er '.data.token | select(length > 20)' <<<"$login")
step opened-inbox
inbox_status=$(curl --silent --show-error -o /verify/evidence/inbox-open.json -w '%{http_code}' \
  -H "authorization: Bearer $token" "$admin_public/api/notifications?limit=100")
[ "$inbox_status" = 200 ] || { echo "opened inbox returned HTTP $inbox_status" >&2; cat /verify/evidence/inbox-open.json >&2; exit 1; }
jq -e --arg event "$open_event" --arg subject "$subject" \
  '.data.items | length == 1 and .[0].topic == "monitor.incident.opened" and .[0].subjectId == $subject and .[0].subjectRevision == 1' \
  /verify/evidence/inbox-open.json >/dev/null

step duplicate-and-auth-denials
python3 /verify/client.py --url "$ingress" --body /verify/evidence/open-event.json \
  > /verify/evidence/duplicate.json
jq -e '.status == 200 and .contentType == "application/json" and .body.code == "duplicate"' \
  /verify/evidence/duplicate.json >/dev/null
python3 /verify/client.py --url "$ingress" --body /verify/evidence/open-event.json --bad-signature \
  > /verify/evidence/bad-signature.json
jq -e '.status == 401 and .body.code == "bad-producer"' /verify/evidence/bad-signature.json >/dev/null
unsigned_status=$(curl --silent --show-error -o /verify/evidence/unsigned.json -w '%{http_code}' \
  -H 'content-type: application/json' --data-binary @/verify/evidence/open-event.json "$ingress")
[ "$unsigned_status" = 400 ]
jq -e '.code == "invalid-protocol"' /verify/evidence/unsigned.json >/dev/null
python3 /verify/client.py --url "$admin_public/internal/v1/notification-events" \
  --body /verify/evidence/open-event.json > /verify/evidence/public-internal.json
jq -e '.status == 404 and .contentType == "application/json" and .body.code == 10001' \
  /verify/evidence/public-internal.json >/dev/null
[ "$(scalar "$selected_root/admin.db" 'SELECT COUNT(*) FROM notification_receipts')" = 1 ]
[ "$(scalar "$selected_root/admin.db" 'SELECT COUNT(*) FROM notifications')" = 1 ]

step incident-resolve-reports
report 4 10
report 5 10
report 6 10
wait_scalar "$selected_root/admin.db" 'SELECT COUNT(*) FROM notification_receipts' 2
wait_scalar "$selected_root/monitor.db" 'SELECT COUNT(*) FROM notification_outbox' 0
step resolved-inbox
inbox_status=$(curl --silent --show-error -o /verify/evidence/inbox-final.json -w '%{http_code}' \
  -H "authorization: Bearer $token" "$admin_public/api/notifications?limit=100")
[ "$inbox_status" = 200 ] || { echo "resolved inbox returned HTTP $inbox_status" >&2; cat /verify/evidence/inbox-final.json >&2; exit 1; }
jq -e --arg subject "$subject" '
  .data.items | length == 2 and
  [.[].topic] == ["monitor.incident.resolved","monitor.incident.opened"] and
  [.[].subjectId] == [$subject,$subject] and [.[].subjectRevision] == [2,1]
' /verify/evidence/inbox-final.json >/dev/null
python3 - "$selected_root/admin.db" "$selected_root/monitor.db" > /verify/evidence/selected-state.json <<'PY'
import json,sqlite3,sys
a=sqlite3.connect(sys.argv[1]); m=sqlite3.connect(sys.argv[2])
rows=a.execute("SELECT event_id,result FROM notification_receipts ORDER BY accepted_at,event_id").fetchall()
messages=a.execute("SELECT topic,subject_id,subject_revision FROM notifications ORDER BY inbox_seq").fetchall()
print(json.dumps({"receipts":rows,"messages":messages,"recipients":a.execute("SELECT COUNT(*) FROM notification_recipients").fetchone()[0],"outbox":m.execute("SELECT COUNT(*) FROM notification_outbox").fetchone()[0]},separators=(",",":")))
PY
jq -e '.receipts | length == 2 and all(.[1] == "stored")' /verify/evidence/selected-state.json >/dev/null
jq -e --arg subject "$subject" '.messages == [["monitor.incident.opened",$subject,1],["monitor.incident.resolved",$subject,2]] and .recipients == 2 and .outbox == 0' /verify/evidence/selected-state.json >/dev/null
stop_processes

step pure-monitor-absence
export RUSTZEN_RUNTIME_ROOT=$plain_root
export RUSTZEN_ADMIN_SQLITE_PATH=$plain_root/admin.db RUSTZEN_MONITOR_SQLITE_PATH=$plain_root/monitor.db
export RUSTZEN_ADMIN_PORT=19820 RUSTZEN_MONITOR_PORT=19821 RUSTZEN_NOTIFICATION_INGRESS_PORT=19822
printf '%s\n' "$owner_password" | /verify/bin/rz-admin-pure bootstrap-owner
/verify/bin/rz-admin-pure bind-database
/verify/bin/rz-monitor-pure init-db
/verify/bin/rz-monitor-pure bind-database
/verify/bin/rz-admin-pure contract selected admin > /verify/evidence/plain-admin-api.json
/verify/bin/rz-admin-pure contract config selected access > /verify/evidence/plain-admin-config.json
/verify/bin/rz-monitor-pure contract selected > /verify/evidence/plain-monitor-api.json
/verify/bin/rz-monitor-pure contract config selected > /verify/evidence/plain-monitor-config.json
if /verify/bin/rz-admin-pure contract selected notifications >/dev/null 2>&1; then exit 1; fi
if /verify/bin/rz-admin-pure contract config selected notifications >/dev/null 2>&1; then exit 1; fi
! grep -Eqi 'notification|/api/notifications' /verify/evidence/plain-*.json
python3 - "$plain_root/admin.db" "$plain_root/monitor.db" > /verify/evidence/plain-absence.json <<'PY'
import json,sqlite3,sys
result={}
for name,path in (("admin",sys.argv[1]),("monitor",sys.argv[2])):
 db=sqlite3.connect(path)
 result[name]=[row[0] for row in db.execute("SELECT name FROM sqlite_master WHERE name LIKE 'notification%' OR name LIKE '%notifications_migrations' ORDER BY name")]
print(json.dumps(result,separators=(",",":")))
PY
jq -e '.admin == [] and .monitor == []' /verify/evidence/plain-absence.json >/dev/null
/verify/bin/rz-admin-pure serve >"$plain_root/admin.log" 2>&1 & pids+=("$!")
/verify/bin/rz-monitor-pure controller >"$plain_root/monitor.log" 2>&1 & pids+=("$!")
wait_health http://127.0.0.1:19820
wait_health http://127.0.0.1:19821
sleep 1
! ss -H -ltn 'sport = :19822' | grep -q .
! ss -H -tn '( sport = :19822 or dport = :19822 )' | grep -q .
plain_status=$(curl --silent -o /verify/evidence/plain-notification-route.json -w '%{http_code}' \
  http://127.0.0.1:19820/api/notifications)
[ "$plain_status" = 404 ]
jq -e '.code == 404 and .message == "Not found" and .data == null' \
  /verify/evidence/plain-notification-route.json >/dev/null
ss -H -ltn > /verify/evidence/plain-listeners.txt
stop_processes

step manifest
rm -rf "$selected_root" "$plain_root"

receipts=$(for file in steps.log open-event.json open-event.json.identity selected-ingress-listener.txt inbox-open.json duplicate.json bad-signature.json unsigned.json public-internal.json inbox-final.json selected-state.json plain-admin-api.json plain-admin-config.json plain-monitor-api.json plain-monitor-config.json plain-absence.json plain-notification-route.json plain-listeners.txt; do
  jq -nc --arg file "$file" --arg sha "$(sha256sum "/verify/evidence/$file" | awk '{print $1}')" --argjson bytes "$(stat -c %s "/verify/evidence/$file")" '{file:$file,sha256:$sha,bytes:$bytes}'
done | jq -s .)
jq -n --arg head "$RUSTZEN_VERIFY_HEAD" --arg state "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" \
  --arg sourceSha "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" --arg architecture "$RUSTZEN_VERIFY_ARCHITECTURE" \
  --arg platform "$RUSTZEN_VERIFY_PLATFORM" --argjson binaries "$RUSTZEN_VERIFY_BINARY_HASHES" \
  --arg provenanceSha "$RUSTZEN_VERIFY_BUILD_PROVENANCE_SHA256" \
  --arg verifierImage "$RUSTZEN_VERIFY_VERIFIER_IMAGE_ID" --arg verifierKey "$RUSTZEN_VERIFY_VERIFIER_KEY" \
  --arg verifierSha "$RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256" \
  --arg openEvent "$open_event" --arg subject "$subject" --argjson receipts "$receipts" \
  '{schemaVersion:1,status:"passed",gitHead:$head,sourceTreeState:$state,sourceTreeSha256:$sourceSha,
    platform:{architecture:$architecture,name:$platform},binaryHashes:$binaries,buildProvenanceSha256:$provenanceSha,
    verifier:{imageId:$verifierImage,key:$verifierKey,provenanceSha256:$verifierSha},
    delivery:{openEventId:$openEvent,subjectId:$subject,opened:"stored",duplicate:"duplicate",resolved:"stored",receiptCount:2,messageCount:2,recipientCount:2},
    authentication:{unsigned:400,badSignature:401,publicInternal:404},
    pureMonitor:{adminNotificationOwner:false,monitorNotificationOwner:false,notificationConfig:false,notificationRoutes:false,relayTask:false,notificationSchemaObjects:0,notificationListeners:0},
    receipts:$receipts,limits:["disposable shared-kernel container","no systemd or production deployment","no Reports, SSE, UI or sustained load"]}' \
  > /verify/evidence/manifest.json
jq -e '.status == "passed" and .delivery == (.delivery + {receiptCount:2,messageCount:2,recipientCount:2}) and .authentication == {unsigned:400,badSignature:401,publicInternal:404} and .pureMonitor.notificationSchemaObjects == 0 and .pureMonitor.notificationListeners == 0' /verify/evidence/manifest.json >/dev/null
echo 'Monitor notification Linux runtime gate passed'
