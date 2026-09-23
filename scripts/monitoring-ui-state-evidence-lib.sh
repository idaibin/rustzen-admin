#!/usr/bin/env bash
# Strict evidence validators and the canonical 23-case inventory.

expected_cases() {
    cat <<'CASES'
overviewLoading	overview-loading
overview403	overview-403
overview500	overview-500
overviewBackground403	overview-background-403
overviewBackground500	overview-background-500
nodesLoading	nodes-loading
nodes403	nodes-403
nodes500	nodes-500
nodesBackground403	nodes-background-403
nodesBackground500	nodes-background-500
incidentsLoading	incidents-loading
incidents403	incidents-403
incidents500	incidents-500
incidentsBackground403	incidents-background-403
incidentsBackground500	incidents-background-500
summariesLoading	summaries-loading
summaries403	summaries-403
summaries500	summaries-500
summariesBackground403	summaries-background-403
summariesBackground500	summaries-background-500
incidentsPaging	incidents-paging
incidentsFilters	incidents-filters
summariesPaging	summaries-paging
CASES
}

expected_names_json() {
    expected_cases | cut -f1 | jq -Rsc 'split("\n") | map(select(length > 0))'
}
verify_fixture_receipt() {
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
    ' "$1" >/dev/null
}

verify_step_receipts() {
    local manifest=$1 expected=$2 names
    names=$(expected_names_json)
    jq -e \
        --argjson names "$names" \
        --slurpfile expected "$expected" '
        . as $manifest
        | ($expected[0] | keys | sort) == ($names | sort)
        and ($manifest.runSteps | keys | sort) == ($names | sort)
        and ([$names[] as $name
            | ($expected[0][$name]) as $expected_steps
            | ($manifest.runSteps[$name]) as $actual_steps
            | ($actual_steps | type) == "array"
            and ($actual_steps | length) == ($expected_steps | length)
            and ([range(0; $expected_steps | length) as $index
                | $actual_steps[$index].action == $expected_steps[$index].action
                and $actual_steps[$index].status == "succeeded"
                and $actual_steps[$index].message == null
            ] | all)
        ] | all)
    ' "$manifest" >/dev/null
}

verify_retry_receipts() {
    local directory=$1 manifest=$2 file hash bytes actual_hash actual_bytes
    file=$(jq -er '.retryReceipts.file' "$manifest") || return 1
    hash=$(jq -er '.retryReceipts.sha256' "$manifest") || return 1
    bytes=$(jq -er '.retryReceipts.bytes' "$manifest") || return 1
    test "$file" = retry-receipts.json
    test -f "$directory/$file" && test ! -L "$directory/$file" || return 1
    actual_hash=$(sha256sum "$directory/$file" | awk '{print $1}') || return 1
    actual_bytes=$(wc -c <"$directory/$file" | tr -d ' ') || return 1
    test "$actual_hash" = "$hash" && test "$actual_bytes" = "$bytes" || return 1
    jq -e --argjson runs "$(jq -c '.runs' "$manifest")" '
      type == "array" and length <= 1
      and all(.[]; . as $receipt |
        ($receipt | keys | sort
         == ["case","childRun","database","fixtureReads","sourceRun","sourceRunApi","sourceStepsApi"])
        and ($receipt.case | type == "string" and length > 0)
        and $receipt.sourceRun != $receipt.childRun and $runs[$receipt.case] == $receipt.childRun
        and ($receipt.fixtureReads | keys | sort == ["after","before"])
        and ($receipt.fixtureReads.before | type == "number")
        and $receipt.fixtureReads.before == $receipt.fixtureReads.after
        and $receipt.sourceRunApi.data.id == $receipt.sourceRun
        and $receipt.sourceRunApi.data.status == "failed"
        and $receipt.sourceRunApi.data.error == "reports service operation failed"
        and (($receipt.sourceStepsApi.data | length == 0)
          or (($receipt.sourceStepsApi.data | length == 1)
            and $receipt.sourceStepsApi.data[0].runId == $receipt.sourceRun
            and ($receipt.sourceStepsApi.data[0] | {stepIndex,action,status}
                 == {stepIndex:0,action:"setUiPreferences",status:"succeeded"})))
        and $receipt.database == {sourceRun:$receipt.sourceRun,childRun:$receipt.childRun,
          sourceFlowId:$receipt.sourceRunApi.data.flowId,childFlowId:$receipt.sourceRunApi.data.flowId,
          childRetrySourceRunId:$receipt.sourceRun,sourceChildCount:1}
      )
    ' "$directory/$file" >/dev/null
}

