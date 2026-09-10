import { expect, test } from "bun:test";
import { chmod } from "node:fs/promises";

const helper = new URL("./monitoring-ui-state-delivery-lib.sh", import.meta.url).pathname;
const run = (script) => Bun.spawnSync({ cmd: ["bash", "-c", script, "delivery", helper] });

test("Monitor settle default is 75 seconds and rejects a 60 second mutation", async () => {
    const source = await Bun.file(helper).text();
    const defaultTimeout = (text) => Number(text.match(/RUSTZEN_MONITOR_DELIVERY_SETTLE_TIMEOUT:-([0-9]+)/)?.[1]);
    expect(defaultTimeout(source)).toBe(75);
    expect(defaultTimeout(source.replace(":-75", ":-60"))).not.toBe(75);
});

test("delivery helper fails closed for relay, browser, and artifact faults", async () => {
    await chmod(helper, 0o755);
    const cases = [
        `source "$1"; curl_json(){ return 22; }; ! monitor_delivery_wait_run run`,
        `source "$1"; curl_json(){ printf '{"data":{"status":"failed"}}'; }; ! monitor_delivery_wait_run run`,
        `source "$1"; jq(){ return 1; }; ! monitor_delivery_browser_case n u p dark en-US 1 1 text`,
        `source "$1"; curl_json(){ return 22; }; ! monitor_delivery_screenshot run n out.png 1 1`,
        `source "$1"; curl_json(){ printf '{"data":[]}'; }; ! monitor_delivery_screenshot run n out.png 1 1`,
        `source "$1"; curl_json(){ printf '{"data":[{"fileName":"n.png","id":"id"}]}'; };
         ! monitor_delivery_screenshot run n out.png 1 1`,
    ];
    for (const script of cases) expect(run(script).exitCode).toBe(0);
});

test("outer assignments preserve browser success and the targeted failure", () => {
    const success = `
        source "$1"; root=$(mktemp -d); trap 'rm -rf "$root"' EXIT
        export RUSTZEN_MONITOR_DELIVERY_EVIDENCE_ROOT="$root"; system=s; admin=http://admin; owner_auth=()
        curl_json(){ case "\${!#}" in
          *flows) printf '%s' '{"data":{"id":"flow","systemId":"s","name":"Monitoring delivery: owner","steps":[{"action":"setUiPreferences","theme":"dark","locale":"en-US"}],"createdAt":"now","updatedAt":"now"}}';;
          *runs/*/steps) printf '%s' '{"data":[{"id":1,"runId":"run","stepIndex":0,"action":"setUiPreferences","status":"succeeded","durationMs":null,"message":null,"createdAt":"now"}]}';;
          *runs) printf '%s' '{"data":{"id":"run","flowId":"flow","status":"queued","error":null,"createdAt":"now","startedAt":null,"finishedAt":null}}';;
        esac; }
        monitor_delivery_wait_run(){ return 0; }
        x=$(monitor_delivery_browser_case owner owner password dark en-US 1 1 text) || exit 1
        test "$x" = run; test -s "$root/delivery-owner-steps.json"
        jq -e '.data.id == "flow" and .data.steps[0].locale == "en-US"' "$root/delivery-owner-flow.json"
        jq -e '.data.id == "run" and .data.flowId == "flow"' "$root/delivery-owner-run.json"
    `;
    const failure = `
        source "$1"; root=$(mktemp -d); trap 'rm -rf "$root"' EXIT
        export RUSTZEN_MONITOR_DELIVERY_EVIDENCE_ROOT="$root"; system=s; admin=http://admin; owner_auth=()
        curl_json(){ return 22; }
        if x=$(monitor_delivery_browser_case owner owner password dark en-US 1 1 text); then exit 1; fi
        test -z "$x"; test ! -e "$root/delivery-owner-steps.json"
    `;
    expect(run(success).exitCode).toBe(0);
    expect(run(failure).exitCode).toBe(0);
});

test("delivery descriptor rejects missing files", () => {
    expect(run(`source "$1"; ! monitor_delivery_descriptor /no/such/file`).exitCode).toBe(0);
});

test("owner auth refresh synchronizes both receipts and fails closed", () => {
    const success = `
        source "$1"; admin=http://admin; login_body='{"username":"owner","password":"rustzen@123"}'
        counter=$(mktemp); trap 'rm -f "$counter"' EXIT; printf 0 >"$counter"
        curl_json(){ calls=$(cat "$counter"); calls=$((calls + 1)); printf %s "$calls" >"$counter"; printf '{"data":{"token":"token-%s"}}' "$calls"; }
        refresh_owner_auth || exit 1
        test "\${auth[1]}" = 'authorization: Bearer token-1'
        test "\${owner_auth[1]}" = 'authorization: Bearer token-1'
        refresh_owner_auth || exit 1
        test "\${auth[1]}" = 'authorization: Bearer token-2'
        test "\${owner_auth[1]}" = 'authorization: Bearer token-2'
    `;
    const failure = `
        source "$1"; admin=http://admin; login_body='{}'; curl_json(){ return 22; }
        ! refresh_owner_auth
    `;
    expect(run(success).exitCode).toBe(0);
    expect(run(failure).exitCode).toBe(0);
});

