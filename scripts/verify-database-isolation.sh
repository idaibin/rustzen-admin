# shellcheck shell=sh
# Sourced by verify-services.sh after lifecycle helpers and cleanup are installed.
# It relies on the outer verifier's service, health, state, and logging helpers.

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

verify_database_isolations() {
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
}
