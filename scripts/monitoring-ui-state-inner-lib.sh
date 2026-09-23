#!/usr/bin/env bash
# Disposable Linux service lifecycle for the Monitoring UI gate.

pids=()
env_value() { printenv "$1" 2>/dev/null || true; }

pid_alive() {
    local state
    kill -0 "$1" 2>/dev/null || return 1
    state=$(ps -o stat= -p "$1" 2>/dev/null | tr -d ' ')
    case "$state" in
        Z*) return 1 ;;
        *) return 0 ;;
    esac
}

collect_pid_tree() {
    local pid=$1 child
    printf '%s\n' "$pid"
    for child in $(pgrep -P "$pid" 2>/dev/null || true); do
        collect_pid_tree "$child"
    done
}

stop_pid_tree() {
    local pid=$1 target attempt any_alive targets_file
    [ -n "$pid" ] || return 0
    targets_file=$(mktemp /tmp/rz-monitoring-ui-inner-targets.XXXXXX)
    collect_pid_tree "$pid" >"$targets_file"
    while read -r target; do
        kill -TERM "$target" 2>/dev/null || true
    done <"$targets_file"
    for attempt in $(seq 1 5); do
        any_alive=0
        while read -r target; do
            if pid_alive "$target"; then
                any_alive=1
                break
            fi
        done <"$targets_file"
        [ "$any_alive" = 0 ] && break
        sleep .05
    done
    while read -r target; do
        if pid_alive "$target"; then
            kill -KILL "$target" 2>/dev/null || true
        fi
    done <"$targets_file"
    rm -f "$targets_file"
    wait "$pid" 2>/dev/null || true
}

cleanup() {
    result=$?
    trap - EXIT INT TERM
    for pid in "${pids[@]}"; do
        stop_pid_tree "$pid"
    done
    stubborn_signal=$(env_value RUSTZEN_MONITORING_UI_STATE_INNER_TEST_STUBBORN)
    if [ -n "$stubborn_signal" ]; then
        ! pid_alive "$inner_stubborn_pid"
        ! pid_alive "$inner_grandchild_pid"
        ! pid_alive "$inner_graceful_pid"
        test "$(cat /tmp/rz-monitoring-ui-inner-graceful.receipt)" = graceful
        rm -f /tmp/rz-monitoring-ui-inner-grandchild.pid
        rm -f /tmp/rz-monitoring-ui-inner-graceful.receipt
        echo "Monitoring UI inner $stubborn_signal stubborn cleanup seam passed" >&2
    fi
    if [ "$result" -ne 0 ] && [ -d /opt/rz/logs ]; then
        for log in /opt/rz/logs/admin.log \
            /opt/rz/logs/monitor.log \
            /opt/rz/logs/insights.log \
            /opt/rz/logs/reports.log \
            /opt/rz/logs/fixture.log; do
            [ -s "$log" ] || continue
            echo "== $log ==" >&2
            tail -n 60 "$log" >&2
        done
    fi
    exit "$result"
}

stubborn_signal=$(env_value RUSTZEN_MONITORING_UI_STATE_INNER_TEST_STUBBORN)
if [ "$stubborn_signal" = INT ] || [ "$stubborn_signal" = TERM ]; then
    stubborn='trap "exit 0" TERM; '
    stubborn=$stubborn'sh -c '\''trap "" TERM; while :; do sleep 60; done'\'' & '
    stubborn=$stubborn'echo $! > "$1"; wait'
    sh -c "$stubborn" sh /tmp/rz-monitoring-ui-inner-grandchild.pid &
    inner_stubborn_pid=$!
    for retry in $(seq 1 20); do
        [ -s /tmp/rz-monitoring-ui-inner-grandchild.pid ] && break
        sleep .01
    done
    inner_grandchild_pid=$(cat /tmp/rz-monitoring-ui-inner-grandchild.pid)
    sh -c \
        'trap '\''printf graceful > "$1"; exit 0'\'' TERM; while :; do sleep 60; done' \
        sh /tmp/rz-monitoring-ui-inner-graceful.receipt &
    inner_graceful_pid=$!
    pids+=("$inner_stubborn_pid" "$inner_graceful_pid")
    trap cleanup EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    kill -"$stubborn_signal" "$$"
fi

for command in chromium curl dpkg-query file groupadd jq pgrep python3 \
    setpriv sha256sum ss useradd; do
    command -v "$command" >/dev/null
done
verify_chromium=$(env_value RUSTZEN_VERIFY_CHROMIUM_VERSION)
[ -n "$verify_chromium" ] || {
    echo "missing RUSTZEN_VERIFY_CHROMIUM_VERSION" >&2
    exit 2
}
test "$(dpkg-query -W -f='${Version}' chromium)" = "$verify_chromium"
for port in 19801 19802 19803 19804 19806; do
    if ss -H -ltn "sport = :$port" | grep -q .; then
        echo "verification port occupied: $port" >&2
        exit 1
    fi
done

