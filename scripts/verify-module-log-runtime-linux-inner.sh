#!/usr/bin/env bash
set -euo pipefail

for command in curl jq setpriv sha256sum stat tar groupadd useradd getent cut; do
  command -v "$command" >/dev/null
done

start_utc_date=$(date -u +%Y-%m-%d)
ports=(19801 19802 19803 19804)
for port in "${ports[@]}"; do
  ! ss -H -ltn "sport = :$port" | grep -q . || { echo "occupied port: $port" >&2; exit 1; }
done

for module in admin monitor insights reports; do
  groupadd --system "rz-$module"
  useradd --system --gid "rz-$module" --home-dir "/opt/rz/data/$module" --shell /usr/sbin/nologin "rz-$module"
done
groupadd --system rz-log-control
log_control_gid=$(getent group rz-log-control | cut -d: -f3)
install -d -m 0755 -o root -g root /opt/rz
install -d -m 0711 -o root -g root /opt/rz/data
install -d -m 0711 -o root -g root /opt/rz/data/db
for module in admin monitor insights; do
  install -d -m 0750 -o "rz-$module" -g "rz-$module" "/opt/rz/data/db/$module"
done
install -d -m 0750 -o rz-admin -g rz-admin \
  /opt/rz/data/uploads /opt/rz/data/avatars /opt/rz/data/update-requests /opt/rz/data/releases
install -d -m 0750 -o rz-reports -g rz-reports \
  /opt/rz/data/reports /opt/rz/data/reports/db /opt/rz/data/reports/.config /opt/rz/data/reports/.cache
install -d -m 0711 -o root -g root /opt/rz/logs
for module in admin monitor insights reports; do
  install -d -m 2770 -o "rz-$module" -g rz-log-control "/opt/rz/logs/$module"
done
for name in rz-admin rz-monitor rz-insights rz-reports; do
  install -m 0755 "/verify/bin/$name" "/opt/rz/$name"
done

common_env=(
  RUSTZEN_ENV=development RUSTZEN_RUNTIME_ROOT=/opt/rz
  RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_ADMIN_PORT=19801
  RUSTZEN_INTERNAL_HOST=127.0.0.1 RUSTZEN_MONITOR_PORT=19802
  RUSTZEN_INSIGHTS_PORT=19803 RUSTZEN_REPORTS_PORT=19804
  RUSTZEN_ADMIN_SQLITE_PATH=./data/db/admin/admin.db
  RUSTZEN_MONITOR_SQLITE_PATH=./data/db/monitor/monitor.db
  RUSTZEN_INSIGHTS_SQLITE_PATH=./data/db/insights/insights.db
  RUSTZEN_REPORTS_SQLITE_PATH=./data/reports/db/reports.db
  RUSTZEN_JWT_SECRET=module-log-runtime-jwt-secret
  RUSTZEN_IPC_TOKEN=module-log-runtime-ipc-secret
  RUSTZEN_MONITOR_AGENT_TOKEN=module-log-runtime-agent-secret
  RUSTZEN_REPORTS_CREDENTIAL_KEY=module-log-runtime-reports-key
  RUSTZEN_BUILD_ID=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
  RUSTZEN_COMPOSITION_ID=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
  RUSTZEN_MONITOR_SCHEMA_FINGERPRINT=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
  RUSTZEN_MONITOR_DATA_CONTRACT_ID=dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd
  RUSTZEN_TIMEZONE=UTC RUST_LOG=info
)

pids=()
cleanup() {
  result=$?
  trap - EXIT INT TERM
  for pid in "${pids[@]}"; do kill -TERM "$pid" 2>/dev/null || true; done
  for pid in "${pids[@]}"; do
    for _ in $(seq 1 50); do kill -0 "$pid" 2>/dev/null || break; sleep .1; done
    kill -KILL "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  done
  if [ "$result" -ne 0 ]; then
    for log in /tmp/verify-*.log; do [ -s "$log" ] && { echo "== $log ==" >&2; tail -n 80 "$log" >&2; }; done
  fi
  exit "$result"
}
trap cleanup EXIT INT TERM

