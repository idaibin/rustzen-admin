#!/usr/bin/env bash

reports_ui_state_cases() {
    cat <<'CASES'
managerProcessing
managerRuntimeFailure
managerPartial
viewOnlyMobile
CASES
}

reports_ui_state_expected_actions() {
    case "$1" in
        managerProcessing)
            printf '%s\n' '["setUiPreferences","setViewport","goto","waitFor","fill","fill","click","waitFor","goto","waitFor","waitFor","assertText","assertText","assertText","assertText","assertText","assertText","assertNoHorizontalOverflow","screenshotViewport","click","waitFor","assertText","assertText","click","pause","waitFor","click","waitFor"]'
            ;;
        managerRuntimeFailure)
            printf '%s\n' '["setUiPreferences","setViewport","goto","waitFor","fill","fill","click","waitFor","goto","waitFor","assertText","click","waitFor","assertText","assertText","assertNoHorizontalOverflow","screenshotViewport","click","waitFor"]'
            ;;
        managerPartial)
            printf '%s\n' '["setUiPreferences","setViewport","goto","waitFor","fill","fill","click","waitFor","goto","waitFor","assertText","assertText","assertText","assertAbsent","waitFor","assertNoHorizontalOverflow","screenshotViewport"]'
            ;;
        viewOnlyMobile)
            printf '%s\n' '["setUiPreferences","setViewport","goto","waitFor","fill","fill","click","waitFor","goto","waitFor","assertAbsent","assertAbsent","assertAbsent","assertAbsent","goto","waitFor","waitFor","assertText","assertText","assertText","assertText","assertText","assertText","assertNoHorizontalOverflow","screenshotViewport","assertAbsent","assertAbsent","assertAbsent","click","waitFor","assertAbsent"]'
            ;;
        *) return 1 ;;
    esac
}

