#!/usr/bin/env bash
set -euo pipefail

if [ "${RUSTZEN_ANALYTICS_TRACKER_INNER:-}" = 1 ]; then
  expected_chromium_version=${RUSTZEN_VERIFY_CHROMIUM_VERSION:?missing pinned Chromium version}
  expected_verifier_provenance_sha=${RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256:?missing verifier provenance hash}
  test "$(dpkg-query -W -f='${Version}' chromium)" = "$expected_chromium_version"
  test "$(sha256sum /usr/local/share/rustzen-browser-verifier.provenance | awk '{print $1}')" = "$expected_verifier_provenance_sha"
  for command in chromium curl jq file setpriv ss python3; do command -v "$command" >/dev/null; done

  service_ports=(19801 19802 19803 19804)
  for port in "${service_ports[@]}" 18080; do
    if ss -H -ltn "sport = :$port" | grep -q .; then
      echo "analytics tracker verification port is already occupied: $port" >&2
      exit 1
    fi
  done

  groupadd --system rustzen
  useradd --system --gid rustzen --home-dir /opt/rz --shell /usr/sbin/nologin rustzen
  install -d -m 0750 -o rustzen -g rustzen /opt/rz /opt/rz/data/db /opt/rz/data/reports/db /opt/rz/logs /opt/rz/logs/reports /opt/rz/output
  for name in rz-admin rz-monitor rz-insights rz-reports; do
    install -m 0755 "/verify/bin/$name" "/opt/rz/$name"
  done
  chown -R rustzen:rustzen /opt/rz

  export HOME=/opt/rz XDG_CONFIG_HOME=/opt/rz/.config XDG_CACHE_HOME=/opt/rz/.cache
  export RUSTZEN_ENV=development RUSTZEN_RUNTIME_ROOT=/opt/rz
  export RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_ADMIN_PORT=19801 RUSTZEN_INTERNAL_HOST=127.0.0.1
  export RUSTZEN_MONITOR_PORT=19802 RUSTZEN_INSIGHTS_PORT=19803 RUSTZEN_REPORTS_PORT=19804
  export RUSTZEN_ADMIN_SQLITE_PATH=./data/db/admin.db RUSTZEN_MONITOR_SQLITE_PATH=./data/db/monitor.db
  export RUSTZEN_INSIGHTS_SQLITE_PATH=./data/db/insights.db RUSTZEN_REPORTS_SQLITE_PATH=./data/reports/db/reports.db
  export RUSTZEN_JWT_SECRET=analytics-tracker-verification-jwt
  export RUSTZEN_IPC_TOKEN=analytics-tracker-verification-ipc
  export RUSTZEN_MONITOR_AGENT_TOKEN=analytics-tracker-verification-agent
  export RUSTZEN_REPORTS_CREDENTIAL_KEY=analytics-tracker-verification-credential
  export RUSTZEN_REPORTS_BROWSER_PATH=/usr/bin/chromium RUSTZEN_REPORTS_MAX_CONCURRENCY=1
  export RUSTZEN_TIMEZONE=UTC RUST_LOG=warn
  export RUSTZEN_BUILD_ID=$(printf 'a%.0s' {1..64}) RUSTZEN_COMPOSITION_ID=$(printf 'b%.0s' {1..64})
  export RUSTZEN_MONITOR_SCHEMA_FINGERPRINT=$(printf 'c%.0s' {1..64})
  export RUSTZEN_MONITOR_DATA_CONTRACT_ID=$(printf 'd%.0s' {1..64})

  pids=()
  cleanup() {
    result=$?
    for pid in "${pids[@]}"; do kill -TERM "$pid" 2>/dev/null || true; done
    for pid in "${pids[@]}"; do wait "$pid" 2>/dev/null || true; done
    if [ "$result" -ne 0 ]; then
      for log in /opt/rz/logs/verify-*.log; do [ -s "$log" ] && { echo "== $log ==" >&2; tail -n 80 "$log" >&2; }; done
    fi
    exit "$result"
  }
  trap cleanup EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM

  start() {
    name=$1
    shift
    setpriv --reuid=rustzen --regid=rustzen --init-groups --no-new-privs -- env \
      HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" XDG_CACHE_HOME="$XDG_CACHE_HOME" \
      RUSTZEN_ENV="$RUSTZEN_ENV" RUSTZEN_RUNTIME_ROOT="$RUSTZEN_RUNTIME_ROOT" \
      RUSTZEN_ADMIN_HOST="$RUSTZEN_ADMIN_HOST" RUSTZEN_ADMIN_PORT="$RUSTZEN_ADMIN_PORT" \
      RUSTZEN_INTERNAL_HOST="$RUSTZEN_INTERNAL_HOST" RUSTZEN_MONITOR_PORT="$RUSTZEN_MONITOR_PORT" \
      RUSTZEN_INSIGHTS_PORT="$RUSTZEN_INSIGHTS_PORT" RUSTZEN_REPORTS_PORT="$RUSTZEN_REPORTS_PORT" \
      RUSTZEN_ADMIN_SQLITE_PATH="$RUSTZEN_ADMIN_SQLITE_PATH" RUSTZEN_MONITOR_SQLITE_PATH="$RUSTZEN_MONITOR_SQLITE_PATH" \
      RUSTZEN_INSIGHTS_SQLITE_PATH="$RUSTZEN_INSIGHTS_SQLITE_PATH" RUSTZEN_REPORTS_SQLITE_PATH="$RUSTZEN_REPORTS_SQLITE_PATH" \
      RUSTZEN_JWT_SECRET="$RUSTZEN_JWT_SECRET" RUSTZEN_IPC_TOKEN="$RUSTZEN_IPC_TOKEN" \
      RUSTZEN_MONITOR_AGENT_TOKEN="$RUSTZEN_MONITOR_AGENT_TOKEN" RUSTZEN_REPORTS_CREDENTIAL_KEY="$RUSTZEN_REPORTS_CREDENTIAL_KEY" \
      RUSTZEN_REPORTS_BROWSER_PATH="$RUSTZEN_REPORTS_BROWSER_PATH" RUSTZEN_REPORTS_MAX_CONCURRENCY=1 \
      RUSTZEN_BUILD_ID="$RUSTZEN_BUILD_ID" RUSTZEN_COMPOSITION_ID="$RUSTZEN_COMPOSITION_ID" \
      RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_MONITOR_SCHEMA_FINGERPRINT" \
      RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_MONITOR_DATA_CONTRACT_ID" \
      RUSTZEN_TIMEZONE=UTC RUST_LOG=warn "$@" >"/opt/rz/logs/verify-$name.log" 2>&1 &
    pids+=("$!")
  }

  initialize_monitor_database() {
    setpriv --reuid=rustzen --regid=rustzen --init-groups --no-new-privs -- env \
      HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" XDG_CACHE_HOME="$XDG_CACHE_HOME" \
      RUSTZEN_ENV="$RUSTZEN_ENV" RUSTZEN_RUNTIME_ROOT="$RUSTZEN_RUNTIME_ROOT" \
      RUSTZEN_MONITOR_SQLITE_PATH="$RUSTZEN_MONITOR_SQLITE_PATH" RUSTZEN_BUILD_ID="$RUSTZEN_BUILD_ID" \
      RUSTZEN_COMPOSITION_ID="$RUSTZEN_COMPOSITION_ID" RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_MONITOR_SCHEMA_FINGERPRINT" \
      RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_MONITOR_DATA_CONTRACT_ID" RUSTZEN_TIMEZONE=UTC /opt/rz/rz-monitor init-db
    setpriv --reuid=rustzen --regid=rustzen --init-groups --no-new-privs -- env \
      HOME="$HOME" XDG_CONFIG_HOME="$XDG_CONFIG_HOME" XDG_CACHE_HOME="$XDG_CACHE_HOME" \
      RUSTZEN_ENV="$RUSTZEN_ENV" RUSTZEN_RUNTIME_ROOT="$RUSTZEN_RUNTIME_ROOT" \
      RUSTZEN_MONITOR_SQLITE_PATH="$RUSTZEN_MONITOR_SQLITE_PATH" RUSTZEN_BUILD_ID="$RUSTZEN_BUILD_ID" \
      RUSTZEN_COMPOSITION_ID="$RUSTZEN_COMPOSITION_ID" RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_MONITOR_SCHEMA_FINGERPRINT" \
      RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_MONITOR_DATA_CONTRACT_ID" RUSTZEN_TIMEZONE=UTC /opt/rz/rz-monitor bind-database
  }

  initialize_monitor_database
  start monitor /opt/rz/rz-monitor controller
  start insights /opt/rz/rz-insights serve
  start reports /opt/rz/rz-reports serve
  start admin /opt/rz/rz-admin serve
  setpriv --reuid=rustzen --regid=rustzen --init-groups --no-new-privs -- \
    python3 -m http.server 18080 --directory /verify/fixture >/opt/rz/logs/verify-fixture.log 2>&1 &
  pids+=("$!")

  curl_json() { curl --fail --silent --show-error --connect-timeout 3 --max-time 15 "$@"; }
  for port in "${service_ports[@]}"; do
    ready=0
    for _ in $(seq 1 180); do
      if curl_json "http://127.0.0.1:$port/health" >/dev/null 2>&1; then ready=1; break; fi
      sleep .1
    done
    test "$ready" = 1
  done
  admin=http://127.0.0.1:19801
  login=$(curl_json -H 'content-type: application/json' -d '{"username":"owner","password":"rustzen@123"}' "$admin/api/auth/login")
  token=$(jq -er '.data.token | select(length > 20)' <<<"$login")
  auth=(-H "authorization: Bearer $token")
  policy=$(jq -nc '{collectionEnabled:true,projectKey:"linux-analytics-project-key",allowedOrigins:["http://127.0.0.1:18080"]}')
  curl_json "${auth[@]}" -H 'content-type: application/json' -X PUT -d "$policy" "$admin/api/insights/collection-policy" >/dev/null
  event_total() { curl_json "${auth[@]}" "$admin/api/insights/events" | jq -er '.data.total'; }

  system=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d '{"name":"Analytics tracker fixture","baseUrl":"http://127.0.0.1:18080","enabled":true}' "$admin/api/reports/systems")
  system_id=$(jq -er '.data.id' <<<"$system")
  start_flow() {
    flow_body=$1
    flow=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg system "$system_id" --argjson steps "$flow_body" '{systemId:$system,name:"Analytics tracker browser gate",steps:$steps}')" "$admin/api/reports/flows")
    flow_id=$(jq -er '.data.id' <<<"$flow")
    run=$(curl_json "${auth[@]}" -H 'content-type: application/json' -d "$(jq -nc --arg flow "$flow_id" '{flowId:$flow,input:{}}')" "$admin/api/reports/runs")
    jq -er '.data.id' <<<"$run"
  }
  wait_run() {
    run_id=$1
    state=queued
    for _ in $(seq 1 900); do
      state=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$run_id" | jq -er '.data.status')
      case "$state" in queued|running|cancelling) sleep .1 ;; *) break ;; esac
    done
    test "$state" = succeeded || { curl_json "${auth[@]}" "$admin/api/reports/runs/$run_id/steps" >&2 || true; return 1; }
  }

  pre_rows=$(event_total)
  pre_steps='[{"action":"goto","url":"/"},{"action":"waitFor","selector":"#state"},{"action":"assertText","selector":"#state","text":"pre-optin:native:no-ids"},{"action":"assertText","selector":"#request-count","text":"0"},{"action":"pause","durationMs":250}]'
  pre_run_id=$(start_flow "$pre_steps")
  wait_run "$pre_run_id"
  test "$(event_total)" = "$pre_rows"

  lifecycle_steps='[{"action":"goto","url":"/"},{"action":"waitFor","selector":"#state"},{"action":"assertText","selector":"#state","text":"pre-optin:native:no-ids"},{"action":"assertText","selector":"#request-count","text":"0"},{"action":"click","selector":"#opt-in"},{"action":"assertText","selector":"#state","text":"opted-in:patched:ids"},{"action":"assertText","selector":"#request-count","text":"1"},{"action":"pause","durationMs":2000},{"action":"click","selector":"#opt-out"},{"action":"pause","durationMs":200},{"action":"assertText","selector":"#state","text":"opted-out:native:no-ids"},{"action":"assertText","selector":"#request-count","text":"1"},{"action":"pause","durationMs":250}]'
  lifecycle_run_id=$(start_flow "$lifecycle_steps")
  optin_rows=$pre_rows
  for _ in $(seq 1 40); do
    optin_rows=$(event_total)
    [ "$optin_rows" -gt "$pre_rows" ] && break
    sleep .1
  done
  test "$optin_rows" -gt "$pre_rows"
  wait_run "$lifecycle_run_id"
  curl_json "${auth[@]}" "$admin/api/insights/events" | jq -e '.data.data[] | select(.eventName == "custom_export" and .pagePath == "/fixture/allowed")' >/dev/null
  optout_rows=$(event_total)
  test "$optout_rows" = "$optin_rows"

  oversized_body=$(python3 - <<'PY'
import json
print(json.dumps({"eventName":"page_view","visitorId":"http-413","pagePath":"/413","properties":{"feature":"x" * 66000}}))
PY
)
  before_413=$optout_rows
  status_413=$(curl --silent --show-error --connect-timeout 3 --max-time 15 -o /tmp/analytics-413.json -w '%{http_code}' \
    -H 'content-type: application/json' -H 'origin: http://127.0.0.1:18080' -H 'x-rustzen-project-key: linux-analytics-project-key' \
    --data-raw "$oversized_body" "$admin/api/insights/track")
  test "$status_413" = 413
  after_413=$(event_total)
  test "$after_413" = "$before_413"

  accepted_preload_count=28
  for index in $(seq 1 "$accepted_preload_count"); do
    curl_json -H 'content-type: application/json' -H 'origin: http://127.0.0.1:18080' \
      -H 'x-rustzen-project-key: linux-analytics-project-key' \
      --data-raw "$(jq -nc --arg visitor "http-429-$index" '{eventName:"page_view",visitorId:$visitor,pagePath:"/rate"}')" \
      "$admin/api/insights/track" >/dev/null
  done
  before_429=$(event_total)
  status_429=$(curl --silent --show-error --connect-timeout 3 --max-time 15 -o /tmp/analytics-429.json -w '%{http_code}' \
    -H 'content-type: application/json' -H 'origin: http://127.0.0.1:18080' -H 'x-rustzen-project-key: linux-analytics-project-key' \
    --data-raw '{"eventName":"page_view","visitorId":"http-429-rejected","pagePath":"/rate-rejected"}' \
    "$admin/api/insights/track")
  test "$status_429" = 429
  after_429=$(event_total)
  test "$after_429" = "$before_429"

  browser_version=$(/usr/bin/chromium --version)
  jq -n \
    --arg head "$RUSTZEN_VERIFY_HEAD" \
    --arg sourceTreeState "$RUSTZEN_VERIFY_SOURCE_TREE_STATE" \
    --arg sourceTreeSha256 "$RUSTZEN_VERIFY_SOURCE_TREE_SHA256" \
    --arg architecture "$RUSTZEN_VERIFY_ARCHITECTURE" \
    --arg verifierImageId "$RUSTZEN_VERIFY_VERIFIER_IMAGE_ID" \
    --arg verifierKey "$RUSTZEN_VERIFY_VERIFIER_KEY" \
    --arg verifierProvenanceSha256 "$RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256" \
    --arg browser "$browser_version" \
    --arg preRunId "$pre_run_id" --arg lifecycleRunId "$lifecycle_run_id" \
    --argjson acceptedPreloadCount "$accepted_preload_count" \
    --argjson preRows "$pre_rows" --argjson optinRows "$optin_rows" --argjson optoutRows "$optout_rows" \
    --argjson status413 "$status_413" --argjson before413 "$before_413" --argjson after413 "$after_413" \
    --argjson status429 "$status_429" --argjson before429 "$before_429" --argjson after429 "$after_429" \
    --argjson verifierLines "$(wc -l </verify/run.sh)" --argjson fixtureLines "$(wc -l </verify/fixture/index.html)" \
    '{schemaVersion:1,status:"partial",gitHead:$head,sourceTreeState:$sourceTreeState,sourceTreeSha256:$sourceTreeSha256,platform:{architecture:$architecture,platform:(if $architecture == "aarch64" then "linux/arm64" else "linux/amd64" end)},browser:{chromium:$browser},verifier:{imageId:$verifierImageId,key:$verifierKey,provenanceSha256:$verifierProvenanceSha256},lines:{verifier:$verifierLines,fixture:$fixtureLines},cases:{preOptIn:{status:"passed",runId:$preRunId,rowsBefore:$preRows,rowsAfter:$preRows},optIn:{status:"passed",runId:$lifecycleRunId,rowsBefore:$preRows,rowsAfter:$optinRows},optOut:{status:"passed",runId:$lifecycleRunId,rowsBefore:$optinRows,rowsAfter:$optoutRows}},http:{"413":{status:"passed",observedStatus:$status413,rowDelta:($after413-$before413)},"429":{status:"passed",observedStatus:$status429,acceptedPreloadCount:$acceptedPreloadCount,rowDelta:($after429-$before429)},"507":{status:"not-verified",observedStatus:null,rowDelta:null,reason:"safe Linux runtime capacity injection is unavailable; controlled Rust route seam covers 507"}}}' \
    >/verify/evidence/manifest.json
  jq -e '.status == "partial" and .http["413"].observedStatus == 413 and .http["413"].rowDelta == 0 and .http["429"].observedStatus == 429 and .http["429"].rowDelta == 0 and .http["507"].status == "not-verified"' /verify/evidence/manifest.json >/dev/null
  echo "Analytics tracker Linux host gate passed (413/429 target-backed; 507 not-verified; $browser_version)"
  exit 0