umask 077
[ "$(umask)" = 0077 ]
env "${common_env[@]}" /opt/rz/rz-monitor init-db
env "${common_env[@]}" /opt/rz/rz-monitor bind-database
chown -R rz-monitor:rz-monitor /opt/rz/data/db/monitor

start_service() {
  name=$1
  user="rz-$name"
  shift
  home="/opt/rz/data/db/$name"
  [ "$name" != reports ] || home=/opt/rz/data/reports
  install -d -m 0750 -o "$user" -g "$user" "$home/.config" "$home/.cache"
  group_args=(--clear-groups)
  [ "$name" != admin ] || group_args=(--groups="$log_control_gid")
  (umask 0027; exec setpriv --reuid="$user" --regid="$user" "${group_args[@]}" --no-new-privs -- \
    env HOME="$home" XDG_CONFIG_HOME="$home/.config" \
    XDG_CACHE_HOME="$home/.cache" "${common_env[@]}" \
    "$@") >"/tmp/verify-$name.log" 2>&1 &
  pids+=("$!")
  printf '%s\n' "$!" >"/tmp/$name.pid"
}

start_service monitor /opt/rz/rz-monitor controller
start_service insights /opt/rz/rz-insights serve
start_service reports /opt/rz/rz-reports serve
start_service admin /opt/rz/rz-admin serve

for port in "${ports[@]}"; do
  ready=0
  for _ in $(seq 1 180); do
    if curl --fail --silent --show-error --connect-timeout 2 --max-time 5 "http://127.0.0.1:$port/health" >/dev/null 2>&1; then
      ready=1
      break
    fi
    sleep .1
  done
  [ "$ready" = 1 ] || { echo "service on port $port did not become healthy" >&2; exit 1; }
done
for pid in "${pids[@]}"; do kill -0 "$pid"; done

process_receipt=/verify/evidence/process-identities.tsv
: >"$process_receipt"
for module in admin monitor insights reports; do
  pid=$(cat "/tmp/$module.pid")
  cp "/proc/$pid/status" "/verify/evidence/process-$module.status"
  uid=$(awk '/^Uid:/{print $2}' "/verify/evidence/process-$module.status")
  gid=$(awk '/^Gid:/{print $2}' "/verify/evidence/process-$module.status")
  printf '%s\t%s\t%s\t%s\t%s\n' "$module" "$pid" "$uid" "$gid" "process-$module.status" >>"$process_receipt"
done
[ "$(stat -c %a /opt/rz/logs)" = 711 ]
for module in admin monitor insights reports; do
  module_uid=$(id -u "rz-$module")
  module_gid=$(id -g "rz-$module")
  [ "$(awk -F '\t' -v module="$module" '$1 == module {print $3}' "$process_receipt")" = "$module_uid" ]
  [ "$(awk -F '\t' -v module="$module" '$1 == module {print $4}' "$process_receipt")" = "$module_gid" ]
  [ "$(stat -c %U "/opt/rz/logs/$module")" = "rz-$module" ]
  [ "$(stat -c %G "/opt/rz/logs/$module")" = rz-log-control ]
  [ "$(stat -c %a "/opt/rz/logs/$module")" = 2770 ]
done
[ "$(stat -c %U /opt/rz/logs)" = root ]
[ "$(stat -c %G /opt/rz/logs)" = root ]
[ "$(stat -c %u /opt/rz/logs)" = 0 ]
[ "$(stat -c %g /opt/rz/logs)" = 0 ]
stat -c $'%n\t%u\t%g\t%a' /opt/rz/logs /opt/rz/logs/admin /opt/rz/logs/monitor /opt/rz/logs/insights /opt/rz/logs/reports \
  >/verify/evidence/directory-identities.tsv

