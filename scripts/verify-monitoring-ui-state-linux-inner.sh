#!/usr/bin/env bash
set -euo pipefail

inner_lib=$(printenv RUSTZEN_MONITORING_UI_STATE_INNER_LIB 2>/dev/null || true)
[ -n "$inner_lib" ] || inner_lib=/verify/inner-lib.sh
. "$inner_lib"

admin=http://127.0.0.1:19801
login_body='{"username":"owner","password":"rustzen@123"}'
token=$(curl_json \
    -H 'content-type: application/json' \
    -d "$login_body" \
    "$admin/api/auth/login" | jq -er '.data.token')
auth=(-H "authorization: Bearer $token")
system_body='{
  "name":"Monitoring UI fixture",
  "baseUrl":"http://127.0.0.1:19806/health",
  "enabled":true
}'
system=$(curl_json \
    "${auth[@]}" \
    -H 'content-type: application/json' \
    -d "$system_body" \
    "$admin/api/reports/systems" | jq -er '.data.id')
steps=$(cat /verify/evidence/browser-steps.json)
mkdir -p /verify/evidence/run-steps
receipt_records=/verify/evidence/.run-receipts.jsonl
: >"$receipt_records"

set_mode() {
    curl_json \
        -X PATCH \
        -H 'content-type: application/json' \
        -d "$1" \
        http://127.0.0.1:19806/__monitoring_fixture/mode >/dev/null
}

diagnostics() {
    local case_name=$1 run_id=$2
    echo "== Monitoring UI browser case failed: $case_name ($run_id) ==" >&2
    curl_json "${auth[@]}" "$admin/api/reports/runs/$run_id" \
        | jq -c '.data | {id,status,error}' >&2 || true
    curl_json "${auth[@]}" "$admin/api/reports/runs/$run_id/steps" \
        | jq -c '.data[] | {stepIndex,action,status,message}' >&2 || true
    curl_json "${auth[@]}" "$admin/api/reports/runs/$run_id/artifacts" \
        >/verify/evidence/failure-run-artifacts.json 2>/dev/null || true
    curl_json http://127.0.0.1:19806/__monitoring_fixture/receipt \
        >/verify/evidence/fixture-receipt-on-failure.json 2>/dev/null || true
}

run_case() {
    local case_name=$1 stem=$2 case_steps body flow run status run_body
    local receipt_tmp receipt_file receipt_hash
    case_steps=$(jq -ce --arg case "$case_name" '.[$case]' <<<"$steps")
    body=$(jq -nc \
        --arg system "$system" \
        --arg name "Monitoring UI state: $case_name" \
        --argjson steps "$case_steps" \
        '{systemId:$system,name:$name,steps:$steps}')
    flow=$(curl_json \
        "${auth[@]}" \
        -H 'content-type: application/json' \
        -d "$body" \
        "$admin/api/reports/flows" | jq -er '.data.id')
    run_body=$(jq -nc --arg flow "$flow" '{flowId:$flow,input:{}}')
    run=$(curl_json \
        "${auth[@]}" \
        -H 'content-type: application/json' \
        -d "$run_body" \
        "$admin/api/reports/runs" | jq -er '.data.id')
    status=queued
    for retry in $(seq 1 900); do
        status=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$run" \
            | jq -er '.data.status')
        case "$status" in
            queued|running) sleep .1 ;;
            *) break ;;
        esac
    done
    if [ "$status" != succeeded ]; then
        diagnostics "$case_name" "$run"
        exit 1
    fi
    receipt_file="$stem.json"
    receipt_tmp="/verify/evidence/run-steps/.$receipt_file.tmp"
    curl_json "${auth[@]}" "$admin/api/reports/runs/$run/steps" \
        | jq -e '.data | map({action,status,message})' >"$receipt_tmp"
    mv -f "$receipt_tmp" "/verify/evidence/run-steps/$receipt_file"
    receipt_hash=$(sha256sum "/verify/evidence/run-steps/$receipt_file" \
        | awk '{print $1}')
    jq -nc \
        --arg case "$case_name" \
        --arg run "$run" \
        --arg file "$receipt_file" \
        --arg sha "$receipt_hash" \
        --slurpfile run_steps "/verify/evidence/run-steps/$receipt_file" \
        '{
            case:$case,
            runId:$run,
            file:$file,
            sha256:$sha,
            steps:$run_steps[0]
        }' >>"$receipt_records"
    last_run_id=$run
}

run_route_matrix() {
    local route=$1 title=$2 after
    after="$route""FailAfterFirstStatus"
    set_mode "$(jq -nc --arg route "$route" '{($route):"slow"}')"
    run_case "$title""Loading" "$route-loading"
    if [ "$route" = overview ]; then
        desktop_run=$last_run_id
    fi
    set_mode "$(jq -nc --arg route "$route" '{($route):"403"}')"
    run_case "$title""403" "$route-403"
    set_mode "$(jq -nc --arg route "$route" '{($route):"500"}')"
    run_case "$title""500" "$route-500"
    set_mode "$(jq -nc \
        --arg route "$route" \
        --arg after "$after" \
        '{($route):"success",($after):"403"}')"
    run_case "$title""Background403" "$route-background-403"
    set_mode "$(jq -nc \
        --arg route "$route" \
        --arg after "$after" \
        '{($route):"success",($after):"500"}')"
    run_case "$title""Background500" "$route-background-500"
}

