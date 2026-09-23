#!/usr/bin/env sh
set -eu

PROJECT_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
ADMIN="${1:-target/release/rz-admin}"
MONITOR="${2:-target/release/rz-monitor}"
INSIGHTS="${3:-target/release/rz-insights}"
REPORTS="${4:-target/release/rz-reports}"
CLI="${5:-target/release/rz}"
AGENT="${6:-target/release/rz-monitor-agent}"

absolute_binary() {
    case "$1" in
        /*) printf '%s\n' "$1" ;;
        *) printf '%s/%s\n' "$PROJECT_ROOT" "$1" ;;
    esac
}

ADMIN="$(absolute_binary "$ADMIN")"
MONITOR="$(absolute_binary "$MONITOR")"
INSIGHTS="$(absolute_binary "$INSIGHTS")"
REPORTS="$(absolute_binary "$REPORTS")"
CLI="$(absolute_binary "$CLI")"
AGENT="$(absolute_binary "$AGENT")"

for binary in "$ADMIN" "$MONITOR" "$INSIGHTS" "$REPORTS" "$CLI" "$AGENT"; do
    if [ ! -x "$binary" ]; then
        echo "verify-services: missing executable: $binary" >&2
        exit 1
    fi
done

RUSTZEN_VERIFY_BUILD_PROFILE="${RUSTZEN_VERIFY_BUILD_PROFILE:-release}"
case "$RUSTZEN_VERIFY_BUILD_PROFILE" in
    debug) latency_output_default="$PROJECT_ROOT/target/rz/gateway-latency-debug.json" ;;
    release) latency_output_default="$PROJECT_ROOT/target/rz/gateway-latency.json" ;;
    *) echo "verify-services: RUSTZEN_VERIFY_BUILD_PROFILE must be debug or release" >&2; exit 1 ;;
esac
export RUSTZEN_VERIFY_BUILD_PROFILE
export RUSTZEN_GATEWAY_LATENCY_OUTPUT="${RUSTZEN_GATEWAY_LATENCY_OUTPUT:-$latency_output_default}"

ROOT="$(mktemp -d "${TMPDIR:-/tmp}/rz-services.XXXXXX")"
initial_cleanup() {
    rm -rf "$ROOT"
}
trap initial_cleanup EXIT INT TERM
mkdir -p "$ROOT/logs" "$ROOT/pids" "$ROOT/backups" "$PROJECT_ROOT/target/rz"
PHASE="startup"
BASE_PORT="${RUSTZEN_VERIFY_BASE_PORT:-19801}"

export RUSTZEN_RUNTIME_ROOT="$ROOT"
export RUSTZEN_ENV=development
export RUSTZEN_ADMIN_SQLITE_PATH=./data/db/admin.db
export RUSTZEN_MONITOR_SQLITE_PATH=./data/db/monitor.db
export RUSTZEN_INSIGHTS_SQLITE_PATH=./data/db/insights.db
export RUSTZEN_REPORTS_SQLITE_PATH=./data/reports/db/reports.db
# The schedule fixture computes its daily/weekly slots in UTC. Keep this
# disposable verifier explicit rather than coupling it to an installation's
# product timezone.
export RUSTZEN_TIMEZONE=UTC
export RUSTZEN_ADMIN_HOST=127.0.0.1
export RUSTZEN_ADMIN_PORT="$BASE_PORT"
export RUSTZEN_INTERNAL_HOST=127.0.0.1
export RUSTZEN_MONITOR_PORT=$((BASE_PORT + 1))
export RUSTZEN_INSIGHTS_PORT=$((BASE_PORT + 2))
export RUSTZEN_REPORTS_PORT=$((BASE_PORT + 3))
export RUSTZEN_JWT_SECRET=local-service-verification-jwt-secret
export RUSTZEN_IPC_TOKEN=local-service-verification-ipc-secret
export RUSTZEN_MONITOR_AGENT_TOKEN=local-service-verification-agent-secret
export RUSTZEN_MONITOR_NODE_ID=verify-monitor-node
export RUSTZEN_MONITOR_CONTROLLER_URL="http://127.0.0.1:$RUSTZEN_ADMIN_PORT"
export RUSTZEN_BUILD_ID=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
export RUSTZEN_COMPOSITION_ID=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
export RUST_LOG=warn

run_bun() {
    pnpm dlx bun@1.3.14 "$@"
}

LIFECYCLE_HELPER="$PROJECT_ROOT/scripts/verify-service-lifecycle.sh"
if [ ! -f "$LIFECYCLE_HELPER" ] || [ -L "$LIFECYCLE_HELPER" ]; then
    rm -rf "$ROOT"
    echo "verify-services: missing regular lifecycle helper: $LIFECYCLE_HELPER" >&2
    exit 1
fi
if ! sh -n "$LIFECYCLE_HELPER"; then
    rm -rf "$ROOT"
    echo "verify-services: invalid lifecycle helper: $LIFECYCLE_HELPER" >&2
    exit 1
fi
if ! . "$LIFECYCLE_HELPER"; then
    rm -rf "$ROOT"
    echo "verify-services: failed to source lifecycle helper: $LIFECYCLE_HELPER" >&2
    exit 1
fi
trap cleanup EXIT INT TERM

AUTH_MODULE_GATEWAY_HELPER="$PROJECT_ROOT/scripts/verify-service-auth-module-gateway.sh"
if [ ! -f "$AUTH_MODULE_GATEWAY_HELPER" ] || [ -L "$AUTH_MODULE_GATEWAY_HELPER" ]; then
    echo "verify-services: missing regular auth/module-gateway helper: $AUTH_MODULE_GATEWAY_HELPER" >&2
    exit 1
fi
if ! . "$AUTH_MODULE_GATEWAY_HELPER"; then
    echo "verify-services: failed to source auth/module-gateway helper: $AUTH_MODULE_GATEWAY_HELPER" >&2
    exit 1
fi

MODULE_LOG_HELPER="$PROJECT_ROOT/scripts/verify-module-log-diagnostics.sh"
if [ ! -f "$MODULE_LOG_HELPER" ] || [ -L "$MODULE_LOG_HELPER" ]; then
    echo "verify-services: missing regular module-log helper: $MODULE_LOG_HELPER" >&2
    exit 1
fi
. "$MODULE_LOG_HELPER"

DATABASE_ISOLATION_HELPER="$PROJECT_ROOT/scripts/verify-database-isolation.sh"
if [ ! -f "$DATABASE_ISOLATION_HELPER" ] || [ -L "$DATABASE_ISOLATION_HELPER" ]; then
    echo "verify-services: missing regular database-isolation helper: $DATABASE_ISOLATION_HELPER" >&2
    exit 1
fi
. "$DATABASE_ISOLATION_HELPER"

PHASE="admin-alone"
start_service admin
wait_for_health admin
owner_login="$(login owner 'rustzen@123')"
RUSTZEN_ADMIN_TOKEN="$(printf '%s' "$owner_login" | parse_json 'value.data.token')"
RUSTZEN_ADMIN_USER_ID="$(printf '%s' "$owner_login" | parse_json 'value.data.userInfo.id')"
export RUSTZEN_ADMIN_TOKEN RUSTZEN_ADMIN_USER_ID
for module in monitor insights reports; do
    wait_for_module_state "$module" false false
    assert_gateway_unavailable "$module"
done
stop_all

order_index=0
while IFS= read -r order; do
    order_index=$((order_index + 1))
    PHASE="startup-order-$order_index"
    for service in $order; do
        start_service "$service"
        sleep 0.05
    done
    for service in admin monitor insights reports; do
        wait_for_health "$service"
    done
    stop_all
done <<'ORDERS'
admin monitor insights reports
admin monitor reports insights
admin insights monitor reports
admin insights reports monitor
admin reports monitor insights
admin reports insights monitor
monitor admin insights reports
monitor admin reports insights
monitor insights admin reports
monitor insights reports admin
monitor reports admin insights
monitor reports insights admin
insights admin monitor reports
insights admin reports monitor
insights monitor admin reports
insights monitor reports admin
insights reports admin monitor
insights reports monitor admin
reports admin monitor insights
reports admin insights monitor
reports monitor admin insights
reports monitor insights admin
reports insights admin monitor
reports insights monitor admin
ORDERS

PHASE="contract"
for service in monitor insights reports admin; do
    start_service "$service"
done
for service in admin monitor insights reports; do
    wait_for_health "$service"
done

cli_status="$($CLI --json doctor)"
CLI_STATUS="$cli_status" run_bun -e '
    const payload = JSON.parse(process.env.CLI_STATUS);
    const services = payload.data?.health?.services;
    if (
        payload.schema_version !== 1
        || payload.ok !== true
        || payload.command !== "doctor"
        || !Array.isArray(services)
        || services.length !== 4
        || services.some((service) => service.reachable !== true || service.state !== "healthy")
    ) {
        throw new Error(`invalid rz doctor response: ${JSON.stringify(payload)}`);
    }
'

owner_login="$(login owner 'rustzen@123')"
RUSTZEN_ADMIN_TOKEN="$(printf '%s' "$owner_login" | parse_json 'value.data.token')"
RUSTZEN_ADMIN_USER_ID="$(printf '%s' "$owner_login" | parse_json 'value.data.userInfo.id')"
export RUSTZEN_ADMIN_TOKEN RUSTZEN_ADMIN_USER_ID

wait_for_module_state monitor true true
wait_for_module_state insights true true
wait_for_module_state reports true true
wait_for_status 200 "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/monitor/nodes" "$RUSTZEN_ADMIN_TOKEN"
wait_for_status 401 "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/monitor/nodes"
wait_for_status 401 "http://127.0.0.1:$RUSTZEN_MONITOR_PORT/api/monitor/nodes"
wait_for_status 404 "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/unknown/path"

sqlite3 "$ROOT/data/db/admin.db" \
    "INSERT INTO users (username, email, password_hash, real_name, status, is_system) SELECT 'verify-denied', 'verify-denied@example.com', password_hash, 'Verify Denied', 1, 0 FROM users WHERE username = 'owner';"
denied_login="$(login verify-denied 'rustzen@123')"
denied_token="$(printf '%s' "$denied_login" | parse_json 'value.data.token')"
wait_for_status 403 "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/monitor/nodes" "$denied_token"
verify_module_log_diagnostics
echo "verify-services: module-log owner, denial, tail cursor/caps, archive metadata/hash, partial cleanup, and one-time confirmation passed"

PHASE="monitor-agent"
agent_nodes_before="$(sqlite3 "$ROOT/data/db/monitor.db" 'SELECT COUNT(*) FROM monitor_nodes;')"
start_service monitor_agent
agent_count=0
while [ "$agent_count" -lt 100 ]; do
    assert_alive monitor_agent
    agent_nodes_after="$(sqlite3 "$ROOT/data/db/monitor.db" 'SELECT COUNT(*) FROM monitor_nodes;' 2>/dev/null || true)"
    if [ -n "$agent_nodes_after" ] && [ "$agent_nodes_after" -gt "$agent_nodes_before" ]; then
        break
    fi
    agent_count=$((agent_count + 1))
    sleep 0.1
done
if [ -z "${agent_nodes_after:-}" ] || [ "$agent_nodes_after" -le "$agent_nodes_before" ]; then
    echo "verify-services: Monitor Agent report was not persisted through Admin" >&2
    dump_logs
    exit 1
fi
stop_service monitor_agent

run_bun "$PROJECT_ROOT/scripts/verify-worker-contracts.mjs"

disable_status="$(http_status \
    -X PUT \
    -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
    -H 'content-type: application/json' \
    -d '{"enabled":false}' \
    "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/modules/monitor/enabled")"
[ "$disable_status" = 200 ] || { echo "verify-services: disabling Monitor returned $disable_status" >&2; exit 1; }
wait_for_module_state monitor false true
wait_for_status 503 "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/monitor/nodes" "$RUSTZEN_ADMIN_TOKEN"
enable_status="$(http_status \
    -X PUT \
    -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
    -H 'content-type: application/json' \
    -d '{"enabled":true}' \
    "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/modules/monitor/enabled")"
[ "$enable_status" = 200 ] || { echo "verify-services: enabling Monitor returned $enable_status" >&2; exit 1; }
wait_for_module_state monitor true true

for service in admin monitor insights reports; do
    PHASE="termination-$service"
    stop_service "$service"
    assert_other_services_healthy "$service"
    if [ "$service" != admin ]; then
        wait_for_module_state "$service" false true
        assert_gateway_unavailable "$service"
        assert_module_gateways_healthy_except "$service"
    fi
    start_service "$service"
    wait_for_health "$service"
    if [ "$service" = admin ]; then
        wait_for_module_state monitor true true
        wait_for_module_state insights true true
        wait_for_module_state reports true true
        assert_module_gateways_healthy_except ""
    else
        wait_for_module_state "$service" true true
    fi
done

verify_database_isolations

echo "verify-services: Admin-alone login, module-log owner/denial/tail/archive/cleanup contracts, Agent persistence, 24 startup orders, rz doctor, unavailable gateways, independent termination, four database restores, contracts, and latency passed"
echo "verify-services: latency evidence: $RUSTZEN_GATEWAY_LATENCY_OUTPUT"