declare -A log_paths=(
  [admin]="/opt/rz/logs/admin/admin.$start_utc_date"
  [monitor]="/opt/rz/logs/monitor/monitor.$start_utc_date"
  [insights]="/opt/rz/logs/insights/insights.$start_utc_date"
  [reports]="/opt/rz/logs/reports/reports.$start_utc_date"
)
declare -A startup_text=(
  [admin]="Server started successfully"
  [monitor]="Monitor Controller started"
  [insights]="Insights service started"
  [reports]="Reports service started"
)
for module in admin monitor insights reports; do
  ready=0
  for _ in $(seq 1 100); do
    if [ -s "${log_paths[$module]}" ] && grep -Fq "${startup_text[$module]}" "${log_paths[$module]}"; then
      ready=1
      break
    fi
    sleep .1
  done
  [ "$ready" = 1 ] || { echo "$module did not emit its current UTC-day startup log" >&2; exit 1; }
done
for module in admin monitor insights reports; do
  [ "$(stat -c %U "${log_paths[$module]}")" = "rz-$module" ]
  [ "$(stat -c %G "${log_paths[$module]}")" = rz-log-control ]
  [ "$(stat -c %a "${log_paths[$module]}")" = 640 ]
done
for module in admin monitor insights reports; do
  setpriv --reuid=rz-admin --regid=rz-admin --groups="$log_control_gid" --no-new-privs -- \
    test -r "${log_paths[$module]}"
done
! setpriv --reuid=rz-admin --regid=rz-admin --groups="$log_control_gid" --no-new-privs -- \
  sh -c ': >> "$1"' sh "${log_paths[monitor]}"
! setpriv --reuid=rz-monitor --regid=rz-monitor --clear-groups --no-new-privs -- \
  test -r "${log_paths[insights]}"
for module in admin monitor insights reports; do
  stat -c $'%n\t%i\t%u\t%g\t%a' "${log_paths[$module]}" |
    awk -v module="$module" 'BEGIN{OFS="\t"} {print module,$0}'
done >/verify/evidence/current-file-identities.tsv

api=http://127.0.0.1:19801
login() {
  curl --fail --silent --show-error --connect-timeout 3 --max-time 15 \
    -H 'content-type: application/json' \
    -d "{\"username\":\"$1\",\"password\":\"rustzen@123\"}" "$api/api/auth/login"
}
owner_token=$(login owner | jq -er '.data.token | select(length > 20)')
owner_auth=(-H "authorization: Bearer $owner_token")
options_menu_id=$(curl --fail --silent --show-error "${owner_auth[@]}" \
  "$api/api/system/menus/options" | jq -er '.data[] | select(.code == "system:user:options") | .value')
curl --fail --silent --show-error "${owner_auth[@]}" -H 'content-type: application/json' \
  -d "$(jq -nc --argjson menu "$options_menu_id" '{name:"Module log non-owner",code:"module_log_non_owner",status:1,menuIds:[$menu],description:"module log authorization gate"}')" \
  "$api/api/system/roles" | jq -e '.code == 0' >/dev/null
viewer_role_id=$(curl --fail --silent --show-error "${owner_auth[@]}" \
  "$api/api/system/roles?current=1&pageSize=100&roleCode=module_log_non_owner" | \
  jq -er '.data[] | select(.code == "module_log_non_owner") | .id')
curl --fail --silent --show-error "${owner_auth[@]}" -H 'content-type: application/json' \
  -d "$(jq -nc --argjson role "$viewer_role_id" '{username:"log_viewer",email:"module-log-viewer@example.test",password:"module-log-viewer-password",realName:"Module log viewer",status:1,roleIds:[$role]}')" \
  "$api/api/system/users" | jq -e '.code == 0 and (.data | type == "number")' >/dev/null
admin_token=$(curl --fail --silent --show-error --connect-timeout 3 --max-time 15 \
  -H 'content-type: application/json' \
  -d '{"username":"log_viewer","password":"module-log-viewer-password"}' \
  "$api/api/auth/login" | jq -er '.data.token | select(length > 20)')
