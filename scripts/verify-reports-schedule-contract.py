#!/usr/bin/env python3
"""Finite future-only Reports schedule API acceptance; no browser/job execution."""
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--plan', type=Path, required=True)
    parser.add_argument('--output-parent', type=Path, default=Path('target/rz/reports-schedule-contract'))
    args = parser.parse_args()
    if not __debug__: raise RuntimeError('Run without Python optimization')
    binary = args.binary.resolve(strict=True)
    plan_bytes = args.plan.read_bytes(); plan = json.loads(plan_bytes)
    plan_sha = hashlib.sha256(plan_bytes).hexdigest()
    assert digest(binary) == plan['binarySha256'], 'Binary differs from pre-run plan'
    assert digest(__file__) == plan['runnerSha256'], 'Runner differs from pre-run plan'
    args.output_parent.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix='run-', dir=args.output_parent.resolve()))
    database = output / 'reports.db'
    hits = []
    class RejectUnexpectedRequest(BaseHTTPRequestHandler):
        def do_GET(self):
            hits.append({'method': self.command, 'path': self.path})
            self.send_response(503); self.end_headers()
        do_POST = do_GET
        def log_message(self, *_): pass
    sink = ThreadingHTTPServer(('127.0.0.1', 0), RejectUnexpectedRequest)
    sink_thread = threading.Thread(target=sink.serve_forever, daemon=True); sink_thread.start()
    sink_url = f'http://127.0.0.1:{sink.server_address[1]}'
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0)); port = sock.getsockname()[1]
    secret = 'reports-schedule-owned-ipc-fixture-only'
    environment = {key: value for key, value in os.environ.items() if not key.startswith('RUSTZEN_')}
    environment.update(RUSTZEN_ENV='development', RUSTZEN_RUNTIME_ROOT=str(output), RUSTZEN_REPORTS_SQLITE_PATH=str(database),
                       RUSTZEN_REPORTS_PORT=str(port), RUSTZEN_INTERNAL_HOST='127.0.0.1', RUSTZEN_TIMEZONE='UTC',
                       RUSTZEN_IPC_TOKEN=secret, RUSTZEN_REPORTS_NOTIFICATION_INGRESS_URL=sink_url + '/internal/v1/notification-events')
    receipts = []; guard_checks = 0; process = None
    started = dt.datetime.now(dt.timezone.utc).isoformat()
    zero_tables = ['automation_runs', 'automation_schedule_occurrences', 'automation_artifacts', 'notification_outbox']

    def guard():
        nonlocal guard_checks
        assert process.poll() is None, 'Owned Reports service exited'
        assert not hits, 'Unexpected target or notification request; stop this experiment'
        children = []
        for path in (Path('/proc') / str(process.pid) / 'task').glob('*/children'):
            try: children.extend(path.read_text().split())
            except FileNotFoundError: pass
        assert not children, 'Unexpected child process (including browser); stop this experiment'
        with sqlite3.connect(f'file:{database}?mode=ro', uri=True) as db:
            counts = {table: db.execute(f'SELECT COUNT(*) FROM {table}').fetchone()[0] for table in zero_tables}
        assert all(count == 0 for count in counts.values()), counts
        guard_checks += 1
        return counts

    def request(method, path, capability, body=None, expected=200):
        assert len(receipts) < 24, 'The 24-HTTP-request budget is exhausted'
        guard()
        headers = {'content-type': 'application/json'}
        if capability is not None:
            timestamp, request_id = str(int(time.time())), str(uuid.uuid4())
            payload = '\n'.join(['1', timestamp, request_id, '1', 'reports', method, path, capability])
            headers.update({'x-rustzen-contract-version': '1', 'x-rustzen-ipc-timestamp': timestamp,
                            'x-rustzen-request-id': request_id, 'x-rustzen-user-id': '1', 'x-rustzen-module': 'reports',
                            'x-rustzen-ipc-capability': capability, 'x-rustzen-ipc-signature': hmac.new(secret.encode(), payload.encode(), hashlib.sha256).hexdigest()})
        req = urllib.request.Request(f'http://127.0.0.1:{port}' + path, method=method, headers=headers,
                                     data=json.dumps(body).encode() if body is not None else None)
        try:
            with urllib.request.urlopen(req, timeout=2) as response: status, value = response.status, json.load(response)
        except urllib.error.HTTPError as error: status, value = error.code, json.load(error)
        receipts.append({'method': method, 'path': path, 'capability': capability, 'status': status, 'body': value})
        assert status == expected, (path, expected, status, value)
        guard()
        return value.get('data')

    def future_due(value):
        assert value['enabled'] and value['timezone'] == 'UTC'
        due = dt.datetime.fromisoformat(value['nextDue'].replace('Z', '+00:00'))
        assert (due - dt.datetime.now(dt.timezone.utc)).total_seconds() >= 7200
        assert value['lastOccurrence'] is None and value['lastRun'] is None

    log = (output / 'reports.log').open('w')
    failure = None
    try:
        process = subprocess.Popen([str(binary), 'serve'], env=environment, cwd=output, stdout=log, stderr=log)
        deadline = time.monotonic() + 30
        while 'Reports service started' not in (output / 'reports.log').read_text():
            assert process.poll() is None, f'Reports startup failed; inspect {output}'
            assert time.monotonic() < deadline, 'Reports startup deadline exceeded'
            time.sleep(.05)
        guard()
        view, manage = 'reports:schedule:view', 'reports:schedule:manage'
        assert request('GET', '/api/reports/settings', view)['timezone'] == 'UTC'
        system = request('POST', '/api/reports/systems', 'reports:system:manage', {'name': 'Owned schedule target', 'baseUrl': sink_url, 'enabled': True})
        flow = request('POST', '/api/reports/flows', 'reports:flow:manage', {'systemId': system['id'], 'name': 'Never executed schedule fixture', 'steps': [{'action': 'goto', 'url': '/owned-fixture'}]})
        now = dt.datetime.now(dt.timezone.utc)
        daily_input = {'flowId': flow['id'], 'cadence': 'daily', 'weekday': None, 'dueTime': (now + dt.timedelta(hours=3)).strftime('%H:%M'), 'input': {}, 'enabled': False}
        daily = request('POST', '/api/reports/schedules', manage, daily_input)
        assert daily['enabled'] is False and daily['nextDue'] is None
        daily_path = '/api/reports/schedules/' + daily['id']
        assert request('GET', daily_path, view)['id'] == daily['id']
        request('PUT', daily_path, view, daily_input, expected=403)
        future_due(request('PUT', daily_path, manage, {**daily_input, 'enabled': True}))
        disabled = request('PUT', daily_path, manage, daily_input)
        assert disabled['enabled'] is False and disabled['nextDue'] is None
        weekly_slot = now + dt.timedelta(days=1, hours=3)
        weekly_input = {**daily_input, 'cadence': 'weekly', 'weekday': weekly_slot.weekday(), 'dueTime': weekly_slot.strftime('%H:%M')}
        weekly = request('POST', '/api/reports/schedules', manage, weekly_input)
        assert weekly['enabled'] is False and weekly['nextDue'] is None
        weekly_path = '/api/reports/schedules/' + weekly['id']
        future_due(request('PUT', weekly_path, manage, {**weekly_input, 'enabled': True}))
        assert len(request('GET', '/api/reports/schedules', view)) == 2
        invalid = [({**daily_input, 'cadence': 'monthly'}, 422),
                   ({**daily_input, 'weekday': 1}, 400), ({**weekly_input, 'weekday': None}, 400),
                   ({**daily_input, 'dueTime': '25:61'}, 400),
                   ({**daily_input, 'input': {'password': 'synthetic-non-secret'}}, 400)]
        for body, expected in invalid:
            request('POST', '/api/reports/schedules', manage, body, expected)
        assert len(request('GET', '/api/reports/schedules', view)) == 2
        request('DELETE', daily_path, manage)
        request('GET', daily_path, view, expected=404)
        request('DELETE', weekly_path, manage)
        assert request('GET', '/api/reports/schedules', view) == []
        request('GET', '/api/reports/schedules', None, expected=401)
        final_counts = guard()
    except Exception as error:
        failure = f'{type(error).__name__}: {error}'
    finally:
        if process is not None and process.poll() is None:
            process.terminate()
            try: process.wait(timeout=10)
            except subprocess.TimeoutExpired: process.kill(); process.wait(timeout=10)
        log.close(); sink.shutdown(); sink.server_close(); sink_thread.join(timeout=2)
    result = {'status': 'failed' if failure else 'passed', 'error': failure, 'receipts': receipts, 'requestCount': len(receipts),
              'plan': plan, 'planSha256': plan_sha, 'startedAt': started, 'completedAt': dt.datetime.now(dt.timezone.utc).isoformat(),
              'runnerSha256After': digest(__file__), 'binarySha256After': digest(binary), 'guardChecks': guard_checks,
              'unexpectedTargetOrNotificationRequests': hits, 'cleanup': 'Owned Reports process and loopback sink stopped',
              'boundaries': 'Real signed Reports module HTTP only. Future schedules never become due; no queued run/browser/notification delivery. Not Admin JWT/RBAC, browser UI, due-occurrence execution, rendering/PDF or production.'}
    if not failure: result['finalZeroRowCounts'] = final_counts
    assert digest(binary) == plan['binarySha256'] and digest(__file__) == plan['runnerSha256']
    assert hashlib.sha256(args.plan.read_bytes()).hexdigest() == plan_sha
    (output / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': result['status'], 'requestCount': len(receipts), 'result': str(output / 'result.json')}))
    if failure: raise RuntimeError(failure)


if __name__ == '__main__':
    main()