verify_manifest() {
    local manifest=$1 expected=$2 head=$3 state=$4 sha=$5 platform=$6 names
    names=$(expected_names_json)
    jq -e \
        --arg head "$head" \
        --arg state "$state" \
        --arg sha "$sha" \
        --arg platform "$platform" \
        --argjson names "$names" '
        .schemaVersion == 1
        and .status == "passed"
        and .runCount == 23
        and .gitHead == $head
        and .sourceTreeState == $state
        and .sourceTreeSha256 == $sha
        and .platform == $platform
        and (.chromiumVersion | type == "string" and length > 0)
        and (.runs | keys | sort) == ($names | sort)
        and ([.runs[] | type == "string" and length > 0] | all)
        and (.runSteps | keys | sort) == ($names | sort)
        and ([.stepReceipts[].case] | sort) == ($names | sort)
        and (.stepReceipts | length) == 23
        and (.fixtureReceipt.requests | type == "array")
        and (.retryReceipts | keys | sort == ["bytes","file","sha256"])
        and (.retryReceipts.file == "retry-receipts.json")
        and (.retryReceipts.sha256 | type == "string" and test("^[0-9a-f]{64}$"))
        and (.retryReceipts.bytes | type == "number" and . > 0)
        and (.artifacts | length) == 4
        and (.deliveryHealth.gapTotal == 15)
        and (.deliveryHealth | keys | sort == ["database","gapTotal","ownerApi","ownerFlow","ownerRun",
                                                "ownerRunReceipt","ownerSteps","viewerApi","viewerFlow","viewerRun",
                                                "viewerRunReceipt","viewerSteps"])
        and (.deliveryHealth.ownerRun | type == "string" and length > 0)
        and (.deliveryHealth.viewerRun | type == "string" and length > 0)
        and (.deliveryHealth.ownerRun != .deliveryHealth.viewerRun)
        and (.deliveryHealth.ownerApi.file == "monitor-delivery-owner.json")
        and (.deliveryHealth.viewerApi.file == "monitor-delivery-viewer.json")
        and (.deliveryHealth.database.file == "monitor-delivery-db.json")
        and (.deliveryHealth.ownerSteps.file == "delivery-monitor-delivery-owner-steps.json")
        and (.deliveryHealth.viewerSteps.file == "delivery-monitor-delivery-viewer-steps.json")
        and (.deliveryHealth.ownerFlow.file == "delivery-monitor-delivery-owner-flow.json")
        and (.deliveryHealth.viewerFlow.file == "delivery-monitor-delivery-viewer-flow.json")
        and (.deliveryHealth.ownerRunReceipt.file == "delivery-monitor-delivery-owner-run.json")
        and (.deliveryHealth.viewerRunReceipt.file == "delivery-monitor-delivery-viewer-run.json")
        and ([.deliveryHealth.ownerApi,.deliveryHealth.viewerApi,.deliveryHealth.database,
              .deliveryHealth.ownerSteps,.deliveryHealth.viewerSteps,
              .deliveryHealth.ownerFlow,.deliveryHealth.viewerFlow,
              .deliveryHealth.ownerRunReceipt,.deliveryHealth.viewerRunReceipt] | all(
            (keys | sort == ["bytes","file","sha256"])
            and (.sha256 | type == "string" and test("^[0-9a-f]{64}$"))
            and (.bytes | type == "number" and . > 0)))
        and ([.deliveryHealth.ownerApi,.deliveryHealth.viewerApi,.deliveryHealth.database,
              .deliveryHealth.ownerSteps,.deliveryHealth.viewerSteps,
              .deliveryHealth.ownerFlow,.deliveryHealth.viewerFlow,
              .deliveryHealth.ownerRunReceipt,.deliveryHealth.viewerRunReceipt | .file]
             | unique | length == 9)
        and ([.artifacts[] | select(.case == "monitor-delivery-owner" or .case == "monitor-delivery-viewer")]
             | length == 2)
        and ([.artifacts[] | select(.case == "monitor-delivery-owner")]
             | length == 1 and .[0].file == "monitor-delivery-owner-desktop-dark-en.png"
             and .[0].dimensions == "1440 x 900")
        and ([.artifacts[] | select(.case == "monitor-delivery-viewer")]
             | length == 1 and .[0].file == "monitor-delivery-viewer-mobile-light-zh.png"
             and .[0].dimensions == "390 x 844")
    ' "$manifest" >/dev/null
    verify_step_receipts "$manifest" "$expected"
}

