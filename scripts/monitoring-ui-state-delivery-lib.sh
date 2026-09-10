#!/usr/bin/env bash
# Notification-delivery extension for the route-exact Monitoring UI gate.

monitor_delivery_expected_jq() { reports_ui_delivery_expected_jq; }
monitor_delivery_evidence_root() { printf '%s\n' "${RUSTZEN_MONITOR_DELIVERY_EVIDENCE_ROOT:-/verify/evidence}"; }
monitor_delivery_bytes() { wc -c < "$1" | tr -d ' '; }

refresh_owner_auth() {
    local response token
    response=$(curl_json -H 'content-type: application/json' -d "$login_body" \
        "$admin/api/auth/login") || return 1
    token=$(jq -er '.data.token | strings | select(length > 0)' <<<"$response") || return 1
    auth=(-H "authorization: Bearer $token")
    owner_auth=(-H "authorization: Bearer $token")
}

monitor_delivery_settle() {
    local database=$1 phase=$2 receipt=$3
    local timeout=${RUSTZEN_MONITOR_DELIVERY_SETTLE_TIMEOUT:-75}
    local interval=${RUSTZEN_MONITOR_DELIVERY_SETTLE_INTERVAL:-0.1}
    python3 -B - "$database" "$phase" "$receipt" "$timeout" "$interval" <<'PY'
import json
import sqlite3
import sys
import time

database, phase, receipt, timeout, interval = sys.argv[1:]
started = time.monotonic()
result = {"schemaVersion": 1, "phase": phase, "settled": False}
try:
    timeout = float(timeout)
    interval = float(interval)
    if timeout <= 0 or interval <= 0:
        raise ValueError("settle timeout and interval must be positive")
    while True:
        with sqlite3.connect(database, timeout=5.0) as connection:
            states = dict(connection.execute(
                "SELECT state,COUNT(*) FROM notification_outbox GROUP BY state"
            ))
            active = connection.execute(
                "SELECT COUNT(*) FROM notification_outbox WHERE state IN ('pending','reconciling') "
                "OR lease_token IS NOT NULL"
            ).fetchone()[0]
            active_rows = connection.execute(
                "SELECT event_id,state,attempts,next_attempt_at,lease_until,reconcile_until,last_error_code "
                "FROM notification_outbox WHERE state IN ('pending','reconciling') "
                "OR lease_token IS NOT NULL ORDER BY event_id"
            ).fetchall()
            leased = connection.execute(
                "SELECT COUNT(*) FROM notification_outbox WHERE lease_token IS NOT NULL"
            ).fetchone()[0]
            status = connection.execute(
                "SELECT pending_count,pending_bytes,quarantine_count,quarantine_bytes,"
                "omitted_count,expired_count,unconfirmed_count,quarantined_count,"
                "quarantine_evicted_count,first_gap_at,last_gap_at,last_success_at "
                "FROM notification_delivery_status WHERE id=1"
            ).fetchone()
        if status is None:
            raise RuntimeError("notification delivery status row was not initialized")
        result.update({"outbox": {"stateCounts": states, "leasedCount": leased,
                       "activeCount": active, "activeRows": [dict(zip(
                       ("eventId", "state", "attempts", "nextAttemptAt", "leaseUntil",
                        "reconcileUntil", "lastErrorCode"), row)) for row in active_rows]},
                       "deliveryStatus": list(status)})
        if active == 0:
            result["settled"] = True
            break
        if time.monotonic() - started >= timeout:
            break
        time.sleep(interval)
except Exception as error:
    result["error"] = str(error)
result["elapsedMs"] = round((time.monotonic() - started) * 1000)
with open(receipt, "w", encoding="utf-8") as output:
    json.dump(result, output, separators=(",", ":"), sort_keys=True)
    output.write("\n")
raise SystemExit(0 if result["settled"] else 1)
PY
}

monitor_delivery_prepare() {
    local database=$1 phase=$2 evidence receipt
    evidence=$(monitor_delivery_evidence_root) || return 1
    receipt="$evidence/.monitor-delivery-prepare-$phase.json"
    monitor_delivery_settle "$database" "$phase" "$receipt" || return 1
    seed_reports_ui_delivery_status "$database" || return 1
}

monitor_delivery_descriptor() {
    local file=$1 hash_line hash bytes
    test -f "$file" && test ! -L "$file" || return 1
    hash_line=$(sha256sum "$file") || return 1
    hash=${hash_line%% *}
    bytes=$(monitor_delivery_bytes "$file") || return 1
    jq -nc --arg file "$(basename -- "$file")" --arg sha "$hash" --argjson bytes "$bytes" \
        '{file:$file,sha256:$sha,bytes:$bytes}'
}

