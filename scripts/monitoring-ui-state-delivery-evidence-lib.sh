#!/usr/bin/env bash
# Delivery evidence verification for the Monitoring route-exact gate.

delivery_bytes() { wc -c < "$1" | tr -d ' '; }

delivery_descriptor() {
    local directory=$1 manifest=$2 key=$3 file expected bytes actual
    file=$(jq -er --arg key "$key" '.deliveryHealth[$key].file' "$manifest") || return 1
    expected=$(jq -er --arg key "$key" '.deliveryHealth[$key].sha256' "$manifest") || return 1
    bytes=$(jq -er --arg key "$key" '.deliveryHealth[$key].bytes' "$manifest") || return 1
    test -f "$directory/$file" && test ! -L "$directory/$file" || return 1
    actual=$(sha256sum "$directory/$file" | awk '{print $1}') || return 1
    test "$actual" = "$expected" || return 1
    test "$(delivery_bytes "$directory/$file")" = "$bytes"
}

delivery_flow() {
    local directory=$1 manifest=$2 actor=$3 flow run file
    flow=$(jq -er --arg actor "$actor" '.deliveryHealth[$actor + "Flow"].file' "$manifest") || return 1
    run=$(jq -er --arg actor "$actor" '.deliveryHealth[$actor + "Run"]' "$manifest") || return 1
    file=$(jq -er --arg actor "$actor" '.deliveryHealth[$actor + "RunReceipt"].file' "$manifest") || return 1
    delivery_descriptor "$directory" "$manifest" "${actor}Flow" || return 1
    delivery_descriptor "$directory" "$manifest" "${actor}RunReceipt" || return 1
    jq -e --arg actor "$actor" --arg run "$run" \
      --arg incident_selector ".ant-table-tbody > tr.ant-table-row .monitoring-incident-primary-column" '
      .data as $flow | ($flow | keys | sort == ["createdAt","id","name","steps","systemId","updatedAt"])
      and ($flow.id | type == "string" and length > 0)
      and ($flow.name == ("Monitoring delivery: monitor-delivery-" + $actor))
      and ($flow.steps | type == "array" and length == (if $actor == "viewer" then 26 else 18 end))
      and (($flow.steps[0] | keys | sort) == ["action","locale","theme"])
      and $flow.steps[0].action == "setUiPreferences"
      and (($flow.steps[1] | keys | sort) == ["action","height","width"])
      and $flow.steps[1].action == "setViewport"
      and ($flow.steps[2] == {action:"goto",url:"/login"})
      and ($flow.steps[3] == {action:"waitFor",selector:"#login_username"})
      and (($flow.steps[4] | keys | sort) == ["action","selector","value"])
      and (($flow.steps[5] | keys | sort) == ["action","selector","value"])
      and ($flow.steps[4].action == "fill" and $flow.steps[4].selector == "#login_username")
      and ($flow.steps[5].action == "fill" and $flow.steps[5].selector == "#login_password")
      and ($flow.steps[6] == {action:"click",selector:"button[type=submit]"})
      and ($flow.steps[7] == {action:"waitFor",selector:".shell-content"})
      and ($flow.steps[8] == {action:"goto",url:"/monitoring/incidents"})
      and ($flow.steps[9] == {action:"waitFor",selector:"[data-testid=notification-delivery-card]"})
      and (($flow.steps[10:16] | map(.action))
           == ["assertText","assertText","assertText","assertText","assertText","assertText"])
      and (($flow.steps[10:16] | map(.selector))
           == ["[data-testid=notification-delivery-card]","[data-testid=notification-delivery-card]",
               "[data-testid=notification-delivery-card]","[data-testid=notification-delivery-card]",
               "[data-testid=notification-delivery-card]","[data-testid=notification-delivery-card]"])
      and (if $actor == "viewer" then
          ($flow.steps[16] == {action:"waitFor",selector:".ant-table-row"})
          and ($flow.steps[17] == {action:"assertText",
               selector:$incident_selector,text:"Fixture incident"})
          and ($flow.steps[18] == {action:"assertElementLayout",
               selector:".ant-table-thead th:not(.ant-table-cell-scrollbar)",elementCount:null,
               visibleCount:3,maxHeight:null,withinViewportRight:false,withinViewport:false})
          and ($flow.steps[19] == {action:"assertElementLayout",
               selector:".ant-table-thead .monitoring-incident-detail-column",elementCount:null,
               visibleCount:0,maxHeight:null,withinViewportRight:false,withinViewport:false})
          and ($flow.steps[20] == {action:"assertElementLayout",
               selector:".ant-table-thead .monitoring-incident-primary-column",elementCount:null,
               visibleCount:1,maxHeight:64,withinViewportRight:true,withinViewport:false})
          and ($flow.steps[21] == {action:"assertElementLayout",
               selector:".ant-table-tbody > tr.ant-table-row",elementCount:20,visibleCount:20,maxHeight:72,
               withinViewportRight:true,withinViewport:false})
          and ($flow.steps[22] == {action:"assertElementLayout",
               selector:".ant-table-body > table",elementCount:null,visibleCount:1,maxHeight:null,
               withinViewportRight:true,withinViewport:false})
          and ($flow.steps[23] == {action:"assertElementLayout",
               selector:".data-table-pagination .ant-pagination",elementCount:null,visibleCount:1,
               maxHeight:null,withinViewportRight:true,withinViewport:false})
          and ($flow.steps[24] == {action:"assertNoHorizontalOverflow"})
          and ($flow.steps[25] == {action:"screenshotViewport",name:"monitor-delivery-viewer"})
        else
          ($flow.steps[16] == {action:"assertNoHorizontalOverflow"})
          and ($flow.steps[17] == {action:"screenshotViewport",name:"monitor-delivery-owner"})
        end)
      and (if $actor == "owner" then
          $flow.steps[0].theme == "dark" and $flow.steps[0].locale == "en-US"
          and $flow.steps[1].width == 1440 and $flow.steps[1].height == 900
          and $flow.steps[4].value == "owner" and $flow.steps[5].value == "rustzen@123"
          and ($flow.steps[10:16] | map(.text))
              == ["15 irreversible notification delivery gaps","Pending 2 (1024 B)","Quarantine 3 (2048 B)",
                  "First gap 09/10/2026, 01:02:03 AM","Last gap 09/10/2026, 02:03:04 AM",
                  "Last success 09/10/2026, 03:04:05 AM"]
        else
          $flow.steps[0].theme == "light" and $flow.steps[0].locale == "zh-CN"
          and $flow.steps[1].width == 390 and $flow.steps[1].height == 844
          and $flow.steps[4].value == "monitor_incident_viewer"
          and $flow.steps[5].value == "monitor-incident-viewer-password"
          and ($flow.steps[10:16] | map(.text))
              == ["通知投递存在 15 个不可恢复缺口","待投递 2（1024 B）","隔离 3（2048 B）",
                  "首个缺口 2026/09/10 01:02:03","最后缺口 2026/09/10 02:03:04",
                  "最后成功 2026/09/10 03:04:05"]
        end)
    ' "$directory/$flow" >/dev/null || return 1
    jq -e --arg flow "$(jq -er '.data.id' "$directory/$flow")" --arg run "$run" '
      .data as $receipt
      | ($receipt | keys | sort
         == ["createdAt","error","finishedAt","flowId","id","startedAt","status"])
      and $receipt.id == $run and $receipt.flowId == $flow
    ' "$directory/$file" >/dev/null
}