admin_auth=(-H "authorization: Bearer $admin_token")

curl --fail --silent --show-error "${owner_auth[@]}" \
  "$api/api/system/status/module-logs?date=$start_utc_date" >/verify/evidence/list-owner.json
jq -e --arg date "$start_utc_date" '
  [.data[] | select(.date == $date and .active and .readable) | .module] | sort
    == ["admin","insights","monitor","reports"]
' /verify/evidence/list-owner.json >/dev/null

for module in admin monitor insights reports; do
  curl --fail --silent --show-error "${owner_auth[@]}" \
    "$api/api/system/status/module-logs/tail?module=$module&date=$start_utc_date" \
    >"/verify/evidence/tail-$module.json"
  jq -e --arg module_id "$module" --arg date "$start_utc_date" --arg text "${startup_text[$module]}" '
    .data.module == $module_id and .data.date == $date and (.data.content | contains($text))
      and .data.byteCount <= 262144 and .data.lineCount <= 2000
  ' "/verify/evidence/tail-$module.json" >/dev/null
done

denials=/verify/evidence/non-owner-denials.jsonl
: >"$denials"
deny() {
  name=$1
  shift
  status=$(curl --silent --show-error --output "/verify/evidence/denied-$name.json" \
    --write-out '%{http_code}' --connect-timeout 3 --max-time 15 "${admin_auth[@]}" "$@")
  [ "$status" = 403 ] || { echo "$name returned $status to non-owner" >&2; exit 1; }
  jq -e 'type == "object" and (keys == ["code","data","message"]) and
    .code == 403 and .message == "Permission denied" and .data == null' "/verify/evidence/denied-$name.json" >/dev/null
  ! grep -Eq 'Server started successfully|Monitor Controller started|Insights service started|Reports service started|/opt/rz/logs|module-log-runtime-(jwt|ipc|agent|reports)' "/verify/evidence/denied-$name.json"
  jq -nc --arg endpoint "$name" --arg bodyFile "denied-$name.json" --argjson status "$status" \
    '{endpoint:$endpoint,status:$status,bodyFile:$bodyFile}' >>"$denials"
}
deny list "$api/api/system/status/module-logs?date=$start_utc_date"
deny tail "$api/api/system/status/module-logs/tail?module=admin&date=$start_utc_date"
deny backup -H 'content-type: application/json' -d '{"files":[{"module":"admin","date":"2000-01-01"}]}' "$api/api/system/status/module-logs/backup"
deny cleanup-preview -X POST "$api/api/system/status/module-logs/cleanup/preview"
deny cleanup-confirm -X POST -H 'content-type: application/json' -d '{"token":"denied"}' "$api/api/system/status/module-logs/cleanup/confirm"
[ "$(jq -s 'length' "$denials")" = 5 ]

backup_request=$(jq -nc --arg date "$start_utc_date" '{files:["admin","monitor","insights","reports"] | map({module:.,date:$date})}')
curl --fail --silent --show-error "${owner_auth[@]}" -H 'content-type: application/json' \
  -d "$backup_request" -D /verify/evidence/archive.headers \
  -o /verify/evidence/rustzen-module-logs.tar "$api/api/system/status/module-logs/backup"
