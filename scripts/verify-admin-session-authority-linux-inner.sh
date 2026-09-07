#!/usr/bin/env bash
set -euo pipefail

runtime=/tmp/rz-admin-session-authority
db=$runtime/admin.db
base=http://127.0.0.1:19860
owner_password='rustzen@123'
user_password='runtime-user-password-123'
jwt_secret='runtime-session-jwt-secret-0123456789'
admin_pid= current_step=bootstrap

step(){ current_step=$1; printf 'STEP %s\n' "$1" | tee -a /verify/evidence/steps.log; }
stop_admin(){ [ -z "$admin_pid" ] && return; kill -TERM "$admin_pid" 2>/dev/null || true; for _ in $(seq 1 100); do kill -0 "$admin_pid" 2>/dev/null || break; sleep .05; done; kill -0 "$admin_pid" 2>/dev/null && kill -KILL "$admin_pid" 2>/dev/null || true; wait "$admin_pid" 2>/dev/null || true; admin_pid=; }
cleanup(){ result=$?; trap - EXIT INT TERM; stop_admin; if [ "$result" -ne 0 ]; then echo "FAILED STEP $current_step ($result)" >&2; tail -n 100 "$runtime/admin.log" 2>/dev/null || true; fi; exit "$result"; }
trap cleanup EXIT; trap 'exit 130' INT; trap 'exit 143' TERM
wait_health(){ for _ in $(seq 1 300); do curl -fsS --connect-timeout 1 --max-time 2 "$base/health" >/dev/null 2>&1 && return; sleep .05; done; return 1; }
start_admin(){ /verify/bin/rz-admin serve >>"$runtime/admin.log" 2>&1 & admin_pid=$!; wait_health; }
login(){ curl -fsS --connect-timeout 2 --max-time 10 -H 'content-type: application/json' -d "$(jq -nc --arg u "$1" --arg p "$2" '{username:$u,password:$p}')" "$base/api/auth/login" | jq -er .data.token; }
api(){ method=$1 path=$2 token=$3 body=${4:--}; args=(-sS --fail --connect-timeout 2 --max-time 10 -X "$method" -H "authorization: Bearer $token"); [ "$body" = - ] || args+=(-H 'content-type: application/json' --data-binary "$body"); curl "${args[@]}" "$base$path"; }
capture(){ method=$1 path=$2 token=$3 body=$4 output=$5; headers=$(mktemp); response=$(mktemp); args=(-sS --connect-timeout 2 --max-time 10 -D "$headers" -o "$response" -w '%{http_code}' -X "$method"); [ "$token" = - ] || args+=(-H "authorization: Bearer $token"); [ "$body" = - ] || args+=(-H 'content-type: application/json' --data-binary "$body"); status=$(curl "${args[@]}" "$base$path"); content_type=$(awk 'BEGIN{IGNORECASE=1} /^content-type:/{sub(/^[^:]*:[[:space:]]*/,""); sub(/\r$/,""); value=$0} END{print value}' "$headers"); jq -n --argjson status "$status" --arg contentType "$content_type" --slurpfile body "$response" '{status:$status,contentType:$contentType,body:$body[0]}' >"$output"; rm -f "$headers" "$response"; }
expect(){ jq -e --argjson status "$2" --argjson code "$3" '.status==$status and .contentType=="application/json" and .body.code==$code and (.body.message|type)=="string"' "$1" >/dev/null; }
scalar(){ python3 - "$db" "$1" <<'PY'
import sqlite3,sys
print(sqlite3.connect(sys.argv[1],timeout=5).execute(sys.argv[2]).fetchone()[0])
PY
}
execute(){ python3 - "$db" "$1" <<'PY'
import sqlite3,sys
with sqlite3.connect(sys.argv[1],timeout=5) as c: c.execute(sys.argv[2])
PY
}

