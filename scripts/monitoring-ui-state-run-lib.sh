#!/usr/bin/env bash
set_mode() {
    curl_json -X PATCH -H 'content-type: application/json' -d "$1" \
        http://127.0.0.1:19806/__monitoring_fixture/mode >/dev/null
}

set_slow_hold() {
    local route=$1
    curl_json -X PATCH -H 'content-type: application/json' \
        -d "$(jq -nc --arg route "$route" '{route:$route}')" \
        http://127.0.0.1:19806/__monitoring_fixture/hold >/dev/null
}

release_slow_hold() {
    local route=$1 trigger=$2
    curl --fail --silent --show-error --connect-timeout 0.3 --max-time 2 \
        "${auth[@]}" -X PATCH -H 'content-type: application/json' \
        -d "$(jq -nc --arg route "$route" --arg trigger "$trigger" '{route:$route,trigger:$trigger}')" \
        http://127.0.0.1:19806/__monitoring_fixture/release >/dev/null
}

release_after_loading_assert() {
    local run=$1 steps=$2 route=$3 assert_index status=pending current trigger=cleanup
    local deadline=$((SECONDS + 120)) started finished
    started=$(date -u +%Y-%m-%dT%H:%M:%S.%NZ)
    assert_index=$(jq -er \
        '[to_entries[] | select(.value.action == "assertText" and
         .value.selector == ".shell-content") | .key] | first' \
        <<<"$steps")
    while [ "$SECONDS" -lt "$deadline" ]; do
        current=$(curl --fail --silent --show-error --connect-timeout 0.3 --max-time 0.8 \
            "${auth[@]}" "$admin/api/reports/runs/$run/steps" 2>/dev/null || true)
        status=$(jq -r --argjson index "$assert_index" \
            '[.data[] | select(.stepIndex == $index) | .status] | first // "pending"' \
            <<<"$current" 2>/dev/null || printf pending)
        case "$status" in
            succeeded) trigger=assertText-succeeded; break ;;
            failed|cancelled) trigger=cleanup; break ;;
            *) sleep 0.1 ;;
        esac
    done
    finished=$(date -u +%Y-%m-%dT%H:%M:%S.%NZ)
    release_slow_hold "$route" "$trigger"
    jq -n \
        --arg run "$run" --arg route "$route" --arg assertionStatus "$status" \
        --arg releaseTrigger "$trigger" --arg startedAt "$started" --arg finishedAt "$finished" \
        '{runId:$run,route:$route,assertionStatus:$assertionStatus,releaseTrigger:$releaseTrigger,
          runObservationBudgetSeconds:120,fixtureHoldTimeoutSeconds:30,
          startedAt:$startedAt,finishedAt:$finishedAt}' \
        >"/verify/evidence/loading-barrier-$route.json"
    [ "$status" = succeeded ]
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
    : > /verify/evidence/failure-artifact-bytes.jsonl
    while IFS=$'\t' read -r artifact_id artifact_kind; do
        case "$artifact_kind" in live-frame|screenshot) ;; *) continue ;; esac
        local file="failure-$artifact_kind-$artifact_id.png"
        if curl -fsS "${auth[@]}" "$admin/api/reports/runs/$run_id/artifacts/$artifact_id" \
            >"/verify/evidence/$file" 2>/dev/null; then
            local hash
            hash=$(sha256sum "/verify/evidence/$file" | awk '{print $1}')
            jq -nc --arg id "$artifact_id" --arg kind "$artifact_kind" \
                --arg file "$file" --arg hash "$hash" \
                '{artifactId:$id,kind:$kind,file:$file,sha256:$hash}' \
                >> /verify/evidence/failure-artifact-bytes.jsonl
        fi
    done < <(jq -r '.data[] | [.id,.kind] | @tsv' \
        /verify/evidence/failure-run-artifacts.json 2>/dev/null || true)
    curl -fsS "${auth[@]}" "$admin/api/reports/runs/$run_id/live-frame" \
        >/verify/evidence/failure-live-frame.png 2>/dev/null || true
    if [ -s /verify/evidence/failure-live-frame.png ]; then
        sha256sum /verify/evidence/failure-live-frame.png \
            >/verify/evidence/failure-live-frame.sha256
    fi
    curl_json http://127.0.0.1:19806/__monitoring_fixture/receipt \
        >/verify/evidence/fixture-receipt-on-failure.json 2>/dev/null || true
}

run_route_matrix() {
    local route=$1 title=$2 after
    after="$route""FailAfterFirstStatus"
    set_mode "$(jq -nc --arg route "$route" '{($route):"slow"}')"
    run_case "$title""Loading" "$route-loading"
    if [ "$route" = overview ]; then desktop_run=$last_run_id; fi
    set_mode "$(jq -nc --arg route "$route" '{($route):"403"}')"
    run_case "$title""403" "$route-403"
    set_mode "$(jq -nc --arg route "$route" '{($route):"500"}')"
    run_case "$title""500" "$route-500"
    set_mode "$(jq -nc --arg route "$route" --arg after "$after" \
        '{($route):"success",($after):"403"}')"
    run_case "$title""Background403" "$route-background-403"
    set_mode "$(jq -nc --arg route "$route" --arg after "$after" \
        '{($route):"success",($after):"500"}')"
    run_case "$title""Background500" "$route-background-500"
}
