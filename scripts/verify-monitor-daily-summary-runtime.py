#!/usr/bin/env python3
"""Verify real Monitor startup generation against disposable historical inputs.

No summary row is inserted by this verifier. Historical raw telemetry is synthetic;
this proves the unchanged startup worker and delegated read API, not Agent transport,
an elapsed hourly tick, Admin RBAC, systemd, or production deployment.
"""
import argparse
import datetime as dt
import hashlib
import hmac
import json
import math
import os
from pathlib import Path
import socket
import sqlite3
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
import uuid


def seed_database(database, day):
    start = dt.datetime.combine(day, dt.time(), dt.timezone.utc)
    stamp = lambda value: value.isoformat()
    with sqlite3.connect(database) as db:
        db.execute('PRAGMA foreign_keys=ON')
        assert db.execute('SELECT COUNT(*) FROM node_daily_summaries').fetchone()[0] == 0
        db.execute('UPDATE alert_settings SET offline_enabled=0')
        for name, created in [('sampled', start), ('empty', start), ('future', start + dt.timedelta(days=1))]:
            at = stamp(created)
            db.execute('INSERT INTO monitor_nodes(node_id,hostname,agent_version,current_boot_id,last_sequence,last_report_at,last_received_at,cpu_percent,memory_used_bytes,memory_total_bytes,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?)',
                       (name, name, 'fixture', str(uuid.uuid4()), 1, at, at, 0, 0, 100, at, at))
        for i, cpu, memory, root, data in [(1, 10, 30, 20, 40), (2, 30, 70, 40, 60)]:
            at = stamp(start + dt.timedelta(hours=i))
            db.execute('INSERT INTO resource_samples(node_id,cpu_percent,memory_used_bytes,memory_total_bytes,collected_at) VALUES(?,?,?,?,?)',
                       ('sampled', cpu, memory, 100, at))
            for mount, used in [('/', root), ('/data', data)]:
                db.execute('INSERT INTO disk_samples(node_id,mount_point,used_bytes,total_bytes,collected_at) VALUES(?,?,?,?,?)',
                           ('sampled', mount, used, 100, at))
        for name, opened, closed in [('cross-day', start - dt.timedelta(seconds=60), start + dt.timedelta(seconds=60)),
                                      ('within-day', start + dt.timedelta(hours=3), start + dt.timedelta(hours=3, seconds=120))]:
            db.execute('INSERT INTO monitor_incidents(id,node_id,kind,target,status,title,opened_at,last_observed_at,resolved_at) VALUES(?,?,?,?,?,?,?,?,?)',
                       (name, 'sampled', 'nodeOffline', '', 'resolved', 'synthetic fixture', stamp(opened), stamp(closed), stamp(closed)))
        assert db.execute('SELECT COUNT(*) FROM node_daily_summaries').fetchone()[0] == 0


def verify_rows(rows, day, count=2):
    assert len(rows) == 2, rows
    by_node = {row['nodeId']: row for row in rows}
    assert set(by_node) == {'sampled', 'empty'}, by_node
    sampled, empty = by_node['sampled'], by_node['empty']
    assert sampled['date'] == str(day)
    assert sampled['sampleCount'] == count
    assert sampled['cpu'] == {'min': 10.0, 'avg': 20.0, 'max': 30.0}
    assert sampled['memory'] == {'min': 30.0, 'avg': 50.0, 'max': 70.0}
    assert sampled['diskSummary'] == {'/': {'min': 20.0, 'avg': 30.0, 'max': 40.0}, '/data': {'min': 40.0, 'avg': 50.0, 'max': 60.0}}
    assert math.isclose(sampled['coveragePercent'], count / 2880 * 100)
    assert sampled['offlineSeconds'] == 180
    assert sampled['incidentCount'] == 2
    assert empty['sampleCount'] == 0 and empty['coveragePercent'] == 0
    assert empty['cpu'] == {'min': None, 'avg': None, 'max': None}
    assert empty['memory'] == {'min': None, 'avg': None, 'max': None}
    assert empty['diskSummary'] == {} and empty['offlineSeconds'] == 0


