#!/usr/bin/env bash
set -euo pipefail

selected=/tmp/rz-message-center-selected; harness=/tmp/rz-message-center-harness
public=http://127.0.0.1:19810; ingress=http://127.0.0.1:19811/internal/v1/notification-events
monitor=http://127.0.0.1:19812; browser_admin=http://127.0.0.1:19901
owner_password=runtime-owner-password; agent_token=runtime-agent-token-0123456789012345
pids=(); current_step=bootstrap
step(){ current_step=$1; printf 'STEP %s\n' "$1"|tee -a /verify/evidence/steps.log; }
stop_all(){ for pid in "${pids[@]}"; do kill -TERM "$pid" 2>/dev/null||true; done; for pid in "${pids[@]}"; do wait "$pid" 2>/dev/null||true; done; pids=(); }
capture_logs(){ for entry in "selected-admin:$selected/admin.log" "selected-monitor:$selected/monitor.log" "selected-proxy:$selected/proxy.log" "harness-admin:$harness/admin.log" "harness-reports:$harness/logs/reports/verifier.log"; do name=${entry%%:*}; path=${entry#*:}; [ ! -f "$path" ] || cp "$path" "/verify/evidence/$name.log"; done; }
cleanup(){ result=$?; trap - EXIT INT TERM; stop_all; if [ "$result" -ne 0 ]; then capture_logs; echo "FAILED STEP $current_step" >&2; find "$selected" "$harness" -name '*.log' -type f -exec tail -n 50 {} \; 2>/dev/null||true; fi; exit "$result"; }
trap cleanup EXIT; trap 'exit 130' INT; trap 'exit 143' TERM
wait_health(){ for _ in $(seq 1 200); do curl -fsS --max-time 2 "$1/health" >/dev/null 2>&1&&return; sleep .05; done; return 1; }
curl_json(){ curl -fsS --connect-timeout 2 --max-time 15 "$@"; }
scalar(){ python3 - "$1" "$2" <<'PY'
import sqlite3,sys
print(sqlite3.connect(sys.argv[1]).execute(sys.argv[2]).fetchone()[0])
PY
}
wait_scalar(){ for _ in $(seq 1 200); do [ "$(scalar "$1" "$2" 2>/dev/null||true)" = "$3" ]&&return; sleep .05; done; return 1; }
wait_file(){ for _ in $(seq 1 200); do [ -s "$1" ]&&return; sleep .05; done; return 1; }

for port in 19800 19810 19811 19812 19820 19821 19822 19901 19904 19911; do
    ss -H -ltn "sport = :$port"|grep -q . || continue
    echo "message center verification port is already occupied: $port" >&2; exit 1
done

rm -rf "$selected" "$harness"; mkdir -p "$selected" "$harness" /verify/evidence
groupadd --system rz-reports; useradd --system --gid rz-reports --home-dir "$harness" --shell /usr/sbin/nologin rz-reports
: >/verify/evidence/steps.log; echo '{}' >/verify/evidence/proxy-mode.json
export RUSTZEN_ENV=development RUSTZEN_TIMEZONE=UTC RUST_LOG=warn
export RUSTZEN_BUILD_ID=$(printf 'a%.0s' {1..64}) RUSTZEN_COMPOSITION_ID=$(printf 'b%.0s' {1..64})
export RUSTZEN_ADMIN_SCHEMA_FINGERPRINT=$(printf 'c%.0s' {1..64}) RUSTZEN_ADMIN_DATA_CONTRACT_ID=$(printf 'd%.0s' {1..64})
export RUSTZEN_MONITOR_SCHEMA_FINGERPRINT=$(printf 'e%.0s' {1..64}) RUSTZEN_MONITOR_DATA_CONTRACT_ID=$(printf 'f%.0s' {1..64})
export RUSTZEN_JWT_SECRET=runtime-jwt-secret-01234567890123456789 RUSTZEN_IPC_TOKEN=runtime-ipc-secret-01234567890123456789
export RUSTZEN_REPORTS_CREDENTIAL_KEY=runtime-reports-key-01234567890123456789
export RUSTZEN_MONITOR_AGENT_TOKEN=$agent_token RUSTZEN_NOTIFICATION_EVENT_KEY_ID=runtime-v1
export RUSTZEN_NOTIFICATION_EVENT_KEY=runtime-notification-secret-0123456789
export RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_INTERNAL_HOST=127.0.0.1

step selected-bootstrap
export RUSTZEN_RUNTIME_ROOT=$selected RUSTZEN_ADMIN_SQLITE_PATH=$selected/admin.db RUSTZEN_MONITOR_SQLITE_PATH=$selected/monitor.db
export RUSTZEN_ADMIN_PORT=19810 RUSTZEN_MONITOR_PORT=19812 RUSTZEN_NOTIFICATION_INGRESS_PORT=19811
export RUSTZEN_NOTIFICATION_INGRESS_URL=$ingress
printf '%s\n' "$owner_password"|/verify/staged/rz-admin-notify bootstrap-owner
/verify/staged/rz-admin-notify bind-database; /verify/staged/rz-admin-notify validate-database
/verify/staged/rz-monitor-notify init-db; /verify/staged/rz-monitor-notify bind-database
/verify/staged/rz-monitor-notify controller >$selected/monitor.log 2>&1 & pids+=("$!")
/verify/staged/rz-admin-notify serve >$selected/admin.log 2>&1 & pids+=("$!")
python3 -u -B /verify/proxy.py --web-root /verify/staged/web/notify/dist --receipt /verify/evidence/proxy.jsonl --mode-file /verify/evidence/proxy-mode.json >$selected/proxy.log 2>&1 & pids+=("$!")
wait_health "$monitor"; wait_health "$public"; wait_health http://127.0.0.1:19800

report_epoch=$(($(date +%s)-30))
report(){ seq=$1 cpu=$2; now=$(python3 -c 'from datetime import datetime,timezone; import sys; print(datetime.fromtimestamp(int(sys.argv[1])+int(sys.argv[2]),timezone.utc).isoformat(timespec="microseconds").replace("+00:00","Z"))' "$report_epoch" "$seq"); response=/verify/evidence/report-response-$seq.json; error=/verify/evidence/report-error-$seq.log; jq -nc --argjson seq "$seq" --argjson cpu "$cpu" --arg now "$now" '{nodeId:"runtime-node",bootId:"11111111-1111-4111-8111-111111111111",sequence:$seq,hostname:"runtime-node",agentVersion:"runtime",collectedAt:$now,cpuPercent:$cpu,memory:{usedBytes:1,totalBytes:2},disks:[]}' >$selected/report.json; set +e; status=$(curl -sS --connect-timeout 2 --max-time 15 -o "$response" -w '%{http_code}' -H 'content-type: application/json' -H "x-rustzen-monitor-agent-token: $agent_token" --data-binary @$selected/report.json "$public/api/monitor/agent-reports" 2>"$error"); curl_status=$?; set -e; if [ "$curl_status" -ne 0 ]; then printf 'agent report curl failed (%s, HTTP %s): ' "$curl_status" "$status" >&2; cat "$error" >&2; return 1; fi; if [ "$status" != 200 ] || ! jq -e '.code == 0 and .data.status == "accepted"' "$response" >/dev/null; then printf 'agent report returned HTTP %s: ' "$status" >&2; cat "$response" >&2; return 1; fi; sleep .02; }
login=$(curl_json -H 'content-type: application/json' -d "{\"username\":\"owner\",\"password\":\"$owner_password\"}" "$public/api/auth/login"); token=$(jq -er .data.token<<<"$login"); auth=(-H "authorization: Bearer $token")

step selected-sse-preflight
jq -nc '{name:"preflight"}' >/verify/evidence/proxy-mode.json
export RUSTZEN_SSE_PREFLIGHT_TOKEN=$token
python3 /verify/sse-preflight.py --label direct --url "$public/api/notifications/stream" >/verify/evidence/sse-preflight-direct.json
python3 /verify/sse-preflight.py --label proxy --url http://127.0.0.1:19800/api/notifications/stream >/verify/evidence/sse-preflight-proxy.json
unset RUSTZEN_SSE_PREFLIGHT_TOKEN
jq -s '{direct:.[0],proxy:.[1]}' /verify/evidence/sse-preflight-direct.json /verify/evidence/sse-preflight-proxy.json >/verify/evidence/sse-preflight.json
rm /verify/evidence/sse-preflight-direct.json /verify/evidence/sse-preflight-proxy.json

step browser-harness
export RUSTZEN_RUNTIME_ROOT=$harness RUSTZEN_ADMIN_SQLITE_PATH=$harness/admin.db RUSTZEN_REPORTS_SQLITE_PATH=$harness/data/reports/db/reports.db
export RUSTZEN_ADMIN_PORT=19901 RUSTZEN_REPORTS_PORT=19904 RUSTZEN_NOTIFICATION_INGRESS_PORT=19911
export RUSTZEN_REPORTS_NOTIFICATION_INGRESS_URL=http://127.0.0.1:19911/internal/v1/notification-events
export RUSTZEN_REPORTS_BROWSER_PATH=/usr/bin/chromium
install -d -m 0750 -o rz-reports -g rz-reports "$harness" "$harness/data" "$harness/data/reports" "$harness/data/reports/db" "$harness/logs" "$harness/logs/reports" "$harness/output" "$harness/.config" "$harness/.cache"
install -m 0600 -o rz-reports -g rz-reports /dev/null "$harness/logs/reports/verifier.log"
chown -R rz-reports:rz-reports "$harness"
as_reports(){ setpriv --reuid=rz-reports --regid=rz-reports --init-groups --no-new-privs -- env HOME="$harness" XDG_CONFIG_HOME="$harness/.config" XDG_CACHE_HOME="$harness/.cache" "$@"; }
as_reports /verify/staged/rz-reports-verifier serve >$harness/logs/reports/verifier.log 2>&1 & pids+=("$!")
/verify/staged/rz-admin-verifier serve >$harness/admin.log 2>&1 & pids+=("$!")
wait_health "$browser_admin"; wait_health http://127.0.0.1:19904
step harness-login
hlogin=$(curl_json -H 'content-type: application/json' -d '{"username":"owner","password":"rustzen@123"}' "$browser_admin/api/auth/login"); htoken=$(jq -er .data.token<<<"$hlogin"); hauth=(-H "authorization: Bearer $htoken")
step harness-system-create
system=$(curl_json "${hauth[@]}" -H 'content-type: application/json' -d '{"name":"Message center Web","baseUrl":"http://127.0.0.1:19800/health","enabled":true}' "$browser_admin/api/reports/systems"|jq -er .data.id)
steps=$(cat /verify/evidence/browser-steps.json); : >/verify/evidence/browser-results.jsonl
case_diagnostics(){ name=$1 run=$2; prefix=/verify/evidence/failed-case-$name; step_response=$(curl_json "${hauth[@]}" "$browser_admin/api/reports/runs/$run/steps" 2>/dev/null||true); jq -nc --arg name "$name" --arg run "$run" --argjson response "${step_response:-null}" '{case:$name,runId:$run,steps:[($response.data[]? // empty)|{stepIndex,action,status,message}]}' >"$prefix-steps.json"; cat "$prefix-steps.json" >&2; artifact_response=$(curl_json "${hauth[@]}" "$browser_admin/api/reports/runs/$run/artifacts" 2>/dev/null||true); printf '%s\n' "${artifact_response:-null}" >"$prefix-artifacts.json"; index=0; while IFS= read -r id; do index=$((index+1)); curl_json "${hauth[@]}" "$browser_admin/api/reports/runs/$run/artifacts/$id" >"$prefix-artifact-$index.bin" 2>/dev/null || rm -f "$prefix-artifact-$index.bin"; done < <(jq -r '.data[]?.id' <<<"${artifact_response:-null}"); }
start_trigger(){
    trigger=$1 delayed=$2 trigger_pid=
    case "$trigger" in
        realtime)
            rm -f /verify/evidence/realtime-stream-ready /verify/evidence/realtime-list-ready /verify/evidence/realtime-timing.json
            (
                wait_file /verify/evidence/realtime-stream-ready
                wait_file /verify/evidence/realtime-list-ready
                sleep .5
                stream_ready=$(cat /verify/evidence/realtime-stream-ready)
                list_ready=$(cat /verify/evidence/realtime-list-ready)
                admitted=$(date +%s%N)
                report 1 97; report 2 97; report 3 97
                rm -f /verify/evidence/report-response-*.json /verify/evidence/report-error-*.log
                wait_scalar "$selected/admin.db" 'SELECT COUNT(*) FROM notifications' 1
                jq -nc --argjson stream "$stream_ready" --argjson list "$list_ready" --argjson admitted "$admitted" \
                    '{case:"realtimeInvalidation",streamReadyAtNs:$stream.readyAtNs,listReadyAtNs:$list.readyAtNs,admissionStartedAtNs:$admitted,notificationCount:1}' \
                    >/verify/evidence/realtime-timing.json
                rm -f /verify/evidence/realtime-stream-ready /verify/evidence/realtime-list-ready
            ) & trigger_pid=$!; pids+=("$trigger_pid")
            ;;
        forbidden)
            rm -f /verify/evidence/forbidden-list-ready /verify/evidence/forbidden-timing.json
            (
                wait_file /verify/evidence/forbidden-list-ready
                ready=$(cat /verify/evidence/forbidden-list-ready)
                admitted=$(date +%s%N)
                admission=$(admit "$delayed")
                jq -nc --argjson ready "$ready" --argjson admission "$admission" --argjson admitted "$admitted" --arg eventId "browser-page-$delayed" \
                    '{case:"forbiddenClears",listStatus:$ready.status,listReadyAtNs:$ready.readyAtNs,admissionStartedAtNs:$admitted,eventId:$eventId,admissionStatus:$admission.status,admissionCode:$admission.body.code}' \
                    >/verify/evidence/forbidden-timing.json
                rm -f /verify/evidence/forbidden-list-ready "/verify/evidence/admission-$delayed.json"
            ) & trigger_pid=$!; pids+=("$trigger_pid")
            ;;
    esac
}
run_case(){
    name=$1 mode=$2 trigger=${3:-} delayed=${4:-}
    step "run-case-$name"
    jq -nc --arg name "$name" --argjson mode "$mode" \
        '$mode+{name:$name}|if (.sseLive == true or .sse401 == true) then . else .+{sse204:true} end' \
        >/verify/evidence/proxy-mode.json
    case_steps=$(jq -ce --arg name "$name" '.[$name]'<<<"$steps")
    flow=$(curl_json "${hauth[@]}" -H 'content-type: application/json' \
        -d "$(jq -nc --arg system "$system" --arg name "$name" --argjson steps "$case_steps" '{systemId:$system,name:$name,steps:$steps}')" \
        "$browser_admin/api/reports/flows"|jq -er .data.id)
    run=$(curl_json "${hauth[@]}" -H 'content-type: application/json' \
        -d "$(jq -nc --arg flow "$flow" '{flowId:$flow,input:{}}')" \
        "$browser_admin/api/reports/runs"|jq -er .data.id)
    start_trigger "$trigger" "$delayed"
    status=queued response=
    for _ in $(seq 1 900); do
        if ! response=$(curl_json "${hauth[@]}" "$browser_admin/api/reports/runs/$run"); then
            printf '{"case":"%s","runId":"%s","phase":"poll","error":"request-failed"}\n' "$name" "$run" >/verify/evidence/failed-case-$name-poll.json
            case_diagnostics "$name" "$run" || true; return 1
        fi
        status=$(jq -er .data.status<<<"$response") || { printf '%s\n' "$response" >/verify/evidence/failed-case-$name-poll-response.json; return 1; }
        case "$status" in queued|running) sleep .1;; *) break;; esac
    done
    [ "$status" = succeeded ] || { case_diagnostics "$name" "$run" || true; echo "$response" >&2; return 1; }
    [ -z "$trigger_pid" ] || wait "$trigger_pid"
    if ! final_steps=$(curl_json "${hauth[@]}" "$browser_admin/api/reports/runs/$run/steps"); then
        case_diagnostics "$name" "$run" || true; return 1
    fi
    jq -c --arg name "$name" '{case:$name,steps:.data}' <<<"$final_steps" >>/verify/evidence/browser-results.jsonl
    last_run=$run
}