groupadd --system rustzen
useradd \
    --system \
    --gid rustzen \
    --home-dir /opt/rz \
    --shell /usr/sbin/nologin \
    rustzen
install -d -m 0750 -o rustzen -g rustzen \
    /opt/rz/data/db \
    /opt/rz/data/reports/db \
    /opt/rz/logs \
    /opt/rz/output
for name in rz-admin rz-monitor rz-insights rz-reports; do
    install -m 0755 "/verify/bin/$name" "/opt/rz/$name"
done
chown -R rustzen:rustzen /opt/rz

export HOME=/opt/rz
export XDG_CONFIG_HOME=/opt/rz/.config
export XDG_CACHE_HOME=/opt/rz/.cache
export RUSTZEN_ENV=development
export RUSTZEN_RUNTIME_ROOT=/opt/rz
export RUSTZEN_ADMIN_HOST=127.0.0.1
export RUSTZEN_ADMIN_PORT=19801
export RUSTZEN_INTERNAL_HOST=127.0.0.1
export RUSTZEN_MONITOR_PORT=19802
export RUSTZEN_INSIGHTS_PORT=19803
export RUSTZEN_REPORTS_PORT=19804
export RUSTZEN_ADMIN_SQLITE_PATH=./data/db/admin.db
export RUSTZEN_MONITOR_SQLITE_PATH=./data/db/monitor.db
export RUSTZEN_INSIGHTS_SQLITE_PATH=./data/db/insights.db
export RUSTZEN_REPORTS_SQLITE_PATH=./data/reports/db/reports.db
export RUSTZEN_JWT_SECRET=monitoring-ui-jwt-secret
export RUSTZEN_IPC_TOKEN=monitoring-ui-ipc-secret
export RUSTZEN_MONITOR_AGENT_TOKEN=monitoring-ui-agent-secret
export RUSTZEN_REPORTS_CREDENTIAL_KEY=monitoring-ui-reports-key
export RUSTZEN_REPORTS_BROWSER_PATH=/usr/bin/chromium
: "${RUSTZEN_REPORTS_MAX_CONCURRENCY:=1}"
export RUSTZEN_REPORTS_MAX_CONCURRENCY
export RUSTZEN_TIMEZONE=UTC
export RUSTZEN_BUILD_ID
RUSTZEN_BUILD_ID=$(printf 'a%.0s' {1..64})
export RUSTZEN_COMPOSITION_ID
RUSTZEN_COMPOSITION_ID=$(printf 'b%.0s' {1..64})
export RUSTZEN_MONITOR_SCHEMA_FINGERPRINT
RUSTZEN_MONITOR_SCHEMA_FINGERPRINT=$(printf 'c%.0s' {1..64})
export RUSTZEN_MONITOR_DATA_CONTRACT_ID
RUSTZEN_MONITOR_DATA_CONTRACT_ID=$(printf 'd%.0s' {1..64})
export RUST_LOG=warn

as_rustzen() {
    setpriv \
        --reuid=rustzen \
        --regid=rustzen \
        --init-groups \
        --no-new-privs \
        -- env "$@"
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

as_rustzen /opt/rz/rz-monitor init-db
as_rustzen /opt/rz/rz-monitor bind-database
start_service() {
    local name=$1
    shift
    as_rustzen "$@" >"/opt/rz/logs/$name.log" 2>&1 &
    pids+=("$!")
}
start_service monitor /opt/rz/rz-monitor controller
start_service insights /opt/rz/rz-insights serve
start_service reports /opt/rz/rz-reports serve
start_service admin /opt/rz/rz-admin serve

curl_json() {
    curl \
        --fail \
        --silent \
        --show-error \
        --connect-timeout 3 \
        --max-time 15 \
        "$@"
}

wait_for_health() {
    local name=$1 url=$2
    for retry in $(seq 1 150); do
        if curl_json "$url" >/dev/null 2>&1; then
            return
        fi
        sleep .1
    done
    echo "$name did not become healthy at $url" >&2
    curl_json "$url" >/dev/null
}

wait_for_health admin http://127.0.0.1:19801/health
wait_for_health monitor http://127.0.0.1:19802/health
wait_for_health insights http://127.0.0.1:19803/health
wait_for_health reports http://127.0.0.1:19804/health

python3 -B /verify/fixture.py >/opt/rz/logs/fixture.log 2>&1 &
pids+=("$!")
wait_for_health fixture http://127.0.0.1:19806/__monitoring_fixture/health

monitor_fixture_reads() {
    curl_json http://127.0.0.1:19806/__monitoring_fixture/receipt \
        | jq -er '.requests | length'
}

monitor_wait_run() {
    local run=$1 status
    for retry in $(seq 1 900); do
        status=$(curl_json "${auth[@]}" "$admin/api/reports/runs/$run" \
            | jq -er '.data.status') || return 1
        case "$status" in queued|running) sleep .1 ;; *) printf '%s\n' "$status"; return 0 ;; esac
    done
    return 1
}
