#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
. "$root/scripts/monitoring-ui-state-gate-lib.sh"
. "$root/scripts/monitoring-ui-state-evidence-lib.sh"

cleanup_mode=$(env_value RUSTZEN_MONITORING_UI_STATE_TEST_BLOCKING_CLEANUP)
if [ -n "$cleanup_mode" ]; then
    test_root=$(env_value RUSTZEN_MONITORING_UI_STATE_TEST_ROOT)
    [ -n "$test_root" ] || {
        echo "blocking cleanup seam requires a test root" >&2
        exit 2
    }
    case "$cleanup_mode" in
        124|INT|TERM) ;;
        *)
            echo "unsupported blocking cleanup mode: $cleanup_mode" >&2
            exit 2
            ;;
    esac
    evidence_root=$test_root
    current="$test_root/current"
    lock_dir="$test_root/.verify.lock"
    run_id=test
    candidate="$test_root/.candidate"
    staged="$test_root/.staged"
    published_run="$test_root/runs/$run_id"
    publish_link="$test_root/.current-$run_id"
    container=test
    cleanup_timeout=2
    docker_bin="$test_root/fake-docker.sh"
    mkdir -p "$test_root/runs/old" "$candidate" "$staged" "$lock_dir"
    printf old >"$test_root/runs/old/manifest.json"
    printf candidate >"$candidate/manifest.json"
    ln -s runs/old "$current"
    ln -s runs/test "$publish_link"
    cat >"$docker_bin" <<'SH'
#!/usr/bin/env bash
printf '%s\n' "$$" >>"$RUSTZEN_MONITORING_UI_STATE_TEST_ROOT/fake-docker-pids"
printf started >"$RUSTZEN_MONITORING_UI_STATE_TEST_ROOT/fake-docker-$1.started"
trap '' TERM
while :; do sleep 60; done
SH
    chmod +x "$docker_bin"
    trap outer_cleanup EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    if [ "$cleanup_mode" = 124 ]; then
        exit 124
    fi
    kill -"$cleanup_mode" "$$"
fi

if [ "$(env_value RUSTZEN_MONITORING_UI_STATE_TEST_TIMEOUT)" = 1 ]; then
    test_root=$(env_value RUSTZEN_MONITORING_UI_STATE_TEST_ROOT)
    [ -n "$test_root" ] || \
        test_root=$(mktemp -d "$tmp_root/rz-monitoring-ui-timeout-seam.XXXXXX")
    mkdir -p "$test_root"
    timeout_state="$test_root/timeout-state"
    watchdog_receipt="$test_root/watchdog.pid"
    stubborn='trap "exit 0" TERM; '
    stubborn=$stubborn'sh -c '\''trap "" TERM; while :; do sleep 60; done'\'' & '
    stubborn=$stubborn'echo $! > "$1"; wait'
    if RUSTZEN_MONITORING_UI_STATE_TIMEOUT_STATE_DIR="$timeout_state" \
        RUSTZEN_MONITORING_UI_STATE_TEST_TIMEOUT_WATCHDOG_RECEIPT="$watchdog_receipt" \
        run_bounded 1 sh -c "$stubborn" sh "$test_root/grandchild.pid" \
        >"$test_root/output" 2>&1; then
        echo "timeout seam unexpectedly succeeded" >&2
        exit 1
    else
        timeout_status=$?
    fi
    grandchild=$(cat "$test_root/grandchild.pid")
    watchdog=$(cat "$watchdog_receipt")
    test "$timeout_status" = 124
    ! pid_alive "$grandchild"
    ! pid_alive "$watchdog"
    ! test -e "$timeout_state"
    rm -rf "$test_root"
    echo "Monitoring UI state natural timeout tree seam passed"
    exit 0
fi

