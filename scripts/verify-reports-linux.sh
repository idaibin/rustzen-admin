#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
binary=${1:-"$root/target/rz/build/x86_64/bin/rz-reports"}
test -x "$binary"
name="rz-reports-linux-$$"
cleanup() {
  status=$?
  docker exec "$name" sh -c 'cat /tmp/reports-service.log 2>/dev/null || true' 2>&1 || true
  docker logs "$name" 2>&1 || true
  docker inspect "$name" --format 'exit={{.State.ExitCode}} status={{.State.Status}}' 2>/dev/null || true
  if [ "${RZ_REPORTS_LINUX_KEEP_CONTAINER:-}" != 1 ]; then
    docker rm -f "$name" >/dev/null 2>&1 || true
  fi
  exit "$status"
}
trap cleanup EXIT
docker run --name "$name" --platform linux/amd64 --security-opt seccomp=unconfined -v "$binary:/tmp/rz-reports:ro" debian:bookworm-slim bash -ec '
  trap "status=\$?; printf \"reports Linux verifier failed line %s status %s: %s\\n\" \"\$LINENO\" \"\$status\" \"\${BASH_COMMAND}\" >&2; cat /tmp/reports-service.log 2>/dev/null || true; ps -ef || true; ls -ld /opt/rz /opt/rz/data /opt/rz/data/reports /opt/rz/data/reports/db || true" ERR
  apt-get update >/dev/null
  apt-get install -y --no-install-recommends chromium chromium-sandbox sqlite3 curl jq openssl busybox procps util-linux >/dev/null
  groupadd --system rz-reports; useradd --system --gid rz-reports --home-dir /opt/rz/data/reports --shell /usr/sbin/nologin rz-reports
  umask 077; mkdir -p /opt/rz/releases/current/bin /opt/rz/data/reports/db /opt/rz/data/reports/.config /opt/rz/data/reports/.cache /opt/rz/logs/reports /srv; chmod 0755 /opt/rz /opt/rz/releases /opt/rz/releases/current /opt/rz/releases/current/bin; chmod 0711 /opt/rz/data /opt/rz/logs; chown -R rz-reports:rz-reports /opt/rz/data/reports /opt/rz/logs/reports
  install -m 0755 /tmp/rz-reports /opt/rz/releases/current/bin/rz-reports; ln -s releases/current /opt/rz/current
  printf "<main>reports fixture</main>" >/srv/index.html
  busybox httpd -f -p 18080 -h /srv &
  touch /opt/rz/data/recovery-blocked
  ! setpriv --reuid=rz-reports --regid=rz-reports --init-groups --no-new-privs -- test ! -e /opt/rz/data/recovery-blocked
  rm /opt/rz/data/recovery-blocked
  setpriv --reuid=rz-reports --regid=rz-reports --init-groups --no-new-privs -- unshare -Ur true
  setpriv --reuid=rz-reports --regid=rz-reports --init-groups --no-new-privs -- env HOME=/opt/rz/data/reports XDG_CONFIG_HOME=/opt/rz/data/reports/.config XDG_CACHE_HOME=/opt/rz/data/reports/.cache RUSTZEN_ENV=production RUSTZEN_RUNTIME_ROOT=/opt/rz RUSTZEN_IPC_TOKEN=linux-ipc RUSTZEN_REPORTS_CREDENTIAL_KEY=linux-key RUSTZEN_REPORTS_BROWSER_PATH=/usr/bin/chromium RUSTZEN_REPORTS_PORT=19804 /opt/rz/current/bin/rz-reports serve >/tmp/reports-service.log 2>&1 & pid=$!
  for n in $(seq 1 100); do curl -fsS http://127.0.0.1:19804/health >/dev/null && break; sleep .1; done
  curl -fsS http://127.0.0.1:19804/health >/dev/null
  test "$(awk "/^Uid:/{print \$2}" /proc/$pid/status)" = "$(id -u rz-reports)"
  grep -q "NoNewPrivs:[[:space:]]*1" /proc/$pid/status
  signed() {
    method=$1 path=$2 capability=$3 body=${4:-}
    timestamp=$(date +%s); request_id=$(cat /proc/sys/kernel/random/uuid)
    payload=$(printf "1\\n%s\\n%s\\n1\\nreports\\n%s\\n%s\\n%s" "$timestamp" "$request_id" "$method" "$path" "$capability")
    signature=$(printf "%s" "$payload" | openssl dgst -sha256 -hmac linux-ipc -hex | sed "s/^.* //")
    common_args=( -fsS -X "$method" "http://127.0.0.1:19804$path" \
      -H "content-type: application/json" -H "x-rustzen-contract-version: 1" \
      -H "x-rustzen-ipc-timestamp: $timestamp" -H "x-rustzen-request-id: $request_id" \
      -H "x-rustzen-user-id: 1" -H "x-rustzen-module: reports" \
      -H "x-rustzen-ipc-capability: $capability" -H "x-rustzen-ipc-signature: $signature" )
    if [ -n "$body" ]; then
      curl "${common_args[@]}" --data-raw "$body"
    else
      curl "${common_args[@]}"
    fi
  }
  system=$(signed POST /api/reports/systems reports:system:manage "{\"name\":\"Linux fixture\",\"baseUrl\":\"http://127.0.0.1:18080\"}" | jq -r .data.id)
  flow=$(signed POST /api/reports/flows reports:flow:manage "{\"systemId\":\"$system\",\"name\":\"Sandbox browser\",\"steps\":[{\"action\":\"goto\",\"url\":\"/\"},{\"action\":\"pause\",\"durationMs\":15000},{\"action\":\"screenshot\",\"name\":\"sandbox\"}]}" | jq -r .data.id)
  run=$(signed POST /api/reports/runs reports:run:manage "{\"flowId\":\"$flow\",\"input\":{}}" | jq -r .data.id)
  reports_uid=$(id -u rz-reports)
  find_browser() {
    ps -eo uid=,pid=,args= | awk -v reports_uid="$reports_uid" "\$1 == reports_uid && /--user-data-dir=/ && /browser-/ { print \$2; exit }"
  }
  browser_pid=""
  for n in $(seq 1 240); do
    kill -0 "$pid"
    status=$(signed GET "/api/reports/runs/$run" reports:run:view | jq -r .data.status)
    case "$status" in
      queued|running|cancelling) ;;
      *) break ;;
    esac
    browser_pid=$(find_browser || true)
    test -n "$browser_pid" && break
    sleep .25
  done
  if [ -z "$browser_pid" ]; then
    signed GET "/api/reports/runs/$run" reports:run:view || true
    signed GET "/api/reports/runs/$run/steps" reports:run:view || true
    ps -eo uid,pid,args | grep -E "chromium|browser-" || true
    exit 1
  fi
  test "$(awk "/^Uid:/{print \$2}" /proc/$browser_pid/status)" = "$reports_uid"
  browser_seccomp=$(awk "/^Seccomp:/{print \$2}" /proc/$browser_pid/status)
  case "$browser_seccomp" in
    2) seccomp_evidence="browser seccomp=2" ;;
    0) seccomp_evidence="browser seccomp not verified because the Colima amd64 container uses an unconfined outer profile" ;;
    *) echo "unexpected browser Seccomp value: $browser_seccomp" >&2; exit 1 ;;
  esac
  browser_cmdline=$(cat /proc/$browser_pid/cmdline 2>/dev/null | tr "\0" " " || true)
  test -n "$browser_cmdline"
  ! printf "%s\n" "$browser_cmdline" | grep -q -- "--no-sandbox"
  result=""
  for n in $(seq 1 1200); do
    kill -0 "$pid"
    result=$(signed GET "/api/reports/runs/$run" reports:run:view | jq -r .data.status)
    case "$result" in
      succeeded) break ;;
      queued|running|cancelling) sleep .1 ;;
      *)
        signed GET "/api/reports/runs/$run" reports:run:view || true
        signed GET "/api/reports/runs/$run/steps" reports:run:view || true
        ps -ef | grep chromium || true
        exit 1 ;;
    esac
  done
  if [ "$result" != succeeded ]; then
    signed GET "/api/reports/runs/$run" reports:run:view || true
    signed GET "/api/reports/runs/$run/steps" reports:run:view || true
    ps -ef | grep chromium || true
    exit 1
  fi
  signed GET "/api/reports/runs/$run/artifacts" reports:run:view | jq -e ".data[] | select(.kind == \"screenshot\")" >/dev/null
  setpriv --reuid=rz-reports --regid=rz-reports --init-groups -- sqlite3 /opt/rz/data/reports/db/reports.db "PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS verify_gate(value); INSERT INTO verify_gate VALUES (1);" >/dev/null
  test -f /opt/rz/data/reports/db/reports.db-wal; test -f /opt/rz/data/reports/db/reports.db-shm
  find /opt/rz/logs/reports -type f -name "reports*" -user rz-reports | grep -q .
  kill -TERM $pid; wait $pid || true
  ! pgrep -x chromium >/dev/null 2>&1
  ! find /opt/rz/data/reports -type d -name "browser-*" -print -quit | grep -q .
  chromium_version=$(/usr/bin/chromium --version)
  echo "reports Linux delegated browser/userns/WAL/recovery/log gate passed ($chromium_version; $seccomp_evidence); real systemd unit is not verified"
'