test("Monitor settlement retains delayed, terminal, and timeout state receipts", () => {
    const script = `
        source "$1"; root=$(mktemp -d); trap 'rm -rf "$root"' EXIT
        database="$root/monitor.db"; terminal="$root/terminal.json"; delayed="$root/delayed.json"; stalled="$root/stalled.json"
        python3 - "$database" <<'PY'
import sqlite3
import sys
with sqlite3.connect(sys.argv[1]) as connection:
    connection.executescript('''
      CREATE TABLE notification_outbox (
        event_id TEXT, state TEXT NOT NULL, attempts INTEGER, next_attempt_at TEXT,
        lease_until TEXT, lease_token TEXT, reconcile_until TEXT, last_error_code TEXT
      );
      CREATE TABLE notification_delivery_status (
        id INTEGER PRIMARY KEY, pending_count INTEGER, pending_bytes INTEGER,
        quarantine_count INTEGER, quarantine_bytes INTEGER, omitted_count INTEGER,
        expired_count INTEGER, unconfirmed_count INTEGER, quarantined_count INTEGER,
        quarantine_evicted_count INTEGER, first_gap_at TEXT, last_gap_at TEXT, last_success_at TEXT
      );
      INSERT INTO notification_delivery_status VALUES (1,0,0,0,0,0,0,0,0,0,NULL,NULL,NULL);
      INSERT INTO notification_outbox VALUES ('terminal','quarantined',0,'next',NULL,NULL,NULL,NULL);
    ''')
PY
        monitor_delivery_settle "$database" terminal "$terminal"
        jq -e '.settled and .outbox.stateCounts.quarantined == 1 and .outbox.activeCount == 0 and (.deliveryStatus | length == 12)' "$terminal"
        python3 - "$database" <<'PY'
import sqlite3
import sys
with sqlite3.connect(sys.argv[1]) as connection:
    connection.execute("UPDATE notification_outbox SET state='pending'")
PY
        (sleep 60.2; python3 - "$database" <<'PY'
import sqlite3
import sys
with sqlite3.connect(sys.argv[1]) as connection:
    connection.execute("DELETE FROM notification_outbox")
PY
        ) &
        monitor_delivery_settle "$database" delayed "$delayed"
        jq -e '.settled and .elapsedMs >= 60000 and .outbox.activeCount == 0' "$delayed"
        python3 - "$database" <<'PY'
import sqlite3
import sys
with sqlite3.connect(sys.argv[1]) as connection:
    connection.execute("INSERT INTO notification_outbox VALUES "
        "('pending','pending',2,'pending-next',NULL,'lease-only',NULL,'pending_error')")
    connection.execute("INSERT INTO notification_outbox VALUES "
        "('reconciling','reconciling',3,'reconcile-next','until-only',NULL,'reconcile-until','relay_failed')")
PY
        if RUSTZEN_MONITOR_DELIVERY_SETTLE_TIMEOUT=1 monitor_delivery_settle "$database" stalled "$stalled"; then exit 1; fi
        jq -e '(.settled | not) and .outbox.stateCounts.pending == 1 and .outbox.stateCounts.reconciling == 1 and .outbox.leasedCount == 1 and .outbox.activeCount == 2 and (.outbox.activeRows == [{eventId:"pending",state:"pending",attempts:2,nextAttemptAt:"pending-next",leaseUntil:null,reconcileUntil:null,lastErrorCode:"pending_error"},{eventId:"reconciling",state:"reconciling",attempts:3,nextAttemptAt:"reconcile-next",leaseUntil:"until-only",reconcileUntil:"reconcile-until",lastErrorCode:"relay_failed"}]) and (.deliveryStatus | length == 12)' "$stalled"
    `;
    expect(run(script).exitCode).toBe(0);
}, 80_000);

test("delivery run failures retain candidate diagnostics and their nonzero code", () => {
    const script = `
        source "$1"; root=$(mktemp -d); trap 'rm -rf "$root"' EXIT
        export RUSTZEN_MONITOR_DELIVERY_EVIDENCE_ROOT="$root"; admin=http://admin; owner_auth=()
        curl_json(){ case "\${!#}" in
          */runs/run) printf '%s' '{"data":{"id":"run","status":"failed","error":"viewer browser failed"}}';;
          */runs/run/steps) printf '%s' '{"data":[{"runId":"run"}]}' ;;
          */runs/run/artifacts) printf '%s' '{"data":[{"id":"artifact"}]}' ;;
        esac; }
        if monitor_delivery_wait_run run viewer; then exit 1; else code=$?; fi
        test "$code" -eq 1
        jq -e '.data.status == "failed" and .data.error == "viewer browser failed"' "$root/.delivery-viewer-final-run.json"
        jq -e '.data[0].runId == "run"' "$root/.delivery-viewer-final-steps.json"
        jq -e '.data[0].id == "artifact"' "$root/.delivery-viewer-final-artifacts.json"
        jq -e '.reason == "terminal" and .status == "failed" and .error == "viewer browser failed"' "$root/.delivery-viewer-final-status.json"
        curl_json(){ case "\${!#}" in
          */runs/run) printf '%s' '{"data":{"id":"run","status":"running","error":null}}';;
          */runs/run/steps) printf '%s' '{"data":[]}' ;;
          */runs/run/artifacts) printf '%s' '{"data":[]}' ;;
        esac; }
        if RUSTZEN_MONITOR_DELIVERY_RUN_POLLS=1 monitor_delivery_wait_run run exhausted; then exit 1; else code=$?; fi
        test "$code" -eq 1
        jq -e '.reason == "exhausted" and .status == "running" and .error == null' "$root/.delivery-exhausted-final-status.json"
        curl_json(){ case "\${!#}" in
          *flows) printf '%s' '{"data":{"id":"flow"}}';;
          *runs) printf '%s' '{"data":{"id":"run"}}';;
        esac; }
        monitor_delivery_wait_run(){ return 37; }
        if monitor_delivery_browser_case preserve owner password dark en-US 1 1 text; then exit 1; else code=$?; fi
        test "$code" -eq 37
    `;
    expect(run(script).exitCode).toBe(0);
});