for command in curl jq python3 ss; do command -v "$command" >/dev/null; done
for port in 19860 19861 19862 19863 19864; do ! ss -H -ltn "sport = :$port" | grep -q .; done
rm -rf "$runtime"; mkdir -p "$runtime" /verify/evidence; : >/verify/evidence/steps.log
export HOME=$runtime XDG_CONFIG_HOME=$runtime/.config XDG_CACHE_HOME=$runtime/.cache
export RUSTZEN_ENV=development RUSTZEN_TIMEZONE=UTC RUST_LOG=warn RUSTZEN_RUNTIME_ROOT=$runtime
export RUSTZEN_ADMIN_SQLITE_PATH=$db RUSTZEN_MONITOR_SQLITE_PATH=$runtime/monitor.db RUSTZEN_INSIGHTS_SQLITE_PATH=$runtime/insights.db RUSTZEN_REPORTS_SQLITE_PATH=$runtime/reports.db
export RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_ADMIN_PORT=19860 RUSTZEN_INTERNAL_HOST=127.0.0.1 RUSTZEN_NOTIFICATION_INGRESS_PORT=19861 RUSTZEN_MONITOR_PORT=19862 RUSTZEN_INSIGHTS_PORT=19863 RUSTZEN_REPORTS_PORT=19864
export RUSTZEN_JWT_SECRET=$jwt_secret RUSTZEN_IPC_TOKEN=runtime-session-ipc-secret-0123456789 RUSTZEN_MONITOR_AGENT_TOKEN=runtime-session-agent-secret-0123456789 RUSTZEN_REPORTS_CREDENTIAL_KEY=runtime-session-report-secret
export RUSTZEN_NOTIFICATION_EVENT_KEY_ID=runtime-monitor-v1 RUSTZEN_NOTIFICATION_EVENT_KEY=runtime-session-monitor-event-secret
export RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY_ID=runtime-reports-v1 RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY=runtime-session-reports-event-secret

step fresh-admin
start_admin; owner=$(login owner "$owner_password")
menus=$(api GET /api/system/menus/options "$owner"); list_menu=$(jq -er '.data[]|select(.code=="system:user:list")|.value' <<<"$menus"); options_menu=$(jq -er '.data[]|select(.code=="system:user:options")|.value' <<<"$menus")
role_body=$(jq -nc --argjson list "$list_menu" --argjson options "$options_menu" '{name:"Runtime reader",code:"runtime_reader",status:1,menuIds:[$list,$options],description:"runtime gate"}')
api POST /api/system/roles "$owner" "$role_body" >/dev/null
role_id=$(api GET '/api/system/roles?current=1&pageSize=100&roleCode=runtime_reader' "$owner" | jq -er '.data[]|select(.code=="runtime_reader")|.id')
create_user(){ api POST /api/system/users "$owner" "$(jq -nc --arg u "$1" --arg p "$user_password" --argjson role "$role_id" '{username:$u,email:($u+"@runtime.invalid"),password:$p,realName:$u,status:1,roleIds:[$role]}')" | jq -er .data; }
sid_user=$(create_user runtime_sid); revoke_user=$(create_user runtime_revoke); password_user=$(create_user runtime_password); disable_user=$(create_user runtime_disable); grant_user=$(create_user runtime_grant)

step two-sids-and-logout
sid_a=$(login runtime_sid "$user_password"); sid_b=$(login runtime_sid "$user_password"); api GET /api/auth/me "$sid_a" >/dev/null; api GET /api/auth/logout "$sid_a" >/dev/null
capture GET /api/auth/me "$sid_a" - /verify/evidence/sid-a-after-logout.json; expect /verify/evidence/sid-a-after-logout.json 401 401
capture GET /api/auth/me "$sid_b" - /verify/evidence/sid-b-after-logout.json; expect /verify/evidence/sid-b-after-logout.json 200 0

