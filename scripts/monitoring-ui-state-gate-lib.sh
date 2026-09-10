#!/usr/bin/env bash
# Shared process, staging, and publication helpers for the Monitoring UI gate.

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
env_value() { printenv "$1" 2>/dev/null || true; }
docker_bin=$(env_value RUSTZEN_MONITORING_UI_STATE_DOCKER)
[ -n "$docker_bin" ] || docker_bin=docker
timeout=$(env_value RUSTZEN_MONITORING_UI_STATE_TIMEOUT)
[ -n "$timeout" ] || timeout=2100
info_timeout=$(env_value RUSTZEN_MONITORING_UI_STATE_DOCKER_INFO_TIMEOUT)
[ -n "$info_timeout" ] || info_timeout=10
cleanup_timeout=$(env_value RUSTZEN_MONITORING_UI_STATE_DOCKER_CLEANUP_TIMEOUT)
[ -n "$cleanup_timeout" ] || cleanup_timeout=3
tmp_root=$(env_value TMPDIR)
[ -n "$tmp_root" ] || tmp_root=/tmp
bounded_child_pid=
bounded_watchdog_pid=
published_committed=0

validate_timeout() {
    local value=$1 name=$2 maximum=$3
    case "$value" in
        ''|*[!0-9]*)
            echo "$name must be a positive integer" >&2
            exit 2
            ;;
    esac
    if [ "$value" -le 0 ] || [ "$value" -gt "$maximum" ]; then
        echo "$name must be 1..$maximum seconds" >&2
        exit 2
    fi
}

pid_alive() {
    local state
    kill -0 "$1" 2>/dev/null || return 1
    state=$(ps -o stat= -p "$1" 2>/dev/null | tr -d ' ')
    case "$state" in
        Z*) return 1 ;;
        *) return 0 ;;
    esac
}

collect_process_tree() {
    local pid=$1 child
    printf '%s\n' "$pid"
    for child in $(pgrep -P "$pid" 2>/dev/null || true); do
        collect_process_tree "$child"
    done
}

stop_frozen_processes() {
    local targets_file=$1 attempt target any_alive
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
}

stop_process_tree() {
    local pid=$1 targets_file
    [ -n "$pid" ] || return 0
    targets_file=$(mktemp "$tmp_root/rz-monitoring-ui-targets.XXXXXX") || return 1
    collect_process_tree "$pid" >"$targets_file"
    stop_frozen_processes "$targets_file"
    rm -f "$targets_file"
    wait "$pid" 2>/dev/null || true
}

cleanup_bounded_processes() {
    stop_process_tree "$bounded_child_pid"
    stop_process_tree "$bounded_watchdog_pid"
    bounded_child_pid=
    bounded_watchdog_pid=
}

run_bounded_quietly() {
    run_bounded "$cleanup_timeout" "$@" >/dev/null 2>&1 || true
}

run_bounded() {
    local seconds=$1 status state_dir done_file fired_file targets_file
    shift
    state_dir=$(env_value RUSTZEN_MONITORING_UI_STATE_TIMEOUT_STATE_DIR)
    if [ -z "$state_dir" ]; then
        state_dir=$(mktemp -d "$tmp_root/rz-monitoring-ui-timeout.XXXXXX")
    fi
    mkdir -p "$state_dir"
    done_file="$state_dir/done"
    fired_file="$state_dir/fired"
    targets_file="$state_dir/targets"
    "$@" &
    bounded_child_pid=$!
    (
        watchdog_receipt=$(env_value \
            RUSTZEN_MONITORING_UI_STATE_TEST_TIMEOUT_WATCHDOG_RECEIPT)
        [ -z "$watchdog_receipt" ] || printf '%s\n' "$BASHPID" >"$watchdog_receipt"
        sleep "$seconds"
        [ ! -e "$done_file" ] || exit 0
        : >"$fired_file"
        collect_process_tree "$bounded_child_pid" >"$targets_file"
        stop_frozen_processes "$targets_file"
    ) >/dev/null 2>&1 &
    bounded_watchdog_pid=$!
    if wait "$bounded_child_pid"; then
        status=0
    else
        status=$?
    fi
    if [ -e "$fired_file" ]; then
        wait "$bounded_watchdog_pid" 2>/dev/null || true
        bounded_watchdog_pid=
        bounded_child_pid=
        rm -rf "$state_dir"
        return 124
    fi
    : >"$done_file"
    stop_process_tree "$bounded_watchdog_pid"
    bounded_watchdog_pid=
    bounded_child_pid=
    rm -rf "$state_dir"
    return "$status"
}

