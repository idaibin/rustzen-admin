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

ROOT="$(mktemp -d "${TMPDIR:-/tmp}/rz-services.XXXXXX")"
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
export RUSTZEN_MONITOR_SCHEMA_FINGERPRINT=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
export RUSTZEN_MONITOR_DATA_CONTRACT_ID=dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd
export RUSTZEN_GATEWAY_LATENCY_OUTPUT="${RUSTZEN_GATEWAY_LATENCY_OUTPUT:-$PROJECT_ROOT/target/rz/gateway-latency.json}"
export RUST_LOG=warn

run_bun() {
    pnpm dlx bun@1.3.14 "$@"
}

dump_logs() {
    for log in "$ROOT"/logs/*.log; do
        [ -s "$log" ] || continue
        echo "verify-services: tail $log" >&2
        tail -n 40 "$log" >&2 || true
    done
}

cleanup() {
    stop_all || true
    rm -rf "$ROOT"
}
trap cleanup EXIT INT TERM

start_service() {
    name="$1"
    log="$ROOT/logs/$PHASE-$name.log"
    case "$name" in
        admin) "$ADMIN" serve >"$log" 2>&1 & ;;
        monitor) "$MONITOR" controller >"$log" 2>&1 & ;;
        insights) "$INSIGHTS" serve >"$log" 2>&1 & ;;
        reports) "$REPORTS" serve >"$log" 2>&1 & ;;
        monitor_agent) "$AGENT" >"$log" 2>&1 & ;;
        *) echo "verify-services: unknown service $name" >&2; exit 1 ;;
    esac
    pid=$!
    printf '%s\n' "$pid" >"$ROOT/pids/$name"
    printf '%s\n' "$log" >"$ROOT/pids/$name.log"
}

stop_service() {
    name="$1"
    pid_file="$ROOT/pids/$name"
    [ -f "$pid_file" ] || return 0
    pid="$(cat "$pid_file")"
    if kill -0 "$pid" 2>/dev/null; then
        kill -TERM "$pid" 2>/dev/null || true
        count=0
        while kill -0 "$pid" 2>/dev/null && [ "$count" -lt 50 ]; do
            count=$((count + 1))
            sleep 0.1
        done
        if kill -0 "$pid" 2>/dev/null; then
            echo "verify-services: $name did not stop after SIGTERM; sending SIGKILL" >&2
            kill -KILL "$pid" 2>/dev/null || true
        fi
    fi
    wait "$pid" 2>/dev/null || true
    rm -f "$pid_file" "$ROOT/pids/$name.log"
}

stop_all() {
    for name in monitor_agent admin reports insights monitor; do
        stop_service "$name"
    done
}

service_pid() {
    cat "$ROOT/pids/$1"
}

assert_alive() {
    name="$1"
    pid="$(service_pid "$name")"
    if ! kill -0 "$pid" 2>/dev/null; then
        echo "verify-services: $name exited unexpectedly" >&2
        dump_logs
        exit 1
    fi
}

service_health_url() {
    case "$1" in
        admin) printf 'http://127.0.0.1:%s/health\n' "$RUSTZEN_ADMIN_PORT" ;;
        monitor) printf 'http://127.0.0.1:%s/health\n' "$RUSTZEN_MONITOR_PORT" ;;
        insights) printf 'http://127.0.0.1:%s/health\n' "$RUSTZEN_INSIGHTS_PORT" ;;
        reports) printf 'http://127.0.0.1:%s/health\n' "$RUSTZEN_REPORTS_PORT" ;;
        *) return 1 ;;
    esac
}

http_status() {
    curl --silent --show-error --output /dev/null --write-out '%{http_code}' "$@" 2>/dev/null || true
}

wait_for_status() {
    expected="$1"
    url="$2"
    token="${3:-}"
    count=0
    while [ "$count" -lt 180 ]; do
        if [ -n "$token" ]; then
            status="$(http_status -H "authorization: Bearer $token" "$url")"
        else
            status="$(http_status "$url")"
        fi
        if [ "$status" = "$expected" ]; then
            return 0
        fi
        count=$((count + 1))
        sleep 0.1
    done
    echo "verify-services: expected HTTP $expected from $url, got ${status:-none}" >&2
    dump_logs
    exit 1
}

wait_for_health() {
    name="$1"
    wait_for_status 200 "$(service_health_url "$name")"
    assert_alive "$name"
}

assert_other_services_healthy() {
    stopped="$1"
    for name in admin monitor insights reports; do
        if [ "$name" != "$stopped" ]; then
            wait_for_health "$name"
        fi
    done
}

parse_json() {
    expression="$1"
    run_bun -e "const value = JSON.parse(await Bun.stdin.text()); console.log($expression)"
}

login() {
    username="$1"
    password="$2"
    response="$(curl --fail --silent --show-error \
        -H 'content-type: application/json' \
        -d "{\"username\":\"$username\",\"password\":\"$password\"}" \
        "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/auth/login")"
    printf '%s' "$response"
}

wait_for_module_state() {
    module="$1"
    available="$2"
    compatible="$3"
    count=0
    while [ "$count" -lt 180 ]; do
        body="$(curl --silent --show-error \
            -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
            "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/modules" 2>/dev/null || true)"
        if BODY="$body" MODULE_ID="$module" EXPECT_AVAILABLE="$available" \
            EXPECT_COMPATIBLE="$compatible" run_bun -e '
                try {
                    const payload = JSON.parse(process.env.BODY);
                    const module = payload.data?.find((item) => item.id === process.env.MODULE_ID);
                    const matches = module
                        && String(module.available) === process.env.EXPECT_AVAILABLE
                        && String(module.compatible) === process.env.EXPECT_COMPATIBLE;
                    process.exit(matches ? 0 : 1);
                } catch {
                    process.exit(1);
                }
            '
        then
            return 0
        fi
        count=$((count + 1))
        sleep 0.1
    done
    echo "verify-services: module state did not converge: $module available=$available compatible=$compatible" >&2
    dump_logs
    exit 1
}

module_gateway_url() {
    case "$1" in
        monitor) printf 'http://127.0.0.1:%s/api/monitor/nodes\n' "$RUSTZEN_ADMIN_PORT" ;;
        insights) printf 'http://127.0.0.1:%s/api/insights/overview\n' "$RUSTZEN_ADMIN_PORT" ;;
        reports) printf 'http://127.0.0.1:%s/api/reports/systems\n' "$RUSTZEN_ADMIN_PORT" ;;
        *) return 1 ;;
    esac
}

assert_gateway_unavailable() {
    module="$1"
    url="$(module_gateway_url "$module")"
    wait_for_status 503 "$url" "$RUSTZEN_ADMIN_TOKEN"
    body="$(curl --silent --show-error \
        -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" "$url" 2>/dev/null || true)"
    BODY="$body" MODULE_ID="$module" run_bun -e '
        const payload = JSON.parse(process.env.BODY);
        const expected = `${process.env.MODULE_ID} worker is temporarily unavailable.`;
        if (payload.code !== 40001 || payload.message !== expected || payload.data !== null) {
            throw new Error(`invalid unavailable envelope: ${JSON.stringify(payload)}`);
        }
    '
}

assert_module_gateways_healthy_except() {
    excluded="$1"
    for module in monitor insights reports; do
        if [ "$module" != "$excluded" ]; then
            wait_for_status 200 "$(module_gateway_url "$module")" "$RUSTZEN_ADMIN_TOKEN"
        fi
    done
}

verify_module_log_diagnostics() {
    module_log_date="$(run_bun -e 'console.log(new Date().toISOString().slice(0, 10))')"
    module_log_old_date="$(run_bun -e 'const date = new Date(); date.setUTCDate(date.getUTCDate() - 90); console.log(date.toISOString().slice(0, 10))')"
    module_log_dir="$ROOT/logs"
    mkdir -p "$module_log_dir"
    MODULE_LOG_DIR="$module_log_dir" MODULE_LOG_DATE="$module_log_date" MODULE_LOG_OLD_DATE="$module_log_old_date" run_bun -e '
        const dir = process.env.MODULE_LOG_DIR;
        const tailFixture = Array.from(
            { length: 24_000 },
            (_, index) => `fixture-${String(index).padStart(6, "0")}\n`,
        ).join("");
        await Bun.write(`${dir}/admin.${process.env.MODULE_LOG_DATE}`, tailFixture);
        await Bun.write(`${dir}/monitor.${process.env.MODULE_LOG_OLD_DATE}`, "cleanup-candidate\n");
    '

    module_log_list="$(curl --fail --silent --show-error \
        -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
        "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/status/module-logs?module=admin&date=$module_log_date")"
    MODULE_LOG_LIST="$module_log_list" MODULE_LOG_DATE="$module_log_date" run_bun -e '
        const payload = JSON.parse(process.env.MODULE_LOG_LIST);
        if (!payload.data?.some((item) => item.module === "admin" && item.date === process.env.MODULE_LOG_DATE && item.readable)) {
            throw new Error(`module-log list did not return the readable admin fixture: ${JSON.stringify(payload)}`);
        }
    '
    wait_for_status 403 "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/status/module-logs?module=admin&date=$module_log_date" "$denied_token"

    MODULE_LOG_BASE_URL="http://127.0.0.1:$RUSTZEN_ADMIN_PORT" MODULE_LOG_DATE="$module_log_date" MODULE_LOG_TOKEN="$RUSTZEN_ADMIN_TOKEN" run_bun -e '
        const expectedLines = 24_000;
        const pages = [];
        let cursor;
        for (let pageIndex = 0; pageIndex < 32; pageIndex += 1) {
            const params = new URLSearchParams({ module: "admin", date: process.env.MODULE_LOG_DATE });
            if (cursor) params.set("cursor", cursor);
            const response = await fetch(`${process.env.MODULE_LOG_BASE_URL}/api/system/status/module-logs/tail?${params}`, {
                headers: { authorization: `Bearer ${process.env.MODULE_LOG_TOKEN}` },
            });
            if (!response.ok) throw new Error(`module-log tail returned HTTP ${response.status}`);
            const tail = (await response.json()).data;
            const ids = tail.content ? tail.content.split("\n").map((line) => Number(line.slice("fixture-".length))) : [];
            if (tail.byteCount > 256 * 1024 || tail.lineCount > 2000 || ids.length !== tail.lineCount) {
                throw new Error(`module-log tail cap contract failed: ${JSON.stringify(tail)}`);
            }
            if (ids.some((id, index) => !Number.isInteger(id) || (index > 0 && id !== ids[index - 1] + 1))) {
                throw new Error(`module-log tail page is not an ordered fixture range: ${JSON.stringify(ids)}`);
            }
            pages.push(ids);
            if (!tail.nextCursor) break;
            if (!tail.truncated || tail.nextCursor === cursor) {
                throw new Error(`module-log cursor did not advance: ${JSON.stringify(tail)}`);
            }
            cursor = tail.nextCursor;
        }
        if (pages.length < 2 || pages.at(-1)?.length === 0 || pages.at(-1)?.[0] !== 0) {
            throw new Error(`module-log tail did not converge: ${JSON.stringify(pages)}`);
        }
        const first = pages[0];
        const second = pages[1];
        if (first.some((id) => second.includes(id)) || first[0] <= second.at(-1)) {
            throw new Error(`module-log first and second pages overlap or are unordered: ${JSON.stringify({ first, second })}`);
        }
        const reconstructed = pages.toReversed().flat();
        if (reconstructed.length !== expectedLines || reconstructed.some((id, index) => id !== index)) {
            throw new Error(`module-log tail reconstruction failed: ${JSON.stringify({ length: reconstructed.length, first: reconstructed[0], last: reconstructed.at(-1) })}`);
        }
    '

    module_log_headers="$ROOT/module-log-backup.headers"
    module_log_archive="$ROOT/module-log-backup.tar"
    curl --fail --silent --show-error \
        -D "$module_log_headers" \
        -o "$module_log_archive" \
        -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
        -H 'content-type: application/json' \
        -d "{\"files\":[{\"module\":\"admin\",\"date\":\"$module_log_date\"}]}" \
        "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/status/module-logs/backup"
    MODULE_LOG_HEADERS="$module_log_headers" MODULE_LOG_ARCHIVE="$module_log_archive" run_bun -e '
        const headers = await Bun.file(process.env.MODULE_LOG_HEADERS).text();
        const value = (name) => headers.match(new RegExp(`^${name}:\s*(.+)\r?$`, "im"))?.[1]?.trim();
        if (!/^attachment;\s*filename=rustzen-module-logs\.tar$/i.test(value("content-disposition") ?? "")) throw new Error("missing backup filename header");
        const archiveSha256 = value("x-rustzen-archive-sha256");
        if (!/^[a-f0-9]{64}$/.test(archiveSha256 ?? "")) throw new Error("missing backup hash header");
        if (value("x-rustzen-archive-file-count") !== "1") throw new Error("backup count header did not match one selected file");
        const archive = await Bun.file(process.env.MODULE_LOG_ARCHIVE).arrayBuffer();
        if (archive.byteLength === 0) throw new Error("empty backup archive");
        const digest = [...new Uint8Array(await crypto.subtle.digest("SHA-256", archive))].map((byte) => byte.toString(16).padStart(2, "0")).join("");
        if (digest !== archiveSha256) throw new Error("backup archive hash does not match its header");
    '

    cleanup_preview="$(curl --fail --silent --show-error \
        -X POST -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
        "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/status/module-logs/cleanup/preview")"
    cleanup_token="$(CLEANUP_PREVIEW="$cleanup_preview" MODULE_LOG_OLD_DATE="$module_log_old_date" run_bun -e '
        const preview = JSON.parse(process.env.CLEANUP_PREVIEW).data;
        if (typeof preview?.token !== "string" || !preview.candidates?.some((item) => item.module === "monitor" && item.date === process.env.MODULE_LOG_OLD_DATE)) {
            throw new Error(`module-log cleanup preview fixture missing: ${JSON.stringify(preview)}`);
        }
        console.log(preview.token);
    ')"
    printf '%s\n' changed >>"$module_log_dir/monitor.$module_log_old_date"
    cleanup_result="$(curl --fail --silent --show-error \
        -X POST -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" -H 'content-type: application/json' \
        -d "{\"token\":\"$cleanup_token\"}" \
        "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/status/module-logs/cleanup/confirm")"
    CLEANUP_RESULT="$cleanup_result" run_bun -e '
        const result = JSON.parse(process.env.CLEANUP_RESULT).data;
        if (!result.partial || !Array.isArray(result.failures) || result.failures.length === 0) {
            throw new Error(`module-log cleanup change must be partial: ${JSON.stringify(result)}`);
        }
    '
    repeated_status="$(http_status -X POST -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" -H 'content-type: application/json' \
        -d "{\"token\":\"$cleanup_token\"}" \
        "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/status/module-logs/cleanup/confirm")"
    [ "$repeated_status" = 400 ] || { echo "verify-services: reused module-log cleanup token returned $repeated_status" >&2; exit 1; }
}

"$MONITOR" init-db
"$MONITOR" bind-database
"$MONITOR" validate-database

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

cli_status="$($CLI --json status all)"
CLI_STATUS="$cli_status" run_bun -e '
    const payload = JSON.parse(process.env.CLI_STATUS);
    const services = payload.data?.services;
    if (
        payload.schema_version !== 1
        || payload.ok !== true
        || payload.command !== "status"
        || payload.data?.selection !== "all"
        || !Array.isArray(services)
        || services.length !== 4
        || services.some((service) => service.reachable !== true || service.state !== "healthy")
    ) {
        throw new Error(`invalid rz status response: ${JSON.stringify(payload)}`);
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

expect_corrupt_start_failure() {
    name="$1"
    start_service "$name"
    pid="$(service_pid "$name")"
    count=0
    while kill -0 "$pid" 2>/dev/null && [ "$count" -lt 40 ]; do
        count=$((count + 1))
        sleep 0.1
    done
    if kill -0 "$pid" 2>/dev/null; then
        echo "verify-services: $name unexpectedly accepted a corrupt database" >&2
        dump_logs
        exit 1
    fi
    wait "$pid" 2>/dev/null || true
    rm -f "$ROOT/pids/$name" "$ROOT/pids/$name.log"
}

verify_database_isolation() {
    db_service="$1"
    database="$2"
    case "$db_service" in
        reports) path="$ROOT/data/reports/db/reports.db" ;;
        *) path="$ROOT/data/db/$database.db" ;;
    esac
    backup="$ROOT/backups/$database.db"
    PHASE="database-$db_service"

    stop_service "$db_service"
    sqlite3 "$path" ".backup '$backup'"
    rm -f "$path" "$path-wal" "$path-shm"
    printf 'not-a-sqlite-database' >"$path"
    expect_corrupt_start_failure "$db_service"
    assert_other_services_healthy "$db_service"
    if [ "$db_service" != admin ]; then
        wait_for_module_state "$db_service" false true
    fi

    rm -f "$path" "$path-wal" "$path-shm"
    cp "$backup" "$path"
    start_service "$db_service"
    wait_for_health "$db_service"
    if [ "$db_service" = admin ]; then
        wait_for_module_state monitor true true
        wait_for_module_state insights true true
        wait_for_module_state reports true true
    else
        wait_for_module_state "$db_service" true true
    fi
}

verify_database_isolation monitor monitor
verify_database_isolation insights insights
verify_database_isolation reports reports
verify_database_isolation admin admin

for database in admin monitor insights; do
    [ -s "$ROOT/data/db/$database.db" ] || {
        echo "verify-services: missing restored database $database" >&2
        exit 1
    }
done
[ -s "$ROOT/data/reports/db/reports.db" ] || {
    echo "verify-services: missing restored database reports" >&2
    exit 1
}

echo "verify-services: Admin-alone login, module-log owner/denial/tail/archive/cleanup contracts, Agent persistence, 24 startup orders, rz status, unavailable gateways, independent termination, four database restores, contracts, and latency passed"
echo "verify-services: latency evidence: $RUSTZEN_GATEWAY_LATENCY_OUTPUT"