monitor_delivery_run_diagnostics() {
    local run=$1 name=$2 reason=$3 response=$4 evidence status error
    evidence=$(monitor_delivery_evidence_root) || return 0
    status=$(jq -r '.data.status // "unknown"' <<<"$response" 2>/dev/null) || status=unknown
    error=$(jq -r '.data.error // null' <<<"$response" 2>/dev/null) || error=null
    printf '%s\n' "$response" >"$evidence/.delivery-$name-final-run.json" 2>/dev/null || true
    curl_json "${owner_auth[@]}" "$admin/api/reports/runs/$run/steps" \
        >"$evidence/.delivery-$name-final-steps.json" 2>/dev/null || true
    curl_json "${owner_auth[@]}" "$admin/api/reports/runs/$run/artifacts" \
        >"$evidence/.delivery-$name-final-artifacts.json" 2>/dev/null || true
    jq -nc --arg run "$run" --arg reason "$reason" --arg status "$status" --arg error "$error" \
        '{run:$run,reason:$reason,status:$status,error:($error | if . == "null" then null else . end)}' \
        >"$evidence/.delivery-$name-final-status.json" 2>/dev/null || true
    printf 'delivery %s run %s %s: %s\n' "$name" "$run" "$status" "$error" >&2
}

monitor_delivery_wait_run() {
    local run=$1 name=$2 response status polls
    polls=${RUSTZEN_MONITOR_DELIVERY_RUN_POLLS:-900}
    case "$polls" in ''|*[!0-9]*) return 1 ;; esac
    [ "$polls" -gt 0 ] || return 1
    for _ in $(seq 1 "$polls"); do
        response=$(curl_json "${owner_auth[@]}" "$admin/api/reports/runs/$run") || return 1
        status=$(jq -er '.data.status' <<<"$response") || return 1
        case "$status" in
            queued|running) sleep .1 ;;
            succeeded) return 0 ;;
            *) monitor_delivery_run_diagnostics "$run" "$name" terminal "$response"; return 1 ;;
        esac
    done
    monitor_delivery_run_diagnostics "$run" "$name" exhausted "$response"
    return 1
}

