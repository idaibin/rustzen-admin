"""Owned Monitor outage/restart checks; navigation is an API assertion, not UI proof."""
import subprocess
import time


def verify_recovery(request, owner_token, processes, restart_monitor):
    path = '/api/monitor/daily-summaries?current=2&pageSize=20'
    status, before = request(path, owner_token)
    assert status == 200 and before['data']['total'] == 23
    status, navigation = request('/api/system/modules/navigation', owner_token)
    assert status == 200
    paths_before = sorted(row['path'] for row in navigation['data'] if row['module'] == 'monitor')
    assert '/monitoring/summaries' in paths_before
    # The passed handle is the Monitor child created by this unique fixture only.
    monitor = processes[0]
    monitor.terminate()
    try:
        monitor.wait(timeout=10)
    except subprocess.TimeoutExpired:
        monitor.kill()
        monitor.wait(timeout=10)
    assert monitor.poll() is not None
    assert processes[1].poll() is None, 'Admin stopped with Monitor'
    status, unavailable = request(path, owner_token)
    assert status == 503 and unavailable['code'] == 40001 and unavailable['data'] is None, unavailable
    assert 'temporarily unavailable' in unavailable['message']
    # Wait for the real ten-second discovery loop to mark the module unavailable;
    # checking navigation before that transition would miss the degraded-state case.
    deadline = time.monotonic() + 30
    unavailable_attempts = 0
    while True:
        unavailable_attempts += 1
        assert processes[1].poll() is None
        status, body = request('/api/system/modules', owner_token)
        assert status == 200, body
        module = next(row for row in body['data'] if row['id'] == 'monitor')
        assert module['enabled'] is True
        if module['available'] is False:
            break
        assert time.monotonic() < deadline, 'Module unavailability discovery deadline exceeded'
        time.sleep(.1)
    assert request('/health')[0] == 200
    status, login = request('/api/auth/login', body={'username': 'owner', 'password': 'rustzen@123'})
    assert status == 200, (status, login.get('message'))
    assert request('/api/auth/me', login['data']['token'])[0] == 200
    status, navigation = request('/api/system/modules/navigation', owner_token)
    assert status == 200
    paths_during = sorted(row['path'] for row in navigation['data'] if row['module'] == 'monitor')
    assert paths_during == paths_before
    processes[0] = restart_monitor()
    deadline = time.monotonic() + 30
    recovery_attempts = 0
    while True:
        recovery_attempts += 1
        assert all(process.poll() is None for process in processes)
        status, recovered = request(path, owner_token)
        if status == 200:
            break
        assert status == 503, recovered
        assert time.monotonic() < deadline, 'Module recovery deadline exceeded'
        time.sleep(.1)
    assert recovered == before, 'Persisted generated summaries changed after restart'
    status, body = request('/api/system/modules', owner_token)
    assert status == 200
    restored = next(row for row in body['data'] if row['id'] == 'monitor')
    assert restored['enabled'] is True and restored['available'] is True
    return {'outageStatus': 503, 'outageBody': unavailable, 'adminHealthStatus': 200,
            'freshLoginStatus': 200, 'freshSessionStatus': 200, 'moduleUnavailableObserved': True,
            'navigationBefore': paths_before, 'navigationDuringOutage': paths_during,
            'recoveryStatus': 200, 'recoveredPage': recovered,
            'unavailabilityAttempts': unavailable_attempts, 'recoveryAttempts': recovery_attempts,
            'boundary': 'One owned Monitor restart; same original owner JWT and SQLite; persisted navigation API only. No browser, other-module independence, concurrency or performance acceptance.'}
