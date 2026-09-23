#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
outer="$root/scripts/verify-module-log-runtime-linux.sh"
inner="$root/scripts/verify-module-log-runtime-linux-inner.sh"
bash -n "$outer" "$inner"
jq -n --arg module_id admin '{data:{module:"admin"}} | .data.module == $module_id' >/dev/null

grep -Fq 'RUSTZEN_VERIFY_BINARY_HASHES' "$outer"
grep -Fq 'admin-browser-source-identity.sh' "$outer"
grep -Fq 'mv "$evidence_source" "$failed_run"' "$outer"
grep -Fq 'run_bounded "$cleanup_timeout" "$docker_bin" rm -f "$container"' "$outer"
grep -Fq "trap 'cleanup_with_status 130' INT" "$outer"
grep -Fq "trap 'cleanup_with_status 143' TERM" "$outer"
grep -Fq 'publish_candidate' "$outer"
grep -Fq 'cmp "$candidate/source-$module.log" "$verify_tmp/$member"' "$outer"
grep -Fq 'jq -e -s' "$outer"
grep -Fq 'setpriv --reuid="$user" --regid="$user"' "$inner"
grep -Fq '[ "$(stat -c %a "/opt/rz/logs/$module")" = 2770 ]' "$inner"
grep -Fq 'grep -Fq "${startup_text[$module]}" "${log_paths[$module]}"' "$inner"
grep -Fq 'cmp "$source" "$archived"' "$inner"
grep -Fq 'cmp /verify/evidence/current-before.json /verify/evidence/current-after.json' "$inner"
grep -Fq '[ "$end_utc_date" = "$start_utc_date" ]' "$inner"
grep -Fq 'deny cleanup-confirm' "$inner"
grep -Fq '.code == 403 and .message == "Permission denied" and .data == null' "$inner"
grep -Fq 'process-identities.tsv' "$inner"
grep -Fq 'directory-identities.tsv' "$inner"
for unit in rz-admin rz-monitor rz-insights rz-reports; do
  grep -Fqx 'UMask=0027' "$root/deploy/$unit.service"
done

test_root=$(mktemp -d "${TMPDIR:-/tmp}/rz-module-log-gate-test.XXXXXX")
trap 'rm -rf "$test_root"' EXIT
fake_docker="$test_root/docker"
cat >"$fake_docker" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >>"$RUSTZEN_MODULE_LOG_TEST_DOCKER_RECEIPT"
command=${1:-}
case ",${RUSTZEN_MODULE_LOG_TEST_BLOCK:-}," in
*",$command,"*)
  trap 'printf "%s terminated\n" "$command" >>"$RUSTZEN_MODULE_LOG_TEST_DOCKER_RECEIPT"; exit 143' INT TERM
  printf '%s started\n' "$command" >>"$RUSTZEN_MODULE_LOG_TEST_DOCKER_RECEIPT"
  while :; do sleep 1 & wait "$!"; done
  ;;
esac
[ "$command" != logs ] || printf 'test container log\n'
EOF
chmod +x "$fake_docker"
receipt="$test_root/docker.receipt"
RUSTZEN_MODULE_LOG_DOCKER="$fake_docker" RUSTZEN_MODULE_LOG_TEST_ROOT="$test_root/publish" \
  RUSTZEN_MODULE_LOG_TEST_DOCKER_RECEIPT="$receipt" RUSTZEN_MODULE_LOG_TEST_PUBLISH=1 "$outer"
[ "$(readlink "$test_root/publish/current")" = runs/test-publish ]
for command in run logs rm; do
  set +e
  RUSTZEN_MODULE_LOG_DOCKER="$fake_docker" RUSTZEN_MODULE_LOG_TEST_DOCKER_RECEIPT="$receipt" \
    RUSTZEN_MODULE_LOG_TEST_BLOCK="$command" RUSTZEN_MODULE_LOG_TEST_BOUNDED="$command" "$outer"
  status=$?
  set -e
  [ "$status" = 124 ]
  grep -Fq "$command terminated" "$receipt"
