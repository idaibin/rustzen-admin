#!/usr/bin/env bash

schedule_form_case_map() {
    printf '%s\n' '{"cancel":"cancelCreate","malformedInput":"malformedInput","missingTime":"missingTime","secretRejected":"secretRejected","daily":"createDaily","weekly":"editWeekly","viewer":"viewer"}'
}

save_schedule_form_run_steps() {
    local run_id=$1
    local case_name=$2
    local receipt="run-steps/$case_name.json"
    local root=${RUSTZEN_SCHEDULE_FORM_EVIDENCE_ROOT:-/verify/evidence}
    mkdir -p "$root/run-steps" || return 1
    curl_json "${auth[@]}" "$admin/api/reports/runs/$run_id/steps" >"$root/$receipt" || return 1
    jq -e --arg run "$run_id" '(.data | type == "array" and length > 0) and all(.data[]; .runId == $run)' "$root/$receipt" >/dev/null || return 1
    jq -nc --arg run "$run_id" --arg file "$receipt" --arg sha "$(sha256sum "$root/$receipt" | awk '{print $1}')" '{runId:$run,file:$file,sha256:$sha}'
}

verify_schedule_form_manifest() (
    set -e
    local manifest=$1 head=$2 state=$3 sha=$4 platform=$5 browser_steps=$6 expected
    expected=$(schedule_form_case_map)
    jq -e --arg head "$head" --arg state "$state" --arg sha "$sha" --arg platform "$platform" --argjson expected "$expected" '
        .schemaVersion == 2 and .status == "passed"
        and .gitHead == $head and .sourceTreeState == $state and .sourceTreeSha256 == $sha and .platform == $platform
        and (.chromiumVersion | type == "string" and length > 0)
        and (.runs | keys | sort) == ($expected | keys | sort)
        and (.runSteps | keys | sort) == ($expected | keys | sort)
        and ([.runs[] | type == "string" and length > 0] | all)
        and ([.runSteps | to_entries[] | (.value | keys | sort) == ["file","runId","sha256"] and (.value.file == ("run-steps/" + .key + ".json")) and (.value.sha256 | type == "string" and test("^[0-9a-f]{64}$"))] | all)
        and (.localValidation | keys | sort) == ["proxyPostCount","rowsAfter","rowsBefore"]
        and ([.localValidation.rowsBefore,.localValidation.rowsAfter,.localValidation.proxyPostCount] | all(type == "number" and floor == . and . >= 0))
        and .localValidation.proxyPostCount == 0 and .localValidation.rowsBefore == .localValidation.rowsAfter
        and (.secretPolicy | keys | sort) == ["proxyPostCount","rowDelta","rowsAfter","rowsBefore"]
        and ([.secretPolicy.rowsBefore,.secretPolicy.rowsAfter,.secretPolicy.rowDelta,.secretPolicy.proxyPostCount] | all(type == "number" and floor == . and . >= 0))
        and .secretPolicy.proxyPostCount == 1 and .secretPolicy.rowsBefore == .secretPolicy.rowsAfter and .secretPolicy.rowDelta == (.secretPolicy.rowsAfter - .secretPolicy.rowsBefore)
        and .schedule == {cadence:"weekly",weekday:0,dueTime:"10:16",timezone:"UTC"}
        and .viewOnly == {managementVisible:false,dueTime:"10:16 · UTC"}
        and (.artifacts | type == "array" and length == 2)
        and ([.artifacts[].file] | sort) == ["schedule-form-desktop-dark-en.png","schedule-form-mobile-light-zh.png"]
        and ([.artifacts[] | (.file == "schedule-form-desktop-dark-en.png" and .dimensions == "1440 x 900") or (.file == "schedule-form-mobile-light-zh.png" and .dimensions == "390 x 844")] | all)
        and ([.artifacts[] | .sha256 | type == "string" and test("^[0-9a-f]{64}$")] | all)
    ' "$manifest" >/dev/null
)

verify_schedule_form_receipts() (
    set -e
    local directory=$1 manifest=$2 browser_steps=$3 item case_name source_case run_id file_name expected_hash actual_hash expected_actions
    while IFS= read -r item; do
        case_name=$(jq -er '.key' <<<"$item")
        source_case=$(schedule_form_case_map | jq -er --arg case "$case_name" '.[$case]')
        run_id=$(jq -er --arg case "$case_name" '.runs[$case]' "$manifest")
        test "$run_id" = "$(jq -er '.value.runId' <<<"$item")" || return 1
        file_name=$(jq -er '.value.file' <<<"$item")
        expected_hash=$(jq -er '.value.sha256' <<<"$item")
        test -f "$directory/$file_name" || return 1
        actual_hash=$(shasum -a 256 "$directory/$file_name" | awk '{print $1}')
        test "$actual_hash" = "$expected_hash" || return 1
        expected_actions=$(jq -ce --arg case "$source_case" '.[$case] | map(.action)' "$browser_steps")
        jq -e --arg run "$run_id" --argjson expected "$expected_actions" '(.data | type == "array" and length > 0) and ([.data[].action] == $expected) and all(.data[]; .runId == $run and .status == "succeeded")' "$directory/$file_name" >/dev/null || return 1
    done < <(jq -c '.runSteps | to_entries[]' "$manifest")
)

verify_schedule_form_artifacts() (
    set -e
    local directory=$1 manifest=$2 item file_name expected_hash actual_hash dimensions actual_dimensions
    while IFS= read -r item; do
        file_name=$(jq -er '.file' <<<"$item")
        expected_hash=$(jq -er '.sha256' <<<"$item")
        dimensions=$(jq -er '.dimensions' <<<"$item")
        test -f "$directory/$file_name" || return 1
        actual_hash=$(shasum -a 256 "$directory/$file_name" | awk '{print $1}')
        test "$actual_hash" = "$expected_hash" || return 1
        actual_dimensions=$(file "$directory/$file_name" | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
        test "$actual_dimensions" = "$dimensions" || return 1
    done < <(jq -c '.artifacts[]' "$manifest")
)