step identity-invalidations
revoke_token=$(login runtime_revoke "$user_password"); api POST "/api/system/users/$revoke_user/sessions/revoke-all" "$owner" >/dev/null; capture GET /api/auth/me "$revoke_token" - /verify/evidence/revoke-all.json; expect /verify/evidence/revoke-all.json 401 401
password_token=$(login runtime_password "$user_password"); api PUT "/api/system/users/$password_user/password" "$owner" '{"password":"runtime-new-password-456"}' >/dev/null; capture GET /api/auth/me "$password_token" - /verify/evidence/password-old.json; expect /verify/evidence/password-old.json 401 401
password_new=$(login runtime_password runtime-new-password-456); capture GET /api/auth/me "$password_new" - /verify/evidence/password-new.json; expect /verify/evidence/password-new.json 200 0
disable_token=$(login runtime_disable "$user_password"); api PUT "/api/system/users/$disable_user/status" "$owner" '{"status":2}' >/dev/null; capture GET /api/auth/me "$disable_token" - /verify/evidence/disable-old.json; expect /verify/evidence/disable-old.json 401 401
capture POST /api/auth/login - "$(jq -nc --arg p "$user_password" '{username:"runtime_disable",password:$p}')" /verify/evidence/disable-login.json; expect /verify/evidence/disable-login.json 403 10004

step current-grant
grant_token=$(login runtime_grant "$user_password"); capture GET /api/system/users "$grant_token" - /verify/evidence/grant-before.json; expect /verify/evidence/grant-before.json 200 0
grant_epoch_before=$(scalar 'SELECT authz_epoch FROM access_policy_state WHERE id=1')
api PUT "/api/system/roles/$role_id" "$owner" "$(jq -nc --argjson options "$options_menu" '{name:"Runtime reader",code:"runtime_reader",status:1,menuIds:[$options],description:"runtime gate"}')" >/dev/null
grant_epoch_removed=$(scalar 'SELECT authz_epoch FROM access_policy_state WHERE id=1')
capture GET /api/system/users "$grant_token" - /verify/evidence/grant-removed.json; expect /verify/evidence/grant-removed.json 403 403
api PUT "/api/system/roles/$role_id" "$owner" "$role_body" >/dev/null
grant_epoch_restored=$(scalar 'SELECT authz_epoch FROM access_policy_state WHERE id=1')
capture GET /api/system/users "$grant_token" - /verify/evidence/grant-restored.json; expect /verify/evidence/grant-restored.json 200 0

step token-validation
capture GET /api/auth/me bad.jwt.token - /verify/evidence/bad-jwt.json; expect /verify/evidence/bad-jwt.json 401 401
capture GET /api/auth/me "$grant_token" - /verify/evidence/valid-before-expiry.json; expect /verify/evidence/valid-before-expiry.json 200 0
# The pinned Linux verifier includes Python's SQLite/HMAC standard library but not Bun;
# these fixture-only operations avoid installing another runtime into the evidence image.
expired=$(python3 - "$jwt_secret" "$grant_token" <<'PY'
import base64,hashlib,hmac,json,sys,time
pad=lambda v:v+'='*((4-len(v)%4)%4); head,body,_=sys.argv[2].split('.'); claims=json.loads(base64.urlsafe_b64decode(pad(body))); claims['exp']=int(time.time())-1
body=base64.urlsafe_b64encode(json.dumps(claims,separators=(',',':')).encode()).rstrip(b'=').decode(); sig=base64.urlsafe_b64encode(hmac.new(sys.argv[1].encode(),f'{head}.{body}'.encode(),hashlib.sha256).digest()).rstrip(b'=').decode(); print(f'{head}.{body}.{sig}')
PY
)
capture GET /api/auth/me "$expired" - /verify/evidence/expired-jwt.json; expect /verify/evidence/expired-jwt.json 401 401