verify_delivery_health() (
    set -e
    local directory=$1 manifest=$2 key file run viewer_run
    jq -e '
      .deliveryHealth as $h
      | ($h.ownerRun != $h.viewerRun)
      and ($h.ownerApi.file == "monitor-delivery-owner.json")
      and ($h.viewerApi.file == "monitor-delivery-viewer.json")
      and ($h.database.file == "monitor-delivery-db.json")
      and ($h.ownerSteps.file == "delivery-monitor-delivery-owner-steps.json")
      and ($h.viewerSteps.file == "delivery-monitor-delivery-viewer-steps.json")
      and ($h.ownerFlow.file == "delivery-monitor-delivery-owner-flow.json")
      and ($h.viewerFlow.file == "delivery-monitor-delivery-viewer-flow.json")
      and ($h.ownerRunReceipt.file == "delivery-monitor-delivery-owner-run.json")
      and ($h.viewerRunReceipt.file == "delivery-monitor-delivery-viewer-run.json")
      and ([$h.ownerApi,$h.viewerApi,$h.database,$h.ownerSteps,$h.viewerSteps,$h.ownerFlow,
            $h.viewerFlow,$h.ownerRunReceipt,$h.viewerRunReceipt | .file] | unique | length == 9)
    ' "$manifest" >/dev/null || return 1
    for key in ownerApi viewerApi database ownerSteps viewerSteps ownerFlow viewerFlow \
        ownerRunReceipt viewerRunReceipt; do
        delivery_descriptor "$directory" "$manifest" "$key" || return 1
    done
    for key in ownerApi viewerApi; do
        file=$(jq -er --arg key "$key" '.deliveryHealth[$key].file' "$manifest") || return 1
        jq -e '.data == {pendingCount:2,pendingBytes:1024,quarantineCount:3,quarantineBytes:2048,
          omittedCount:1,expiredCount:2,unconfirmedCount:3,quarantinedCount:4,
          quarantineEvictedCount:5,firstGapAt:"2026-09-10T01:02:03Z",
          lastGapAt:"2026-09-10T02:03:04Z",lastSuccessAt:"2026-09-10T03:04:05Z"}' \
          "$directory/$file" >/dev/null || return 1
        cmp <(jq -Sc '.data' "$directory/$file") \
            "$directory/$(jq -er '.deliveryHealth.database.file' "$manifest")" || return 1
    done
    viewer_run=$(jq -er '.deliveryHealth.viewerRun' "$manifest") || return 1
    for key in ownerSteps viewerSteps; do
        file=$(jq -er --arg key "$key" '.deliveryHealth[$key].file' "$manifest") || return 1
        run=$(jq -er --arg key "$key" '.deliveryHealth[$key | sub("Steps"; "Run")]' "$manifest") || return 1
        jq -e --arg run "$run" --arg viewer "$viewer_run" '
          .data | [.[].action] == ["setUiPreferences","setViewport","goto","waitFor","fill","fill",
            "click","waitFor","goto","waitFor","assertText","assertText","assertText","assertText",
            "assertText","assertText"]
          + (if $run == $viewer then ["waitFor","assertText","assertElementLayout","assertElementLayout",
              "assertElementLayout","assertElementLayout","assertElementLayout","assertElementLayout"] else [] end)
          + ["assertNoHorizontalOverflow","screenshotViewport"]
          and all(.[]; .runId == $run and .status == "succeeded")
          and (to_entries | all(.key == .value.stepIndex))
          and all(.[]; keys | sort == ["action","createdAt","durationMs","id","message","runId","status","stepIndex"])
        ' "$directory/$file" >/dev/null || return 1
    done
    delivery_flow "$directory" "$manifest" owner || return 1
    delivery_flow "$directory" "$manifest" viewer
)

