"""Fixed local read observation, not a production benchmark or throughput/SLO gate."""
from concurrent.futures import ThreadPoolExecutor
import json
import math
import os
from pathlib import Path
import threading
import time

REQUESTS = 100
CLIENTS = 4
GLOBAL_RPS = 10
DEADLINE_SECONDS = 30
RSS_LIMIT = 1024 ** 3


def process_usage(process):
    assert process.poll() is None, 'Owned service exited during read observation'
    status = (Path('/proc') / str(process.pid) / 'status').read_text()
    rss_kib = int(next(line.split()[1] for line in status.splitlines() if line.startswith('VmRSS:')))
    fields = (Path('/proc') / str(process.pid) / 'stat').read_text().rsplit(')', 1)[1].split()
    cpu = (int(fields[11]) + int(fields[12])) / os.sysconf('SC_CLK_TCK')
    return {'rssBytes': rss_kib * 1024, 'cpuSeconds': cpu}


def verify_read_observation(request, token, processes, baseline_receipts, output):
    expected = {item['page']: item['body'] for item in baseline_receipts}
    before = [process_usage(process) for process in processes]
    start = time.monotonic()
    deadline = start + DEADLINE_SECONDS
    stopped = threading.Event()
    dispatch_lock = threading.Lock()
    results_lock = threading.Lock()
    state = {'index': 0, 'nextStart': start, 'inFlight': 0, 'maxInFlight': 0,
             'peakServiceRssBytes': sum(item['rssBytes'] for item in before)}
    samples = []
    failures = []

    def worker():
        while not stopped.is_set():
            with dispatch_lock:
                if stopped.is_set() or state['index'] >= REQUESTS:
                    return
                if stopped.wait(max(0, state['nextStart'] - time.monotonic())):
                    return
                if time.monotonic() >= deadline:
                    with results_lock:
                        failures.append('Load resource deadline exceeded')
                    stopped.set()
                    return
                # Global dispatcher spacing prevents per-client or catch-up bursts.
                index = state['index']
                state['index'] += 1
                state['nextStart'] = time.monotonic() + 1 / GLOBAL_RPS
                with results_lock:
                    state['inFlight'] += 1
                    state['maxInFlight'] = max(state['maxInFlight'], state['inFlight'])
            page = 1 + index % 2
            sent = time.monotonic()
            try:
                rss = sum(process_usage(process)['rssBytes'] for process in processes)
                assert rss <= RSS_LIMIT, 'Owned-service RSS resource ceiling exceeded'
                request_started = time.monotonic()
                status, body = request(f'/api/monitor/daily-summaries?current={page}&pageSize=20', token)
                latency_ms = (time.monotonic() - request_started) * 1000
                assert status == 200 and body == expected[page], f'Read correctness failed at sample {index}, status {status}'
                with results_lock:
                    state['peakServiceRssBytes'] = max(state['peakServiceRssBytes'], rss)
                    samples.append({'index': index, 'page': page, 'status': status,
                                    'startOffsetSeconds': sent - start, 'latencyMs': latency_ms})
            except Exception as error:
                with results_lock:
                    failures.append(f'{type(error).__name__}: {error}')
                stopped.set()
            finally:
                with results_lock:
                    state['inFlight'] -= 1

    with ThreadPoolExecutor(max_workers=CLIENTS) as pool:
        futures = [pool.submit(worker) for _ in range(CLIENTS)]
        for future in futures:
            future.result()
    elapsed = time.monotonic() - start
    try:
        after = [process_usage(process) for process in processes]
    except Exception as error:
        failures.append(f'Final resource observation failed: {type(error).__name__}: {error}')
        after = None
    samples.sort(key=lambda item: item['index'])
    latencies = sorted(item['latencyMs'] for item in samples)
    result = {'status': 'passed' if not failures and len(samples) == REQUESTS else 'failed',
              'workload': {'clients': CLIENTS, 'requestCap': REQUESTS, 'globalRpsCap': GLOBAL_RPS,
                           'deadlineSeconds': DEADLINE_SECONDS, 'serviceRssCeilingBytes': RSS_LIMIT},
              'sampleCount': len(samples), 'errorCount': len(failures), 'failures': failures,
              'elapsedSeconds': elapsed, 'observedThroughputRps': len(samples) / elapsed,
              'maxObservedInFlight': state['maxInFlight'],
              'peakObservedCombinedServiceRssBytes': state['peakServiceRssBytes'],
              'serviceCpuDeltaSeconds': [max(0, end['cpuSeconds'] - begin['cpuSeconds']) for begin, end in zip(before, after)] if after else None,
              'samples': samples,
              'interpretation': 'Single local warm-cache point after ordinary paging checks; four client workers with actual in-flight count recorded. No sequential comparison, improvement, production capacity, performance budget, tail-latency or multi-user claim. Shared-executor scheduling and HTTP generator overhead are not isolated.'}
    if latencies:
        result['latencyMs'] = {'minimum': latencies[0], 'p50NearestRank': latencies[math.ceil(.5 * len(latencies)) - 1],
                               'p95NearestRank': latencies[math.ceil(.95 * len(latencies)) - 1], 'maximum': latencies[-1]}
    (output / 'load-observation.json').write_text(json.dumps(result, indent=2) + '\n')
    assert result['status'] == 'passed', f'Read observation failed; inspect {output / "load-observation.json"}'
    return result