signal=$(env_value RUSTZEN_MONITORING_UI_STATE_TEST_SIGNAL)
if [ "$signal" = INT ] || [ "$signal" = TERM ]; then
    test_root=$(env_value RUSTZEN_MONITORING_UI_STATE_TEST_ROOT)
    [ -n "$test_root" ] || \
        test_root=$(mktemp -d "$tmp_root/rz-monitoring-ui-signal.XXXXXX")
    evidence_root=$test_root
    current="$test_root/current"
    lock_dir="$test_root/.verify.lock"
    candidate="$test_root/.candidate"
    staged="$test_root/.staged"
    published_run="$test_root/runs/unused"
    run_id=unused
    mkdir -p "$test_root/runs/old" "$candidate" "$staged" "$lock_dir"
    printf old >"$test_root/runs/old/manifest.json"
    ln -s runs/old "$current"
    sh -c 'trap "exit 0" TERM; while :; do sleep 60; done' &
    bounded_child_pid=$!
    sh -c \
        'trap '\''printf graceful > "$1"; exit 0'\'' TERM; while :; do sleep 60; done' \
        sh "$test_root/graceful.receipt" &
    bounded_watchdog_pid=$!
    child=$bounded_child_pid
    watchdog=$bounded_watchdog_pid
    cleanup_signal() {
        result=$?
        trap - EXIT INT TERM
        cleanup_bounded_processes
        rm -rf "$candidate" "$staged" "$lock_dir"
        expected=130
        [ "$signal" != TERM ] || expected=143
        test "$result" = "$expected"
        ! pid_alive "$child"
        ! pid_alive "$watchdog"
        test "$(cat "$test_root/graceful.receipt")" = graceful
        test "$(readlink "$current")" = runs/old
        ! test -e "$candidate"
        ! test -e "$staged"
        ! test -e "$lock_dir"
        echo "Monitoring UI state $signal signal seam passed" >&2
        exit "$result"
    }
    trap cleanup_signal EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    kill -"$signal" "$$"
fi

signal=$(env_value RUSTZEN_MONITORING_UI_STATE_TEST_AFTER_REPLACE_SIGNAL)
if [ "$signal" = INT ] || [ "$signal" = TERM ]; then
    test_root=$(mktemp -d "$tmp_root/rz-monitoring-ui-commit.XXXXXX")
    evidence_root=$test_root
    current="$test_root/current"
    lock_dir="$test_root/.verify.lock"
    run_id=test
    candidate="$test_root/.candidate"
    staged="$test_root/.staged"
    published_run="$test_root/runs/$run_id"
    publish_link="$test_root/.current-$run_id"
    mkdir -p "$test_root/runs/old" "$candidate" "$staged" "$lock_dir"
    printf old >"$test_root/runs/old/manifest.json"
    printf new >"$candidate/manifest.json"
    ln -s runs/old "$current"
    cleanup_commit() {
        result=$?
        trap - EXIT INT TERM
        rm -rf "$staged" "$publish_link" "$lock_dir"
        move_failure_evidence
        expected=130
        [ "$signal" != TERM ] || expected=143
        test "$result" = "$expected"
        current_is_published
        test "$(cat "$current/manifest.json")" = new
        ! test -e "$publish_link"
        ! test -e "$lock_dir"
        echo "Monitoring UI state $signal after-replace signal seam passed" >&2
        exit "$result"
    }
    trap cleanup_commit EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    publish_candidate
    kill -"$signal" "$$"
fi

failure=$(env_value RUSTZEN_MONITORING_UI_STATE_TEST_PUBLISH_FAILURE)
if [ -n "$failure" ]; then
    test_root=$(mktemp -d "$tmp_root/rz-monitoring-ui-publish.XXXXXX")
    evidence_root=$test_root
    current="$test_root/current"
    lock_dir="$test_root/.verify.lock"
    run_id=test
    candidate="$test_root/.candidate"
    staged="$test_root/.staged"
    published_run="$test_root/runs/$run_id"
    publish_link="$test_root/.current-$run_id"
    mkdir -p "$test_root/runs/old" "$candidate" "$staged" "$lock_dir"
    printf old >"$test_root/runs/old/manifest.json"
    printf new >"$candidate/manifest.json"
    ln -s runs/old "$current"
    if publish_candidate; then
        echo "forced publication failure unexpectedly succeeded" >&2
        exit 1
    fi
    move_failure_evidence
    rm -rf "$staged" "$publish_link" "$lock_dir"
    if [ "$failure" = after-replace ]; then
        current_is_published
        test "$(cat "$current/manifest.json")" = new
    else
        test "$(readlink "$current")" = runs/old
        ! test -e "$candidate"
        ! test -e "$published_run"
        test "$(cat "$test_root/failed-runs/test/manifest.json")" = new
    fi
    ! test -e "$publish_link"
    ! test -e "$lock_dir"
    echo "Monitoring UI state $failure publication rollback seam passed"
    rm -rf "$test_root"
    exit 0