verify_artifacts() {
    local directory=$1 manifest=$2 desktop mobile desktop_size mobile_size
    desktop=$(shasum -a 256 \
        "$directory/monitoring-overview-desktop-dark-en.png" | awk '{print $1}')
    mobile=$(shasum -a 256 \
        "$directory/monitoring-summaries-mobile-light-zh.png" | awk '{print $1}')
    desktop_size=$(file "$directory/monitoring-overview-desktop-dark-en.png" \
        | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
    mobile_size=$(file "$directory/monitoring-summaries-mobile-light-zh.png" \
        | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
    jq -e \
        --arg desktop "$desktop" \
        --arg mobile "$mobile" '
        ([.artifacts[]
            | select(
                .file == "monitoring-overview-desktop-dark-en.png"
                and .sha256 == $desktop
                and .dimensions == "1440 x 900"
            )] | length) == 1
        and ([.artifacts[]
            | select(
                .file == "monitoring-summaries-mobile-light-zh.png"
                and .sha256 == $mobile
                and .dimensions == "390 x 844"
            )] | length) == 1
    ' "$manifest" >/dev/null
    test "$desktop_size" = "1440 x 900"
    test "$mobile_size" = "390 x 844"
    for item in \
        'monitor-delivery-owner|monitor-delivery-owner-desktop-dark-en.png|1440 x 900' \
        'monitor-delivery-viewer|monitor-delivery-viewer-mobile-light-zh.png|390 x 844'; do
        IFS='|' read -r case name dimensions <<<"$item"
        hash=$(sha256sum "$directory/$name" | awk '{print $1}')
        jq -e --arg case "$case" --arg file "$name" --arg hash "$hash" \
            --arg dimensions "$dimensions" \
            '[.artifacts[] | select(.case == $case and .file == $file
              and .sha256 == $hash and .dimensions == $dimensions
              and (keys | sort == ["bytes","case","dimensions","file","sha256"]))] | length == 1' \
            "$manifest" >/dev/null
    done
}

verify_evidence_files() {
    local directory=$1 manifest=$2 case receipt stem expected_hash actual_hash
    local root_files step_files expected_step_files
    root_files=$(find "$directory" -maxdepth 1 -type f -exec basename {} \; | sort)
    expected_root_files=$(cat <<'FILES'
browser-steps.json
build-provenance.txt
delivery-monitor-delivery-owner-flow.json
delivery-monitor-delivery-owner-run.json
delivery-monitor-delivery-owner-steps.json
delivery-monitor-delivery-viewer-flow.json
delivery-monitor-delivery-viewer-run.json
delivery-monitor-delivery-viewer-steps.json
fixture-receipt.json
manifest.json
monitor-delivery-db.json
monitor-delivery-owner-desktop-dark-en.png
monitor-delivery-owner.json
monitor-delivery-viewer-mobile-light-zh.png
monitor-delivery-viewer.json
monitoring-overview-desktop-dark-en.png
monitoring-summaries-mobile-light-zh.png
retry-receipts.json
FILES
)
    test "$root_files" = "$expected_root_files"
    expected_step_files=$(expected_cases | cut -f2 | sed 's/$/.json/' | sort)
    step_files=$(find "$directory/run-steps" -maxdepth 1 -type f \
        -exec basename {} \; | sort)
    test "$step_files" = "$expected_step_files"
    test -z "$(find "$directory" -type l -print -quit)"
    jq -e --slurpfile receipt "$directory/fixture-receipt.json" \
        '.fixtureReceipt == $receipt[0]' "$manifest" >/dev/null
    while IFS=$'\t' read -r case stem; do
        receipt="$stem.json"
        jq -e \
            --arg case "$case" \
            --slurpfile value "$directory/run-steps/$receipt" \
            '.runSteps[$case] == $value[0]' "$manifest" >/dev/null
        expected_hash=$(jq -er \
            --arg case "$case" \
            --arg file "$receipt" \
            '.stepReceipts[]
                | select(.case == $case and .file == $file)
                | .sha256' "$manifest")
        actual_hash=$(shasum -a 256 "$directory/run-steps/$receipt" \
            | awk '{print $1}')
        test "$actual_hash" = "$expected_hash"
    done < <(expected_cases)
    verify_retry_receipts "$directory" "$manifest"
}

. "$(dirname "${BASH_SOURCE[0]}")/monitoring-ui-state-delivery-evidence-lib.sh"