step browser-empty-loading
run_case emptyDesktop '{}'
step selected-trigger-incident
run_case realtimeInvalidation '{"sseLive":true,"streamReadyFile":"/verify/evidence/realtime-stream-ready","readyFile":"/verify/evidence/realtime-list-ready"}' realtime
wait_scalar $selected/admin.db 'SELECT COUNT(*) FROM notifications' 1
incident=$(scalar $selected/monitor.db 'SELECT id FROM monitor_incidents ORDER BY opened_at DESC LIMIT 1')
admit(){ index=$1; occurred=$(date -u +%Y-%m-%dT%H:%M:%SZ); expires=$(date -u -d '6 days' +%Y-%m-%dT%H:%M:%SZ); subject=$(printf '00000000-0000-4000-8000-%012d' "$index"); receipt=/verify/evidence/admission-$index.json; jq -nc --arg id "browser-page-$index" --arg subject "$subject" --arg occurred "$occurred" --arg expires "$expires" '{schemaVersion:1,eventId:$id,producer:"monitor",topic:"monitor.incident.opened",subject:{kind:"monitor-incident",id:$subject,revision:1},audience:{policy:"monitor-incident-readers"},occurredAt:$occurred,expiresAt:$expires,content:{title:("Runtime page "+$id),summary:"Runtime browser paging"}}' >$selected/event.json; python3 /verify/client.py --url "$ingress" --body $selected/event.json >"$receipt"; if ! jq -e '.status == 201 and .body.code == "stored"' "$receipt" >/dev/null; then cat "$receipt" >&2; return 1; fi; cat "$receipt"; }
run_case loading '{"listDelayMs":1200}'
step browser-populated-detail
run_case populatedDetail '{}'; desktop_run=$last_run
for index in $(seq 1 21); do admit "$index" >/dev/null; done; rm -f /verify/evidence/admission-*.json
run_case unreadFilterPaging '{}'; run_case singleRead '{}'; run_case readAll '{}'
step browser-reconcile-and-auth
run_case forbiddenClears '{"list403After":1,"readyFile":"/verify/evidence/forbidden-list-ready"}' forbidden 22
run_case unauthorized '{"sse401":true}'
run_case incidentDeepLink '{}'; run_case mobile '{}'; mobile_run=$last_run