def main():
    if not __debug__:
        raise RuntimeError('Run without Python optimization: acceptance assertions must remain enabled')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output-parent', type=Path, default=Path('target/rz/daily-summary-runtime'))
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    args.output_parent.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix='run-', dir=args.output_parent.resolve()))
    database = output / 'monitor.db'
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    secret = 'daily-summary-owned-fixture-ipc-only'
    environment = {key: value for key, value in os.environ.items() if not key.startswith('RUSTZEN_')}
    environment.update(RUSTZEN_ENV='development', RUSTZEN_RUNTIME_ROOT=str(output),
                       RUSTZEN_MONITOR_SQLITE_PATH=str(database), RUSTZEN_INTERNAL_HOST='127.0.0.1',
                       RUSTZEN_MONITOR_PORT=str(port), RUSTZEN_IPC_TOKEN=secret)
    subprocess.run([str(binary), 'init-db'], env=environment, cwd=output, check=True, timeout=30)
    day = dt.datetime.now(dt.timezone.utc).date() - dt.timedelta(days=1)
    seed_database(database, day)
    base = f'http://127.0.0.1:{port}'
    path = '/api/monitor/daily-summaries'

    def request(route, capability=None):
        headers = {}
        if capability:
            timestamp, request_id = str(int(time.time())), str(uuid.uuid4())
            payload = '\n'.join(['1', timestamp, request_id, '1', 'monitor', 'GET', route.split('?')[0], capability])
            headers = {'x-rustzen-contract-version': '1', 'x-rustzen-ipc-timestamp': timestamp,
                       'x-rustzen-request-id': request_id, 'x-rustzen-user-id': '1', 'x-rustzen-module': 'monitor',
                       'x-rustzen-ipc-capability': capability,
                       'x-rustzen-ipc-signature': hmac.new(secret.encode(), payload.encode(), hashlib.sha256).hexdigest()}
        try:
            with urllib.request.urlopen(urllib.request.Request(base + route, headers=headers), timeout=2) as response:
                return response.status, json.load(response)
        except urllib.error.HTTPError as error:
            return error.code, json.load(error)

    receipts = []
    for iteration in range(2):
        expected_count = 2 + iteration
        if iteration:
            # A new raw observation proves the second worker tick actually ran;
            # merely reading pre-existing summary rows would not establish that.
            at = dt.datetime.combine(day, dt.time(4), dt.timezone.utc).isoformat()
            with sqlite3.connect(database) as db:
                db.execute('INSERT INTO resource_samples(node_id,cpu_percent,memory_used_bytes,memory_total_bytes,collected_at) VALUES(?,?,?,?,?)',
                           ('sampled', 20, 50, 100, at))
        with (output / f'controller-{iteration}.log').open('w') as log:
            process = subprocess.Popen([str(binary), 'controller'], cwd=output, env=environment, stdout=log, stderr=log)
            try:
                deadline = time.monotonic() + 30
                while True:
                    assert process.poll() is None, f'controller exited; inspect {log.name}'
                    try:
                        status, body = request(path, 'monitor:node:view')
                        if status == 200 and body['data']['total'] == 2 and any(row['nodeId'] == 'sampled' and row['sampleCount'] == expected_count for row in body['data']['data']):
                            break
                    except (OSError, ValueError):
                        pass
                    assert time.monotonic() < deadline, f'generation deadline exceeded; inspect {log.name}'
                    time.sleep(.1)
                rows = body['data']['data']
                verify_rows(rows, day, expected_count)
                assert request(path)[0] == 401
                assert request(path, 'monitor:incident:view')[0] == 403
                with sqlite3.connect(database) as db:
                    stored = db.execute('SELECT node_id,summary_date,sample_count,offline_seconds,incident_count FROM node_daily_summaries ORDER BY node_id').fetchall()
                assert stored == [('empty', str(day), 0, 0, 0), ('sampled', str(day), expected_count, 180, 2)]
                receipts.append({'iteration': iteration, 'rows': rows, 'databaseRows': stored, 'unsignedStatus': 401, 'wrongCapabilityStatus': 403})
            finally:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=10)
    result = {'status': 'passed', 'binarySha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
              'day': str(day), 'receipts': receipts,
              'boundaries': 'Synthetic historical input; real unchanged startup worker, SQLite persistence, delegated HTTP, and restart upsert. Hourly elapsed tick, Agent transport, Admin RBAC, systemd and production are not verified.'}
    (output / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': 'passed', 'result': str(output / 'result.json')}))


if __name__ == '__main__':
    main()
