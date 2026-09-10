#!/usr/bin/env bash

seed_reports_ui_delivery_status() {
    local database=$1
    python3 -B - "$database" <<'PY'
import sqlite3
import sys

with sqlite3.connect(sys.argv[1], timeout=5.0) as connection:
    connection.execute("BEGIN IMMEDIATE")
    cursor = connection.execute(
        """
        UPDATE notification_delivery_status SET
          pending_count=2, pending_bytes=1024,
          quarantine_count=3, quarantine_bytes=2048,
          omitted_count=1, expired_count=2, unconfirmed_count=3,
          quarantined_count=4, quarantine_evicted_count=5,
          first_gap_at='2026-09-10T01:02:03Z',
          last_gap_at='2026-09-10T02:03:04Z',
          last_success_at='2026-09-10T03:04:05Z'
        WHERE id=1
        """
    )
    if cursor.rowcount != 1:
        raise RuntimeError("notification delivery status row was not initialized")
PY
}

wait_reports_ui_outbox_settled() {
    local database=$1
    python3 -B - "$database" <<'PY'
import sqlite3
import sys
import time

for _ in range(100):
    with sqlite3.connect(sys.argv[1], timeout=5.0) as connection:
        pending = connection.execute(
            "SELECT COUNT(*) FROM notification_outbox "
            "WHERE state IN ('pending','reconciling') OR lease_token IS NOT NULL"
        ).fetchone()[0]
    if pending == 0:
        raise SystemExit(0)
    time.sleep(0.1)
raise RuntimeError("notification outbox did not settle")
PY
}

seed_partial_schedule_fixture() {
    python3 -B - "$1" "$2" "$3" "$4" <<'PY'
import sqlite3
import sys
database, enqueued, skipped, run = sys.argv[1:]
with sqlite3.connect(database, timeout=5.0) as connection:
    connection.execute("PRAGMA foreign_keys = ON")
    if connection.execute("PRAGMA foreign_keys").fetchone() != (1,): raise RuntimeError("foreign key enforcement is unavailable")
    connection.execute("INSERT INTO automation_schedule_occurrences (schedule_id,occurrence_key,due_local,due_at,decided_at,decision,reason,run_id,run_id_snapshot) VALUES (?,?,?,?,?,?,?,?,?)", (enqueued,"fixture-enqueued","2026-09-07T10:00","2026-09-07T10:00:00+00:00","2026-09-07T10:00:01+00:00","enqueued",None,run,run))
    connection.execute("INSERT INTO automation_schedule_occurrences (schedule_id,occurrence_key,due_local,due_at,decided_at,decision,reason,run_id,run_id_snapshot) VALUES (?,?,?,?,?,?,?,?,?)", (skipped,"fixture-skipped","2026-09-07T10:01",None,"2026-09-07T10:01:01+00:00","skipped","missed",None,None))
PY
}

reports_ui_delivery_expected_jq() {
    cat <<'JQ'
.data == {
  pendingCount:2,pendingBytes:1024,quarantineCount:3,quarantineBytes:2048,
  omittedCount:1,expiredCount:2,unconfirmedCount:3,quarantinedCount:4,
  quarantineEvictedCount:5,firstGapAt:"2026-09-10T01:02:03Z",
  lastGapAt:"2026-09-10T02:03:04Z",lastSuccessAt:"2026-09-10T03:04:05Z"
}
JQ
}

reports_ui_delivery_db_json() {
    python3 -B - "$1" <<'PY'
import json
import sqlite3
import sys
columns = ["pendingCount","pendingBytes","quarantineCount","quarantineBytes","omittedCount","expiredCount","unconfirmedCount","quarantinedCount","quarantineEvictedCount","firstGapAt","lastGapAt","lastSuccessAt"]
with sqlite3.connect(sys.argv[1], timeout=5.0) as connection:
    row = connection.execute("SELECT pending_count,pending_bytes,quarantine_count,quarantine_bytes,omitted_count,expired_count,unconfirmed_count,quarantined_count,quarantine_evicted_count,first_gap_at,last_gap_at,last_success_at FROM notification_delivery_status WHERE id=1").fetchone()
if row is None: raise RuntimeError("notification delivery status row was not initialized")
print(json.dumps(dict(zip(columns, row)), separators=(",", ":"), sort_keys=True))
PY
}
