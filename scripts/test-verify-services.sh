#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
VERIFY="${VERIFY_SERVICES_SCRIPT:-$ROOT/scripts/verify-services.sh}"
JUSTFILE="${JUSTFILE_PATH:-$ROOT/justfile}"
MODULE_LOG_HELPER="${VERIFY_SERVICES_MODULE_LOG_HELPER:-$ROOT/scripts/verify-module-log-diagnostics.sh}"
DATABASE_ISOLATION_HELPER="${VERIFY_SERVICES_DATABASE_ISOLATION_HELPER:-$ROOT/scripts/verify-database-isolation.sh}"
LIFECYCLE_HELPER="${VERIFY_SERVICES_LIFECYCLE_HELPER:-$ROOT/scripts/verify-service-lifecycle.sh}"

bash -n "$VERIFY"
sh -n "$MODULE_LOG_HELPER"
sh -n "$DATABASE_ISOLATION_HELPER"
sh -n "$LIFECYCLE_HELPER"

grep -Fqx 'AGENT="${6:-target/release/rz-monitor-agent}"' "$VERIFY"
grep -Fqx 'AGENT="$(absolute_binary "$AGENT")"' "$VERIFY"
grep -Fqx 'for binary in "$ADMIN" "$MONITOR" "$INSIGHTS" "$REPORTS" "$CLI" "$AGENT"; do' "$VERIFY"
grep -Fqx 'export RUSTZEN_MONITOR_NODE_ID=verify-monitor-node' "$VERIFY"
grep -Fqx 'export RUSTZEN_REPORTS_SQLITE_PATH=./data/reports/db/reports.db' "$VERIFY"
grep -Fqx 'LIFECYCLE_HELPER="$PROJECT_ROOT/scripts/verify-service-lifecycle.sh"' "$VERIFY"
grep -Fqx 'trap initial_cleanup EXIT INT TERM' "$VERIFY"
grep -Fqx 'if ! sh -n "$LIFECYCLE_HELPER"; then' "$VERIFY"
grep -Fqx 'if ! . "$LIFECYCLE_HELPER"; then' "$VERIFY"
grep -Fqx 'if [ ! -f "$LIFECYCLE_HELPER" ] || [ -L "$LIFECYCLE_HELPER" ]; then' "$VERIFY"
grep -Fqx '    rm -rf "$ROOT"' "$VERIFY"
grep -Fqx 'if ! sh -n "$LIFECYCLE_HELPER"; then' "$VERIFY"
grep -Fqx 'if ! . "$LIFECYCLE_HELPER"; then' "$VERIFY"
grep -Fqx 'trap cleanup EXIT INT TERM' "$VERIFY"
if grep -Fq 'start_service() {' "$VERIFY"; then
    echo "verify-services must source lifecycle helpers instead of defining them inline" >&2
    exit 1
fi
grep -Fqx 'start_service() {' "$LIFECYCLE_HELPER"
grep -Fqx '        admin) "$ADMIN" serve >"$log" 2>&1 & ;;' "$LIFECYCLE_HELPER"
grep -Fqx '        monitor) "$MONITOR" controller >"$log" 2>&1 & ;;' "$LIFECYCLE_HELPER"
grep -Fqx '        insights) "$INSIGHTS" serve >"$log" 2>&1 & ;;' "$LIFECYCLE_HELPER"
grep -Fqx '        reports) "$REPORTS" serve >"$log" 2>&1 & ;;' "$LIFECYCLE_HELPER"
grep -Fqx '        monitor_agent) "$AGENT" >"$log" 2>&1 & ;;' "$LIFECYCLE_HELPER"
grep -Fqx '    for name in monitor_agent admin reports insights monitor; do' "$LIFECYCLE_HELPER"
grep -Fqx '        while kill -0 "$pid" 2>/dev/null && [ "$count" -lt 50 ]; do' "$LIFECYCLE_HELPER"
grep -Fqx '            sleep 0.1' "$LIFECYCLE_HELPER"
grep -Fqx '            kill -KILL "$pid" 2>/dev/null || true' "$LIFECYCLE_HELPER"
grep -Fqx '    while [ "$count" -lt 180 ]; do' "$LIFECYCLE_HELPER"
grep -Fqx 'MODULE_LOG_HELPER="$PROJECT_ROOT/scripts/verify-module-log-diagnostics.sh"' "$VERIFY"
grep -Fqx 'if [ ! -f "$MODULE_LOG_HELPER" ] || [ -L "$MODULE_LOG_HELPER" ]; then' "$VERIFY"
grep -Fqx '. "$MODULE_LOG_HELPER"' "$VERIFY"
grep -Fqx 'DATABASE_ISOLATION_HELPER="$PROJECT_ROOT/scripts/verify-database-isolation.sh"' "$VERIFY"
grep -Fqx 'if [ ! -f "$DATABASE_ISOLATION_HELPER" ] || [ -L "$DATABASE_ISOLATION_HELPER" ]; then' "$VERIFY"
grep -Fqx '. "$DATABASE_ISOLATION_HELPER"' "$VERIFY"
if grep -Fq 'verify_database_isolation() {' "$VERIFY"; then
    echo "verify-services must source database isolation instead of defining it inline" >&2
    exit 1