verify_reports_ui_state_manifest() (
    set -e
    local manifest=$1 head=$2 state=$3 sha=$4 platform=$5 expected_cases expected_artifacts
    expected_cases=$(reports_ui_state_cases | jq -Rsc 'split("\n") | map(select(length > 0))')
    expected_artifacts='{"managerProcessing":{"file":"reports-processing-desktop-dark-en.png","width":1440,"height":900},"managerRuntimeFailure":{"file":"reports-runtime-failure-desktop-dark-en.png","width":1440,"height":900},"managerPartial":{"file":"reports-partial-desktop-dark-en.png","width":1440,"height":900},"viewOnlyMobile":{"file":"reports-view-only-mobile-light-zh.png","width":390,"height":844}}'
    jq -e \
        --arg head "$head" --arg state "$state" --arg sha "$sha" --arg platform "$platform" \
        --argjson expected_cases "$expected_cases" --argjson expected_artifacts "$expected_artifacts" '
        .schemaVersion == 3
        and .status == "passed"
        and .gitHead == $head and .sourceTreeState == $state and .sourceTreeSha256 == $sha and .platform == $platform
        and (.chromiumVersion | type == "string" and length > 0)
        and (.runs | keys | sort) == ($expected_cases | sort)
        and ([.runs[] | type == "string" and length > 0] | all)
        and (.runSteps | keys | sort) == ($expected_cases | sort)
        and ([.runSteps | to_entries[] | (.value.runId | type == "string" and length > 0) and (.value.file | type == "string" and test("^run-steps/[a-z-]+\\.json$")) and (.value.sha256 | type == "string" and test("^[0-9a-f]{64}$"))] | all)
        and (.sourceEvidence.runId | type == "string" and length > 0)
        and (.sourceEvidence.status == "failed")
        and (.sourceEvidence.error | type == "string" and contains("assertText did not match"))
        and (.sourceEvidence.failedStep | .action == "assertText" and .status == "failed" and (.message | type == "string" and contains("assertText did not match")))
        and (.sourceEvidence.before.run.file == "source-run.before.json" and .sourceEvidence.after.run.file == "source-run.after.json")
        and (.sourceEvidence.before.steps.file == "source-steps.before.json" and .sourceEvidence.after.steps.file == "source-steps.after.json")
        and (.sourceEvidence.before.artifacts.file == "source-artifacts.before.json" and .sourceEvidence.after.artifacts.file == "source-artifacts.after.json")
        and (.sourceEvidence as $source | ["run", "steps", "artifacts"] | all(.[]; . as $kind | ($source.before[$kind].file | type == "string") and ($source.after[$kind].file | type == "string") and ($source.before[$kind].sha256 | test("^[0-9a-f]{64}$")) and ($source.after[$kind].sha256 | test("^[0-9a-f]{64}$")) and ($source.before[$kind].sha256 == $source.after[$kind].sha256)))
        and (.retry | .childId | type == "string" and length > 0)
        and (.retry.sourcePreserved == true)
        and (.processing | .runId | type == "string" and length > 0)
        and (.processing | .status == "cancelled" and .pauseDurationMs == 30000)
        and (.processing.pauseStep | .action == "pause" and .status == "cancelled")
        and (.processing.stepReceipt.file == "processing-run-steps.json" and (.processing.stepReceipt.sha256 | test("^[0-9a-f]{64}$")))
        and (.partialFixture | .enqueuedRunId | type == "string" and length > 0)
        and (.partialFixture.skippedRunLinked == false)
        and (.viewOnly | keys | sort) == ["retryReceipt","scheduleMutationReceipt"]
        and (.viewOnly.scheduleMutationReceipt | keys | sort) == ["file","sha256","status"]
        and (.viewOnly.retryReceipt | keys | sort) == ["file","sha256","status"]
        and .viewOnly.scheduleMutationReceipt == {status:403,file:"viewer-schedule-mutation.json",sha256:.viewOnly.scheduleMutationReceipt.sha256}
        and .viewOnly.retryReceipt == {status:403,file:"viewer-retry.json",sha256:.viewOnly.retryReceipt.sha256}
        and ([.viewOnly.scheduleMutationReceipt,.viewOnly.retryReceipt][] | .sha256 | type == "string" and test("^[0-9a-f]{64}$"))
        and (.deliveryHealth.gapTotal == 15)
        and (.deliveryHealth.ownerReceipt.file == "reports-delivery-owner.json")
        and (.deliveryHealth.viewerReceipt.file == "reports-delivery-viewer.json")
        and (.deliveryHealth.ownerReceipt.file != .deliveryHealth.viewerReceipt.file)
        and ([.deliveryHealth.ownerReceipt,.deliveryHealth.viewerReceipt] | all(
            (.file | type == "string" and test("^reports-delivery-(owner|viewer)\\.json$"))
            and (.sha256 | type == "string" and test("^[0-9a-f]{64}$"))
        ))
        and (.artifacts | type == "array" and length == 4)
        and ([.artifacts[] | .case] | sort) == ($expected_cases | sort)
        and ([.artifacts[] | . as $artifact | $expected_artifacts[$artifact.case] as $expected | $expected != null and $artifact.file == $expected.file and $artifact.viewport.width == $expected.width and $artifact.viewport.height == $expected.height and $artifact.dimensions == ("\($expected.width) x \($expected.height)") and ($artifact.sha256 | type == "string" and test("^[0-9a-f]{64}$"))] | all)
    ' "$manifest" >/dev/null
)

verify_reports_ui_state_delivery_receipts() (
    set -e
    local directory=$1 manifest=$2 receipt file_name expected_hash actual_hash
    for receipt in ownerReceipt viewerReceipt; do
        file_name=$(jq -er --arg receipt "$receipt" '.deliveryHealth[$receipt].file' "$manifest")
        test "$file_name" = "reports-delivery-${receipt%Receipt}.json" || return 1
        expected_hash=$(jq -er --arg receipt "$receipt" '.deliveryHealth[$receipt].sha256' "$manifest")
        actual_hash=$(shasum -a 256 "$directory/$file_name" | awk '{print $1}')
        test "$actual_hash" = "$expected_hash" || return 1
        jq -e '
            .data == {
              pendingCount:2,pendingBytes:1024,quarantineCount:3,quarantineBytes:2048,
              omittedCount:1,expiredCount:2,unconfirmedCount:3,quarantinedCount:4,
              quarantineEvictedCount:5,firstGapAt:"2026-09-10T01:02:03Z",
              lastGapAt:"2026-09-10T02:03:04Z",lastSuccessAt:"2026-09-10T03:04:05Z"
            }
        ' "$directory/$file_name" >/dev/null || return 1
    done
)