done
for pair in INT:130 TERM:143; do
  signal=${pair%%:*}
  expected=${pair##*:}
  signal_root="$test_root/signal-$signal"
  set +e
  RUSTZEN_MODULE_LOG_DOCKER="$fake_docker" RUSTZEN_MODULE_LOG_TEST_ROOT="$signal_root" \
    RUSTZEN_MODULE_LOG_TEST_DOCKER_RECEIPT="$receipt" RUSTZEN_MODULE_LOG_TEST_BLOCK=run,logs,rm \
    RUSTZEN_MODULE_LOG_CLEANUP_TIMEOUT=1 \
    RUSTZEN_MODULE_LOG_TEST_SIGNAL="$signal" "$outer"
  status=$?
  set -e
  [ "$status" = "$expected" ]
  [ -f "$signal_root/failed-runs/test-signal/container.log" ]
  [ ! -e "$signal_root/.verify.lock" ]
  [ ! -e "$signal_root/.binaries-test-signal" ]
done
grep -Fq 'rm -f module-log-test-container' "$receipt"

denials="$test_root/denials"
mkdir "$denials"
: >"$denials/non-owner-denials.jsonl"
for endpoint in list tail backup cleanup-preview cleanup-confirm; do
  body="denied-$endpoint.json"
  printf '{"code":403,"message":"Permission denied","data":null}\n' >"$denials/$body"
  jq -nc --arg endpoint "$endpoint" --arg bodyFile "$body" '{endpoint:$endpoint,status:403,bodyFile:$bodyFile}' \
    >>"$denials/non-owner-denials.jsonl"
done
RUSTZEN_MODULE_LOG_TEST_ROOT="$denials" RUSTZEN_MODULE_LOG_TEST_VERIFY=denials "$outer"
cp -R "$denials" "$test_root/denials-tampered"
printf '{"code":403,"message":"Permission denied module-log-runtime-jwt-secret","data":null}\n' \
  >"$test_root/denials-tampered/denied-tail.json"
if RUSTZEN_MODULE_LOG_TEST_ROOT="$test_root/denials-tampered" RUSTZEN_MODULE_LOG_TEST_VERIFY=denials "$outer"; then
  echo 'secret-bearing denial receipt was accepted' >&2
  exit 1
fi

identity="$test_root/identity"
mkdir "$identity"
cat >"$identity/process-identities.tsv" <<'EOF'
admin	101	988	988	process-admin.status
monitor	102	989	989	process-monitor.status
insights	103	990	990	process-insights.status
reports	104	991	991	process-reports.status
EOF
for record in admin:101:988 monitor:102:989 insights:103:990 reports:104:991; do
  service=${record%%:*}; rest=${record#*:}; pid=${rest%%:*}; identity_id=${record##*:}
  groups=$identity_id
  [ "$service" != admin ] || groups="$identity_id 987"
  printf 'Name:\trz-%s\nPid:\t%s\nUid:\t%s\t%s\t%s\t%s\nGid:\t%s\t%s\t%s\t%s\nGroups:\t%s\n' \
    "$service" "$pid" "$identity_id" "$identity_id" "$identity_id" "$identity_id" \
    "$identity_id" "$identity_id" "$identity_id" "$identity_id" "$groups" >"$identity/process-$service.status"
done
cat >"$identity/directory-identities.tsv" <<'EOF'
/opt/rz/logs	0	0	711
/opt/rz/logs/admin	988	987	2770
/opt/rz/logs/monitor	989	987	2770
/opt/rz/logs/insights	990	987	2770
/opt/rz/logs/reports	991	987	2770
EOF
cat >"$identity/current-file-identities.tsv" <<'EOF'
admin	/opt/rz/logs/admin/admin.2026-09-07	201	988	987	640
monitor	/opt/rz/logs/monitor/monitor.2026-09-07	202	989	987	640
insights	/opt/rz/logs/insights/insights.2026-09-07	203	990	987	640
reports	/opt/rz/logs/reports/reports.2026-09-07	204	991	987	640
EOF
jq -n '{utc:{startDate:"2026-09-07"},processes:[
  {service:"admin",pid:101,uid:988,gid:988,statusFile:"process-admin.status"},
  {service:"monitor",pid:102,uid:989,gid:989,statusFile:"process-monitor.status"},
  {service:"insights",pid:103,uid:990,gid:990,statusFile:"process-insights.status"},
  {service:"reports",pid:104,uid:991,gid:991,statusFile:"process-reports.status"}],
  directories:{root:{path:"/opt/rz/logs",uid:0,gid:0,mode:"0711"},modules:[
    {module:"admin",path:"/opt/rz/logs/admin",uid:988,gid:987,mode:"02770"},
    {module:"monitor",path:"/opt/rz/logs/monitor",uid:989,gid:987,mode:"02770"},
    {module:"insights",path:"/opt/rz/logs/insights",uid:990,gid:987,mode:"02770"},
    {module:"reports",path:"/opt/rz/logs/reports",uid:991,gid:987,mode:"02770"}]},
  cleanup:{currentBefore:[
    {module:"admin",path:"/opt/rz/logs/admin/admin.2026-09-07",inode:201,uid:988,gid:987,mode:"640"},
    {module:"monitor",path:"/opt/rz/logs/monitor/monitor.2026-09-07",inode:202,uid:989,gid:987,mode:"640"},
    {module:"insights",path:"/opt/rz/logs/insights/insights.2026-09-07",inode:203,uid:990,gid:987,mode:"640"},
    {module:"reports",path:"/opt/rz/logs/reports/reports.2026-09-07",inode:204,uid:991,gid:987,mode:"640"}]}}' \
  >"$identity/manifest.json"
RUSTZEN_MODULE_LOG_TEST_ROOT="$identity" RUSTZEN_MODULE_LOG_TEST_VERIFY=identity "$outer"
for tamper in process directory file; do
  tampered="$test_root/identity-$tamper"
  cp -R "$identity" "$tampered"
  case "$tamper" in
    process) sed -i.bak $'s/reports\t104\t991\t991/reports\t104\t992\t991/' "$tampered/process-identities.tsv" ;;
    directory) sed -i.bak $'s#/opt/rz/logs/reports\t991\t987#/opt/rz/logs/reports\t992\t987#' "$tampered/directory-identities.tsv" ;;
    file) sed -i.bak $'s#reports\t/opt/rz/logs/reports/reports.2026-09-07\t204\t991\t987#reports\t/opt/rz/logs/reports/reports.2026-09-07\t204\t991\t992#' "$tampered/current-file-identities.tsv" ;;
  esac
  rm -f "$tampered"/*.bak
  if RUSTZEN_MODULE_LOG_TEST_ROOT="$tampered" RUSTZEN_MODULE_LOG_TEST_VERIFY=identity "$outer"; then
    echo "$tamper identity tamper was accepted" >&2
    exit 1
  fi
done

echo "module-log runtime Linux gate contract passed"