run_bounded_capture() {
    local capture status
    capture=$(mktemp "$tmp_root/rz-monitoring-ui-capture.XXXXXX") || return 1
    if run_bounded "$@" >"$capture"; then
        status=0
    else
        status=$?
    fi
    cat "$capture"
    rm -f "$capture"
    return "$status"
}

atomic_replace_symlink() {
    case "$(uname -s)" in
        Darwin|FreeBSD) mv -fh "$1" "$2" ;;
        *) mv -Tf "$1" "$2" ;;
    esac
}

provenance_hash() {
    awk -F '\t' -v name="$2" '$1 == name { print $2; exit }' "$1"
}

verify_build_provenance() {
    local provenance=$1
    local expected_head=$2
    local expected_state=$3
    local expected_sha=$4
    local expected_architecture=$5
    local expected_triple=$6
    local expected_platform=$7
    awk \
        -F '\t' \
        -v head="$expected_head" \
        -v state="$expected_state" \
        -v sha="$expected_sha" \
        -v arch="$expected_architecture" \
        -v triple="$expected_triple" \
        -v platform="$expected_platform" '
        $1 == "schemaVersion" && $2 == "1" { schema = 1 }
        $1 == "gitHead" && $2 == head { git = 1 }
        $1 == "sourceTreeState" && $2 == state { tree = 1 }
        $1 == "sourceTreeSha256" && $2 == sha { digest = 1 }
        $1 == "architecture" && $2 == arch { architecture = 1 }
        $1 == "targetTriple" && $2 == triple { target = 1 }
        $1 == "platform" && $2 == platform { os = 1 }
        $1 == "distribution" && $2 == "full" { distribution = 1 }
        END {
            valid = schema && git && tree && digest && architecture
            valid = valid && target && os && distribution
            exit(valid ? 0 : 1)
        }
    ' "$provenance"
}

stage_binary() {
    local name=$1 expected actual
    cp "$bin_dir/$name" "$staged/$name"
    file "$staged/$name" | grep -q "$file_pattern"
    expected=$(provenance_hash "$candidate/build-provenance.txt" "$name")
    actual=$(shasum -a 256 "$staged/$name" | awk '{print $1}')
    [ -n "$expected" ] && test "$actual" = "$expected"
}

verify_staged_binaries() {
    local name expected actual
    for name in rz-admin rz-monitor rz-insights rz-reports; do
        expected=$(provenance_hash "$candidate/build-provenance.txt" "$name")
        actual=$(shasum -a 256 "$staged/$name" | awk '{print $1}')
        [ -n "$expected" ] && test "$actual" = "$expected"
    done
}
current_is_published() {
    [ -L "$current" ] && [ "$(readlink "$current")" = "runs/$run_id" ]
}

move_failure_evidence() {
    local source=
    if current_is_published || [ "$published_committed" = 1 ]; then
        return 0
    fi
    if [ -d "$published_run" ]; then
        source=$published_run
    elif [ -d "$candidate" ]; then
        source=$candidate
    fi
    [ -n "$source" ] || return 0
    failed_dir="$evidence_root/failed-runs/$run_id"
    mkdir -p "$evidence_root/failed-runs"
    rm -rf "$failed_dir"
    mv "$source" "$failed_dir"
}

publish_candidate() {
    failure=$(env_value RUSTZEN_MONITORING_UI_STATE_TEST_PUBLISH_FAILURE)
    [ "$failure" != before-move ] || return 1
    mv "$candidate" "$published_run" || return 1
    [ "$failure" != after-move ] || return 1
    ln -s "runs/$run_id" "$publish_link" || return 1
    [ "$failure" != after-link ] || return 1
    [ "$failure" != replace ] || return 1
    atomic_replace_symlink "$publish_link" "$current"
    published_committed=1
    [ "$failure" != after-replace ] || return 1
}

outer_cleanup() {
    local result=$?
    trap - EXIT INT TERM
    cleanup_bounded_processes
    if [ -d "$candidate" ]; then
        run_bounded "$cleanup_timeout" "$docker_bin" logs "$container" \
            >"$candidate/container.log" 2>&1 || true
    fi
    run_bounded_quietly "$docker_bin" rm -f "$container"
    rm -rf "$staged" "$publish_link"
    if [ "$result" -ne 0 ]; then
        move_failure_evidence
    fi
    rm -rf "$lock_dir"
    exit "$result"
}