verify_reports_ui_state_artifacts() (
    set -e
    local directory=$1 manifest=$2 item file_name hash dimensions actual_hash actual_dimensions
    while IFS= read -r item; do
        file_name=$(jq -er '.file' <<<"$item")
        hash=$(jq -er '.sha256' <<<"$item")
        dimensions=$(jq -er '.dimensions' <<<"$item")
        test -f "$directory/$file_name"
        actual_hash=$(shasum -a 256 "$directory/$file_name" | awk '{print $1}')
        actual_dimensions=$(file "$directory/$file_name" | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
        test "$actual_hash" = "$hash"
        test "$actual_dimensions" = "$dimensions"
    done < <(jq -c '.artifacts[]' "$manifest")
)

verify_reports_ui_state_receipts() (
    set -e
    local directory=$1 manifest=$2 item case_name run_id file_name expected_hash actual_hash
    while IFS= read -r item; do
        case_name=$(jq -er '.key' <<<"$item")
        run_id=$(jq -er '.value.runId' <<<"$item")
        test "$run_id" = "$(jq -er --arg case "$case_name" '.runs[$case]' "$manifest")"
        file_name=$(jq -er '.value.file' <<<"$item")
        expected_hash=$(jq -er '.value.sha256' <<<"$item")
        test -f "$directory/$file_name"
        actual_hash=$(shasum -a 256 "$directory/$file_name" | awk '{print $1}')
        test "$actual_hash" = "$expected_hash"
        jq -e --arg run "$run_id" --argjson expected "$(reports_ui_state_expected_actions "$case_name")" '
            (.data | type == "array" and length > 0)
            and ([.data[].action] == $expected)
            and all(.data[]; .runId == $run and .status == "succeeded")
        ' "$directory/$file_name" >/dev/null || return 1
    done < <(jq -c '.runSteps | to_entries[]' "$manifest")

    local processing_file processing_hash
    processing_file=$(jq -er '.processing.stepReceipt.file' "$manifest")
    processing_hash=$(jq -er '.processing.stepReceipt.sha256' "$manifest")
    test "$(shasum -a 256 "$directory/$processing_file" | awk '{print $1}')" = "$processing_hash"
    jq -e --arg run "$(jq -er '.processing.runId' "$manifest")" '(.data | any(.runId == $run and .action == "pause" and .status == "cancelled"))' "$directory/$processing_file" >/dev/null
)

verify_reports_ui_state_forbidden_receipts() (
    set -e
    local directory=$1 manifest=$2 receipt file_name expected_hash actual_hash
    for receipt in scheduleMutationReceipt retryReceipt; do
        file_name=$(jq -er --arg receipt "$receipt" '.viewOnly[$receipt].file' "$manifest")
        expected_hash=$(jq -er --arg receipt "$receipt" '.viewOnly[$receipt].sha256' "$manifest")
        actual_hash=$(shasum -a 256 "$directory/$file_name" | awk '{print $1}')
        test "$actual_hash" = "$expected_hash" || return 1
        jq -e '. == {code:403,message:"Permission denied",data:null}' "$directory/$file_name" >/dev/null || return 1
    done
)

verify_reports_ui_state_source_evidence() (
    set -e
    local directory=$1 manifest=$2 phase kind file_name expected_hash actual_hash
    for phase in before after; do
        for kind in run steps artifacts; do
            file_name=$(jq -er --arg phase "$phase" --arg kind "$kind" '.sourceEvidence[$phase][$kind].file' "$manifest")
            expected_hash=$(jq -er --arg phase "$phase" --arg kind "$kind" '.sourceEvidence[$phase][$kind].sha256' "$manifest")
            test -f "$directory/$file_name"
            actual_hash=$(shasum -a 256 "$directory/$file_name" | awk '{print $1}')
            test "$actual_hash" = "$expected_hash"
        done
    done
    local run_id
    run_id=$(jq -er '.sourceEvidence.runId' "$manifest")
    jq -e --arg run "$run_id" '.data.id == $run and .data.status == "failed" and (.data.error | contains("assertText did not match"))' "$directory/$(jq -er '.sourceEvidence.before.run.file' "$manifest")" >/dev/null
    jq -e --arg run "$run_id" '.data | any(.runId == $run and .action == "assertText" and .status == "failed" and (.message | contains("assertText did not match")))' "$directory/$(jq -er '.sourceEvidence.before.steps.file' "$manifest")" >/dev/null
    jq -e --arg run "$run_id" '(.data | type == "array" and length > 0) and all(.data[]; .runId == $run)' "$directory/$(jq -er '.sourceEvidence.before.artifacts.file' "$manifest")" >/dev/null
)
