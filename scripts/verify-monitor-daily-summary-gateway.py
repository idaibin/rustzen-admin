#!/usr/bin/env python3
"""Real Admin login and Monitor gateway paging over application-generated summaries.

Both processes and HTTP checks share one execution environment. Only raw historical
inputs are synthetic. This is API integration acceptance, not browser/UI acceptance.
"""
import argparse
import datetime as dt
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import socket
import sqlite3
import subprocess
import tempfile
import time
import urllib.error
import urllib.request

SPEC = importlib.util.spec_from_file_location(
    'daily_runtime', Path(__file__).with_name('verify-monitor-daily-summary-runtime.py'))
FIXTURE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(FIXTURE)


def main():
    if not __debug__:
        raise RuntimeError('Run without Python optimization: acceptance assertions must remain enabled')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--admin-binary', type=Path, required=True)
    parser.add_argument('--monitor-binary', type=Path, required=True)
    parser.add_argument('--verify-overlap', action='store_true', help='Exactly one four-GET cohort; no automatic replay')
    parser.add_argument('--observe-reads', action='store_true', help='Fixed 100-GET local read observation; no performance SLO')
    parser.add_argument('--verify-recovery', action='store_true', help='Stop/restart only this fixture Monitor once')
    parser.add_argument('--plan', type=Path, help='Pre-execution acceptance plan; required for recovery')
    parser.add_argument('--verify-roles', action='store_true', help='Also run owned non-owner role/revocation cases')
    parser.add_argument('--output-parent', type=Path, default=Path('target/rz/daily-summary-gateway'))
    args = parser.parse_args()
    if (args.verify_recovery or args.observe_reads or args.verify_overlap) and not args.plan:
        parser.error('--verify-recovery/--observe-reads/--verify-overlap requires a pre-execution --plan')
    plan_bytes = args.plan.read_bytes() if args.plan else None
    plan = json.loads(plan_bytes) if plan_bytes else None
    plan_sha256 = hashlib.sha256(plan_bytes).hexdigest() if plan_bytes else None
    started_at = dt.datetime.now(dt.timezone.utc).isoformat()
    binaries = {'admin': args.admin_binary.resolve(strict=True), 'monitor': args.monitor_binary.resolve(strict=True)}
    args.output_parent.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix='run-', dir=args.output_parent.resolve()))
    database = output / 'monitor.db'
    # Reserve distinct ephemeral ports together before passing them to the services.
    with socket.socket() as admin_socket, socket.socket() as monitor_socket:
        admin_socket.bind(('127.0.0.1', 0))
        monitor_socket.bind(('127.0.0.1', 0))
        admin_port, monitor_port = admin_socket.getsockname()[1], monitor_socket.getsockname()[1]
    environment = {key: value for key, value in os.environ.items() if not key.startswith('RUSTZEN_')}
    environment.update(RUSTZEN_ENV='development', RUSTZEN_RUNTIME_ROOT=str(output),
                       RUSTZEN_MONITOR_SQLITE_PATH=str(database), RUSTZEN_ADMIN_SQLITE_PATH=str(output / 'admin.db'),
                       RUSTZEN_ADMIN_HOST='127.0.0.1', RUSTZEN_INTERNAL_HOST='127.0.0.1',
                       RUSTZEN_ADMIN_PORT=str(admin_port), RUSTZEN_MONITOR_PORT=str(monitor_port),
                       RUSTZEN_IPC_TOKEN='daily-summary-gateway-owned-fixture-only')
    subprocess.run([str(binaries['monitor']), 'init-db'], env=environment, cwd=output, check=True, timeout=30)
    day = dt.datetime.now(dt.timezone.utc).date() - dt.timedelta(days=1)
    FIXTURE.seed_database(database, day)
    with sqlite3.connect(database) as db:
        for i in range(21):
            db.execute("INSERT INTO monitor_nodes SELECT ?,?,agent_version,current_boot_id,last_sequence,last_report_at,last_received_at,cpu_percent,memory_used_bytes,memory_total_bytes,created_at,updated_at FROM monitor_nodes WHERE node_id='empty'",
                       (f'pagination-{i:02}', f'pagination-{i:02}'))
        assert db.execute('SELECT COUNT(*) FROM node_daily_summaries').fetchone()[0] == 0

    def request(path, token=None, body=None, method=None):
        headers = {'content-type': 'application/json'}
        if token:
            headers['Authorization'] = 'Bearer ' + token
        try:
            req = urllib.request.Request(f'http://127.0.0.1:{admin_port}' + path, headers=headers,
                                         data=json.dumps(body).encode() if body else None, method=method)
            with urllib.request.urlopen(req, timeout=2) as response:
                return response.status, json.load(response)
        except urllib.error.HTTPError as error:
            return error.code, json.load(error)

    processes, logs = [], []
    try:
        for name, command in [('monitor', 'controller'), ('admin', 'serve')]:
            log = (output / f'{name}.log').open('w')
            logs.append(log)
            processes.append(subprocess.Popen([str(binaries[name]), command], env=environment, cwd=output, stdout=log, stderr=log))
        deadline = time.monotonic() + 30
        while True:
            assert all(process.poll() is None for process in processes), f'Service exited; inspect {output}'
            try:
                if request('/health')[0] == 200:
                    break
            except OSError:
                pass
            assert time.monotonic() < deadline, f'Health deadline exceeded; inspect {output}'
            time.sleep(.1)
        status, login = request('/api/auth/login', body={'username': 'owner', 'password': 'rustzen@123'})
        assert status == 200, (status, login.get('message'))
        token = login['data']['token']  # Owned development fixture; never written to evidence.
        # /health does not prove asynchronous module discovery and generation.
        # Poll the actual authorized gateway postcondition before paging assertions.
        deadline = time.monotonic() + 30
        readiness_attempts = 0
        while True:
            readiness_attempts += 1
            assert all(process.poll() is None for process in processes), f'Service exited; inspect {output}'
            try:
                status, body = request('/api/monitor/daily-summaries?current=1&pageSize=20', token)
                if status == 200 and isinstance(body.get('data'), dict) and body['data'].get('total') == 23:
                    break
            except OSError:
                pass
            assert time.monotonic() < deadline, f'Gateway generation readiness deadline exceeded; inspect {output}'
            time.sleep(.1)
        receipts = []
        for page in [1, 2, 1]:
            status, body = request(f'/api/monitor/daily-summaries?current={page}&pageSize=20', token)
            assert status == 200, body
            assert body['data']['total'] == 23, body
            rows = body['data']['data']
            assert len(rows) == (20 if page == 1 else 3)
            receipts.append({'page': page, 'status': status, 'body': body})
        all_rows = receipts[0]['body']['data']['data'] + receipts[1]['body']['data']['data']
        assert len({row['nodeId'] for row in all_rows}) == 23
        FIXTURE.verify_rows([row for row in all_rows if row['nodeId'] in ('empty', 'sampled')], day)
        assert receipts[0]['body'] == receipts[2]['body']
        role_receipts = []
        if args.verify_roles:
            from monitor_daily_summary_roles import verify_role_access
            role_receipts = verify_role_access(request, token)
        recovery_receipt = None
        if args.verify_recovery:
            from monitor_daily_summary_recovery import verify_recovery
            def restart_monitor():
                log = (output / 'monitor-restarted.log').open('w')
                logs.append(log)
                return subprocess.Popen([str(binaries['monitor']), 'controller'], env=environment, cwd=output, stdout=log, stderr=log)
            recovery_receipt = verify_recovery(request, token, processes, restart_monitor)
        read_observation = None
        if args.observe_reads:
            from monitor_daily_summary_load import verify_read_observation
            read_observation = verify_read_observation(request, token, processes, receipts, output)
        overlap_receipt = None
        if args.verify_overlap:
            from monitor_daily_summary_overlap import verify_overlap
            overlap_receipt = verify_overlap(request, token, receipts, output, plan_sha256)
        unsigned_status = request('/api/monitor/daily-summaries')[0]
        assert unsigned_status == 401
        with sqlite3.connect(database) as db:
            assert db.execute('SELECT COUNT(*) FROM node_daily_summaries').fetchone()[0] == 23
    finally:
        for process in processes:
            if process.poll() is None:
                process.terminate()
        for process in processes:
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)
        for log in logs:
            log.close()
    result = {'status': 'passed', 'overlapReceipt': overlap_receipt, 'readObservation': read_observation, 'planSha256': plan_sha256, 'plan': plan, 'recoveryReceipt': recovery_receipt, 'roleReceipts': role_receipts, 'day': str(day),
              'startedAt': started_at, 'completedAt': dt.datetime.now(dt.timezone.utc).isoformat(),
              'verifierSha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              'fixtureSha256': hashlib.sha256(Path(FIXTURE.__file__).read_bytes()).hexdigest(), 'receipts': receipts, 'unsignedStatus': unsigned_status,
              'binaries': {name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in binaries.items()},
              'cleanup': 'Both owned processes stopped', 'readinessAttempts': readiness_attempts,
              'boundaries': ('Real development-owner plus two synthetic custom roles and serial grant/revoke/disable checks. ' if args.verify_roles else 'Real development-owner only. ') + 'JWT/gateway/module/SQLite with raw telemetry fixtures. Browser/UI, all other roles, unlisted concurrency behavior, performance budgets, other modules, systemd, hourly elapsed ticks and production not verified.'}
    if args.plan:
        assert hashlib.sha256(args.plan.read_bytes()).hexdigest() == plan_sha256, 'Plan changed during execution'
    if args.verify_overlap:
        result['overlapVerifierSha256'] = hashlib.sha256(Path(__file__).with_name('monitor_daily_summary_overlap.py').read_bytes()).hexdigest()
    if args.observe_reads:
        result['loadVerifierSha256'] = hashlib.sha256(Path(__file__).with_name('monitor_daily_summary_load.py').read_bytes()).hexdigest()
    if args.verify_recovery:
        result['recoveryVerifierSha256'] = hashlib.sha256(Path(__file__).with_name('monitor_daily_summary_recovery.py').read_bytes()).hexdigest()
    if args.verify_roles:
        result['roleVerifierSha256'] = hashlib.sha256(Path(__file__).with_name('monitor_daily_summary_roles.py').read_bytes()).hexdigest()
    (output / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': 'passed', 'result': str(output / 'result.json')}))


if __name__ == '__main__':
    main()