monitor_delivery_browser_case() {
    local name=$1 username=$2 password=$3 theme=$4 locale=$5 width=$6 height=$7 text=$8
    local steps body flow_response flow run_response run run_body file evidence
    local pending quarantine first last success incident wait_code
    if [ "$locale" = zh-CN ]; then
        pending='待投递 2（1024 B）'; quarantine='隔离 3（2048 B）'
        first='首个缺口 2026/09/10 01:02:03'; last='最后缺口 2026/09/10 02:03:04'
        success='最后成功 2026/09/10 03:04:05'
    else
        pending='Pending 2 (1024 B)'; quarantine='Quarantine 3 (2048 B)'
        first='First gap 09/10/2026, 01:02:03 AM'; last='Last gap 09/10/2026, 02:03:04 AM'
        success='Last success 09/10/2026, 03:04:05 AM'
    fi
    incident='Fixture incident'
    steps=$(jq -nc --arg username "$username" --arg password "$password" \
        --arg theme "$theme" --arg locale "$locale" --arg text "$text" --arg name "$name" \
        --arg pending "$pending" --arg quarantine "$quarantine" --arg first "$first" \
        --arg last "$last" --arg success "$success" --arg incident "$incident" \
        --arg incident_selector ".ant-table-tbody > tr.ant-table-row .monitoring-incident-primary-column" \
        --argjson width "$width" --argjson height "$height" '
        [
          {action:"setUiPreferences",theme:$theme,locale:$locale},
          {action:"setViewport",width:$width,height:$height},
          {action:"goto",url:"/login"},{action:"waitFor",selector:"#login_username"},
          {action:"fill",selector:"#login_username",value:$username},
          {action:"fill",selector:"#login_password",value:$password},
          {action:"click",selector:"button[type=submit]"},{action:"waitFor",selector:".shell-content"},
          {action:"goto",url:"/monitoring/incidents"},
          {action:"waitFor",selector:"[data-testid=notification-delivery-card]"},
          {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:$text},
          {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:$pending},
          {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:$quarantine},
          {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:$first},
          {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:$last},
          {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:$success}
        ] + (if $width == 390 then [
          {action:"waitFor",selector:".ant-table-row"},
          {action:"assertText",selector:$incident_selector,text:$incident},
          {action:"assertElementLayout",selector:".ant-table-thead th:not(.ant-table-cell-scrollbar)",visibleCount:3},
          {action:"assertElementLayout",selector:".ant-table-thead .monitoring-incident-detail-column",visibleCount:0},
          {action:"assertElementLayout",selector:".ant-table-thead .monitoring-incident-primary-column",
           visibleCount:1,maxHeight:64,withinViewportRight:true},
          {action:"assertElementLayout",selector:".ant-table-tbody > tr.ant-table-row",
           elementCount:20,visibleCount:20,maxHeight:72,withinViewportRight:true},
          {action:"assertElementLayout",selector:".ant-table-body > table",visibleCount:1,withinViewportRight:true},
          {action:"assertElementLayout",selector:".data-table-pagination .ant-pagination",
           visibleCount:1,withinViewportRight:true}
        ] else [] end) + [
          {action:"assertNoHorizontalOverflow"},{action:"screenshotViewport",name:$name}
        ]') || return 1
    body=$(jq -nc --arg system "$system" --arg name "Monitoring delivery: $name" --argjson steps "$steps" \
        '{systemId:$system,name:$name,steps:$steps}') || return 1
    evidence=$(monitor_delivery_evidence_root) || return 1
    flow_response=$(curl_json "${owner_auth[@]}" -H 'content-type: application/json' -d "$body" \
        "$admin/api/reports/flows") || return 1
    flow=$(jq -er '.data.id' <<<"$flow_response") || return 1
    printf '%s\n' "$flow_response" >"$evidence/delivery-$name-flow.json" || return 1
    run_body=$(jq -nc --arg flow "$flow" '{flowId:$flow,input:{}}') || return 1
    run_response=$(curl_json "${owner_auth[@]}" -H 'content-type: application/json' -d "$run_body" \
        "$admin/api/reports/runs") || return 1
    run=$(jq -er '.data.id' <<<"$run_response") || return 1
    printf '%s\n' "$run_response" >"$evidence/delivery-$name-run.json" || return 1
    monitor_delivery_wait_run "$run" "$name"
    wait_code=$?
    [ "$wait_code" -eq 0 ] || return "$wait_code"
    file="delivery-$name-steps.json"
    curl_json "${owner_auth[@]}" "$admin/api/reports/runs/$run/steps" >"$evidence/$file" || return 1
    jq -e --arg run "$run" '.data | length > 0 and all(.[]; .runId == $run and .status == "succeeded")' \
        "$evidence/$file" >/dev/null || return 1
    printf '%s\n' "$run"
}

monitor_delivery_screenshot() {
    local run=$1 name=$2 output=$3 width=$4 height=$5 response ids id image_info size evidence
    evidence=$(monitor_delivery_evidence_root) || return 1
    response=$(curl_json "${owner_auth[@]}" "$admin/api/reports/runs/$run/artifacts") || return 1
    ids=$(jq -er --arg name "$name" '
        [.data[] | select(.fileName | startswith($name)) | .id]
        | if length == 1 then .[0] else error("artifact count") end
    ' <<<"$response") || return 1
    id=$ids
    curl_json "${owner_auth[@]}" "$admin/api/reports/runs/$run/artifacts/$id" >"$evidence/$output" || return 1
    image_info=$(file "$evidence/$output") || return 1
    size=$(sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/' <<<"$image_info") || return 1
    test "$size" = "$width x $height" || return 1
    monitor_delivery_descriptor "$evidence/$output" | jq --arg case "$name" --arg dimensions "$size" \
        '. + {case:$case,dimensions:$dimensions}' || return 1
}

monitor_delivery_create_viewer() {
    local menu role login
    refresh_owner_auth || return 1
    menu=$(curl_json "${owner_auth[@]}" "$admin/api/system/menus/options?limit=500" \
        | jq -er '.data[] | select(.code == "monitor:incident:view") | .value') || return 1
    curl_json "${owner_auth[@]}" -H 'content-type: application/json' \
        -d "$(jq -nc --argjson menu "$menu" \
          '{name:"Monitor incident viewer",code:"monitor_incident_viewer",status:1,menuIds:[$menu]}')" \
        "$admin/api/system/roles" >/dev/null || return 1
    role=$(curl_json "${owner_auth[@]}" "$admin/api/system/roles/options?limit=500" \
        | jq -er '.data[] | select(.code == "monitor_incident_viewer") | .value') || return 1
    curl_json "${owner_auth[@]}" -H 'content-type: application/json' \
        -d "$(jq -nc --argjson role "$role" '
          {username:"monitor_incident_viewer",email:"monitor-incident-viewer@example.test",
           password:"monitor-incident-viewer-password",realName:"Monitor incident viewer",
           status:1,roleIds:[$role]}')" \
        "$admin/api/system/users" >/dev/null || return 1
    login=$(curl_json -H 'content-type: application/json' \
        -d '{"username":"monitor_incident_viewer","password":"monitor-incident-viewer-password"}' \
        "$admin/api/auth/login") || return 1
    viewer_token=$(jq -er '.data.token' <<<"$login") || return 1
}

monitor_delivery_capture() {
    local monitor_db=/opt/rz/data/db/monitor.db evidence
    refresh_owner_auth || return 1
    evidence=$(monitor_delivery_evidence_root) || return 1
    monitor_delivery_prepare "$monitor_db" before-browser || return 1
    delivery_owner_run=$(monitor_delivery_browser_case \
        monitor-delivery-owner owner rustzen@123 dark en-US 1440 900 \
        '15 irreversible notification delivery gaps') || return 1
    delivery_viewer_run=$(monitor_delivery_browser_case \
        monitor-delivery-viewer monitor_incident_viewer monitor-incident-viewer-password light zh-CN 390 844 \
        '通知投递存在 15 个不可恢复缺口') || return 1
    monitor_delivery_prepare "$monitor_db" after-browser || return 1
    reports_ui_delivery_db_json "$monitor_db" > "$evidence/monitor-delivery-db.json" || return 1
    curl_json "${auth[@]}" "$admin/api/monitor/notification-delivery" \
        > "$evidence/monitor-delivery-owner.json" || return 1
    curl_json -H "authorization: Bearer $viewer_token" "$admin/api/monitor/notification-delivery" \
        > "$evidence/monitor-delivery-viewer.json" || return 1
    jq -e "$(monitor_delivery_expected_jq)" "$evidence/monitor-delivery-owner.json" >/dev/null || return 1
    jq -e "$(monitor_delivery_expected_jq)" "$evidence/monitor-delivery-viewer.json" >/dev/null || return 1
    cmp <(jq -Sc '.data' "$evidence/monitor-delivery-owner.json") "$evidence/monitor-delivery-db.json" || return 1
    cmp <(jq -Sc '.data' "$evidence/monitor-delivery-viewer.json") "$evidence/monitor-delivery-db.json" || return 1
    delivery_owner_api=$(monitor_delivery_descriptor "$evidence/monitor-delivery-owner.json") || return 1
    delivery_viewer_api=$(monitor_delivery_descriptor "$evidence/monitor-delivery-viewer.json") || return 1
    delivery_db=$(monitor_delivery_descriptor "$evidence/monitor-delivery-db.json") || return 1
    delivery_owner_steps=$(monitor_delivery_descriptor \
        "$evidence/delivery-monitor-delivery-owner-steps.json") || return 1
    delivery_viewer_steps=$(monitor_delivery_descriptor \
        "$evidence/delivery-monitor-delivery-viewer-steps.json") || return 1
    delivery_owner_flow=$(monitor_delivery_descriptor "$evidence/delivery-monitor-delivery-owner-flow.json") || return 1
    delivery_viewer_flow=$(monitor_delivery_descriptor \
        "$evidence/delivery-monitor-delivery-viewer-flow.json") || return 1
    delivery_owner_run_receipt=$(monitor_delivery_descriptor \
        "$evidence/delivery-monitor-delivery-owner-run.json") || return 1
    delivery_viewer_run_receipt=$(monitor_delivery_descriptor \
        "$evidence/delivery-monitor-delivery-viewer-run.json") || return 1
    delivery_owner_artifact=$(monitor_delivery_screenshot "$delivery_owner_run" monitor-delivery-owner \
        monitor-delivery-owner-desktop-dark-en.png 1440 900) || return 1
    delivery_viewer_artifact=$(monitor_delivery_screenshot "$delivery_viewer_run" monitor-delivery-viewer \
        monitor-delivery-viewer-mobile-light-zh.png 390 844) || return 1
    rm -f "$evidence/.monitor-delivery-prepare-before-browser.json" \
        "$evidence/.monitor-delivery-prepare-after-browser.json"
}
