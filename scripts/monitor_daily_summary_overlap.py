"""One four-request synchronized client cohort; never retries or scales its budget."""
from concurrent.futures import ThreadPoolExecutor
import json
import threading
import time


def maximum_overlap(intervals):
    events = []
    for start, end in intervals:
        if end < start:
            raise ValueError('HTTP interval ends before it starts')
        if end == start:
            continue
        events.extend([(start, 1), (end, -1)])
    active = peak = 0
    # Ends sort before starts at equal times; touching intervals do not overlap.
    for _, delta in sorted(events):
        active += delta
        peak = max(peak, active)
    return peak


def verify_overlap(request, token, baseline_receipts, output, plan_sha256):
    expected = {item['page']: item['body'] for item in baseline_receipts}
    barrier = threading.Barrier(4, timeout=5)
    origin = time.monotonic()

    def worker(index):
        page = 1 + index % 2
        try:
            barrier.wait()
        except threading.BrokenBarrierError:
            return {'index': index, 'page': page, 'error': 'Cohort barrier unavailable'}
        # No hold inside the measured interval; clocks bracket only the real call.
        started = time.monotonic() - origin
        try:
            status, body = request(f'/api/monitor/daily-summaries?current={page}&pageSize=20', token)
            ended = time.monotonic() - origin
            return {'index': index, 'page': page, 'startSeconds': started, 'endSeconds': ended,
                    'status': status, 'matchesBaseline': status == 200 and body == expected[page]}
        except Exception as error:
            ended = time.monotonic() - origin
            return {'index': index, 'page': page, 'startSeconds': started, 'endSeconds': ended,
                    'error': f'{type(error).__name__}: {error}'}

    with ThreadPoolExecutor(max_workers=4) as pool:
        samples = list(pool.map(worker, range(4)))
    peak = maximum_overlap([(sample['startSeconds'], sample['endSeconds']) for sample in samples if 'startSeconds' in sample])
    responses_match = len(samples) == 4 and all(sample.get('matchesBaseline') is True for sample in samples)
    result = {'status': 'passed' if responses_match and peak >= 2 else 'blocked' if responses_match else 'failed',
              'planSha256': plan_sha256, 'cohortsExecuted': 1, 'requestsAttempted': sum('startSeconds' in sample for sample in samples),
              'samples': samples, 'maxObservedClientHttpOverlap': peak,
              'responseCorrectness': 'passed' if responses_match else 'failed',
              'overlapEvidence': 'passed' if responses_match and peak >= 2 else 'not-run',
              'boundary': 'Exactly one four-request cohort; client HTTP call overlap only. No artificial hold, server-critical-section simultaneity, multi-user/write concurrency, performance/SLO or production-capacity claim. No automatic replay.'}
    (output / 'overlap-observation.json').write_text(json.dumps(result, indent=2) + '\n')
    assert responses_match, f'Cohort response correctness failed; inspect {output}'
    assert peak >= 2, f'No client HTTP overlap observed in the one authorized cohort; inspect {output}; do not retry automatically'
    return result
