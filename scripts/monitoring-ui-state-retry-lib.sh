#!/usr/bin/env bash
# One bounded retry receipt for an infrastructure failure before Chromium reads the fixture.

monitor_retry_db_binding() {
    local source_run=$1 child_run=$2
    python3 -B - /opt/rz/data/reports/db/reports.db "$source_run" "$child_run" <<'PY'
import json
import sqlite3
import sys

database, source, child = sys.argv[1:]
with sqlite3.connect(database, timeout=5.0) as connection:
    source_row = connection.execute(
        "SELECT id, flow_id FROM automation_runs WHERE id=?", (source,)).fetchone()
    child_row = connection.execute(
        "SELECT id, flow_id, retry_source_run_id FROM automation_runs WHERE id=?", (child,)).fetchone()
    child_count = connection.execute(
        "SELECT COUNT(*) FROM automation_runs WHERE retry_source_run_id=?", (source,)).fetchone()[0]
if source_row is None or child_row is None:
    raise RuntimeError("retry run rows were not persisted")
if child_row[2] != source or source_row[1] != child_row[1] or child_count != 1:
    raise RuntimeError("retry run binding is not one source child on one flow")
print(json.dumps({"sourceRun": source_row[0], "childRun": child_row[0], "sourceFlowId": source_row[1],
    "childFlowId": child_row[1], "childRetrySourceRunId": child_row[2], "sourceChildCount": child_count},
    separators=(",", ":"), sort_keys=True))
PY
}

monitor_retry_append_receipt() {
    local case_name=$1 source_run=$2 child_run=$3 reads_before=$4 reads_after=$5
    local source_api=$6 source_steps=$7 binding=$8 receipt temporary
    receipt=${monitor_retry_receipts:?}
    temporary="$receipt.tmp"
    jq -n --arg case "$case_name" --arg source "$source_run" --arg child "$child_run" \
        --argjson before "$reads_before" --argjson after "$reads_after" \
        --slurpfile source_api "$source_api" --slurpfile source_steps "$source_steps" \
        --argjson binding "$binding" '
        {case:$case,sourceRun:$source,childRun:$child,fixtureReads:{before:$before,after:$after},
         sourceRunApi:$source_api[0],sourceStepsApi:$source_steps[0],database:$binding}' >"$temporary" || return 1
    jq -s '.[0] + [.[1]]' "$receipt" "$temporary" >"$temporary.next" || return 1
    mv -f "$temporary.next" "$receipt"
    rm -f "$temporary"
}

monitor_retry_prepage_cdp() {
    local case_name=$1 source_run=$2 reads_before=$3 run steps reads_after child binding
    local source_api=${monitor_retry_source_api:?} source_steps=${monitor_retry_source_steps:?}
    run=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$source_run") || return 1
    printf '%s\n' "$run" >"$source_api" || return 1
    steps=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$source_run/steps") || return 1
    printf '%s\n' "$steps" >"$source_steps" || return 1
    reads_after=$(monitor_fixture_reads) || return 1
    jq -e --argjson before "$reads_before" --argjson after "$reads_after" --argjson steps "$steps" '
      .data as $run | $steps.data as $steps | $before == $after and $run.status == "failed"
      and $run.error == "reports service operation failed"
      and (($steps | length == 0)
           or ($steps | length == 1 and .[0].stepIndex == 0
               and .[0].action == "setUiPreferences" and .[0].status == "succeeded"))
    ' <<<"$run" >/dev/null || return 1
    child=$(curl_json "${auth[@]}" -X POST "$admin/api/reports/runs/$source_run/retry" | jq -er '.data.id') || return 1
    binding=$(monitor_retry_db_binding "$source_run" "$child") || return 1
    monitor_retry_append_receipt "$case_name" "$source_run" "$child" "$reads_before" \
        "$reads_after" "$source_api" "$source_steps" "$binding" || return 1
    printf '%s\n' "$child"
}
