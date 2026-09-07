def no_notification: tostring | test("notification"; "i") | not;
def run_topic($runs):
  [
    ["reports.run.completed", $runs.success.id],
    ["reports.run.failed", $runs.failed.id],
    ["reports.run.cancelled", $runs.queuedCancel.id],
    ["reports.run.cancelled", $runs.cooperativeCancel.id],
    ["reports.run.failed", $runs.recoveryFailed.id],
    ["reports.run.cancelled", $runs.recoveryCancelled.id],
    ["reports.run.completed", $runs.dropResponse.id]
  ];
def valid_reports_identity($identity;$prefix;$uid;$gid):
  $identity.process == {uid:$uid,gid:$gid}
  and $identity.directories == [
    {kind:"runtime",path:($prefix+"/reports"),uid:$uid,gid:$gid,mode:"0750"},
    {kind:"db",path:($prefix+"/reports/db"),uid:$uid,gid:$gid,mode:"0750"},
    {kind:"log",path:($prefix+"/reports/logs/reports"),uid:$uid,gid:$gid,mode:"0750"},
    {kind:"artifact",path:($prefix+"/reports/data/reports"),uid:$uid,gid:$gid,mode:"0750"}
  ];

. as $manifest
| $lifecycle[0] as $runs
| $retry[0] as $retryState
| .schemaVersion == 1 and .status == "passed"
  and .gitHead == $head and .sourceTreeState == $state and .sourceTreeSha256 == $sourceSha
  and .platform == {architecture:$architecture,name:$platform}
  and .binaryHashes == $binaries and .buildProvenanceSha256 == $provenanceSha
  and .verifier == {imageId:$verifierImage,key:$verifierKey,provenanceSha256:$verifierSha}
  and .reportsRuntime == {user:"rz-reports",nonRoot:true,directoryMode:"0750",selected:true,pure:true}
  and .delivery == {manualTerminalClasses:6,retryInitiatorImmutable:true,scheduledSilent:true,
    revoked:"no-recipients",outageBackfilled:true,droppedResponse:"duplicate",
    authentication:{unsigned:400,badSignature:401,publicInternal:404}}
  and .pureReports == {notificationSchemaObjects:0,notificationConfig:false,
    notificationRoute:false,notificationTask:false,notificationListeners:0}
  and (.receipts|type=="array" and length==25)
  and ([.receipts[].file]|length==(unique|length))
  and ([.receipts[].file]|sort==($allowlist|sort))
  and all(.receipts[];
    (.file|type=="string" and test("^[A-Za-z0-9][A-Za-z0-9._-]*$"))
    and (.sha256|type=="string" and test("^[0-9a-f]{64}$"))
    and (.bytes|type=="number" and floor==. and .>0))
  and [$runs.success.status,$runs.failed.status,$runs.queuedCancel.status,
       $runs.cooperativeCancel.status,$runs.recoveryFailed.status,
       $runs.recoveryCancelled.status,$runs.dropResponse.status]
      == ["succeeded","failed","cancelled","cancelled","failed","cancelled","succeeded"]
  and $outage[0] == {runId:$runs.success.id,status:"succeeded",pending:1}
  and $retryState.source == $runs.failed.id
  and $retryState.firstChild == $retryState.secondChild
  and $retryState.initiatorUserId == 1
  and $scheduled[0] == {runId:"70000000-0000-4000-8000-000000000001",
    initiator:null,receiptCount:7,gapCount:0}
  and $revoked[0].runId != "" and $revoked[0].eventId != ""
  and $revoked[0].result == "no-recipients"
  and $proxy[0].accepted >= 2 and $proxy[0].firstResponseDropped == true
  and $duplicate[0] == {status:200,contentType:"application/json",body:{code:"duplicate"}}
  and $bad[0] == {status:401,contentType:"application/json",body:{code:"bad-producer"}}
  and $unsigned[0].code == "invalid-protocol"
  and $public[0] == {status:404,contentType:"application/json",
    body:{code:10001,message:"API route not found.",data:null}}
  and ($selectedListener|test("127[.]0[.]0[.]1:19831"))
  and ($selectedConfig[0]|tostring|contains("RUSTZEN_REPORTS_NOTIFICATION_INGRESS_URL"))
  and ($selectedApi[0].routes|any(.path=="/notification-delivery"))
  and $runtimeIdentity[0].user.name == "rz-reports"
  and $runtimeIdentity[0].user.uid > 0 and $runtimeIdentity[0].user.gid > 0
  and valid_reports_identity($runtimeIdentity[0].selected;"/tmp/rz-reports-notification-runtime/selected";$runtimeIdentity[0].user.uid;$runtimeIdentity[0].user.gid)
  and valid_reports_identity($runtimeIdentity[0].pure;"/tmp/rz-reports-notification-runtime/pure";$runtimeIdentity[0].user.uid;$runtimeIdentity[0].user.gid)
  and $selectedState[0].receipts == [["no-recipients",1],["stored",8]]
  and $selectedState[0].recipients == 8 and $selectedState[0].outbox == 0
  and $selectedState[0].gaps == [0,0,0,0,0]
  and ([($selectedState[0].messages[])]|sort ==
      ((run_topic($runs)+[["reports.run.failed",$retryState.firstChild]])|sort))
  and ($inbox[0].data.items|length==8)
  and all($inbox[0].data.items[];.producer=="reports" and .subjectKind=="reports-run")
  and ($pureAdminApi[0]|no_notification) and ($pureAdminConfig[0]|no_notification)
  and ($pureReportsConfig[0]|no_notification)
  and ($pureReportsApi[0].routes|all(.path!="/notification-delivery"))
  and $pureAbsence[0] == {admin:[],reports:[]}
  and $pureRoute[0] == {code:404,message:"Not found",data:null}
  and ($pureListeners|test("19841")|not)