fi

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
docker_bin=${RUSTZEN_ANALYTICS_TRACKER_DOCKER:-docker}
architecture=${RUSTZEN_UI_LINUX_ARCH:-}
evidence_root="$root/target/rz/analytics-tracker"
current="$evidence_root/current"
if [ -e "$current" ] && [ ! -L "$current" ]; then
  echo "refusing to replace legacy Analytics tracker evidence directory: $current; move it under runs/ first" >&2
  exit 1
fi
run_timeout=${RUSTZEN_ANALYTICS_TRACKER_TIMEOUT:-480}
case "$run_timeout" in
  ''|*[!0-9]*) echo 'RUSTZEN_ANALYTICS_TRACKER_TIMEOUT must be a positive integer' >&2; exit 2 ;;
esac
[ "$run_timeout" -gt 0 ] && [ "$run_timeout" -le 900 ] || {
  echo 'RUSTZEN_ANALYTICS_TRACKER_TIMEOUT must be 1..900 seconds' >&2
  exit 2
}
info_timeout=${RUSTZEN_ANALYTICS_TRACKER_DOCKER_INFO_TIMEOUT:-10}
case "$info_timeout" in
  ''|*[!0-9]*) echo 'RUSTZEN_ANALYTICS_TRACKER_DOCKER_INFO_TIMEOUT must be a positive integer' >&2; exit 2 ;;
