def exact_error($value; $status; $code):
  $value.status == $status
  and $value.contentType == "application/json"
  and $value.body.code == $code
  and $value.body.data == null;

def exact_stream($value):
  $value.status == 200
  and $value.contentType == "text/event-stream"
  and $value.cacheControl == "no-cache, no-store, must-revalidate"
  and $value.accelBuffering == "no";

. as $manifest
| .schemaVersion == 1
  and .status == "passed"
  and .gitHead == $head
  and .sourceTreeState == $state
  and .sourceTreeSha256 == $sourceSha
  and .platform == {architecture:$architecture,name:$platform}
  and .binarySha256 == $binary
  and .buildProvenanceSha256 == $provenance
  and .verifier == {imageId:$verifierImage,key:$verifierKey,provenanceSha256:$verifierSha}
  and .cases == {
    initialRevision:true,durableAdmission:true,durableRead:true,retention:true,
    lastEventIdNoReplay:true,unauthorized:true,forbidden:true,
    queryRejectedAndRedacted:true,perUserQuota:true,authorityChangeEof:true,
    expiryEof:true,authorityOutage:true
  }
  and (.receipts|length == 22)
  and ([.receipts[].file]|length == (unique|length))
  and ([.receipts[].file]|sort == ($allowlist|sort))
  and all(.receipts[];
    (.file|type == "string" and test("^[A-Za-z0-9][A-Za-z0-9._-]*$"))
    and (.sha256|type == "string" and test("^[0-9a-f]{64}$"))
    and (.bytes|type == "number" and floor == . and . > 0))
  and exact_stream($initial[0])
  and ($initial[0].body|contains("reason\":\"connected") and contains("id: 0"))
  and ($event[0].producer == "monitor" and $event[0].topic == "monitor.incident.opened")
  and ($ingest[0].status == 201 and $ingest[0].body.code == "stored")
  and ($live[0].body|contains("id: 0") and contains("id: 1") and contains("id: 2"))
  and exact_stream($live[0])
  and ($inbox[0].status == 200 and ($inbox[0].body.data.items|length) == 1)
  and ($inbox[0].body.data.items[0].topic == "monitor.incident.opened")
  and ($read[0].status == 200 and $read[0].body.data.revision == 2)
  and ($retention[0].status == 200 and $retention[0].body.data == {count:0,revision:3})
  and ($reconnect[0].body|contains("reason\":\"connected") and contains("id: 3"))
  and exact_stream($reconnect[0])
  and (($reconnect[0].body|test("(^|\\n)id: (1|2)(\\n|$)"))|not)
  and exact_error($unauthorized[0];401;401)
  and exact_error($forbidden[0];403;403)
  and exact_error($query[0];400;400)
  and ($redaction[0].secretAbsent and $redaction[0].queryAbsent)
  and $quota[0] == {opened:4,fifth:{status:429,code:42901,retryAfter:60},released:true,reopened:200}
  and $authorityChange[0] == {connected:true,closed:true}
  and $expiry[0] == {connected:true,closed:true}
  and exact_error($outage[0];503;50302)
  and $selected[0].receipts == 0
  and $selected[0].messages == 0
  and $selected[0].recipients == 0
  and $selected[0].revision == 3
  and ($selected[0].operationLogTablePresent|not)
  and ($phaseStatus|contains("build\tpassed") and contains("verifier-helper\tpassed") and contains("runtime\tpassed"))
  and ($buildLog|length > 0) and ($helperLog|length > 0) and ($runtimeLog|length > 0)