download(){ run=$1 prefix=$2 output=$3; response=$(curl_json "${hauth[@]}" "$browser_admin/api/reports/runs/$run/artifacts"); id=$(jq -er --arg p "$prefix" '.data[]|select(.fileName|startswith($p))|.id'<<<"$response"); curl_json "${hauth[@]}" "$browser_admin/api/reports/runs/$run/artifacts/$id" >"/verify/evidence/$output"; }
download "$desktop_run" message-center-desktop-en message-center-desktop-en.png
download "$mobile_run" message-center-mobile-en message-center-mobile-en.png
stop_all
jq -s . /verify/evidence/browser-results.jsonl >/verify/evidence/browser-results.json
jq -s . /verify/evidence/proxy.jsonl >/verify/evidence/proxy-receipt.json
rm -f /verify/evidence/browser-results.jsonl /verify/evidence/proxy.jsonl /verify/evidence/proxy-mode.json

step selected-and-pure-state
python3 - $selected/admin.db $selected/monitor.db > /verify/evidence/selected-state.json <<'PY'
import json,sqlite3,sys
a=sqlite3.connect(sys.argv[1]);m=sqlite3.connect(sys.argv[2])
print(json.dumps({'messages':a.execute('select count(*) from notifications').fetchone()[0],'recipients':a.execute('select count(*) from notification_recipients').fetchone()[0],'reads':a.execute('select count(*) from notification_recipients where read_at is not null').fetchone()[0],'incidents':m.execute('select count(*) from monitor_incidents').fetchone()[0]},separators=(',',':')))
PY
export RUSTZEN_RUNTIME_ROOT=/verify/evidence/runtime-pure RUSTZEN_ADMIN_SQLITE_PATH=/verify/evidence/runtime-pure/admin.db RUSTZEN_MONITOR_SQLITE_PATH=/verify/evidence/runtime-pure/monitor.db RUSTZEN_ADMIN_PORT=19820 RUSTZEN_MONITOR_PORT=19822
mkdir -p "$RUSTZEN_RUNTIME_ROOT"; printf '%s\n' "$owner_password"|/verify/staged/rz-admin-pure bootstrap-owner; /verify/staged/rz-admin-pure bind-database; /verify/staged/rz-monitor-pure init-db; /verify/staged/rz-monitor-pure bind-database
notification_owners=true; /verify/staged/rz-admin-pure contract selected notifications >/dev/null 2>&1||notification_owners=false; [ "$notification_owners" = false ]
/verify/staged/rz-admin-pure serve >/verify/evidence/runtime-pure/admin.log 2>&1 & pids+=("$!"); /verify/staged/rz-monitor-pure controller >/verify/evidence/runtime-pure/monitor.log 2>&1 & pids+=("$!"); wait_health http://127.0.0.1:19820; wait_health http://127.0.0.1:19822
route_status=$(curl -sS -o /verify/evidence/runtime-pure/route.json -w '%{http_code}' http://127.0.0.1:19820/api/notifications); ingress_listener=false; ss -H -ltn 'sport = :19821'|grep -q .&&ingress_listener=true; stop_all
python3 - "$RUSTZEN_ADMIN_SQLITE_PATH" "$RUSTZEN_MONITOR_SQLITE_PATH" /verify/staged/web/pure/inventory.json "$route_status" "$ingress_listener" "$notification_owners" > /verify/evidence/pure-absence.json <<'PY'
import json,sqlite3,sys
objects=[]
for path in sys.argv[1:3]: objects += [r[0] for r in sqlite3.connect(path).execute("select name from sqlite_master where name like '%notification%'")]
inventory=json.load(open(sys.argv[3])); owners=('apps/web/src/notifications/','apps/web/src/api/notifications/'); print(json.dumps({'schemaObjects':objects,'notificationModules':[x for x in inventory['moduleIds'] if x.startswith(owners)],'preset':inventory['preset'],'routeStatus':int(sys.argv[4]),'ingressListener':sys.argv[5]=='true','notificationOwners':sys.argv[6]=='true'},separators=(',',':')))
PY
rm -rf /verify/evidence/runtime-pure "$selected" "$harness"

step manifest
receipts=$(for file in steps.log browser-results.json proxy-receipt.json sse-preflight.json realtime-timing.json forbidden-timing.json selected-state.json pure-absence.json message-center-desktop-en.png message-center-mobile-en.png; do jq -nc --arg file "$file" --arg sha "$(sha256sum /verify/evidence/$file|awk '{print $1}')" --argjson bytes "$(stat -c %s /verify/evidence/$file)" '{file:$file,sha256:$sha,bytes:$bytes}'; done|jq -s .)
jq -n --arg head "$RUSTZEN_VERIFY_HEAD" --arg state "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" --arg source "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" --arg arch "$RUSTZEN_VERIFY_ARCHITECTURE" --arg platform "$RUSTZEN_VERIFY_PLATFORM" --argjson binaries "$RUSTZEN_VERIFY_BINARY_HASHES" --arg web "$RUSTZEN_VERIFY_WEB_INVENTORY_SHA256" --arg pureWeb "$RUSTZEN_VERIFY_PURE_INVENTORY_SHA256" --arg provenance "$RUSTZEN_VERIFY_BUILD_PROVENANCE_SHA256" --arg image "$RUSTZEN_VERIFY_VERIFIER_IMAGE_ID" --arg key "$RUSTZEN_VERIFY_VERIFIER_KEY" --arg verifierSha "$RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256" --argjson receipts "$receipts" '{schemaVersion:1,status:"passed",gitHead:$head,sourceTreeState:$state,sourceTreeSha256:$source,platform:{architecture:$arch,name:$platform},binaryHashes:$binaries,web:{selectedInventorySha256:$web,pureInventorySha256:$pureWeb},buildProvenanceSha256:$provenance,verifier:{imageId:$image,key:$key,provenanceSha256:$verifierSha},cases:{browser:11,sseBearer:true,sseUrlSecret:false,browserSseLifecycle:true,singleLifecycle:true,durableReconcile:true,forbiddenClears:true,unauthorizedLogin:true,incidentDeepLink:true,pureAbsent:true},screenshots:[{file:"message-center-desktop-en.png",dimensions:"1440 x 900"},{file:"message-center-mobile-en.png",dimensions:"390 x 844"}],receipts:$receipts,limitations:["keyboard and lifecycle source tests remain authoritative","no Reports notification deep-link browser DSL","no sustained load, native systemd, production reverse proxy or deployment"]}' >/verify/evidence/manifest.json
