#!/usr/bin/env bash
set -euo pipefail

# The processing capture needs a second browser worker while the observed run
# is paused. The shared runtime keeps its default of one worker for all other
# gates.
export RUSTZEN_REPORTS_MAX_CONCURRENCY=2
. /verify/inner-lib.sh
. /verify/delivery-lib.sh
. /verify/browser-lib.sh
test "$(date +%Z)" = UTC

admin=http://127.0.0.1:19801
owner_login=$(curl_json \
    -H 'content-type: application/json' \
    -d '{"username":"owner","password":"rustzen@123"}' \
    "$admin/api/auth/login")
owner_token=$(jq -er '.data.token' <<<"$owner_login")
owner_auth=(-H "authorization: Bearer $owner_token")
seed_reports_ui_delivery_status /opt/rz/data/reports/db/reports.db

system_id=$(create_system 'Reports UI state target')
active_steps=$(jq -nc '[{action:"goto",url:"/health"}]+[range(0;10)|{action:"pause",durationMs:30000}]')
active_flow=$(create_flow 'Reports processing source' "$active_steps")
active_run=$(create_run "$active_flow")
wait_for_status "$active_run" running

processing_steps=$(jq -nc --arg run "$active_run" '
    [
      {action:"setUiPreferences",theme:"dark",locale:"en-US"},
      {action:"setViewport",width:1440,height:900},
      {action:"goto",url:"/login"},
      {action:"waitFor",selector:"#login_username"},
      {action:"fill",selector:"#login_username",value:"owner"},
      {action:"fill",selector:"#login_password",value:"rustzen@123"},
      {action:"click",selector:"button[type=submit]"},
      {action:"waitFor",selector:".shell-content"},
      {action:"goto",url:"/reports/runs"},
      {action:"waitFor",selector:"[data-testid=run-view-\($run)]"},
      {action:"waitFor",selector:"[data-testid=notification-delivery-card]"},
      {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:"15 irreversible notification delivery gaps"},
      {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:"Pending 2 (1024 B)"},
      {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:"Quarantine 3 (2048 B)"},
      {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:"First gap 09/10/2026, 01:02:03 AM"},
      {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:"Last gap 09/10/2026, 02:03:04 AM"},
      {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:"Last success 09/10/2026, 03:04:05 AM"},
      {action:"assertNoHorizontalOverflow"},
      {action:"screenshotViewport",name:"reports-processing-desktop-dark-en"},
      {action:"click",selector:"[data-testid=run-view-\($run)]"},
      {action:"waitFor",selector:"[data-testid=run-audit]"},
      {action:"assertText",selector:".ant-modal",text:"Report run in progress"},
      {action:"assertText",selector:".ant-modal",text:"1. goto"},
      {action:"click",selector:"button.ant-modal-close"},
      {action:"pause",durationMs:300},
      {action:"waitFor",selector:"[data-testid=run-cancel-\($run)]"},
      {action:"click",selector:"[data-testid=run-cancel-\($run)]"},
      {action:"waitFor",selector:"[data-testid=run-cancel-\($run)][disabled]"}
    ]')
processing_browser_run=$(run_browser_case managerProcessing "$processing_steps")
wait_for_status "$active_run" cancelled
curl_json "${owner_auth[@]}" "$admin/api/reports/runs/$active_run" > /verify/evidence/processing-run.json
curl_json "${owner_auth[@]}" "$admin/api/reports/runs/$active_run/steps" > /verify/evidence/processing-run-steps.json
jq -e '.data.status == "cancelled"' /verify/evidence/processing-run.json >/dev/null
jq -e --arg run "$active_run" '.data | any(.runId == $run and .action == "pause" and .status == "cancelled")' \
    /verify/evidence/processing-run-steps.json >/dev/null

failure_flow=$(create_flow 'Reports runtime failure source' \
    '[{"action":"goto","url":"/health"},{"action":"screenshotViewport","name":"source-runtime-failure-witness"},{"action":"assertText","selector":"body","text":"runtime failure witness"}]')
failure_run=$(create_run "$failure_flow")
wait_for_status "$failure_run" failed
curl_json "${owner_auth[@]}" "$admin/api/reports/runs/$failure_run" \
    > /verify/evidence/source-run.before.json
curl_json "${owner_auth[@]}" "$admin/api/reports/runs/$failure_run/steps" \
    > /verify/evidence/source-steps.before.json
curl_json "${owner_auth[@]}" "$admin/api/reports/runs/$failure_run/artifacts" \
    > /verify/evidence/source-artifacts.before.json
jq -e '.data.status == "failed" and (.data.error | contains("assertText did not match"))' \
    /verify/evidence/source-run.before.json >/dev/null
jq -e '.data | any(.status == "failed" and (.message | contains("assertText did not match")))' \
    /verify/evidence/source-steps.before.json >/dev/null

failure_steps=$(jq -nc --arg run "$failure_run" '
    [
      {action:"setUiPreferences",theme:"dark",locale:"en-US"},
      {action:"setViewport",width:1440,height:900},
      {action:"goto",url:"/login"},
      {action:"waitFor",selector:"#login_username"},
      {action:"fill",selector:"#login_username",value:"owner"},
      {action:"fill",selector:"#login_password",value:"rustzen@123"},
      {action:"click",selector:"button[type=submit]"},
      {action:"waitFor",selector:".shell-content"},
      {action:"goto",url:"/reports/runs"},
      {action:"waitFor",selector:"[data-testid=run-retry-list-\($run)]"},
      {action:"assertText",selector:"tr:has([data-testid=run-retry-list-\($run)])",text:"assertText did not match"},
      {action:"click",selector:"[data-testid=run-view-\($run)]"},
      {action:"waitFor",selector:"[data-testid=run-audit][data-run-id=\"\($run)\"]"},
      {action:"assertText",selector:".ant-modal",text:"Failed"},
      {action:"assertText",selector:".ant-modal",text:"assertText did not match"},
      {action:"assertNoHorizontalOverflow"},
      {action:"screenshotViewport",name:"reports-runtime-failure-desktop-dark-en"},
      {action:"click",selector:"[data-testid=run-retry-audit-\($run)]"},
      {action:"waitFor",selector:"[data-testid=run-audit]:not([data-run-id=\"\($run)\"])"}
    ]')
failure_browser_run=$(run_browser_case managerRuntimeFailure "$failure_steps")
retry_child=$(curl_json "${owner_auth[@]}" -X POST "$admin/api/reports/runs/$failure_run/retry")
retry_child_id=$(jq -er '.data.id' <<<"$retry_child")
curl_json "${owner_auth[@]}" "$admin/api/reports/runs/$failure_run" \
    > /verify/evidence/source-run.after.json
curl_json "${owner_auth[@]}" "$admin/api/reports/runs/$failure_run/steps" \
    > /verify/evidence/source-steps.after.json
curl_json "${owner_auth[@]}" "$admin/api/reports/runs/$failure_run/artifacts" \
    > /verify/evidence/source-artifacts.after.json
cmp -s /verify/evidence/source-run.before.json /verify/evidence/source-run.after.json
cmp -s /verify/evidence/source-steps.before.json /verify/evidence/source-steps.after.json
cmp -s /verify/evidence/source-artifacts.before.json /verify/evidence/source-artifacts.after.json

create_schedule() {
    curl_json "${owner_auth[@]}" -H 'content-type: application/json' \
        -d "$(jq -nc --arg flow "$failure_flow" --arg description "$1" '{flowId:$flow,cadence:"daily",dueTime:"23:59",input:{},description:$description,enabled:false}')" \
        "$admin/api/reports/schedules" | jq -er '.data.id'
}
enqueued_schedule=$(create_schedule 'partial enqueued fixture')
skipped_schedule=$(create_schedule 'partial skipped fixture')
reports_db=/opt/rz/data/reports/db/reports.db
seed_partial_schedule_fixture "$reports_db" "$enqueued_schedule" "$skipped_schedule" "$failure_run"
partial_steps=$(jq -nc --arg enqueued "$enqueued_schedule" --arg skipped "$skipped_schedule" '
    [
      {action:"setUiPreferences",theme:"dark",locale:"en-US"},
      {action:"setViewport",width:1440,height:900},
      {action:"goto",url:"/login"},
      {action:"waitFor",selector:"#login_username"},
      {action:"fill",selector:"#login_username",value:"owner"},
      {action:"fill",selector:"#login_password",value:"rustzen@123"},
      {action:"click",selector:"button[type=submit]"},
      {action:"waitFor",selector:".shell-content"},
      {action:"goto",url:"/reports/templates"},
      {action:"waitFor",selector:"[data-testid=schedule-panel]"},
      {action:"assertText",selector:"[data-testid=schedule-panel]",text:"Schedules contain mixed occurrence outcomes"},
      {action:"assertText",selector:"[data-testid=schedule-occurrence-\($skipped)]",text:"Skipped"},
      {action:"assertText",selector:"[data-testid=schedule-occurrence-skipped-\($skipped)]",text:"missed"},
      {action:"assertAbsent",selector:"[data-testid=schedule-occurrence-run-\($skipped)]"},
      {action:"waitFor",selector:"[data-testid=schedule-occurrence-run-\($enqueued)]"},
      {action:"assertNoHorizontalOverflow"},
      {action:"screenshotViewport",name:"reports-partial-desktop-dark-en"}
    ]')
partial_browser_run=$(run_browser_case managerPartial "$partial_steps")

schedule_menu=$(curl_json "${owner_auth[@]}" "$admin/api/system/menus/options?limit=500" \
    | jq -er '.data[] | select(.code == "reports:schedule:view") | .value')
run_menu=$(curl_json "${owner_auth[@]}" "$admin/api/system/menus/options?limit=500" \
    | jq -er '.data[] | select(.code == "reports:run:view") | .value')
curl_json "${owner_auth[@]}" -H 'content-type: application/json' \
    -d "$(jq -nc --argjson schedule "$schedule_menu" --argjson run "$run_menu" '{name:"Reports state viewer",code:"reports_state_viewer",status:1,menuIds:[$schedule,$run]}')" \
    "$admin/api/system/roles" >/dev/null
viewer_role=$(curl_json "${owner_auth[@]}" "$admin/api/system/roles/options?limit=500" \
    | jq -er '.data[] | select(.code == "reports_state_viewer") | .value')
curl_json "${owner_auth[@]}" -H 'content-type: application/json' \
    -d "$(jq -nc --argjson role "$viewer_role" '{username:"reports_state_viewer",email:"reports-state-viewer@example.test",password:"reports-state-viewer-password",realName:"Reports state viewer",status:1,roleIds:[$role]}')" \
    "$admin/api/system/users" >/dev/null
viewer_login=$(curl_json -H 'content-type: application/json' \
    -d '{"username":"reports_state_viewer","password":"reports-state-viewer-password"}' \
    "$admin/api/auth/login")
viewer_token=$(jq -er '.data.token' <<<"$viewer_login")
viewer_schedule_status=$(curl --silent --show-error --output /verify/evidence/viewer-schedule-mutation.json \
    --write-out '%{http_code}' --connect-timeout 3 --max-time 15 \
    -H "authorization: Bearer $viewer_token" -H 'content-type: application/json' -X PUT \
    -d '{}' "$admin/api/reports/schedules/$enqueued_schedule")
viewer_retry_status=$(curl --silent --show-error --output /verify/evidence/viewer-retry.json \
    --write-out '%{http_code}' --connect-timeout 3 --max-time 15 \
    -H "authorization: Bearer $viewer_token" -X POST "$admin/api/reports/runs/$failure_run/retry")
test "$viewer_schedule_status" = 403
test "$viewer_retry_status" = 403
for receipt in viewer-schedule-mutation.json viewer-retry.json; do jq -e '. == {code:403,message:"Permission denied",data:null}' "/verify/evidence/$receipt" >/dev/null; done
wait_reports_ui_outbox_settled /opt/rz/data/reports/db/reports.db
seed_reports_ui_delivery_status /opt/rz/data/reports/db/reports.db

viewer_steps=$(jq -nc --arg run "$failure_run" '
    [
      {action:"setUiPreferences",theme:"light",locale:"zh-CN"},
      {action:"setViewport",width:390,height:844},
      {action:"goto",url:"/login"},
      {action:"waitFor",selector:"#login_username"},
      {action:"fill",selector:"#login_username",value:"reports_state_viewer"},
      {action:"fill",selector:"#login_password",value:"reports-state-viewer-password"},
      {action:"click",selector:"button[type=submit]"},
      {action:"waitFor",selector:".shell-content"},
      {action:"goto",url:"/reports/templates"},
      {action:"waitFor",selector:"[data-testid=schedule-panel]"},
      {action:"assertAbsent",selector:"[data-testid=schedule-create]"},
      {action:"assertAbsent",selector:"[data-testid=schedule-edit]"},
      {action:"assertAbsent",selector:"[data-testid=schedule-toggle]"},
      {action:"assertAbsent",selector:"[data-testid=schedule-delete]"},
      {action:"goto",url:"/reports/runs"},
      {action:"waitFor",selector:"[data-testid=run-view-\($run)]"},
      {action:"waitFor",selector:"[data-testid=notification-delivery-card]"},
      {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:"通知投递存在 15 个不可恢复缺口"},
      {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:"待投递 2（1024 B）"},
      {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:"隔离 3（2048 B）"},
      {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:"首个缺口 2026/09/10 01:02:03"},
      {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:"最后缺口 2026/09/10 02:03:04"},
      {action:"assertText",selector:"[data-testid=notification-delivery-card]",text:"最后成功 2026/09/10 03:04:05"},
      {action:"assertNoHorizontalOverflow"},
      {action:"screenshotViewport",name:"reports-view-only-mobile-light-zh"},
      {action:"assertAbsent",selector:"[data-testid=run-create]"},
      {action:"assertAbsent",selector:"[data-testid^=run-cancel-]"},
      {action:"assertAbsent",selector:"[data-testid=run-retry-list-\($run)]"},
      {action:"click",selector:"[data-testid=run-view-\($run)]"},
      {action:"waitFor",selector:"[data-testid=run-audit]"},
      {action:"assertAbsent",selector:"[data-testid=run-retry-audit-\($run)]"}
    ]')
viewer_browser_run=$(run_browser_case viewOnlyMobile "$viewer_steps")

processing_artifact=$(download_screenshot "$processing_browser_run" reports-processing-desktop-dark-en reports-processing-desktop-dark-en.png managerProcessing 1440 900)
failure_artifact=$(download_screenshot "$failure_browser_run" reports-runtime-failure-desktop-dark-en reports-runtime-failure-desktop-dark-en.png managerRuntimeFailure 1440 900)
partial_artifact=$(download_screenshot "$partial_browser_run" reports-partial-desktop-dark-en reports-partial-desktop-dark-en.png managerPartial 1440 900)
viewer_artifact=$(download_screenshot "$viewer_browser_run" reports-view-only-mobile-light-zh reports-view-only-mobile-light-zh.png viewOnlyMobile 390 844)
processing_receipt=$(save_run_steps "$processing_browser_run" manager-processing.json)
failure_receipt=$(save_run_steps "$failure_browser_run" manager-runtime-failure.json)
partial_receipt=$(save_run_steps "$partial_browser_run" manager-partial.json)
viewer_receipt=$(save_run_steps "$viewer_browser_run" view-only-mobile.json)
wait_reports_ui_outbox_settled /opt/rz/data/reports/db/reports.db
seed_reports_ui_delivery_status /opt/rz/data/reports/db/reports.db
reports_ui_delivery_db_json /opt/rz/data/reports/db/reports.db > /verify/evidence/reports-delivery-db.json
curl_json "${owner_auth[@]}" "$admin/api/reports/notification-delivery" > /verify/evidence/reports-delivery-owner.json
curl_json -H "authorization: Bearer $viewer_token" "$admin/api/reports/notification-delivery" > /verify/evidence/reports-delivery-viewer.json
jq -e "$(reports_ui_delivery_expected_jq)" /verify/evidence/reports-delivery-owner.json >/dev/null
jq -e "$(reports_ui_delivery_expected_jq)" /verify/evidence/reports-delivery-viewer.json >/dev/null
cmp <(jq -Sc '.data' /verify/evidence/reports-delivery-owner.json) /verify/evidence/reports-delivery-db.json
cmp <(jq -Sc '.data' /verify/evidence/reports-delivery-viewer.json) /verify/evidence/reports-delivery-db.json
delivery_owner_receipt=$(evidence_file_descriptor reports-delivery-owner.json)
delivery_viewer_receipt=$(evidence_file_descriptor reports-delivery-viewer.json)
viewer_schedule_receipt=$(jq -nc --arg file viewer-schedule-mutation.json --arg sha "$(sha256sum /verify/evidence/viewer-schedule-mutation.json | awk '{print $1}')" '{status:403,file:$file,sha256:$sha}')
viewer_retry_receipt=$(jq -nc --arg file viewer-retry.json --arg sha "$(sha256sum /verify/evidence/viewer-retry.json | awk '{print $1}')" '{status:403,file:$file,sha256:$sha}')
processing_step_receipt=$(evidence_file_descriptor processing-run-steps.json)
source_run_before=$(evidence_file_descriptor source-run.before.json)
source_steps_before=$(evidence_file_descriptor source-steps.before.json)
source_artifacts_before=$(evidence_file_descriptor source-artifacts.before.json)
source_run_after=$(evidence_file_descriptor source-run.after.json)
source_steps_after=$(evidence_file_descriptor source-steps.after.json)
source_artifacts_after=$(evidence_file_descriptor source-artifacts.after.json)
processing_pause_step=$(jq -c --arg run "$active_run" '.data | map(select(.runId == $run and .action == "pause" and .status == "cancelled")) | .[0]' /verify/evidence/processing-run-steps.json)
failed_step=$(jq -c --arg run "$failure_run" '.data | map(select(.runId == $run and .action == "assertText" and .status == "failed")) | .[0]' /verify/evidence/source-steps.after.json)
processing_status=$(jq -er '.data.status' /verify/evidence/processing-run.json)
failure_error=$(jq -er '.data.error' /verify/evidence/source-run.after.json)

jq -n \
    --arg head "$RUSTZEN_VERIFY_HEAD" \
    --arg state "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" \
    --arg sha "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" \
    --arg platform "$RUSTZEN_VERIFY_PLATFORM" \
    --arg processing "$processing_browser_run" \
    --arg failure "$failure_browser_run" \
    --arg partial "$partial_browser_run" \
    --arg viewer "$viewer_browser_run" \
    --arg child "$retry_child_id" \
    --arg enqueued "$failure_run" \
    --arg active "$active_run" \
    --arg processing_status "$processing_status" \
    --arg failure_error "$failure_error" \
    --argjson viewer_schedule_receipt "$viewer_schedule_receipt" \
    --argjson viewer_retry_receipt "$viewer_retry_receipt" \
    --argjson processing_receipt "$processing_receipt" \
    --argjson failure_receipt "$failure_receipt" \
    --argjson partial_receipt "$partial_receipt" \
    --argjson viewer_receipt "$viewer_receipt" \
    --argjson delivery_owner_receipt "$delivery_owner_receipt" \
    --argjson delivery_viewer_receipt "$delivery_viewer_receipt" \
    --argjson processing_step_receipt "$processing_step_receipt" \
    --argjson processing_pause_step "$processing_pause_step" \
    --argjson failed_step "$failed_step" \
    --argjson source_run_before "$source_run_before" \
    --argjson source_steps_before "$source_steps_before" \
    --argjson source_artifacts_before "$source_artifacts_before" \
    --argjson source_run_after "$source_run_after" \
    --argjson source_steps_after "$source_steps_after" \
    --argjson source_artifacts_after "$source_artifacts_after" \
    --argjson processing_artifact "$processing_artifact" \
    --argjson failure_artifact "$failure_artifact" \
    --argjson partial_artifact "$partial_artifact" \
    --argjson viewer_artifact "$viewer_artifact" '
    {
      schemaVersion:3,
      status:"passed",
      gitHead:$head,
      sourceTreeState:$state,
      sourceTreeSha256:$sha,
      platform:$platform,
      chromiumVersion:env.RUSTZEN_VERIFY_CHROMIUM_VERSION,
      runs:{managerProcessing:$processing,managerRuntimeFailure:$failure,managerPartial:$partial,viewOnlyMobile:$viewer},
      runSteps:{managerProcessing:$processing_receipt,managerRuntimeFailure:$failure_receipt,managerPartial:$partial_receipt,viewOnlyMobile:$viewer_receipt},
      processing:{runId:$active,status:$processing_status,pauseDurationMs:30000,pauseStep:$processing_pause_step,stepReceipt:$processing_step_receipt},
      sourceEvidence:{runId:$enqueued,status:"failed",error:$failure_error,failedStep:$failed_step,before:{run:$source_run_before,steps:$source_steps_before,artifacts:$source_artifacts_before},after:{run:$source_run_after,steps:$source_steps_after,artifacts:$source_artifacts_after}},
      retry:{childId:$child,sourcePreserved:true},
      partialFixture:{enqueuedRunId:$enqueued,skippedRunLinked:false},
      viewOnly:{scheduleMutationReceipt:$viewer_schedule_receipt,retryReceipt:$viewer_retry_receipt},
      deliveryHealth:{gapTotal:15,ownerReceipt:$delivery_owner_receipt,viewerReceipt:$delivery_viewer_receipt},
      artifacts:[$processing_artifact,$failure_artifact,$partial_artifact,$viewer_artifact]
    }
    ' >/verify/evidence/manifest.json
