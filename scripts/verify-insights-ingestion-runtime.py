#!/usr/bin/env python3
"""Start only owned Admin/Insights services for the finite ingestion gateway gate."""
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--admin-binary', required=True, type=Path)
    parser.add_argument('--insights-binary', required=True, type=Path)
    parser.add_argument('--plan', required=True, type=Path)
    parser.add_argument('--output-parent', type=Path, default=Path('target/rz/insights-ingestion-runtime'))
    args = parser.parse_args()
    if not __debug__:
        raise RuntimeError('Run without Python optimization')
    plan_bytes = args.plan.read_bytes()
    plan = json.loads(plan_bytes)
    plan_hash = hashlib.sha256(plan_bytes).hexdigest()
    binaries = {'admin': args.admin_binary.resolve(strict=True), 'insights': args.insights_binary.resolve(strict=True)}
    args.output_parent.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix='run-', dir=args.output_parent.resolve()))
    with socket.socket() as first, socket.socket() as second:
        first.bind(('127.0.0.1', 0)); second.bind(('127.0.0.1', 0))
        admin_port, insights_port = first.getsockname()[1], second.getsockname()[1]
    environment = {key: value for key, value in os.environ.items() if not key.startswith('RUSTZEN_')}
    environment.update(RUSTZEN_ENV='development', RUSTZEN_RUNTIME_ROOT=str(output),
                       RUSTZEN_ADMIN_SQLITE_PATH=str(output / 'admin.db'), RUSTZEN_INSIGHTS_SQLITE_PATH=str(output / 'insights.db'),
                       RUSTZEN_ADMIN_HOST='127.0.0.1', RUSTZEN_INTERNAL_HOST='127.0.0.1',
                       RUSTZEN_ADMIN_PORT=str(admin_port), RUSTZEN_INSIGHTS_PORT=str(insights_port),
                       RUSTZEN_IPC_TOKEN='insights-owned-runtime-fixture-only')
    processes, logs = [], []
    started_at = dt.datetime.now(dt.timezone.utc).isoformat()
    try:
        for name, ready_text in [('insights', 'Insights service started'), ('admin', 'Server started successfully')]:
            log_path = output / f'{name}.log'
            log = log_path.open('w'); logs.append(log)
            process = subprocess.Popen([str(binaries[name]), 'serve'], cwd=output, env=environment, stdout=log, stderr=log)
            processes.append(process)
            deadline = time.monotonic() + 30
            while ready_text not in log_path.read_text():
                assert process.poll() is None, f'{name} exited; inspect {log_path}'
                assert time.monotonic() < deadline, f'{name} startup deadline; inspect {log_path}'
                time.sleep(.05)
        environment.update(RZ_ACCEPT_ADMIN_URL=f'http://127.0.0.1:{admin_port}',
                           RZ_ACCEPT_INSIGHTS_URL=f'http://127.0.0.1:{insights_port}',
                           RZ_ACCEPT_INSIGHTS_DB=str(output / 'insights.db'), RZ_ACCEPT_RESULT=str(output / 'http-result.json'))
        check = subprocess.run(['node', str(Path(__file__).with_suffix('.mjs').resolve())], env=environment,
                               cwd=output, capture_output=True, text=True, timeout=120)
        (output / 'verifier.log').write_text(check.stdout + check.stderr)
        if check.returncode:
            raise RuntimeError(f'HTTP verifier failed; inspect {output / "http-result.json"}')
    finally:
        for process in processes:
            if process.poll() is None: process.terminate()
        for process in processes:
            try: process.wait(timeout=10)
            except subprocess.TimeoutExpired: process.kill(); process.wait(timeout=10)
        for log in logs: log.close()
    assert hashlib.sha256(args.plan.read_bytes()).hexdigest() == plan_hash, 'Plan changed during execution'
    result = json.loads((output / 'http-result.json').read_text())
    assert result['status'] == 'passed' and result['requestCount'] <= 20
    result.update(plan=plan, planSha256=plan_hash, startedAt=started_at, completedAt=dt.datetime.now(dt.timezone.utc).isoformat(),
                  binaries={name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in binaries.items()},
                  sources={path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in [Path(__file__), Path(__file__).with_suffix('.mjs'), Path(__file__).with_name('verify-insights-scenarios.mjs')]},
                  cleanup='Both owned processes stopped', nodeVersion=subprocess.check_output(['node', '--version'], text=True).strip())
    (output / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': 'passed', 'requestCount': result['requestCount'], 'result': str(output / 'result.json')}))


if __name__ == '__main__':
    main()