esac
[ "$info_timeout" -gt 0 ] && [ "$info_timeout" -le 60 ] || {
  echo 'RUSTZEN_ANALYTICS_TRACKER_DOCKER_INFO_TIMEOUT must be 1..60 seconds' >&2
  exit 2
}
run_bounded() {
  seconds=$1; shift
  "$@" & command_pid=$!
  ( sleep "$seconds"; kill -TERM "$command_pid" 2>/dev/null || true; sleep 10; kill -KILL "$command_pid" 2>/dev/null || true ) >/dev/null 2>&1 & watchdog_pid=$!
  if wait "$command_pid"; then command_status=0; else command_status=$?; fi
  kill "$watchdog_pid" 2>/dev/null || true
  pkill -TERM -P "$watchdog_pid" 2>/dev/null || true
  wait "$watchdog_pid" 2>/dev/null || true
  return "$command_status"
}
run_bounded_capture() {
  capture=$(mktemp "${TMPDIR:-/tmp}/rz-analytics-command.XXXXXX") || return 1
  if run_bounded "$@" >"$capture"; then command_status=0; else command_status=$?; fi
  cat "$capture"
  rm -f "$capture"
  return "$command_status"
}
atomic_replace_symlink() {
  source_path=$1
  target_path=$2
  case "$(uname -s)" in
    Darwin|FreeBSD) mv -fh "$source_path" "$target_path" ;;
    *) mv -Tf "$source_path" "$target_path" ;;
  esac
}
on_interrupt() { exit 130; }
on_terminate() { exit 143; }
if [ "${RUSTZEN_ANALYTICS_TRACKER_TEST_PUBLISH_FAILURE:-}" = 1 ]; then
  test_root=$(mktemp -d "${TMPDIR:-/tmp}/rz-analytics-publish.XXXXXX")
  evidence_root="$test_root"
  current="$evidence_root/current"
  candidate="$evidence_root/.candidate-test"
  run_id=test
  publish_candidate() {
    run_dir="$evidence_root/runs/$run_id"
    current_tmp="$evidence_root/.current-$run_id"
    mv "$candidate" "$run_dir" || return 1
    ln -s "runs/$run_id" "$current_tmp" || return 1
    atomic_replace_symlink "$current_tmp" "$current" || return 1
  }
  mkdir -p "$evidence_root/runs/old" "$candidate"
  printf old >"$evidence_root/runs/old/manifest.json"
  ln -s runs/old "$current"
  printf new >"$candidate/manifest.json"
  publish_candidate
  test "$(readlink "$current")" = runs/test
  test "$(cat "$current/manifest.json")" = new
  run_id=failed
  candidate="$evidence_root/.candidate-failed"
  mkdir "$candidate"
  rm -rf "$candidate"
  if publish_candidate; then
    echo 'forced publication failure unexpectedly succeeded' >&2
    exit 1
  fi
  test "$(readlink "$current")" = runs/test
  test "$(cat "$current/manifest.json")" = new
  rm -rf "$test_root"
  echo 'Analytics tracker publication success and failure seams passed'
  exit 0
