# shellcheck shell=sh
# Sourced by verify-services.sh before its cleanup trap is installed.
# It owns only disposable-service process and health lifecycle operations.

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
