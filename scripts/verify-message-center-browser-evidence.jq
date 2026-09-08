def safe_file:
  type == "string" and length > 0 and (startswith("/")|not)
  and (split("/")|all(. != "" and . != "." and . != ".."));
def cases: ["emptyDesktop","realtimeInvalidation","loading","populatedDetail","unreadFilterPaging","singleRead","readAll","forbiddenClears","unauthorized","incidentDeepLink","mobile"];

. as $manifest
| ($manifest.receipts|map(.file)) as $files
| ($browser[0]|map(.case)) as $browserCases
| ($proxy[0]|map(select(.path == "/api/notifications/stream"))) as $streams
| ($streams|map(select(.case == "preflight"))) as $preflightStreams
| ($streams|map(select(.case != "preflight"))) as $browserStreams
| ($streams|map(select(.case == "realtimeInvalidation"))) as $realtimeStreams
| ($streams|map(select(.case == "unauthorized"))) as $unauthorizedStreams
| ($proxy[0]|map(select(.case == "realtimeInvalidation" and (.path == "/api/notifications" or .path == "/api/notifications/unread-count")))) as $realtimeReads
| ($proxy[0]|map(select(.case == "forbiddenClears" and .path == "/api/notifications"))) as $forbiddenLists
| .schemaVersion == 1 and .status == "passed"
and .gitHead == $head and .sourceTreeState == $state and .sourceTreeSha256 == $sourceSha
and .platform == {architecture:$architecture,name:$platform}
and .binaryHashes == $binaries
and .web == {selectedInventorySha256:$web,pureInventorySha256:$pureWeb}
and .buildProvenanceSha256 == $provenance
and .verifier == {imageId:$verifierImage,key:$verifierKey,provenanceSha256:$verifierSha}
and .cases == {browser:11,sseBearer:true,sseUrlSecret:false,browserSseLifecycle:true,singleLifecycle:true,durableReconcile:true,forbiddenClears:true,unauthorizedLogin:true,incidentDeepLink:true,pureAbsent:true}
and (.screenshots|sort_by(.file)) == ([{file:"message-center-desktop-en.png",dimensions:"1440 x 900"},{file:"message-center-mobile-en.png",dimensions:"390 x 844"}]|sort_by(.file))
and ($files|sort) == ($allowlist|sort) and ($files|length) == ($files|unique|length)
and (.receipts|all(.file|safe_file) and all(.sha256|test("^[0-9a-f]{64}$")) and all(.bytes > 0))
and ($browserCases|sort) == (cases|sort) and ($browserCases|length) == 11
and ($browser[0]|all(.steps|type == "array" and length > 0 and all(.status == "succeeded" and .message == null)))
and ($browser[0]|all(. as $result|[.steps[].action] == [$expected[0][$result.case][].action]))
and ($streams|all(.query == false and .tokenInUrl == false and .bearer == true and .accept == "text/event-stream" and .atNs > 0))
and ($preflight[0] == {
  direct:{label:"direct",status:200,contentType:"text/event-stream",path:"/api/notifications/stream",query:false,bearer:true,frames:["reconcile.required","inbox.changed"]},
  proxy:{label:"proxy",status:200,contentType:"text/event-stream",path:"/api/notifications/stream",query:false,bearer:true,frames:["reconcile.required","inbox.changed"]}
})
and ($preflightStreams|length == 1 and .[0].status == 200)
and ($browserStreams|length == 11 and (map(.case)|sort) == (cases|sort) and (group_by(.case)|all(length == 1)))
and ($realtimeStreams|length == 1 and .[0].status == 200)
and ($unauthorizedStreams|length == 1 and .[0].status == 401)
and ([$browserStreams[]|select(.case != "realtimeInvalidation" and .case != "unauthorized")]|all(.status == 204))
and ($realtime[0].case == "realtimeInvalidation" and $realtime[0].notificationCount == 1
  and $realtime[0].streamReadyAtNs <= $realtime[0].admissionStartedAtNs
  and $realtime[0].listReadyAtNs <= $realtime[0].admissionStartedAtNs)
and ([$realtimeReads[]|select(.atNs < $realtime[0].admissionStartedAtNs)|.path]|unique|length) == 2
and ([$realtimeReads[]|select(.atNs > $realtime[0].admissionStartedAtNs)|.path]|unique|length) == 2
and ($forbiddenLists|length >= 2 and .[0].status == 200 and any(.[1:][]; .status == 403))
and ($timing[0].case == "forbiddenClears" and $timing[0].listStatus == 200
  and $timing[0].eventId == "browser-page-22" and $timing[0].admissionStatus == 201
  and $timing[0].admissionCode == "stored" and $timing[0].listReadyAtNs <= $timing[0].admissionStartedAtNs)
and $selected[0].messages >= 22 and $selected[0].recipients == $selected[0].messages
and $selected[0].reads > 0 and $selected[0].incidents >= 1
and $pure[0] == {schemaObjects:[],notificationModules:[],preset:"monitor",routeStatus:404,ingressListener:false,notificationOwners:false}
and ($phaseStatus|split("\n")|map(select(length>0))) == ["source-tests\tpassed","web-build\tpassed","build\tpassed","verifier-helper\tpassed","runtime\tpassed"]