fi
if [ "${RUSTZEN_ANALYTICS_TRACKER_TEST_SETUP_FAILURE:-}" = 1 ]; then
  test_root=$(mktemp -d "${TMPDIR:-/tmp}/rz-analytics-setup.XXXXXX")
  evidence_root="$test_root"
  current="$evidence_root/current"
  lock_dir="$evidence_root/.verify.lock"
  candidate="$evidence_root/.candidate-test"
  staged_bin_dir="$evidence_root/.binaries-test"
  status=1
  cleanup() {
    result=$?
    trap - EXIT INT TERM
    rm -rf "$staged_bin_dir" "$candidate" "$lock_dir"
    test "$result" -ne 0
    test "$(readlink "$current")" = runs/old
    test ! -e "$lock_dir"
    rm -rf "$test_root"
    echo 'Analytics tracker setup failure seam passed' >&2
    exit "$result"
  }
  mkdir -p "$evidence_root/runs/old"
  printf old >"$evidence_root/runs/old/manifest.json"
  ln -s runs/old "$current"
  mkdir "$lock_dir"
  trap cleanup EXIT
  trap on_interrupt INT
  trap on_terminate TERM
  printf blocker >"$evidence_root/blocker"
  candidate="$evidence_root/blocker/candidate"
  mkdir "$candidate"
  exit 1
