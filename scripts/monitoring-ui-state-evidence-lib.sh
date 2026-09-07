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
        and (.artifacts | length) == 2
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
}

verify_evidence_files() {
    local directory=$1 manifest=$2 case receipt stem expected_hash actual_hash
    local root_files step_files expected_step_files
    root_files=$(find "$directory" -maxdepth 1 -type f -exec basename {} \; | sort)
    expected_root_files=$(cat <<'FILES'
browser-steps.json
build-provenance.txt
fixture-receipt.json
manifest.json
monitoring-overview-desktop-dark-en.png
monitoring-summaries-mobile-light-zh.png
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
}
