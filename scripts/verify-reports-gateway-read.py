#!/usr/bin/env python3
"""Bounded real Admin JWT/RBAC gateway reads of a retained worker-generated run."""
import argparse
import datetime as dt
import hashlib
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


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def expected_run(receipt):
    assert receipt['status'] == 'passed'
    assert receipt['requestCount'] == 8 and receipt['processRestarts'] == 1
    assert not receipt['unexpectedTargetOrNotificationRequests']
    before, after = receipt['observations']
    assert before['run'] == after['run']
    run = after['run']
    assert run['status'] == 'failed' and run['error'] == 'target system is disabled'
    assert after['occurrence']['runId'] == run['id']
    return run


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--plan', type=Path, required=True)
    parser.add_argument('--admin-binary', type=Path, required=True)
    parser.add_argument('--reports-binary', type=Path, required=True)
    parser.add_argument('--source-receipt', type=Path, required=True)
    parser.add_argument('--output-parent', type=Path, default=Path('target/rz/reports-gateway-read'))
    args = parser.parse_args()
    if not __debug__:
        raise RuntimeError('Run without Python optimization')
    plan_bytes = args.plan.read_bytes()
    plan = json.loads(plan_bytes)
    binaries = {'admin': args.admin_binary.resolve(strict=True), 'reports': args.reports_binary.resolve(strict=True)}
    assert digest(__file__) == plan['runnerSHA256']
    assert all(digest(path) == plan['binaryHashes'][name] for name, path in binaries.items())
    assert all(digest(path) == value for path, value in plan['sourceHashes'].items())
    assert digest(args.source_receipt) == plan['sourceReceiptSHA256']
    original_db = args.source_receipt.parent / 'reports.db'
    assert digest(original_db) == plan['sourceDatabaseSHA256']
    expected = expected_run(json.loads(args.source_receipt.read_text()))
    args.output_parent.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix='run-', dir=args.output_parent.resolve()))
    database = output / 'reports.db'
    with sqlite3.connect(f'file:{original_db.resolve()}?mode=ro', uri=True) as source, sqlite3.connect(database) as target:
        source.backup(target)
    with sqlite3.connect(database) as db:
        assert db.execute('SELECT id,status,error FROM automation_runs').fetchall() == [(expected['id'], 'failed', 'target system is disabled')]
        assert db.execute('SELECT enabled FROM automation_systems').fetchall() == [(0,)]
        # Isolate retained read evidence from future polling, leaving the source untouched.
        db.execute('UPDATE automation_schedules SET enabled=0')
    hits = []

    class Sink(BaseHTTPRequestHandler):
        def do_GET(self):
            hits.append(self.command)
            self.send_response(503)
            self.end_headers()
        do_POST = do_GET
        def log_message(self, *_):
            pass

    sink = ThreadingHTTPServer(('127.0.0.1', 0), Sink)
    thread = threading.Thread(target=sink.serve_forever, daemon=True)
    thread.start()
    with socket.socket() as a, socket.socket() as r:
        a.bind(('127.0.0.1', 0))
        r.bind(('127.0.0.1', 0))
        admin_port, reports_port = a.getsockname()[1], r.getsockname()[1]
    env = {k: v for k, v in os.environ.items() if not k.startswith('RUSTZEN_')}
    env.update(RUSTZEN_ENV='development', RUSTZEN_RUNTIME_ROOT=str(output),
               RUSTZEN_ADMIN_SQLITE_PATH=str(output/'admin.db'), RUSTZEN_REPORTS_SQLITE_PATH=str(database),
               RUSTZEN_ADMIN_HOST='127.0.0.1', RUSTZEN_INTERNAL_HOST='127.0.0.1',
               RUSTZEN_ADMIN_PORT=str(admin_port), RUSTZEN_REPORTS_PORT=str(reports_port),
               RUSTZEN_TIMEZONE='UTC', RUSTZEN_IPC_TOKEN='reports-gateway-owned-fixture-only',
               RUSTZEN_REPORTS_BROWSER_PATH=str(output/'nonexistent-browser'),
               RUSTZEN_REPORTS_NOTIFICATION_INGRESS_URL=f'http://127.0.0.1:{sink.server_address[1]}/internal/v1/notification-events')
    processes = []
    logs = []
    receipts = []
    reads = []
    guards = 0
    deadline = time.monotonic()+60
    started = dt.datetime.now(dt.timezone.utc).isoformat()

    def guard():
        nonlocal guards
        assert time.monotonic() < deadline, '60-second dispatch deadline'
        assert all(p.poll() is None for p in processes), 'Owned service exited'
        assert not hits, 'Unexpected notification request'
        for process in processes:
            for path in (Path('/proc')/str(process.pid)/'task').glob('*/children'):
                try:
                    assert not path.read_text().split(), 'Unexpected child process'
                except FileNotFoundError:
                    pass
        with sqlite3.connect(f'file:{database}?mode=ro', uri=True) as db:
            for table, count in [('automation_runs',1),('automation_schedule_occurrences',1),('automation_artifacts',0),('automation_run_steps',0),('notification_outbox',0)]:
                assert db.execute(f'SELECT COUNT(*) FROM {table}').fetchone() == (count,), table
            assert db.execute('SELECT enabled FROM automation_systems').fetchall() == [(0,)]
            assert db.execute('SELECT enabled FROM automation_schedules').fetchall() == [(0,)]
        guards += 1

    def request(case, path, token=None, body=None, method='GET', expected_status=200):
        guard()
        assert len(receipts) < 20, '20 HTTP attempts exhausted'
        row = {'case': case, 'method': method, 'path': path, 'expectedStatus': expected_status}
        receipts.append(row)
        headers = {'content-type':'application/json'}
        if token:
            headers['Authorization'] = 'Bearer '+token
        req = urllib.request.Request(f'http://127.0.0.1:{admin_port}'+path, method=method, headers=headers,
                                     data=json.dumps(body).encode() if body is not None else None)
        try:
            with urllib.request.urlopen(req, timeout=2) as response:
                status, value = response.status, json.load(response)
        except urllib.error.HTTPError as exc:
            status, value = exc.code, json.load(exc)
        row['status'] = status  # Never persist login bodies, headers, credentials or JWTs.
        if expected_status is not None:
            assert status == expected_status, (case,status,value.get('message'))
        guard()
        return status, value

    error = None
    try:
        for name in ['reports','admin']:
            path = output/f'{name}.log'
            log = path.open('w')
            logs.append(log)
            processes.append(subprocess.Popen([str(binaries[name]),'serve'],env=env,cwd=output,stdout=log,stderr=log))
            marker = 'Reports service started' if name == 'reports' else 'Server started successfully'
            startup = time.monotonic()+20
            while marker not in path.read_text():
                guard()
                assert time.monotonic() < startup, name+' startup timeout'
                time.sleep(.05)
        _, login = request('owner-login','/api/auth/login',body={'username':'owner','password':'rustzen@123'},method='POST')
        owner = login['data']['token']
        # Wait for persisted module capability discovery without spending extra HTTP.
        ready = time.monotonic()+10
        while True:
            guard()
            with sqlite3.connect(f'file:{output / "admin.db"}?mode=ro',uri=True) as db:
                found = db.execute("SELECT COUNT(*) FROM menus WHERE code='reports:run:view'").fetchone()[0]
            if found:
                break
            assert time.monotonic() < ready, 'Reports capability discovery timeout'
            time.sleep(.05)
        _, menus = request('module-capabilities','/api/system/menus',owner)
        ids = {row['code']:row['id'] for row in menus['data']}
        path = '/api/reports/runs/'+expected['id']
        for attempt in range(3):
            status, value = request('owner-gateway-ready',path,owner,expected_status=None)
            if status == 200:
                break
            assert status == 503 and attempt < 2, ('gateway readiness',status)
            time.sleep(.1)
        assert value['data'] == expected
        reads.append({'case':'owner','run':value['data']})
        request('unsigned-rejected',path,expected_status=401)
        role = {'name':'Owned Reports reader','code':'owned_reports_reader','status':1,'menuIds':[ids['reports:run:view']],'description':'Local fixture only'}
        request('create-owned-role','/api/system/roles',owner,role,'POST')
        _, listing = request('find-owned-role','/api/system/roles?current=1&pageSize=100',owner)
        role_id = next(row['id'] for row in listing['data'] if row['code']==role['code'])
        request('create-owned-reader','/api/system/users',owner,{'username':'owned_reports_reader','email':'reports-reader@example.invalid','password':'OwnedReportsFixture123!','realName':'Owned fixture','roleIds':[role_id],'status':1},'POST')
        _, login = request('reader-login','/api/auth/login',body={'username':'owned_reports_reader','password':'OwnedReportsFixture123!'},method='POST')
        reader = login['data']['token']
        _, me = request('reader-only-capability','/api/auth/me',reader)
        assert me['data']['permissions'] == ['reports:run:view']
        _, value = request('reader-retained-run',path,reader)
        assert value['data'] == expected
        reads.append({'case':'reader','run':value['data']})
        request('reader-no-schedule','/api/reports/schedules',reader,expected_status=403)
        request('reader-no-run-create','/api/reports/runs',reader,{'flowId':expected['flowId'],'input':{}},'POST',403)
        role['menuIds'] = [ids['reports:schedule:view']]
        request('revoke-reader-run-view',f'/api/system/roles/{role_id}',owner,role,'PUT')
        request('same-session-revoked',path,reader,expected_status=403)
        role['menuIds'] = [ids['reports:run:view']]
        request('restore-reader-run-view',f'/api/system/roles/{role_id}',owner,role,'PUT')
        _, value = request('same-session-restored',path,reader)
        assert value['data'] == expected
        reads.append({'case':'restored','run':value['data']})
        _, value = request('owner-unaffected',path,owner)
        assert value['data'] == expected
        guard()
    except Exception as exc:
        error = f'{type(exc).__name__}: {exc}'
    finally:
        for process in processes:
            if process.poll() is None:
                process.terminate()
        for process in processes:
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
        for log in logs:
            log.close()
        sink.shutdown()
        sink.server_close()
        thread.join(timeout=2)
    result = {'status':'failed' if error else 'passed','error':error,'startedAt':started,
              'finishedAt':dt.datetime.now(dt.timezone.utc).isoformat(),'plan':plan,
              'planSHA256':hashlib.sha256(plan_bytes).hexdigest(),'requestCount':len(receipts),
              'receipts':receipts,'reads':reads,'guards':guards,'unexpectedRequests':hits,
              'sourceDatabaseUnchanged':digest(original_db)==plan['sourceDatabaseSHA256'],
              'sourceHashesMatchAfter':all(digest(p)==h for p,h in plan['sourceHashes'].items()),
              'binaryHashesAfter':{name:digest(path) for name,path in binaries.items()},
              'runnerSHA256After':digest(__file__),'cleanup':'Both owned services and sink stopped',
              'boundaries':'Real Admin owner/custom-reader JWT→gateway→Reports→copied application-generated SQLite data. Serial capability revocation/restoration only. Not rendered UI, browser E2E, successful rendering, all roles or concurrent mutation.'}
    (output/'result.json').write_text(json.dumps(result,indent=2)+'\n')
    print(output/'result.json')
    assert result['sourceDatabaseUnchanged'] and result['sourceHashesMatchAfter']
    assert result['binaryHashesAfter']==plan['binaryHashes'] and result['runnerSHA256After']==plan['runnerSHA256']
    assert args.plan.read_bytes()==plan_bytes
    if error:
        raise SystemExit(error)


if __name__=='__main__':
    main()