fi
if [ "${RUSTZEN_ANALYTICS_TRACKER_TEST_SIGNAL:-}" = INT ] || [ "${RUSTZEN_ANALYTICS_TRACKER_TEST_SIGNAL:-}" = TERM ]; then
  test_signal=${RUSTZEN_ANALYTICS_TRACKER_TEST_SIGNAL}
  test_root=$(mktemp -d "${TMPDIR:-/tmp}/rz-analytics-signal.XXXXXX")
  evidence_root="$test_root"
  current="$evidence_root/current"
  lock_dir="$evidence_root/.verify.lock"
  candidate="$evidence_root/.candidate-test"
  staged_bin_dir="$evidence_root/.binaries-test"
  status=1
  cleanup() {
    result=$?
    trap - EXIT INT TERM
    rm -rf "$staged_bin_dir" "$candidate" "$lock_dir"
    test "$result" -eq "$([ "$test_signal" = INT ] && echo 130 || echo 143)"
    test "$(readlink "$current")" = runs/old
    test ! -e "$lock_dir"
    test ! -e "$candidate"
    test ! -e "$staged_bin_dir"
    rm -rf "$test_root"
    echo "Analytics tracker $test_signal signal seam passed" >&2
    exit "$result"
  }
  mkdir -p "$evidence_root/runs/old" "$candidate" "$staged_bin_dir"
  printf old >"$evidence_root/runs/old/manifest.json"
  ln -s runs/old "$current"
  mkdir "$lock_dir"
  trap cleanup EXIT
  trap on_interrupt INT
  trap on_terminate TERM
  kill -"$test_signal" "$$"
