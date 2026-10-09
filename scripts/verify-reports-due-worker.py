#!/usr/bin/env python3
"""One real scheduled-run failure on an owned disabled target; no browser/delivery."""
import argparse
import datetime as dt
import hashlib
import hmac
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import socket
import sqlite3
import subprocess
import tempfile
import threading
import time
import urllib.error
import urllib.request
import uuid


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def due_slot(now):
    """UTC whole minute strictly 30..90 seconds ahead, without changing any clock."""
    return (now + dt.timedelta(seconds=90)).replace(second=0, microsecond=0)


def assert_terminal(row):
    assert row['status'] == 'failed', row
    assert row['error'] == 'target system is disabled', row
    assert row['startedAt'] and row['finishedAt'], row


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--plan', type=Path, required=True)
    parser.add_argument('--output-parent', type=Path, default=Path('target/rz/reports-due-worker'))
    args = parser.parse_args()
    if not __debug__:
        raise RuntimeError('Run without Python optimization')
    binary = args.binary.resolve(strict=True)
    plan_bytes = args.plan.read_bytes()
    plan = json.loads(plan_bytes)
    assert digest(binary) == plan['binarySHA256']
    assert digest(__file__) == plan['runnerSHA256']
    for path, expected in plan['sourceHashes'].items():
        assert digest(path) == expected, path
    args.output_parent.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix='run-', dir=args.output_parent.resolve()))
    database = output / 'reports.db'
    hits = []

    class RejectRequest(BaseHTTPRequestHandler):
        def do_GET(self):
            hits.append({'method': self.command, 'path': self.path})
            self.send_response(503)
            self.end_headers()
        do_POST = do_GET
        def log_message(self, *_):
            pass

    sink = ThreadingHTTPServer(('127.0.0.1', 0), RejectRequest)
    sink_thread = threading.Thread(target=sink.serve_forever, daemon=True)
    sink_thread.start()
    sink_url = f'http://127.0.0.1:{sink.server_address[1]}'
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    fixture_secret = 'reports-due-owned-ipc-fixture-only'
    environment = {key: value for key, value in os.environ.items() if not key.startswith('RUSTZEN_')}
    environment.update(RUSTZEN_ENV='development', RUSTZEN_RUNTIME_ROOT=str(output),
                       RUSTZEN_REPORTS_SQLITE_PATH=str(database), RUSTZEN_REPORTS_PORT=str(port),
                       RUSTZEN_INTERNAL_HOST='127.0.0.1', RUSTZEN_TIMEZONE='UTC',
                       RUSTZEN_IPC_TOKEN=fixture_secret,
                       RUSTZEN_REPORTS_BROWSER_PATH=str(output / 'nonexistent-browser'),
                       RUSTZEN_REPORTS_NOTIFICATION_INGRESS_URL=sink_url + '/internal/v1/notification-events')
    process = None
    logs = []
    requests = []
    observations = []
    guards = 0
    restarts = 0
    started = dt.datetime.now(dt.timezone.utc).isoformat()
    whole_deadline = time.monotonic() + 180

    def guard(schema=True):
        nonlocal guards
        assert time.monotonic() < whole_deadline, '180-second whole-run dispatch deadline'
        assert process is not None and process.poll() is None, 'Owned service exited'
        assert not hits, 'Unexpected target/notification request'
        for path in (Path('/proc') / str(process.pid) / 'task').glob('*/children'):
            try:
                assert not path.read_text().split(), 'Unexpected child process'
            except FileNotFoundError:
                pass
        if schema:
            with sqlite3.connect(f'file:{database}?mode=ro', uri=True) as db:
                counts = {table: db.execute(f'SELECT COUNT(*) FROM {table}').fetchone()[0]
                          for table in ['automation_artifacts', 'automation_run_steps', 'notification_outbox']}
            assert all(value == 0 for value in counts.values()), counts
        guards += 1

    def stop():
        if process is not None and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)

    def start(index):
        nonlocal process
        path = output / f'service-{index}.log'
        log = path.open('w')
        logs.append(log)
        process = subprocess.Popen([str(binary), 'serve'], env=environment, cwd=output,
                                   stdout=log, stderr=log)
        deadline = time.monotonic() + 15
        while 'Reports service started' not in path.read_text():
            guard(False)
            assert time.monotonic() < deadline, 'Startup deadline'
            time.sleep(.05)
        guard()

    def request(method, path, capability, body=None):
        guard()
        assert len(requests) < 8, '8-request budget exhausted'
        timestamp, request_id = str(int(time.time())), str(uuid.uuid4())
        payload = '\n'.join(['1', timestamp, request_id, '1', 'reports', method, path, capability])
        headers = {'content-type': 'application/json', 'x-rustzen-contract-version': '1',
                   'x-rustzen-ipc-timestamp': timestamp, 'x-rustzen-request-id': request_id,
                   'x-rustzen-user-id': '1', 'x-rustzen-module': 'reports',
                   'x-rustzen-ipc-capability': capability,
                   'x-rustzen-ipc-signature': hmac.new(fixture_secret.encode(), payload.encode(), hashlib.sha256).hexdigest()}
        receipt = {'method': method, 'path': path, 'capability': capability}
        requests.append(receipt)  # Count attempts, including network failures.
        req = urllib.request.Request(f'http://127.0.0.1:{port}' + path, method=method,
                                     headers=headers, data=json.dumps(body).encode() if body is not None else None)
        try:
            with urllib.request.urlopen(req, timeout=2) as response:
                status, value = response.status, json.load(response)
        except urllib.error.HTTPError as error:
            status, value = error.code, json.load(error)
        receipt.update(status=status, body=value)
        assert status == 200, receipt
        guard()
        return value['data']

    error = None
    try:
        start(0)
        target_body = {'name': 'Owned disabled-at-due fixture', 'baseUrl': sink_url, 'enabled': True}
        target = request('POST', '/api/reports/systems', 'reports:system:manage', target_body)
        flow = request('POST', '/api/reports/flows', 'reports:flow:manage',
                       {'systemId': target['id'], 'name': 'Never reaches browser', 'steps': [{'action': 'goto', 'url': '/owned-fixture'}]})
        due = due_slot(dt.datetime.now(dt.timezone.utc))
        schedule = request('POST', '/api/reports/schedules', 'reports:schedule:manage',
                           {'flowId': flow['id'], 'cadence': 'daily', 'weekday': None,
                            'dueTime': due.strftime('%H:%M'), 'input': {'fixture': 'owned'}, 'enabled': True})
        assert (due - dt.datetime.now(dt.timezone.utc)).total_seconds() >= 25, 'Insufficient disable safety margin'
        disabled = request('PUT', '/api/reports/systems/' + target['id'], 'reports:system:manage',
                           {**target_body, 'enabled': False})
        assert disabled['enabled'] is False
        assert (due - dt.datetime.now(dt.timezone.utc)).total_seconds() >= 20, 'Disable confirmation too close to due'
        with sqlite3.connect(f'file:{database}?mode=ro', uri=True) as db:
            assert db.execute('SELECT enabled FROM automation_systems WHERE id=?', (target['id'],)).fetchone() == (0,)
            assert db.execute('SELECT COUNT(*) FROM automation_runs').fetchone() == (0,)
        deadline = time.monotonic() + 120
        while True:
            guard()
            with sqlite3.connect(f'file:{database}?mode=ro', uri=True) as db:
                rows = db.execute('SELECT id,status,error FROM automation_runs').fetchall()
                occurrences = db.execute('SELECT decision,run_id,due_at FROM automation_schedule_occurrences').fetchall()
            assert len(rows) <= 1 and len(occurrences) <= 1, 'Duplicate due decision/run'
            if rows and rows[0][1] == 'failed':
                assert rows[0][2] == 'target system is disabled'
                assert len(occurrences) == 1 and occurrences[0][:2] == ('enqueued', rows[0][0])
                assert dt.datetime.fromisoformat(occurrences[0][2]) == due
                run_id = rows[0][0]
                break
            assert not rows or rows[0][1] in ('queued', 'running'), rows
            assert time.monotonic() < deadline, '120-second terminal observation deadline'
            time.sleep(.1)
        schedule_path = '/api/reports/schedules/' + schedule['id']
        first_schedule = request('GET', schedule_path, 'reports:schedule:view')
        first_run = request('GET', '/api/reports/runs/' + run_id, 'reports:run:view')
        assert_terminal(first_run)
        assert first_schedule['lastOccurrence']['runId'] == run_id
        assert first_schedule['lastRun'] == first_run
        observations.append({'phase': 'before-restart', 'run': first_run, 'occurrence': first_schedule['lastOccurrence']})
        stop()
        restarts += 1
        start(1)
        repeat_until = time.monotonic() + 16  # Includes startup poll and a full 15s periodic interval.
        while time.monotonic() < repeat_until:
            guard()
            time.sleep(.1)
        second_run = request('GET', '/api/reports/runs/' + run_id, 'reports:run:view')
        second_schedule = request('GET', schedule_path, 'reports:schedule:view')
        assert second_run == first_run
        assert second_schedule['lastOccurrence'] == first_schedule['lastOccurrence']
        assert second_schedule['lastRun'] == second_run
        with sqlite3.connect(f'file:{database}?mode=ro', uri=True) as db:
            assert db.execute('SELECT COUNT(*) FROM automation_runs').fetchone() == (1,)
            assert db.execute('SELECT COUNT(*) FROM automation_schedule_occurrences').fetchone() == (1,)
            assert db.execute('SELECT initiator_user_id FROM automation_runs').fetchone() == (None,)
            assert db.execute('PRAGMA quick_check').fetchone() == ('ok',)
        observations.append({'phase': 'after-restart', 'run': second_run, 'occurrence': second_schedule['lastOccurrence']})
        guard()
    except Exception as exc:
        error = f'{type(exc).__name__}: {exc}'
    finally:
        stop()
        for log in logs:
            log.close()
        sink.shutdown()
        sink.server_close()
        sink_thread.join(timeout=2)
    result = {'status': 'failed' if error else 'passed', 'error': error, 'startedAt': started,
              'finishedAt': dt.datetime.now(dt.timezone.utc).isoformat(), 'planSHA256': hashlib.sha256(plan_bytes).hexdigest(),
              'plan': plan, 'requestCount': len(requests), 'requests': requests, 'observations': observations,
              'guards': guards, 'processRestarts': restarts, 'unexpectedTargetOrNotificationRequests': hits,
              'binarySHA256After': digest(binary), 'runnerSHA256After': digest(__file__),
              'sourceHashesMatchAfter': all(digest(p) == h for p, h in plan['sourceHashes'].items()),
              'cleanup': 'Owned service process stopped and loopback sink joined',
              'boundaries': 'Real scheduled failure and OS process restart, signed module HTTP/SQLite only; disabled target prevents browser execution. Not successful rendering, Admin gateway/RBAC or browser-entry E2E.'}
    (output / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(output / 'result.json')
    assert result['binarySHA256After'] == plan['binarySHA256']
    assert result['runnerSHA256After'] == plan['runnerSHA256']
    assert result['sourceHashesMatchAfter']
    assert args.plan.read_bytes() == plan_bytes
    if error:
        raise SystemExit(error)


if __name__ == '__main__':
    main()