verify_delivery_artifacts() (
    set -e
    local directory=$1 manifest=$2 item case file dimensions hash bytes actual_hash actual_bytes
    for item in \
        'monitor-delivery-owner|monitor-delivery-owner-desktop-dark-en.png|1440 x 900' \
        'monitor-delivery-viewer|monitor-delivery-viewer-mobile-light-zh.png|390 x 844'; do
        IFS='|' read -r case file dimensions <<<"$item"
        hash=$(jq -er --arg case "$case" '.artifacts[] | select(.case == $case) | .sha256' "$manifest") || return 1
        bytes=$(jq -er --arg case "$case" '.artifacts[] | select(.case == $case) | .bytes' "$manifest") || return 1
        jq -e --arg case "$case" --arg file "$file" --arg dimensions "$dimensions" '
          [.artifacts[] | select(.case == $case and .file == $file and .dimensions == $dimensions
            and (keys | sort == ["bytes","case","dimensions","file","sha256"]))] | length == 1
        ' "$manifest" >/dev/null || return 1
        actual_hash=$(sha256sum "$directory/$file" | awk '{print $1}') || return 1
        actual_bytes=$(delivery_bytes "$directory/$file") || return 1
        test "$actual_hash" = "$hash" && test "$actual_bytes" = "$bytes" || return 1
    done
)