step authority-storage-fail-closed
python3 - "$db" "$grant_token" "$expired" "$grant_epoch_before" "$grant_epoch_removed" "$grant_epoch_restored" > /verify/evidence/selected-state.json <<'PY'
import base64,json,sqlite3,sys
pad=lambda v:v+'='*((4-len(v)%4)%4); decode=lambda token:json.loads(base64.urlsafe_b64decode(pad(token.split('.')[1]))); claims=decode(sys.argv[2]); expired=decode(sys.argv[3]); identity=lambda v:{'sid':v['sid'],'userId':v['user_id'],'username':v['username'],'userAuthEpoch':v['user_auth_epoch'],'exp':v['exp']}; c=sqlite3.connect(sys.argv[1]); session=c.execute('SELECT sid,user_id,auth_epoch_at_issue,expires_at,revoked_at FROM access_sessions WHERE sid=?',(claims['sid'],)).fetchone()
print(json.dumps({'sessions':c.execute('SELECT COUNT(*) FROM access_sessions').fetchone()[0],'active':c.execute('SELECT COUNT(*) FROM access_sessions WHERE revoked_at IS NULL').fetchone()[0],'authzEpoch':c.execute('SELECT authz_epoch FROM access_policy_state WHERE id=1').fetchone()[0],'grantEpochs':{'before':int(sys.argv[4]),'removed':int(sys.argv[5]),'restored':int(sys.argv[6])},'users':c.execute("SELECT username,status,auth_epoch FROM users WHERE username LIKE 'runtime_%' ORDER BY username").fetchall(),'grantCodes':c.execute("SELECT m.code FROM role_menus rm JOIN roles r ON r.id=rm.role_id JOIN menus m ON m.id=rm.menu_id WHERE r.code='runtime_reader' ORDER BY m.code").fetchall(),'expiryIdentity':{'claims':identity(claims),'expiredClaims':identity(expired),'session':session}},separators=(',',':')))
PY
execute 'ALTER TABLE access_sessions RENAME TO access_sessions_unavailable'
capture GET /api/auth/me "$owner" - /verify/evidence/authority-db-failure.json; expect /verify/evidence/authority-db-failure.json 503 50302
stop_admin

step manifest
files=(steps.log sid-a-after-logout.json sid-b-after-logout.json revoke-all.json password-old.json password-new.json disable-old.json disable-login.json grant-before.json grant-removed.json grant-restored.json bad-jwt.json valid-before-expiry.json expired-jwt.json authority-db-failure.json selected-state.json)
receipts=$(for file in "${files[@]}"; do jq -nc --arg file "$file" --arg sha "$(sha256sum "/verify/evidence/$file"|awk '{print $1}')" --argjson bytes "$(stat -c %s "/verify/evidence/$file")" '{file:$file,sha256:$sha,bytes:$bytes}'; done|jq -s .)
jq -n --arg head "$RUSTZEN_VERIFY_HEAD" --arg state "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" --arg sourceSha "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" --arg architecture "$RUSTZEN_VERIFY_ARCHITECTURE" --arg platform "$RUSTZEN_VERIFY_PLATFORM" --arg binary "$RUSTZEN_VERIFY_BINARY_SHA256" --arg provenance "$RUSTZEN_VERIFY_BUILD_PROVENANCE_SHA256" --arg verifierImage "$RUSTZEN_VERIFY_VERIFIER_IMAGE_ID" --arg verifierKey "$RUSTZEN_VERIFY_VERIFIER_KEY" --arg verifierSha "$RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256" --argjson receipts "$receipts" '{schemaVersion:1,status:"passed",gitHead:$head,sourceTreeState:$state,sourceTreeSha256:$sourceSha,platform:{architecture:$architecture,name:$platform},binarySha256:$binary,buildProvenanceSha256:$provenance,verifier:{imageId:$verifierImage,key:$verifierKey,provenanceSha256:$verifierSha},cases:{twoSessions:true,logoutIsolation:true,revokeAll:true,passwordChange:true,userDisable:true,grantRemoved:true,grantRestored:true,badJwt:true,expiredJwt:true,authorityStorageFailClosed:true},receipts:$receipts,limits:["disposable shared-kernel container","authority failure injected by making the fresh session table unavailable","no native systemd, deployment, SSE, Web, proxy or load verification"]}' >/verify/evidence/manifest.json
rm -rf "$runtime"
echo 'Admin session authority Linux runtime gate passed'
