def no_notification:
  tostring | test("notification"; "i") | not;

. as $manifest
| ($identity | rtrimstr("\n") | split("\t")) as $ids
| ($open[0]) as $opened
| ($inbox[0]) as $final
| ($selected[0]) as $selectedState
| .schemaVersion == 1
  and .status == "passed"
  and .gitHead == $head
  and .sourceTreeState == $state
  and .sourceTreeSha256 == $sourceSha
  and .platform == {architecture: $architecture, name: $platform}
  and .binaryHashes == $binaries
  and .buildProvenanceSha256 == $provenanceSha
  and .verifier == {imageId: $verifierImage, key: $verifierKey, provenanceSha256: $verifierSha}
  and .delivery == {
    openEventId: $opened.eventId,
    subjectId: $opened.subject.id,
    opened: "stored",
    duplicate: "duplicate",
    resolved: "stored",
    receiptCount: 2,
    messageCount: 2,
    recipientCount: 2
  }
  and .authentication == {unsigned: 400, badSignature: 401, publicInternal: 404}
  and .pureMonitor == {
    adminNotificationOwner: false,
    monitorNotificationOwner: false,
    notificationConfig: false,
    notificationRoutes: false,
    relayTask: false,
    notificationSchemaObjects: 0,
    notificationListeners: 0
  }
  and (.receipts | type == "array" and length == 18)
  and ([.receipts[].file] | length == (unique | length))
  and ([.receipts[].file] | sort == ($allowlist | sort))
  and all(.receipts[];
    (.file | type == "string" and test("^[A-Za-z0-9][A-Za-z0-9._-]*$"))
    and (.sha256 | type == "string" and test("^[0-9a-f]{64}$"))
    and (.bytes | type == "number" and floor == . and . > 0)
  )
  and $opened.schemaVersion == 1
  and $opened.producer == "monitor"
  and $opened.topic == "monitor.incident.opened"
  and $opened.subject.kind == "monitor-incident"
  and $opened.subject.revision == 1
  and $opened.audience == {policy: "monitor-incident-readers"}
  and $ids == [$opened.eventId, $opened.subject.id]
  and ($selectedListener | test("127[.]0[.]0[.]1:19811"))
  and $inboxOpen[0].code == 0
  and ($inboxOpen[0].data.items | length == 1)
  and $inboxOpen[0].data.items[0].topic == "monitor.incident.opened"
  and $inboxOpen[0].data.items[0].subjectId == $opened.subject.id
  and $inboxOpen[0].data.items[0].subjectRevision == 1
  and $final.code == 0
  and ($final.data.items | length == 2)
  and ([$final.data.items[].topic] == ["monitor.incident.resolved", "monitor.incident.opened"])
  and ([$final.data.items[].subjectId] == [$opened.subject.id, $opened.subject.id])
  and ([$final.data.items[].subjectRevision] == [2, 1])
  and ($selectedState.receipts | length == 2 and all(.[1] == "stored"))
  and $selectedState.receipts[0][0] == $opened.eventId
  and $selectedState.messages == [
    ["monitor.incident.opened", $opened.subject.id, 1],
    ["monitor.incident.resolved", $opened.subject.id, 2]
  ]
  and $selectedState.recipients == 2
  and $selectedState.outbox == 0
  and $duplicate[0] == {status: 200, contentType: "application/json", body: {code: "duplicate"}}
  and $bad[0] == {status: 401, contentType: "application/json", body: {code: "bad-producer"}}
  and $unsigned[0] == {code: "invalid-protocol"}
  and $public[0] == {status: 404, contentType: "application/json", body: {code: 10001, message: "API route not found.", data: null}}
  and $plainAbsence[0] == {admin: [], monitor: []}
  and $plainRoute[0] == {code: 404, data: null, message: "Not found"}
  and ($plainListeners | test("19822") | not)
  and $plainAdminApi[0].version == 1
  and $plainAdminConfig[0].owner == "access"
  and $plainAdminConfig[0].consumer == "rz-admin"
  and $plainMonitorApi[0].module == "monitor"
  and $plainMonitorConfig[0].owner == "monitor"
  and $plainMonitorConfig[0].consumer == "rz-monitor"
  and all([$plainAdminApi[0], $plainAdminConfig[0], $plainMonitorApi[0], $plainMonitorConfig[0]][]; no_notification)