fi
if [ -z "$architecture" ]; then
  architecture=$(run_bounded_capture "$info_timeout" "$docker_bin" info --format '{{.Architecture}}') || {
    echo 'Docker architecture discovery failed or exceeded its timeout' >&2
    exit 1
  }
fi
case "$architecture" in
  aarch64) platform=linux/arm64; file_pattern='ELF 64-bit.*ARM aarch64' ;;
  x86_64) platform=linux/amd64; file_pattern='ELF 64-bit.*x86-64' ;;
  *) echo "unsupported Colima/Docker architecture: $architecture" >&2; exit 1 ;;
esac
read -r verifier_image verifier_key verifier_provenance_sha < <("$root/scripts/ensure-admin-browser-verifier-image.sh" --platform "$platform")
bin_dir=${RUSTZEN_UI_LINUX_BIN_DIR:-"$root/target/rz/build/$architecture/bin"}
for name in rz-admin rz-monitor rz-insights rz-reports; do
  test -x "$bin_dir/$name" || { echo "missing Linux binary: $bin_dir/$name" >&2; exit 1; }
  file "$bin_dir/$name" | grep -q "$file_pattern"
done
read -r head source_tree_state source_tree_sha256 < <("$root/scripts/admin-browser-source-identity.sh")
test -s "$bin_dir/build-provenance.txt"
lock_dir="$evidence_root/.verify.lock"
run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$"
candidate="$evidence_root/.candidate-$run_id"
staged_bin_dir="$evidence_root/.binaries-$run_id"
container="rz-analytics-tracker-$run_id"
publish_candidate() {
  run_dir="$evidence_root/runs/$run_id"
  current_tmp="$evidence_root/.current-$run_id"
  mv "$candidate" "$run_dir" || return 1
  ln -s "runs/$run_id" "$current_tmp" || return 1
  atomic_replace_symlink "$current_tmp" "$current" || return 1
}
status=1
cleanup() {
  status=$?
  trap - EXIT INT TERM
  if [ -d "$candidate" ]; then
    "$docker_bin" logs "$container" >"$candidate/container.log" 2>&1 || true
  fi
  "$docker_bin" rm -f "$container" >/dev/null 2>&1 || true
  rm -rf "$staged_bin_dir"
  if [ "$status" -ne 0 ]; then rm -rf "$candidate"; fi
  rm -rf "$lock_dir"
  exit "$status"
}
mkdir -p "$evidence_root"
if ! mkdir "$lock_dir" 2>/dev/null; then echo "another Analytics tracker verification owns $lock_dir" >&2; exit 1; fi
trap cleanup EXIT
trap on_interrupt INT
trap on_terminate TERM
mkdir -p "$evidence_root/runs"
rm -rf "$candidate" "$staged_bin_dir"
mkdir -p "$candidate" "$staged_bin_dir"
expected_provenance="$candidate/expected-build-provenance.txt"
{
  printf 'schemaVersion\t1\n'
  printf 'gitHead\t%s\n' "$head"
  printf 'sourceTreeState\t%s\n' "$source_tree_state"
  printf 'sourceTreeSha256\t%s\n' "$source_tree_sha256"
  printf 'architecture\t%s\n' "$architecture"
  printf 'targetTriple\t%s\n' "$([ "$architecture" = aarch64 ] && printf aarch64-unknown-linux-musl || printf x86_64-unknown-linux-musl)"
  printf 'platform\t%s\n' "$platform"
  printf 'distribution\tfull\n'
  for name in rz-admin rz-monitor rz-insights rz-reports; do
    printf '%s\t%s\n' "$name" "$(shasum -a 256 "$bin_dir/$name" | awk '{print $1}')"
  done
} >"$expected_provenance"
cmp -s "$expected_provenance" "$bin_dir/build-provenance.txt" || {
  echo "Linux binaries do not match the current source-tree build provenance" >&2
  exit 1
}
mv "$expected_provenance" "$candidate/build-provenance.txt"
for name in rz-admin rz-monitor rz-insights rz-reports; do
  cp "$bin_dir/$name" "$staged_bin_dir/$name"
  expected_hash=$(awk -F '\t' -v name="$name" '$1 == name { print $2 }' "$candidate/build-provenance.txt")
  test "$(shasum -a 256 "$staged_bin_dir/$name" | awk '{print $1}')" = "$expected_hash"