fi

if [ "$(env_value RUSTZEN_MONITORING_UI_STATE_TEST_STAGING_REPLACE)" = 1 ]; then
    test_root=$(mktemp -d "$tmp_root/rz-monitoring-ui-staging.XXXXXX")
    bin_dir="$test_root/bin"
    candidate="$test_root/candidate"
    staged="$test_root/staged"
    file_pattern='.*'
    mkdir -p "$bin_dir" "$candidate" "$staged"
    for name in rz-admin rz-monitor rz-insights rz-reports; do
        printf '%s-original\n' "$name" >"$bin_dir/$name"
        chmod +x "$bin_dir/$name"
    done
    for name in rz-admin rz-monitor rz-insights rz-reports; do
        hash=$(shasum -a 256 "$bin_dir/$name" | awk '{print $1}')
        printf '%s\t%s\n' "$name" "$hash"
    done >"$candidate/build-provenance.txt"
    stage_binary rz-admin
    printf replacement >"$bin_dir/rz-monitor"
    chmod +x "$bin_dir/rz-monitor"
    if stage_binary rz-monitor; then
        echo "concurrent replacement unexpectedly matched provenance" >&2
        exit 1
    fi
    rm -rf "$test_root"
    echo "Monitoring UI state concurrent staging seam passed"
    exit 0
fi

if [ "$(env_value RUSTZEN_MONITORING_UI_STATE_TEST_ARTIFACT_TAMPER)" = 1 ]; then
    test_root=$(mktemp -d "$tmp_root/rz-monitoring-ui-artifact.XXXXXX")
    printf desktop >"$test_root/monitoring-overview-desktop-dark-en.png"
    printf mobile >"$test_root/monitoring-summaries-mobile-light-zh.png"
    cat >"$test_root/manifest.json" <<'JSON'
{"artifacts":[
  {"file":"monitoring-overview-desktop-dark-en.png","sha256":"bad","dimensions":"1440 x 900"},
  {"file":"monitoring-summaries-mobile-light-zh.png","sha256":"bad","dimensions":"390 x 844"}
]}
JSON
    if verify_artifacts "$test_root" "$test_root/manifest.json"; then
        echo "tampered artifact manifest unexpectedly verified" >&2
        exit 1
    fi
    rm -rf "$test_root"
    echo "Monitoring UI state artifact tamper seam passed"
    exit 0
fi

if [ -n "$(env_value RUSTZEN_MONITORING_UI_STATE_TEST_MANIFEST)" ]; then
    test_manifest=$(env_value RUSTZEN_MONITORING_UI_STATE_TEST_MANIFEST)
    test_steps=$(env_value RUSTZEN_MONITORING_UI_STATE_TEST_BROWSER_STEPS)
    [ -n "$test_steps" ] || {
        echo "missing browser steps" >&2
        exit 2
    }
    read -r test_head test_state test_sha test_platform < <(
        jq -r '[.gitHead,.sourceTreeState,.sourceTreeSha256,.platform] | @tsv' \
            "$test_manifest"
    )
    verify_manifest \
        "$test_manifest" \
        "$test_steps" \
        "$test_head" \
        "$test_state" \
        "$test_sha" \
        "$test_platform"
    echo "Monitoring UI state manifest verification seam passed"
    exit 0
fi

echo "no Monitoring UI state seam selected" >&2
exit 2
