def envelope($value; $status; $code):
  $value[0].status == $status
  and $value[0].contentType == "application/json"
  and $value[0].body.code == $code
  and ($value[0].body.message | type) == "string"
  and (if $status >= 400 then $value[0].body.data == null else true end);

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
  twoSessions:true, logoutIsolation:true, revokeAll:true, passwordChange:true,
  userDisable:true, grantRemoved:true, grantRestored:true, badJwt:true,
  expiredJwt:true, authorityStorageFailClosed:true
}
and (.receipts | length) == ($allowlist | length)
and ([.receipts[].file] | length) == ([.receipts[].file] | unique | length)
and ([.receipts[].file] | sort) == ($allowlist | sort)
and ([.receipts[] | (.file | type == "string" and length > 0
  and startswith("/") | not) and (.file | contains("..") | not)
  and (.sha256 | test("^[0-9a-f]{64}$")) and (.bytes | type == "number" and . > 0)] | all)
and envelope($sidA;401;401)
and envelope($sidB;200;0)
and envelope($revokeAll;401;401)
and envelope($passwordOld;401;401)
and envelope($passwordNew;200;0)
and envelope($disableOld;401;401)
and envelope($disableLogin;403;10004)
and envelope($grantBefore;200;0)
and envelope($grantRemoved;403;403)
and envelope($grantRestored;200;0)
and envelope($badJwt;401;401)
and envelope($validBeforeExpiry;200;0)
and envelope($expiredJwt;401;401)
and envelope($authorityFailure;503;50302)
# Each role update deletes its current mappings and inserts the requested set:
# remove is 2 deletes + 1 insert; restore is 1 delete + 2 inserts.
and ($selected[0].grantEpochs.removed == $selected[0].grantEpochs.before + 3)
and ($selected[0].grantEpochs.restored == $selected[0].grantEpochs.removed + 3)
and ($selected[0].authzEpoch == $selected[0].grantEpochs.restored)
and ($selected[0].users | sort_by(.[0])) == [
  ["runtime_disable",2,2],
  ["runtime_grant",1,1],
  ["runtime_password",1,2],
  ["runtime_revoke",1,2],
  ["runtime_sid",1,1]
]
and ($selected[0].grantCodes | map(.[0]) | sort) == ["system:user:list","system:user:options"]
and $selected[0].expiryIdentity.claims.username == "runtime_grant"
and $selected[0].expiryIdentity.claims.sid == $selected[0].expiryIdentity.session[0]
and $selected[0].expiryIdentity.claims.userId == $selected[0].expiryIdentity.session[1]
and $selected[0].expiryIdentity.claims.userAuthEpoch == $selected[0].expiryIdentity.session[2]
and $selected[0].expiryIdentity.claims.exp == $selected[0].expiryIdentity.session[3]
and $selected[0].expiryIdentity.session[4] == null
and $selected[0].expiryIdentity.expiredClaims.sid == $selected[0].expiryIdentity.claims.sid
and $selected[0].expiryIdentity.expiredClaims.userId == $selected[0].expiryIdentity.claims.userId
and $selected[0].expiryIdentity.expiredClaims.username == $selected[0].expiryIdentity.claims.username
and $selected[0].expiryIdentity.expiredClaims.userAuthEpoch == $selected[0].expiryIdentity.claims.userAuthEpoch
and $selected[0].expiryIdentity.expiredClaims.exp < $selected[0].expiryIdentity.claims.exp
and $phaseStatus == "build\tpassed\nverifier-helper\tpassed\nruntime\tpassed\n"
and ($buildLog | length) > 0 and ($helperLog | length) > 0 and ($runtimeLog | length) > 0