header_hash=$(awk 'BEGIN{IGNORECASE=1} /^x-rustzen-archive-sha256:/{gsub("\\r", "", $2); print $2}' /verify/evidence/archive.headers)
header_count=$(awk 'BEGIN{IGNORECASE=1} /^x-rustzen-archive-file-count:/{gsub("\\r", "", $2); print $2}' /verify/evidence/archive.headers)
[ "$header_count" = 4 ]
[ "$header_hash" = "$(sha256sum /verify/evidence/rustzen-module-logs.tar | awk '{print $1}')" ]
mkdir /tmp/archive
tar -xf /verify/evidence/rustzen-module-logs.tar -C /tmp/archive
tar -tf /verify/evidence/rustzen-module-logs.tar | LC_ALL=C sort >/verify/evidence/archive-members.txt
printf '%s\n' admin."$start_utc_date" insights."$start_utc_date" manifest.json monitor."$start_utc_date" reports."$start_utc_date" >/tmp/expected-members.txt
cmp /tmp/expected-members.txt /verify/evidence/archive-members.txt
cp /tmp/archive/manifest.json /verify/evidence/archive-manifest.json
[ "$(jq 'length' /verify/evidence/archive-manifest.json)" = 4 ]
for module in admin monitor insights reports; do
  archived="/tmp/archive/$module.$start_utc_date"
  source=${log_paths[$module]}
  cmp "$source" "$archived"
  cp "$source" "/verify/evidence/source-$module.log"
  cmp "/verify/evidence/source-$module.log" "$archived"
  jq -e --arg module_id "$module" --arg file "$module.$start_utc_date" \
    --arg date "$start_utc_date" --arg sha "$(sha256sum "$archived" | awk '{print $1}')" \
    --argjson bytes "$(wc -c <"$archived")" '
      any(.[]; .module == $module_id and .file_name == $file and .date == $date
        and .sha256 == $sha and .size_bytes == $bytes)
    ' /verify/evidence/archive-manifest.json >/dev/null
done

old_date=$(date -u -d "$start_utc_date - 90 days" +%Y-%m-%d)
for module in admin monitor insights reports; do
  old_path="/opt/rz/logs/$module/$module.$old_date"
  cp "${log_paths[$module]}" "$old_path"
  chown "rz-$module:rz-log-control" "$old_path"
  chmod 0640 "$old_path"
done
current_snapshot() {
  for module in admin monitor insights reports; do
    path=${log_paths[$module]}
    jq -nc --arg module_id "$module" --arg path "$path" \
      --argjson inode "$(stat -c %i "$path")" --argjson uid "$(stat -c %u "$path")" \
      --argjson gid "$(stat -c %g "$path")" --arg mode "$(stat -c %a "$path")" \
      '{module:$module_id,path:$path,inode:$inode,uid:$uid,gid:$gid,mode:$mode}'
  done | jq -s '.'
}
current_snapshot >/verify/evidence/current-before.json

curl --fail --silent --show-error -X POST "${owner_auth[@]}" \
  "$api/api/system/status/module-logs/cleanup/preview" >/verify/evidence/cleanup-preview.json
jq -e --arg date "$old_date" '
  (.data.candidates | length) == 4 and
  ([.data.candidates[] | select(.date == $date) | .module] | sort)
    == ["admin","insights","monitor","reports"]
' /verify/evidence/cleanup-preview.json >/dev/null
cleanup_token=$(jq -er '.data.token' /verify/evidence/cleanup-preview.json)
curl --fail --silent --show-error -X POST "${owner_auth[@]}" -H 'content-type: application/json' \
  -d "$(jq -nc --arg token "$cleanup_token" '{token:$token}')" \
  "$api/api/system/status/module-logs/cleanup/confirm" >/verify/evidence/cleanup-result.json
jq -e --arg date "$old_date" '
  .data.partial == false and (.data.removed | length) == 4 and
  (.data.retained | length) == 0 and (.data.failures | length) == 0 and
  ([.data.removed[] | select(.date == $date) | .module] | sort)
    == ["admin","insights","monitor","reports"]
' /verify/evidence/cleanup-result.json >/dev/null
for module in admin monitor insights reports; do [ ! -e "/opt/rz/logs/$module/$module.$old_date" ]; done
current_snapshot >/verify/evidence/current-after.json
cmp /verify/evidence/current-before.json /verify/evidence/current-after.json
end_utc_date=$(date -u +%Y-%m-%d)
[ "$end_utc_date" = "$start_utc_date" ] || { echo "UTC date changed during module-log gate" >&2; exit 1; }

processes=$(jq -Rn '[inputs | split("\t") |
  {service:.[0],pid:(.[1]|tonumber),uid:(.[2]|tonumber),gid:(.[3]|tonumber),statusFile:.[4]}]' \
  </verify/evidence/process-identities.tsv)