fi
grep -Fqx 'expect_corrupt_start_failure() {' "$DATABASE_ISOLATION_HELPER"
grep -Fqx 'verify_database_isolation() {' "$DATABASE_ISOLATION_HELPER"
grep -Fqx 'verify_database_isolations() {' "$DATABASE_ISOLATION_HELPER"
grep -Fqx '        reports) path="$ROOT/data/reports/db/reports.db" ;;' "$DATABASE_ISOLATION_HELPER"
grep -Fqx '        *) path="$ROOT/data/db/$database.db" ;;' "$DATABASE_ISOLATION_HELPER"
grep -Fqx '        wait_for_module_state monitor true true' "$DATABASE_ISOLATION_HELPER"
grep -Fqx '        wait_for_module_state insights true true' "$DATABASE_ISOLATION_HELPER"
grep -Fqx '        wait_for_module_state reports true true' "$DATABASE_ISOLATION_HELPER"
grep -Fqx '    for database in admin monitor insights; do' "$DATABASE_ISOLATION_HELPER"
grep -Fqx '        [ -s "$ROOT/data/db/$database.db" ] || {' "$DATABASE_ISOLATION_HELPER"
grep -Fqx '    verify_database_isolation monitor monitor' "$DATABASE_ISOLATION_HELPER"
grep -Fqx '    verify_database_isolation insights insights' "$DATABASE_ISOLATION_HELPER"
grep -Fqx '    verify_database_isolation reports reports' "$DATABASE_ISOLATION_HELPER"
grep -Fqx '    verify_database_isolation admin admin' "$DATABASE_ISOLATION_HELPER"
grep -Fqx '    [ -s "$ROOT/data/reports/db/reports.db" ] || {' "$DATABASE_ISOLATION_HELPER"
trap_line="$(grep -nF 'trap cleanup EXIT INT TERM' "$VERIFY" | cut -d: -f1)"
database_source_line="$(grep -nF '. "$DATABASE_ISOLATION_HELPER"' "$VERIFY" | cut -d: -f1)"
database_call_line="$(grep -nF 'verify_database_isolations' "$VERIFY" | tail -n1 | cut -d: -f1)"
[[ "$trap_line" -lt "$database_source_line" && "$database_source_line" -lt "$database_call_line" ]]
if grep -Fq 'verify_module_log_diagnostics() {' "$VERIFY"; then
    echo "verify-services must source module-log diagnostics instead of defining it inline" >&2
    exit 1
fi
grep -Fqx 'verify_module_log_diagnostics() {' "$MODULE_LOG_HELPER"
grep -Fqx '    wait_for_status 403 "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/status/module-logs?module=admin&date=$module_log_date" "$denied_token"' "$MODULE_LOG_HELPER"
grep -Fqx '        const expectedLines = 24_000;' "$MODULE_LOG_HELPER"
grep -Fqx '            if (!tail.truncated || tail.nextCursor === cursor) {' "$MODULE_LOG_HELPER"
grep -Fqx '        if (reconstructed.length !== expectedLines || reconstructed.some((id, index) => id !== index)) {' "$MODULE_LOG_HELPER"
grep -Fqx '        if (!/^attachment;\s*filename=rustzen-module-logs\.tar$/i.test(value("content-disposition") ?? "")) throw new Error("missing backup filename header");' "$MODULE_LOG_HELPER"
grep -Fqx '        if (value("x-rustzen-archive-file-count") !== "1") throw new Error("backup count header did not match one selected file");' "$MODULE_LOG_HELPER"
grep -Fqx '    [ "$repeated_status" = 400 ] || { echo "verify-services: reused module-log cleanup token returned $repeated_status" >&2; exit 1; }' "$MODULE_LOG_HELPER"
source_line="$(grep -nF '. "$MODULE_LOG_HELPER"' "$VERIFY" | cut -d: -f1)"
call_line="$(grep -nF 'verify_module_log_diagnostics' "$VERIFY" | tail -n1 | cut -d: -f1)"
[[ "$trap_line" -lt "$source_line" && "$source_line" -lt "$call_line" ]]
grep -Fqx '        monitor_agent) "$AGENT" >"$log" 2>&1 & ;;' "$LIFECYCLE_HELPER"
if grep -Fq 'monitor_agent) "$MONITOR" agent' "$VERIFY"; then
    echo "verify-services must start the independent rz-monitor-agent binary" >&2
    exit 1
fi

grep -Fqx '    cargo build --release -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent' "$JUSTFILE"
grep -Fqx '    cargo build -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent' "$JUSTFILE"
grep -Fqx '    RUSTZEN_VERIFY_BUILD_PROFILE=release scripts/verify-services.sh target/release/rz-admin target/release/rz-monitor target/release/rz-insights target/release/rz-reports target/release/rz target/release/rz-monitor-agent' "$JUSTFILE"
grep -Fqx '    RUSTZEN_VERIFY_BUILD_PROFILE=debug scripts/verify-services.sh target/debug/rz-admin target/debug/rz-monitor target/debug/rz-insights target/debug/rz-reports target/debug/rz target/debug/rz-monitor-agent' "$JUSTFILE"

echo "verify-services independent Monitor Agent wiring passed"
