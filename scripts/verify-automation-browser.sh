#!/usr/bin/env bash
set -euo pipefail

REPORTS=${1:-target/debug/rz-reports}
BROWSER=${2:?Usage: scripts/verify-automation-browser.sh [reports-binary] BROWSER_PATH}
ROOT=$(mktemp -d "${TMPDIR:-/tmp}/rustzen-automation.XXXXXX")
LOG="$ROOT/reports.log"
export RUSTZEN_ENV="${RUSTZEN_ENV:-development}"
export RUSTZEN_RUNTIME_ROOT="$ROOT"
export RUSTZEN_INTERNAL_HOST=127.0.0.1
reserve_port() {
  bun -e 'const server = Bun.serve({ hostname: "127.0.0.1", port: 0, fetch: () => new Response() }); console.log(server.port); server.stop();'
}
export RUSTZEN_REPORTS_PORT="$(reserve_port)"
export RUSTZEN_AUTOMATION_FIXTURE_PORT="$(reserve_port)"
if [ "$RUSTZEN_REPORTS_PORT" = "$RUSTZEN_AUTOMATION_FIXTURE_PORT" ]; then
  echo "dynamic verifier ports collided" >&2
  exit 1
fi
export RUSTZEN_IPC_TOKEN=automation-browser-verification-token
export RUSTZEN_REPORTS_CREDENTIAL_KEY=automation-browser-credential-key
export RUSTZEN_REPORTS_BROWSER_PATH="$BROWSER"
export RUSTZEN_REPORTS_MAX_CONCURRENCY=2

cleanup() {
  local status=$?
  if [[ -n "${PID:-}" ]] && kill -0 "$PID" 2>/dev/null; then
    kill -TERM "$PID" 2>/dev/null || true
    for _ in $(seq 1 50); do kill -0 "$PID" 2>/dev/null || break; sleep 0.1; done
    kill -0 "$PID" 2>/dev/null && kill -KILL "$PID" 2>/dev/null || true
    wait "$PID" 2>/dev/null || true
  fi
  if pgrep -f -- "$ROOT" >/dev/null 2>&1; then
    pkill -TERM -f -- "$ROOT" 2>/dev/null || true
    for _ in $(seq 1 50); do pgrep -f -- "$ROOT" >/dev/null 2>&1 || break; sleep 0.1; done
    if pgrep -f -- "$ROOT" >/dev/null 2>&1; then
      pkill -KILL -f -- "$ROOT" 2>/dev/null || true
      for _ in $(seq 1 50); do pgrep -f -- "$ROOT" >/dev/null 2>&1 || break; sleep 0.1; done
    fi
    if pgrep -f -- "$ROOT" >/dev/null 2>&1; then
      echo "automation verifier leaked process for $ROOT" >&2
      return 1
    fi
  fi
  rm -rf "$ROOT"
  return "$status"
}
trap cleanup EXIT

"$REPORTS" serve >"$LOG" 2>&1 &
PID=$!
for _ in $(seq 1 100); do
  kill -0 "$PID" 2>/dev/null || { cat "$LOG"; exit 1; }
  if curl --silent --fail "http://127.0.0.1:$RUSTZEN_REPORTS_PORT/health" >/dev/null; then break; fi
  sleep 0.05
done
curl --silent --fail "http://127.0.0.1:$RUSTZEN_REPORTS_PORT/health" >/dev/null || { cat "$LOG"; exit 1; }
kill -0 "$PID" 2>/dev/null || { cat "$LOG"; exit 1; }
REPORTS_DB="$RUSTZEN_RUNTIME_ROOT/data/reports/db/reports.db"
[ -f "$REPORTS_DB" ] || { echo "Reports database was not created: $REPORTS_DB" >&2; cat "$LOG"; exit 1; }
if ! bun scripts/verify-automation-browser.mjs; then
  cat "$LOG"
  exit 1
fi
