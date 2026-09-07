#!/usr/bin/env bash
set -euo pipefail

runtime=/verify/evidence/runtime
public=http://127.0.0.1:19870
stream=$public/api/notifications/stream
ingress=http://127.0.0.1:19871/internal/v1/notification-events
owner_password=runtime-owner-password
jwt_secret=runtime-jwt-secret-01234567890123456789
query_secret=sse-query-secret-must-never-appear
pids=()
current_step=bootstrap

step(){ current_step=$1; printf 'STEP %s\n' "$current_step" | tee -a /verify/evidence/steps.log; }
stop_pid(){ pid=${1:-}; [ -n "$pid" ] || return 0; kill -TERM "$pid" 2>/dev/null || true; for _ in $(seq 1 100); do kill -0 "$pid" 2>/dev/null || break; sleep .05; done; kill -KILL "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; }
stop_all(){ for pid in "${pids[@]}"; do stop_pid "$pid"; done; pids=(); }
cleanup(){ result=$?; trap - EXIT INT TERM; stop_all; if [ "$result" -ne 0 ]; then echo "FAILED STEP $current_step (exit $result)" >&2; [ ! -s "$runtime/admin.log" ] || tail -n 120 "$runtime/admin.log" >&2; fi; exit "$result"; }
trap cleanup EXIT; trap 'exit 130' INT; trap 'exit 143' TERM

export RUSTZEN_ENV=development RUSTZEN_TIMEZONE=UTC RUST_LOG=debug
export RUSTZEN_RUNTIME_ROOT=$runtime RUSTZEN_ADMIN_SQLITE_PATH=$runtime/admin.db
export RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_ADMIN_PORT=19870 RUSTZEN_INTERNAL_HOST=127.0.0.1
export RUSTZEN_NOTIFICATION_INGRESS_PORT=19871
export RUSTZEN_BUILD_ID=$(printf 'a%.0s' {1..64}) RUSTZEN_COMPOSITION_ID=$(printf 'b%.0s' {1..64})
export RUSTZEN_ADMIN_SCHEMA_FINGERPRINT=$(printf 'c%.0s' {1..64}) RUSTZEN_ADMIN_DATA_CONTRACT_ID=$(printf 'd%.0s' {1..64})
export RUSTZEN_JWT_SECRET=$jwt_secret RUSTZEN_IPC_TOKEN=runtime-ipc-secret-01234567890123456789
export RUSTZEN_MONITOR_AGENT_TOKEN=runtime-agent-token-0123456789012345
export RUSTZEN_NOTIFICATION_EVENT_KEY_ID=runtime-v1
export RUSTZEN_NOTIFICATION_EVENT_KEY=runtime-notification-secret-0123456789
export RUSTZEN_NOTIFICATION_FREE_SPACE_RESERVE_BYTES=1

scalar(){ python3 - "$runtime/admin.db" "$1" <<'PY'
import sqlite3,sys
db=sqlite3.connect(sys.argv[1],timeout=5)
value=db.execute(sys.argv[2]).fetchone()[0]
print(value)
PY
}
execute(){ python3 - "$runtime/admin.db" "$1" <<'PY'
import sqlite3,sys
db=sqlite3.connect(sys.argv[1],timeout=5)
db.execute(sys.argv[2]); db.commit()
PY
}
wait_health(){ for _ in $(seq 1 200); do curl -fsS --connect-timeout 1 --max-time 2 "$public/health" >/dev/null 2>&1 && return; sleep .05; done; echo 'Admin health timeout' >&2; return 1; }
wait_scalar(){ sql=$1 expected=$2; for _ in $(seq 1 200); do [ "$(scalar "$sql" 2>/dev/null || true)" = "$expected" ] && return; sleep .05; done; echo "SQLite timeout: $sql = $expected" >&2; return 1; }
start_admin(){ /verify/bin/rz-admin serve >>"$runtime/admin.log" 2>&1 & admin_pid=$!; pids+=("$admin_pid"); wait_health; }
stop_admin(){ stop_pid "$admin_pid"; pids=(); admin_pid=; }
login(){ curl -fsS -H 'content-type: application/json' --data-binary "{\"username\":\"$1\",\"password\":\"$owner_password\"}" "$public/api/auth/login" | jq -er '.data.token'; }
capture(){ name=$1 method=$2 url=$3 token=${4:-}; body=${5:-}; headers=$runtime/$name.headers; args=(--silent --show-error --connect-timeout 2 --max-time 8 -D "$headers" -o "/verify/evidence/$name.json" -w '%{http_code}' -X "$method"); [ -z "$token" ] || args+=(-H "authorization: Bearer $token"); [ -z "$body" ] || args+=(-H 'content-type: application/json' --data-binary "$body"); status=$(curl "${args[@]}" "$url" || true); jq -nc --argjson status "${status:-0}" --arg contentType "$(awk 'BEGIN{IGNORECASE=1}/^content-type:/{gsub(/\r/,"");sub(/^[^:]+:[ ]*/,"");print;exit}' "$headers")" --slurpfile body "/verify/evidence/$name.json" '{status:$status,contentType:$contentType,body:$body[0]}' >"/verify/evidence/.$name"; mv "/verify/evidence/.$name" "/verify/evidence/$name.json"; }
sse_once(){ name=$1 token=$2 seconds=$3 last=${4:-}; : >"/verify/evidence/$name.body"; args=(--silent --show-error -N --connect-timeout 2 --max-time "$seconds" -D "/verify/evidence/$name.headers" -o "/verify/evidence/$name.body" -H "authorization: Bearer $token"); [ -z "$last" ] || args+=(-H "Last-Event-ID: $last"); curl "${args[@]}" "$stream" >/dev/null 2>&1 || [ "$?" = 28 ]; }
sse_start(){ name=$1 token=$2; : >"/verify/evidence/$name.body"; curl --silent --show-error -N --connect-timeout 2 --max-time 30 -D "/verify/evidence/$name.headers" -o "/verify/evidence/$name.body" -H "authorization: Bearer $token" "$stream" >/dev/null 2>&1 & started_pid=$!; }
wait_text(){ file=$1 text=$2; for _ in $(seq 1 200); do grep -Fq "$text" "$file" 2>/dev/null && return; sleep .05; done; echo "text timeout: $text" >&2; return 1; }
json_sse(){ name=$1 body=$2; python3 - "/verify/evidence/$name.headers" "$body" >"/verify/evidence/$name.json" <<'PY'
import json,sys
status=0; headers={}
for raw in open(sys.argv[1],errors='replace'):
    line=raw.rstrip('\r\n')
    if line.startswith('HTTP/'):
        status=int(line.split()[1]); headers={}
    elif ':' in line:
        key,value=line.split(':',1); headers[key.lower().strip()]=value.strip()
body=open(sys.argv[2],errors='replace').read()
print(json.dumps({'status':status,'contentType':headers.get('content-type','').split(';',1)[0].strip(),'cacheControl':headers.get('cache-control',''),'accelBuffering':headers.get('x-accel-buffering',''),'body':body},separators=(',',':')))
PY
}

rm -rf "$runtime"; mkdir -p "$runtime" /verify/evidence; : >/verify/evidence/steps.log; : >"$runtime/admin.log"
step database-bootstrap
printf '%s\n' "$owner_password" | /verify/bin/rz-admin bootstrap-owner
/verify/bin/rz-admin bind-database; /verify/bin/rz-admin validate-database
python3 - "$runtime/admin.db" <<'PY'
import sqlite3
db=sqlite3.connect(__import__('sys').argv[1])
password=db.execute("SELECT password_hash FROM users WHERE username='owner'").fetchone()[0]
db.execute("INSERT INTO users(username,email,password_hash,status,is_system) VALUES('runtime_denied','denied@runtime.invalid',?,1,0)",(password,))
db.commit()
PY

step initial-and-auth-matrix
start_admin; owner=$(login owner); denied=$(login runtime_denied)
sse_once initial "$owner" 2
grep -Fq 'reason":"connected' /verify/evidence/initial.body; grep -Fq 'id: 0' /verify/evidence/initial.body
json_sse initial /verify/evidence/initial.body
capture auth-401 GET "$stream" invalid.jwt
capture auth-403 GET "$stream" "$denied"
jq -e '.status==401 and .body.code==401' /verify/evidence/auth-401.json >/dev/null
jq -e '.status==403 and .body.code==403' /verify/evidence/auth-403.json >/dev/null
capture query-rejection GET "$stream?token=$query_secret" "$owner"
jq -e '.status==400 and .body.code==400' /verify/evidence/query-rejection.json >/dev/null
sleep .1
python3 - "$runtime/admin.db" "$runtime/admin.log" "$query_secret" > /verify/evidence/redaction.json <<'PY'
import json,sqlite3,sys
db=sqlite3.connect(sys.argv[1]); log=open(sys.argv[2],errors='replace').read(); secret=sys.argv[3]
has_operation_logs=db.execute("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='operation_logs')").fetchone()[0]
rows=db.execute("SELECT description FROM operation_logs WHERE description LIKE '%notification%' OR description LIKE '%token%'").fetchall() if has_operation_logs else []
text='\n'.join(x[0] for x in rows)
print(json.dumps({'secretAbsent':secret not in log and secret not in text,'queryAbsent':'?token=' not in log and '?token=' not in text,'operationLogTablePresent':bool(has_operation_logs),'operationDescriptions':[x[0] for x in rows]},separators=(',',':')))
PY
jq -e '.secretAbsent and .queryAbsent' /verify/evidence/redaction.json >/dev/null

step durable-change-and-read
sse_start live-change "$owner"; live_pid=$started_pid; pids+=("$live_pid")
wait_text /verify/evidence/live-change.body 'id: 0'
python3 - /verify/evidence/event.json <<'PY'
from datetime import datetime,timedelta,timezone
import json,sys
now=datetime.now(timezone.utc)
z=lambda d:d.isoformat(timespec='seconds').replace('+00:00','Z')
event={'schemaVersion':1,'eventId':'sse-runtime-open','producer':'monitor','topic':'monitor.incident.opened','subject':{'kind':'monitor-incident','id':'sse-runtime-subject','revision':1},'occurredAt':z(now),'expiresAt':z(now+timedelta(minutes=5)),'audience':{'policy':'monitor-incident-readers'},'content':{'title':'Runtime incident opened','summary':'Runtime SSE evidence'}}
open(sys.argv[1],'w').write(json.dumps(event,separators=(',',':')))
PY
python3 /verify/client.py --url "$ingress" --body /verify/evidence/event.json > /verify/evidence/ingest.json
jq -e '.status==201 and .body.code=="stored"' /verify/evidence/ingest.json >/dev/null
wait_scalar 'SELECT COUNT(*) FROM notification_recipients' 1; wait_text /verify/evidence/live-change.body 'id: 1'
capture inbox GET "$public/api/notifications?limit=100" "$owner"
notification_id=$(jq -er '.body.data.items[0].id' /verify/evidence/inbox.json)
capture read PUT "$public/api/notifications/$notification_id/read" "$owner"
jq -e '.status==200 and .body.data.revision==2' /verify/evidence/read.json >/dev/null
wait_text /verify/evidence/live-change.body 'id: 2'; stop_pid "$live_pid"; pids=("$admin_pid")
json_sse live-change /verify/evidence/live-change.body

step retention-and-reconnect
stop_admin
execute "UPDATE notifications SET accepted_at=datetime('now','-31 days')"
execute "UPDATE notification_receipts SET accepted_at=datetime('now','-31 days'),retain_until=datetime('now','-1 day')"
start_admin; wait_scalar 'SELECT COUNT(*) FROM notification_recipients' 0
capture retention GET "$public/api/notifications/unread-count" "$owner"
jq -e '.status==200 and .body.data=={count:0,revision:3}' /verify/evidence/retention.json >/dev/null
sse_once reconnect "$owner" 2 1
grep -Fq 'reason":"connected' /verify/evidence/reconnect.body; grep -Fq 'id: 3' /verify/evidence/reconnect.body
! grep -Eq '^id: (1|2)$' /verify/evidence/reconnect.body
json_sse reconnect /verify/evidence/reconnect.body

step per-user-quota
quota_pids=()
for i in 1 2 3 4; do sse_start "quota-$i" "$owner"; pid=$started_pid; quota_pids+=("$pid"); pids+=("$pid"); wait_text "/verify/evidence/quota-$i.body" 'reason":"connected'; done
capture quota-fifth GET "$stream" "$owner"
jq -e '.status==429 and .body.code==42901' /verify/evidence/quota-fifth.json >/dev/null
grep -qi '^retry-after: 60' "$runtime/quota-fifth.headers"
for pid in "${quota_pids[@]}"; do stop_pid "$pid"; done; pids=("$admin_pid")
sse_once quota-recovered "$owner" 1
grep -Fq 'reason":"connected' /verify/evidence/quota-recovered.body
jq -nc '{opened:4,fifth:{status:429,code:42901,retryAfter:60},released:true,reopened:200}' >/verify/evidence/quota.json

step authority-change-eof
sse_start authority-change "$owner"; authority_pid=$started_pid; pids+=("$authority_pid"); wait_text /verify/evidence/authority-change.body 'reason":"connected'
capture logout GET "$public/api/auth/logout" "$owner"
for _ in $(seq 1 160); do kill -0 "$authority_pid" 2>/dev/null || break; sleep .05; done
closed=false; kill -0 "$authority_pid" 2>/dev/null || closed=true; [ "$closed" = true ]; wait "$authority_pid" 2>/dev/null || true; pids=("$admin_pid")
jq -nc --argjson closed "$closed" --arg body "$(cat /verify/evidence/authority-change.body)" '{connected:($body|contains("connected")),closed:$closed}' >/verify/evidence/authority-change.json

step expiry-eof
owner=$(login owner)
python3 - "$owner" "$jwt_secret" >"$runtime/short.jwt" <<'PY'
import base64,hashlib,hmac,json,sys,time
token,key=sys.argv[1:]; head,payload,_=token.split('.')
claims=json.loads(base64.urlsafe_b64decode(payload+'='*(-len(payload)%4))); claims['exp']=int(time.time())+2
enc=lambda b:base64.urlsafe_b64encode(b).decode().rstrip('=')
payload=enc(json.dumps(claims,separators=(',',':')).encode()); signing=f'{head}.{payload}'
print(signing+'.'+enc(hmac.new(key.encode(),signing.encode(),hashlib.sha256).digest()))
PY
short=$(cat "$runtime/short.jwt"); sse_start expiry "$short"; expiry_pid=$started_pid; pids+=("$expiry_pid"); wait_text /verify/evidence/expiry.body 'reason":"connected'
for _ in $(seq 1 100); do kill -0 "$expiry_pid" 2>/dev/null || break; sleep .05; done
expired_closed=false; kill -0 "$expiry_pid" 2>/dev/null || expired_closed=true; [ "$expired_closed" = true ]; wait "$expiry_pid" 2>/dev/null || true; pids=("$admin_pid")
jq -nc --argjson closed "$expired_closed" --arg body "$(cat /verify/evidence/expiry.body)" '{connected:($body|contains("connected")),closed:$closed}' >/verify/evidence/expiry.json

step authority-outage
execute 'ALTER TABLE access_sessions RENAME TO access_sessions_unavailable'
capture authority-outage GET "$stream" "$owner"
jq -e '.status==503 and .body.code==50302' /verify/evidence/authority-outage.json >/dev/null
grep -qi '^retry-after: 60' "$runtime/authority-outage.headers"
grep -qi '^cache-control: no-store' "$runtime/authority-outage.headers"

step selected-state
python3 - "$runtime/admin.db" > /verify/evidence/selected-state.json <<'PY'
import json,sqlite3,sys
db=sqlite3.connect(sys.argv[1])
one=lambda q:db.execute(q).fetchone()[0]
has_operation_logs=one("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='operation_logs')")
print(json.dumps({'receipts':one('SELECT COUNT(*) FROM notification_receipts'),'messages':one('SELECT COUNT(*) FROM notifications'),'recipients':one('SELECT COUNT(*) FROM notification_recipients'),'revision':one("SELECT revision FROM notification_user_state JOIN users ON users.id=notification_user_state.user_id WHERE username='owner'"),'operationLogTablePresent':bool(has_operation_logs)},separators=(',',':')))
PY
jq -e '.receipts==0 and .messages==0 and .recipients==0 and .revision==3 and (.operationLogTablePresent|not)' /verify/evidence/selected-state.json >/dev/null
stop_all

step manifest
receipts='["steps.log","initial.json","event.json","ingest.json","live-change.json","inbox.json","read.json","retention.json","reconnect.json","auth-401.json","auth-403.json","query-rejection.json","redaction.json","quota.json","authority-change.json","expiry.json","authority-outage.json","selected-state.json"]'
jq -nc --arg head "$RUSTZEN_VERIFY_HEAD" --arg state "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" --arg source "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" --arg architecture "$RUSTZEN_VERIFY_ARCHITECTURE" --arg platform "$RUSTZEN_VERIFY_PLATFORM" --arg binary "$RUSTZEN_VERIFY_BINARY_SHA256" --arg provenance "$RUSTZEN_VERIFY_BUILD_PROVENANCE_SHA256" --arg image "$RUSTZEN_VERIFY_VERIFIER_IMAGE_ID" --arg key "$RUSTZEN_VERIFY_VERIFIER_KEY" --arg verifier "$RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256" --argjson files "$receipts" '{schemaVersion:1,status:"passed",gitHead:$head,sourceTreeState:$state,sourceTreeSha256:$source,platform:{architecture:$architecture,name:$platform},binarySha256:$binary,buildProvenanceSha256:$provenance,verifier:{imageId:$image,key:$key,provenanceSha256:$verifier},cases:{initialRevision:true,durableAdmission:true,durableRead:true,retention:true,lastEventIdNoReplay:true,unauthorized:true,forbidden:true,queryRejectedAndRedacted:true,perUserQuota:true,authorityChangeEof:true,expiryEof:true,authorityOutage:true},limitations:["disposable Linux runtime","retention age advanced by direct SQLite fixture before real startup cleanup","per-user 4/5 quota exercised; global 1000 and sustained load not exercised","browser, reverse proxy, Web UI, native systemd and production deployment not exercised"],receipts:[]}' >/verify/evidence/manifest.json
for file in $(jq -r '.[]' <<<"$receipts"); do sha=$(sha256sum "/verify/evidence/$file"|cut -d' ' -f1); bytes=$(stat -c %s "/verify/evidence/$file"); jq --arg file "$file" --arg sha "$sha" --argjson bytes "$bytes" '.receipts += [{file:$file,sha256:$sha,bytes:$bytes}]' /verify/evidence/manifest.json >/verify/evidence/.manifest; mv /verify/evidence/.manifest /verify/evidence/manifest.json; done
echo 'Admin notification SSE Linux runtime evidence complete'