log_files=$(jq -n --arg date "$start_utc_date" '
  ["admin","monitor","insights","reports"] | map({module:.,date:$date,fileName:(. + "." + $date)})
')
receipts=$(for file in process-identities.tsv process-admin.status process-monitor.status process-insights.status process-reports.status directory-identities.tsv current-file-identities.tsv list-owner.json tail-admin.json tail-monitor.json tail-insights.json tail-reports.json non-owner-denials.jsonl denied-list.json denied-tail.json denied-backup.json denied-cleanup-preview.json denied-cleanup-confirm.json archive.headers archive-members.txt archive-manifest.json cleanup-preview.json cleanup-result.json current-before.json current-after.json; do
  jq -nc --arg file "$file" --arg sha256 "$(sha256sum "/verify/evidence/$file" | awk '{print $1}')" --argjson bytes "$(wc -c <"/verify/evidence/$file")" '{file:$file,sha256:$sha256,bytes:$bytes}'
done | jq -s '.')
jq -n \
  --arg head "$RUSTZEN_VERIFY_HEAD" --arg state "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" \
  --arg sourceSha "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" --arg architecture "$RUSTZEN_VERIFY_ARCHITECTURE" \
  --arg imageId "$RUSTZEN_VERIFY_VERIFIER_IMAGE_ID" --arg verifierKey "$RUSTZEN_VERIFY_VERIFIER_KEY" \
  --arg verifierSha "$RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256" \
  --arg startDate "$start_utc_date" --arg endDate "$end_utc_date" --arg oldDate "$old_date" --arg archiveSha "$header_hash" \
  --argjson binaryHashes "$RUSTZEN_VERIFY_BINARY_HASHES" --argjson processes "$processes" --argjson logControlGid "$log_control_gid" \
  --argjson logFiles "$log_files" --argjson receipts "$receipts" \
  --argjson archiveBytes "$(wc -c </verify/evidence/rustzen-module-logs.tar)" \
  --slurpfile archiveManifest /verify/evidence/archive-manifest.json \
  --slurpfile cleanupPreview /verify/evidence/cleanup-preview.json \
  --slurpfile cleanupResult /verify/evidence/cleanup-result.json \
  --slurpfile currentBefore /verify/evidence/current-before.json \
  --slurpfile currentAfter /verify/evidence/current-after.json \
  '{schemaVersion:1,gitHead:$head,sourceTreeState:$state,sourceTreeSha256:$sourceSha,architecture:$architecture,
    verifier:{imageId:$imageId,key:$verifierKey,provenanceSha256:$verifierSha},binaryHashes:$binaryHashes,
    utc:{startDate:$startDate,endDate:$endDate},umask:"0027",processes:$processes,directories:{root:{path:"/opt/rz/logs",uid:0,gid:0,mode:"0711"},modules:[$processes[]|{module:.service,path:("/opt/rz/logs/"+.service),uid:.uid,gid:$logControlGid,mode:"02770"}]},
    logFiles:$logFiles,api:{ownerList:"list-owner.json",ownerTails:["tail-admin.json","tail-monitor.json","tail-insights.json","tail-reports.json"],nonOwnerDenials:"non-owner-denials.jsonl"},
    archive:{file:"rustzen-module-logs.tar",sha256:$archiveSha,bytes:$archiveBytes,fileCount:4,manifest:$archiveManifest[0]},
    cleanup:{oldDate:$oldDate,oldFiles:["admin","monitor","insights","reports"]|map({module:.,path:("/opt/rz/logs/"+.+"/"+.+"."+$oldDate)}),preview:$cleanupPreview[0].data,result:$cleanupResult[0].data,currentBefore:$currentBefore[0],currentAfter:$currentAfter[0]},receipts:$receipts}' \
  >/verify/evidence/manifest.json

echo "four-service module-log runtime passed"