done
cp -R "$root/scripts/fixtures/analytics-tracker" "$candidate/fixture"
run_bounded "$run_timeout" "$docker_bin" run --name "$container" --platform "$platform" --security-opt seccomp=unconfined \
  --env RUSTZEN_ANALYTICS_TRACKER_INNER=1 --env RUSTZEN_VERIFY_HEAD="$head" \
  --env RUSTZEN_VERIFY_SOURCE_TREE_STATE="$source_tree_state" --env RUSTZEN_VERIFY_SOURCE_TREE_SHA256="$source_tree_sha256" \
  --env RUSTZEN_VERIFY_ARCHITECTURE="$architecture" --env RUSTZEN_VERIFY_CHROMIUM_VERSION=120.0.6099.224-1~deb11u1 \
  --env RUSTZEN_VERIFY_VERIFIER_IMAGE_ID="$verifier_image" --env RUSTZEN_VERIFY_VERIFIER_KEY="$verifier_key" \
  --env RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256="$verifier_provenance_sha" \
  --mount "type=bind,src=$staged_bin_dir,dst=/verify/bin,readonly" \
  --mount "type=bind,src=$candidate,dst=/verify/evidence" \
  --mount "type=bind,src=$root/scripts/fixtures/analytics-tracker,dst=/verify/fixture,readonly" \
  --mount "type=bind,src=$root/scripts/verify-analytics-tracker-linux.sh,dst=/verify/run.sh,readonly" \
  "$verifier_image" bash /verify/run.sh
test -s "$candidate/manifest.json"
jq -e '.schemaVersion == 1 and .status == "partial" and .sourceTreeSha256 == $sha and .http["413"].rowDelta == 0 and .http["429"].rowDelta == 0 and .http["507"].status == "not-verified" and (.lines.verifier > 0) and (.lines.fixture > 0)' --arg sha "$source_tree_sha256" "$candidate/manifest.json" >/dev/null
read -r final_head final_source_tree_state final_source_tree_sha256 < <("$root/scripts/admin-browser-source-identity.sh")
test "$final_head" = "$head"
test "$final_source_tree_state" = "$source_tree_state"
test "$final_source_tree_sha256" = "$source_tree_sha256"
"$docker_bin" rm "$container" >/dev/null
rm -rf "$staged_bin_dir"
publish_candidate
status=0
rm -rf "$lock_dir"
trap - EXIT INT TERM
echo "Analytics tracker Linux evidence published: $current/manifest.json"
