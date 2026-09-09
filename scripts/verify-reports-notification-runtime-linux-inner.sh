#!/usr/bin/env bash
set -euo pipefail

runtime_base=/tmp/rz-reports-notification-runtime
selected=$runtime_base/selected
pure=$runtime_base/pure
reports_user=rz-reports
reports_home=$selected/reports
reports_db=$reports_home/db/reports.db
reports_log=$reports_home/logs/reports/service.log
pure_reports_home=$pure/reports
pure_reports_db=$pure_reports_home/db/reports.db
pure_reports_log=$pure_reports_home/logs/reports/service.log
admin=http://127.0.0.1:19830
ingress=http://127.0.0.1:19831/internal/v1/notification-events
reports=http://127.0.0.1:19834
reports_key=reports-runtime-notification-secret
pids=(); admin_pid=; reports_pid=; current_step=bootstrap

step(){ current_step=$1; printf 'STEP %s\n' "$1" | tee -a /verify/evidence/steps.log; }
scalar(){ python3 - "$1" "$2" <<'PY'
import sqlite3,sys
print(sqlite3.connect(sys.argv[1],timeout=5).execute(sys.argv[2]).fetchone()[0])
PY
}
execute(){ python3 - "$@" <<'PY'
import sqlite3,sys,time
db,query,*args=sys.argv[1:]
with sqlite3.connect(db,timeout=5) as connection:
    # The verifier image's Python SQLite can predate schema triggers using unixepoch().
    connection.create_function("unixepoch", 0, lambda: int(time.time()))
    connection.execute(query,args)
PY
}
wait_scalar(){ for _ in $(seq 1 400); do [ "$(scalar "$1" "$2" 2>/dev/null || true)" = "$3" ] && return; sleep .05; done; echo "timeout: $2 = $3" >&2; return 1; }
wait_health(){ for _ in $(seq 1 300); do curl -fsS --connect-timeout 1 --max-time 2 "$1/health" >/dev/null 2>&1 && return; sleep .05; done; return 1; }
stop_pid(){ local pid=${1:-}; [ -z "$pid" ] && return; kill -TERM "$pid" 2>/dev/null || true; for _ in $(seq 1 100); do kill -0 "$pid" 2>/dev/null || break; sleep .05; done; kill -0 "$pid" 2>/dev/null && kill -KILL "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; for i in "${!pids[@]}"; do [ "${pids[$i]}" != "$pid" ] || unset 'pids[i]'; done; }
stop_all(){ for pid in "${pids[@]}"; do stop_pid "$pid"; done; pids=(); admin_pid=; reports_pid=; }
preserve_failure_logs(){ mkdir -p /verify/evidence/failure-logs; for pair in "selected-admin:$selected/admin.log" "selected-reports:$reports_log" "selected-fixture:$selected/fixture.log" "selected-proxy:$selected/proxy.log" "pure-admin:$pure/admin.log" "pure-reports:$pure_reports_log"; do name=${pair%%:*}; file=${pair#*:}; [ ! -f "$file" ] || cp "$file" "/verify/evidence/failure-logs/$name.log"; done; }
cleanup(){ result=$?; trap - EXIT INT TERM; stop_all; if [ "$result" -ne 0 ]; then preserve_failure_logs; echo "FAILED STEP $current_step ($result)" >&2; tail -n 80 "$selected"/*.log "$reports_log" "$pure"/*.log "$pure_reports_log" 2>/dev/null || true; fi; exit "$result"; }
trap cleanup EXIT; trap 'exit 130' INT; trap 'exit 143' TERM

for command in getent groupadd useradd setpriv; do command -v "$command" >/dev/null; done
getent group "$reports_user" >/dev/null || groupadd --system "$reports_user"
id "$reports_user" >/dev/null 2>&1 || useradd --system --gid "$reports_user" --home-dir "$reports_home" --shell /usr/sbin/nologin "$reports_user"
prepare_reports_layout(){ local home=$1; install -d -m 0750 -o "$reports_user" -g "$reports_user" "$home" "$home/db" "$home/logs" "$home/logs/reports" "$home/data" "$home/data/reports" "$home/.config" "$home/.cache"; }
rm -rf "$runtime_base"; install -d -m 0711 -o root -g root "$runtime_base" "$selected" "$pure"; prepare_reports_layout "$reports_home"; prepare_reports_layout "$pure_reports_home"
install -m 0600 -o "$reports_user" -g "$reports_user" /dev/null "$reports_log"; install -m 0600 -o "$reports_user" -g "$reports_user" /dev/null "$pure_reports_log"
mkdir -p "$selected/admin" "$pure/admin" /verify/evidence/fixture
: >/verify/evidence/steps.log; printf '<main id="status">reports fixture ready</main>' >/verify/evidence/fixture/index.html
python3 -m http.server 18080 --bind 127.0.0.1 --directory /verify/evidence/fixture >"$selected/fixture.log" 2>&1 & pids+=("$!")
export HOME=$selected/admin XDG_CONFIG_HOME=$selected/admin/.config XDG_CACHE_HOME=$selected/admin/.cache
export RUSTZEN_ENV=development RUSTZEN_TIMEZONE=UTC RUST_LOG=warn RUSTZEN_RUNTIME_ROOT=$selected/admin
export RUSTZEN_ADMIN_SQLITE_PATH=$selected/admin.db RUSTZEN_REPORTS_SQLITE_PATH=$reports_db
export RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_ADMIN_PORT=19830 RUSTZEN_INTERNAL_HOST=127.0.0.1 RUSTZEN_REPORTS_PORT=19834
export RUSTZEN_NOTIFICATION_INGRESS_PORT=19831 RUSTZEN_REPORTS_NOTIFICATION_INGRESS_URL=$ingress
export RUSTZEN_JWT_SECRET=reports-runtime-jwt-secret-0123456789 RUSTZEN_IPC_TOKEN=reports-runtime-ipc-secret-0123456789
export RUSTZEN_REPORTS_CREDENTIAL_KEY=reports-runtime-credential-secret RUSTZEN_REPORTS_BROWSER_PATH=/usr/bin/chromium RUSTZEN_REPORTS_MAX_CONCURRENCY=1
export RUSTZEN_NOTIFICATION_EVENT_KEY_ID=monitor-runtime-v1 RUSTZEN_NOTIFICATION_EVENT_KEY=monitor-runtime-notification-secret-001
export RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY_ID=reports-runtime-v1 RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY=$reports_key
export RUSTZEN_MONITOR_AGENT_TOKEN=reports-runtime-agent-secret-001
export RUSTZEN_BUILD_ID=$(printf 'a%.0s' {1..64}) RUSTZEN_COMPOSITION_ID=$(printf 'b%.0s' {1..64})
export RUSTZEN_ADMIN_SCHEMA_FINGERPRINT=$(printf 'c%.0s' {1..64}) RUSTZEN_ADMIN_DATA_CONTRACT_ID=$(printf 'd%.0s' {1..64})
export RUSTZEN_MONITOR_SCHEMA_FINGERPRINT=$(printf 'e%.0s' {1..64}) RUSTZEN_MONITOR_DATA_CONTRACT_ID=$(printf 'f%.0s' {1..64})
export RUSTZEN_REPORTS_SCHEMA_FINGERPRINT=$(printf '1%.0s' {1..64}) RUSTZEN_REPORTS_DATA_CONTRACT_ID=$(printf '2%.0s' {1..64})

start_admin(){ /verify/bin/rz-admin-selected serve >>"$selected/admin.log" 2>&1 & admin_pid=$!; pids+=("$admin_pid"); wait_health "$admin"; }
start_reports(){ local binary=${1:-/verify/bin/rz-reports-selected} home=${2:-$reports_home} db=${3:-$reports_db} log=${4:-$reports_log} health=${5:-$reports}; setpriv --reuid="$reports_user" --regid="$reports_user" --init-groups --no-new-privs -- env HOME="$home" XDG_CONFIG_HOME="$home/.config" XDG_CACHE_HOME="$home/.cache" RUSTZEN_RUNTIME_ROOT="$home" RUSTZEN_REPORTS_SQLITE_PATH="$db" "$binary" serve >>"$log" 2>&1 & reports_pid=$!; pids+=("$reports_pid"); wait_health "$health"; }
capture_reports_identity(){ local pid=$1 home=$2 output=$3 uid gid directories='[]' kind path stat_row; uid=$(awk '/^Uid:/{print $2}' "/proc/$pid/status"); gid=$(awk '/^Gid:/{print $2}' "/proc/$pid/status"); [ "$uid" -gt 0 ] && [ "$uid" = "$(id -u "$reports_user")" ] && [ "$gid" = "$(id -g "$reports_user")" ]; for entry in "runtime:$home" "db:$home/db" "log:$home/logs/reports" "artifact:$home/data/reports"; do kind=${entry%%:*}; path=${entry#*:}; stat_row=$(stat -c '%u %g %a' "$path"); read -r dir_uid dir_gid mode <<<"$stat_row"; [ "$dir_uid:$dir_gid:$mode" = "$uid:$gid:750" ]; directories=$(jq -nc --argjson current "$directories" --arg kind "$kind" --arg path "$path" --argjson uid "$dir_uid" --argjson gid "$dir_gid" --arg mode "0$mode" '$current+[{kind:$kind,path:$path,uid:$uid,gid:$gid,mode:$mode}]'); done; jq -nc --argjson uid "$uid" --argjson gid "$gid" --argjson directories "$directories" '{process:{uid:$uid,gid:$gid},directories:$directories}' >"$output"; }
login(){ curl -fsS -H 'content-type: application/json' -d "{\"username\":\"$1\",\"password\":\"rustzen@123\"}" "$admin/api/auth/login" | jq -er .data.token; }
api(){ local method=$1 path=$2 body=${3:-}; if [ -n "$body" ]; then curl -fsS -X "$method" -H "authorization: Bearer $token" -H 'content-type: application/json' -d "$body" "$admin$path"; else curl -fsS -X "$method" -H "authorization: Bearer $token" "$admin$path"; fi; }
wait_gateway(){ for _ in $(seq 1 240); do api GET /api/reports/systems >/dev/null 2>&1 && return; sleep .05; done; return 1; }
new_run(){ api POST /api/reports/runs "$(jq -nc --arg f "$1" '{flowId:$f,input:{}}')" | jq -er .data.id; }
wait_run(){ wait_scalar "$reports_db" "SELECT status FROM automation_runs WHERE id='$1'" "$2"; }

step selected-fresh-start
start_reports; start_admin; token=$(login owner); wait_gateway
system=$(api POST /api/reports/systems '{"name":"Runtime fixture","baseUrl":"http://127.0.0.1:18080","enabled":true}' | jq -er .data.id)
flow(){ api POST /api/reports/flows "$(jq -nc --arg s "$system" --arg n "$1" --argjson x "$2" '{systemId:$s,name:$n,steps:$x}')" | jq -er .data.id; }
success_flow=$(flow success '[{"action":"pause","durationMs":3000}]')
failed_flow=$(flow failed '[{"action":"assertText","selector":"body","text":"never present"}]')
long_flow=$(flow long '[{"action":"pause","durationMs":30000}]')
revoked_flow=$(flow revoked '[{"action":"pause","durationMs":3000},{"action":"assertText","selector":"body","text":"never present"}]')

step admin-outage-and-backfill
success=$(new_run "$success_flow"); wait_run "$success" running; stop_pid "$admin_pid"; admin_pid=
wait_run "$success" succeeded; wait_scalar "$reports_db" 'SELECT COUNT(*) FROM notification_outbox' 1
jq -nc --arg run "$success" --arg status "$(scalar "$reports_db" "SELECT status FROM automation_runs WHERE id='$success'")" --argjson pending "$(scalar "$reports_db" 'SELECT pending_count FROM notification_delivery_status WHERE id=1')" '{runId:$run,status:$status,pending:$pending}' >/verify/evidence/outage-state.json
start_admin; token=$(login owner); wait_gateway; wait_scalar "$selected/admin.db" "SELECT COUNT(*) FROM notification_receipts WHERE producer='reports'" 1; wait_scalar "$reports_db" 'SELECT COUNT(*) FROM notification_outbox' 0

step terminal-lifecycle
failed=$(new_run "$failed_flow"); wait_run "$failed" failed
coop=$(new_run "$long_flow"); wait_run "$coop" running
queued=$(new_run "$long_flow"); wait_run "$queued" queued; api POST "/api/reports/runs/$queued/cancel" >/dev/null; wait_run "$queued" cancelled
api POST "/api/reports/runs/$coop/cancel" >/dev/null; wait_run "$coop" cancelled
wait_scalar "$selected/admin.db" "SELECT COUNT(*) FROM notification_receipts WHERE producer='reports'" 4

step retry-initiator-immutable
execute "$selected/admin.db" "INSERT OR IGNORE INTO user_roles(user_id,role_id) SELECT 2,id FROM roles WHERE code='owner'"
first_retry=$(api POST "/api/reports/runs/$failed/retry" | jq -er .data.id)
admin_token=$(login admin); token=$admin_token; second_retry=$(api POST "/api/reports/runs/$failed/retry" | jq -er .data.id); token=$(login owner)
[ "$first_retry" = "$second_retry" ]; [ "$(scalar "$reports_db" "SELECT initiator_user_id FROM automation_runs WHERE id='$first_retry'")" = 1 ]; wait_run "$first_retry" failed
jq -nc --arg source "$failed" --arg child "$first_retry" --arg second "$second_retry" --argjson initiator "$(scalar "$reports_db" "SELECT initiator_user_id FROM automation_runs WHERE id='$first_retry'")" '{source:$source,firstChild:$child,secondChild:$second,initiatorUserId:$initiator}' >/verify/evidence/retry-state.json

step restart-recovery
stop_pid "$reports_pid"; reports_pid=
python3 - "$reports_db" "$long_flow" <<'PY'
import sqlite3,sys
db,flow=sys.argv[1:]; now='2026-09-07T12:00:00Z'
with sqlite3.connect(db) as c:
 for row in [('60000000-0000-4000-8000-000000000001',None),('60000000-0000-4000-8000-000000000002',now)]:
  c.execute("INSERT INTO automation_runs(id,flow_id,initiator_user_id,status,input_json,created_at,started_at,cancel_requested_at) VALUES(?,?,1,'running','{}',?,?,?)",(row[0],flow,now,now,row[1]))
PY
start_reports; wait_run 60000000-0000-4000-8000-000000000001 failed; wait_run 60000000-0000-4000-8000-000000000002 cancelled
wait_scalar "$selected/admin.db" "SELECT COUNT(*) FROM notification_receipts WHERE producer='reports'" 7

step scheduled-silence
before_receipts=$(scalar "$selected/admin.db" "SELECT COUNT(*) FROM notification_receipts WHERE producer='reports'"); before_gaps=$(scalar "$reports_db" 'SELECT omitted_count+expired_count+unconfirmed_count+quarantined_count+quarantine_evicted_count FROM notification_delivery_status WHERE id=1')
execute "$reports_db" "INSERT INTO automation_runs(id,flow_id,initiator_user_id,status,input_json,created_at) VALUES('70000000-0000-4000-8000-000000000001',?,NULL,'queued','{}','2026-09-07T13:00:00Z')" "$failed_flow"
wait_run 70000000-0000-4000-8000-000000000001 failed; sleep 1
[ "$(scalar "$selected/admin.db" "SELECT COUNT(*) FROM notification_receipts WHERE producer='reports'")" = "$before_receipts" ]; [ "$(scalar "$reports_db" 'SELECT omitted_count+expired_count+unconfirmed_count+quarantined_count+quarantine_evicted_count FROM notification_delivery_status WHERE id=1')" = "$before_gaps" ]
jq -nc --arg run 70000000-0000-4000-8000-000000000001 --argjson receipts "$before_receipts" --argjson gaps "$before_gaps" '{runId:$run,initiator:null,receiptCount:$receipts,gapCount:$gaps}' >/verify/evidence/scheduled-state.json

step current-permission-revocation
revoked=$(new_run "$revoked_flow"); wait_run "$revoked" running; stop_pid "$admin_pid"; admin_pid=; execute "$selected/admin.db" "UPDATE modules SET enabled=0 WHERE id='reports'"; wait_run "$revoked" failed
wait_scalar "$reports_db" "SELECT COUNT(*) FROM notification_outbox WHERE subject_id='$revoked'" 1; revoked_event=$(scalar "$reports_db" "SELECT event_id FROM notification_outbox WHERE subject_id='$revoked'")
start_admin; wait_scalar "$selected/admin.db" "SELECT COUNT(*) FROM notification_receipts WHERE producer='reports' AND event_id='$revoked_event' AND result='no-recipients'" 1
stop_pid "$admin_pid"; admin_pid=; execute "$selected/admin.db" "UPDATE modules SET enabled=1 WHERE id='reports'"; start_admin; token=$(login owner); wait_gateway
jq -nc --arg run "$revoked" --arg event "$revoked_event" '{runId:$run,eventId:$event,result:"no-recipients"}' >/verify/evidence/revoked-state.json

step dropped-response-duplicate
stop_pid "$reports_pid"; reports_pid=; export RUSTZEN_REPORTS_NOTIFICATION_INGRESS_URL=http://127.0.0.1:19835/internal/v1/notification-events
python3 /verify/client.py --proxy --listen 127.0.0.1:19835 --upstream 127.0.0.1:19831 --body-out /verify/evidence/drop-event.json --state-out /verify/evidence/drop-proxy.json >"$selected/proxy.log" 2>&1 & pids+=("$!"); sleep .2
start_reports; wait_gateway; dropped=$(new_run "$success_flow"); wait_run "$dropped" succeeded; wait_scalar "$reports_db" 'SELECT COUNT(*) FROM notification_outbox' 0
for _ in $(seq 1 200); do jq -e '.accepted >= 2 and .firstResponseDropped == true' /verify/evidence/drop-proxy.json >/dev/null 2>&1 && break; sleep .05; done
jq -e '.accepted >= 2 and .firstResponseDropped == true' /verify/evidence/drop-proxy.json >/dev/null
python3 /verify/client.py --url "$ingress" --body /verify/evidence/drop-event.json >/verify/evidence/duplicate.json
python3 /verify/client.py --url "$ingress" --body /verify/evidence/drop-event.json --bad-signature >/verify/evidence/bad-signature.json
python3 /verify/client.py --url "$admin/internal/v1/notification-events" --body /verify/evidence/drop-event.json >/verify/evidence/public-internal.json
unsigned_status=$(curl -sS -o /verify/evidence/unsigned.json -w '%{http_code}' -H 'content-type: application/json' --data-binary @/verify/evidence/drop-event.json "$ingress"); [ "$unsigned_status" = 400 ]
jq -e '.status==200 and .body.code=="duplicate"' /verify/evidence/duplicate.json >/dev/null; jq -e '.status==401 and .body.code=="bad-producer"' /verify/evidence/bad-signature.json >/dev/null; jq -e '.status==404 and .body.code==10001' /verify/evidence/public-internal.json >/dev/null

step selected-evidence
ss -H -ltn 'sport = :19831' >/verify/evidence/selected-ingress-listener.txt; /verify/bin/rz-reports-selected contract config selected >/verify/evidence/selected-reports-config.json; curl -fsS "$reports/internal/v1/manifest" >/verify/evidence/selected-reports-api.json
capture_reports_identity "$reports_pid" "$reports_home" /verify/evidence/.selected-reports-identity.json
curl -fsS -H "authorization: Bearer $token" "$admin/api/notifications?limit=100" >/verify/evidence/inbox-final.json
jq -nc --arg success "$success" --arg failed "$failed" --arg queued "$queued" --arg cooperative "$coop" --arg recoveryFailed 60000000-0000-4000-8000-000000000001 --arg recoveryCancelled 60000000-0000-4000-8000-000000000002 --arg dropped "$dropped" '{success:{id:$success,status:"succeeded"},failed:{id:$failed,status:"failed"},queuedCancel:{id:$queued,status:"cancelled"},cooperativeCancel:{id:$cooperative,status:"cancelled"},recoveryFailed:{id:$recoveryFailed,status:"failed"},recoveryCancelled:{id:$recoveryCancelled,status:"cancelled"},dropResponse:{id:$dropped,status:"succeeded"}}' >/verify/evidence/lifecycle.json
python3 - "$selected/admin.db" "$reports_db" >/verify/evidence/selected-state.json <<'PY'
import json,sqlite3,sys
a=sqlite3.connect(sys.argv[1]); r=sqlite3.connect(sys.argv[2])
print(json.dumps({'receipts':a.execute("SELECT result,COUNT(*) FROM notification_receipts WHERE producer='reports' GROUP BY result ORDER BY result").fetchall(),'messages':a.execute("SELECT topic,subject_id FROM notifications WHERE producer='reports' ORDER BY inbox_seq").fetchall(),'recipients':a.execute("SELECT COUNT(*) FROM notification_recipients nr JOIN notifications n ON n.id=nr.notification_id WHERE n.producer='reports'").fetchone()[0],'outbox':r.execute('SELECT COUNT(*) FROM notification_outbox').fetchone()[0],'gaps':r.execute('SELECT omitted_count,expired_count,unconfirmed_count,quarantined_count,quarantine_evicted_count FROM notification_delivery_status WHERE id=1').fetchone()},separators=(',',':')))
PY

step pure-selection-absence
stop_all; export RUSTZEN_RUNTIME_ROOT=$pure/admin HOME=$pure/admin XDG_CONFIG_HOME=$pure/admin/.config XDG_CACHE_HOME=$pure/admin/.cache RUSTZEN_ADMIN_SQLITE_PATH=$pure/admin.db RUSTZEN_MONITOR_SQLITE_PATH=$pure/monitor.db RUSTZEN_REPORTS_SQLITE_PATH=$pure_reports_db RUSTZEN_ADMIN_PORT=19840 RUSTZEN_REPORTS_PORT=19844 RUSTZEN_NOTIFICATION_INGRESS_PORT=19841
export RUSTZEN_COMPOSITION_ID="$RUSTZEN_VERIFY_PURE_COMPOSITION_ID"
printf 'pure-owner-password\n' | /verify/bin/rz-admin-pure bootstrap-owner; /verify/bin/rz-admin-pure bind-database
/verify/bin/rz-admin-pure contract selected admin >/verify/evidence/pure-admin-api.json; /verify/bin/rz-admin-pure contract config selected access >/verify/evidence/pure-admin-config.json; /verify/bin/rz-reports-pure contract config selected >/verify/evidence/pure-reports-config.json
/verify/bin/rz-admin-pure serve >"$pure/admin.log" 2>&1 & pids+=("$!"); start_reports /verify/bin/rz-reports-pure "$pure_reports_home" "$pure_reports_db" "$pure_reports_log" http://127.0.0.1:19844; wait_health http://127.0.0.1:19840; curl -fsS http://127.0.0.1:19844/internal/v1/manifest >/verify/evidence/pure-reports-api.json
curl -fsS http://127.0.0.1:19840/__web-binding >/verify/evidence/pure-web-binding.json
pure_token=$(curl -fsS -H 'content-type: application/json' -d '{"username":"owner","password":"pure-owner-password"}' http://127.0.0.1:19840/api/auth/login | jq -er .data.token)
curl -fsS -H "authorization: Bearer $pure_token" http://127.0.0.1:19840/api/installation >/verify/evidence/pure-installation.json
pure_status=$(curl -sS -o /verify/evidence/pure-notification-route.json -w '%{http_code}' http://127.0.0.1:19840/api/notifications); [ "$pure_status" = 404 ]; jq -e '.code==404 and .data==null' /verify/evidence/pure-notification-route.json >/dev/null
python3 - "$pure/admin.db" "$pure_reports_db" >/verify/evidence/pure-absence.json <<'PY'
import json,sqlite3,sys
out={}
for name,path in [('admin',sys.argv[1]),('reports',sys.argv[2])]: out[name]=[r[0] for r in sqlite3.connect(path).execute("SELECT name FROM sqlite_master WHERE name LIKE 'notification%' OR name LIKE '%notifications_migrations' ORDER BY name")]
print(json.dumps(out,separators=(',',':')))
PY
sleep 1; { ss -H -ltn; ss -H -tn; } >/verify/evidence/pure-listeners.txt; capture_reports_identity "$reports_pid" "$pure_reports_home" /verify/evidence/.pure-reports-identity.json
jq -s --arg user "$reports_user" --argjson uid "$(id -u "$reports_user")" --argjson gid "$(id -g "$reports_user")" '{user:{name:$user,uid:$uid,gid:$gid},selected:.[0],pure:.[1]}' /verify/evidence/.selected-reports-identity.json /verify/evidence/.pure-reports-identity.json >/verify/evidence/reports-runtime-identity.json; rm /verify/evidence/.selected-reports-identity.json /verify/evidence/.pure-reports-identity.json; stop_all

step manifest
rm -rf "$runtime_base" /verify/evidence/fixture
files=(steps.log lifecycle.json outage-state.json retry-state.json scheduled-state.json revoked-state.json drop-event.json drop-proxy.json duplicate.json bad-signature.json unsigned.json public-internal.json selected-ingress-listener.txt selected-reports-config.json selected-reports-api.json inbox-final.json selected-state.json reports-runtime-identity.json pure-admin-api.json pure-admin-config.json pure-reports-config.json pure-reports-api.json pure-absence.json pure-notification-route.json pure-listeners.txt pure-web-binding.json pure-installation.json)
receipts=$(for file in "${files[@]}"; do jq -nc --arg file "$file" --arg sha "$(sha256sum "/verify/evidence/$file"|awk '{print $1}')" --argjson bytes "$(stat -c %s "/verify/evidence/$file")" '{file:$file,sha256:$sha,bytes:$bytes}'; done|jq -s .)
jq -n --arg head "$RUSTZEN_VERIFY_HEAD" --arg state "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" --arg sourceSha "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" --arg architecture "$RUSTZEN_VERIFY_ARCHITECTURE" --arg platform "$RUSTZEN_VERIFY_PLATFORM" --argjson binaries "$RUSTZEN_VERIFY_BINARY_HASHES" --arg provenanceSha "$RUSTZEN_VERIFY_BUILD_PROVENANCE_SHA256" --arg verifierImage "$RUSTZEN_VERIFY_VERIFIER_IMAGE_ID" --arg verifierKey "$RUSTZEN_VERIFY_VERIFIER_KEY" --arg verifierSha "$RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256" --arg pureCompositionId "$RUSTZEN_VERIFY_PURE_COMPOSITION_ID" --arg pureWebDigest "$RUSTZEN_VERIFY_PURE_WEB_DIGEST" --argjson receipts "$receipts" '{schemaVersion:1,status:"passed",gitHead:$head,sourceTreeState:$state,sourceTreeSha256:$sourceSha,platform:{architecture:$architecture,name:$platform},binaryHashes:$binaries,buildProvenanceSha256:$provenanceSha,verifier:{imageId:$verifierImage,key:$verifierKey,provenanceSha256:$verifierSha},reportsRuntime:{user:"rz-reports",nonRoot:true,directoryMode:"0750",selected:true,pure:true},delivery:{manualTerminalClasses:6,retryInitiatorImmutable:true,scheduledSilent:true,revoked:"no-recipients",outageBackfilled:true,droppedResponse:"duplicate",authentication:{unsigned:400,badSignature:401,publicInternal:404}},pureReports:{notificationSchemaObjects:0,notificationConfig:false,notificationRoute:false,notificationTask:false,notificationListeners:0},pureWeb:{compositionId:$pureCompositionId,webDigest:$pureWebDigest},receipts:$receipts,limits:["disposable shared-kernel container","controlled SQLite only establishes scheduled and crashed-running preconditions","no systemd, production deployment, UI or sustained load"]}' >/verify/evidence/manifest.json
echo 'Reports notification Linux runtime gate passed'