run_route_matrix overview overview
run_route_matrix nodes nodes
run_route_matrix incidents incidents
run_route_matrix summaries summaries

set_mode '{"incidents":"success","summaries":"success"}'
run_case incidentsPaging incidents-paging
run_case incidentsFilters incidents-filters
run_case summariesPaging summaries-paging
mobile_run=$last_run_id

receipt_tmp=/verify/evidence/.fixture-receipt.json.tmp
curl_json http://127.0.0.1:19806/__monitoring_fixture/receipt \
    | jq -e . >"$receipt_tmp"
mv -f "$receipt_tmp" /verify/evidence/fixture-receipt.json
fixture_receipt=$(cat /verify/evidence/fixture-receipt.json)

jq -e '
    . as $receipt
    | def modes($name):
        [$receipt.requests[] | select(.name == $name) | .mode];
    def expected:
        ["slow", "403", "500", "success", "403", "success", "500"];
    (["overview", "nodes", "incidents", "summaries"]
        | all(. as $name | modes($name)[0:7] == expected))
    and ([$receipt.requests[]
        | select(
            .name == "incidents"
            and .mode == "success"
            and .query.current == ["2"]
        )] | length >= 2)
    and ([$receipt.requests[]
        | select(
            .name == "incidents"
            and .mode == "success"
            and .query.current == ["1"]
            and .query.status == ["active"]
        )] | length >= 1)
    and ([$receipt.requests[]
        | select(
            .name == "incidents"
            and .mode == "success"
            and .query.current == ["1"]
            and .query.status == ["active"]
            and .query.kind == ["cpuHigh"]
        )] | length >= 1)
    and ([$receipt.requests[]
        | select(
            .name == "summaries"
            and .mode == "success"
            and .query.current == ["2"]
        )] | length >= 1)
' <<<"$fixture_receipt" >/dev/null

download_artifact() {
    local run=$1 prefix=$2 output=$3 response id
    response=$(curl_json "${auth[@]}" \
        "$admin/api/reports/runs/$run/artifacts")
    test "$(jq -r \
        --arg prefix "$prefix" \
        '[.data[] | select(.fileName | startswith($prefix))] | length' \
        <<<"$response")" = 1
    id=$(jq -er \
        --arg prefix "$prefix" \
        '.data[] | select(.fileName | startswith($prefix)) | .id' \
        <<<"$response")
    curl_json "${auth[@]}" \
        "$admin/api/reports/runs/$run/artifacts/$id" \
        >"/verify/evidence/$output"
    sha256sum "/verify/evidence/$output" | awk '{print $1}'
}

desktop_file=monitoring-overview-desktop-dark-en.png
mobile_file=monitoring-summaries-mobile-light-zh.png
desktop_sha=$(download_artifact \
    "$desktop_run" monitoring-overview-desktop-dark-en "$desktop_file")
mobile_sha=$(download_artifact \
    "$mobile_run" monitoring-summaries-mobile-light-zh "$mobile_file")
desktop_size=$(file "/verify/evidence/$desktop_file" \
    | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
mobile_size=$(file "/verify/evidence/$mobile_file" \
    | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
test "$desktop_size" = "1440 x 900"
test "$mobile_size" = "390 x 844"

records=$(jq -s . "$receipt_records")
test "$(jq 'length' <<<"$records")" = 23
verify_head=$(env_value RUSTZEN_VERIFY_HEAD)
verify_state=$(env_value RUSTZEN_VERIFY_SOURCE_TREE_STATE)
verify_sha=$(env_value RUSTZEN_VERIFY_SOURCE_TREE_SHA256)
verify_platform=$(env_value RUSTZEN_VERIFY_PLATFORM)
for value in "$verify_head" "$verify_state" "$verify_sha" "$verify_platform"; do
    [ -n "$value" ]
done
jq -n \
    --arg head "$verify_head" \
    --arg state "$verify_state" \
    --arg sha "$verify_sha" \
    --arg platform "$verify_platform" \
    --arg chromium "$verify_chromium" \
    --arg desktop_file "$desktop_file" \
    --arg desktop_sha "$desktop_sha" \
    --arg desktop_size "$desktop_size" \
    --arg mobile_file "$mobile_file" \
    --arg mobile_sha "$mobile_sha" \
    --arg mobile_size "$mobile_size" \
    --argjson records "$records" \
    --argjson fixture "$fixture_receipt" '
    {
        schemaVersion:1,
        status:"passed",
        runCount:($records | length),
        gitHead:$head,
        sourceTreeState:$state,
        sourceTreeSha256:$sha,
        platform:$platform,
        chromiumVersion:$chromium,
        runs:(
            $records
            | map({key:.case,value:.runId})
            | from_entries
        ),
        runSteps:(
            $records
            | map({key:.case,value:.steps})
            | from_entries
        ),
        stepReceipts:(
            $records
            | map({
                case:.case,
                file:.file,
                sha256:.sha256
            })
        ),
        fixtureReceipt:$fixture,
        artifacts:[
            {
                file:$desktop_file,
                sha256:$desktop_sha,
                dimensions:$desktop_size
            },
            {
                file:$mobile_file,
                sha256:$mobile_sha,
                dimensions:$mobile_size
            }
        ]
    }
' >/verify/evidence/manifest.json
rm -f "$receipt_records"
