"""Bounded real-HTTP role/revocation cases for owned gateway acceptance fixtures."""


def verify_role_access(request, owner_token):
    path = '/api/monitor/daily-summaries'
    status, menus = request('/api/system/menus', owner_token)
    assert status == 200
    ids = {item['code']: item['id'] for item in menus['data']}
    assert 'monitor:node:view' in ids and 'monitor:overview:view' in ids
    roles = {}
    tokens = {}
    users = {}
    receipts = []
    for kind, capability in [('allowed', 'monitor:node:view'), ('denied', 'monitor:overview:view')]:
        role = {'name': f'Daily acceptance {kind}', 'code': f'daily_acceptance_{kind}',
                'status': 1, 'menuIds': [ids[capability]], 'description': 'Owned local acceptance fixture'}
        status, body = request('/api/system/roles', owner_token, role)
        assert status == 200, body
        status, body = request('/api/system/roles?current=1&pageSize=100', owner_token)
        assert status == 200
        role_id = next(item['id'] for item in body['data'] if item['code'] == role['code'])
        roles[kind] = (role_id, role)
        username = f'daily_{kind}'
        status, body = request('/api/system/users', owner_token,
                               {'username': username, 'email': f'{username}@example.invalid',
                                'password': 'OwnedDailyFixture123!', 'realName': 'Owned fixture',
                                'roleIds': [role_id], 'status': 1})
        assert status == 200, body
        users[kind] = body['data']
        status, body = request('/api/auth/login', body={'username': username, 'password': 'OwnedDailyFixture123!'})
        assert status == 200, (status, body.get('message'))
        tokens[kind] = body['data']['token']
        status, body = request('/api/auth/me', tokens[kind])
        assert status == 200 and body['data']['permissions'] == [capability], body
        status, body = request(path, tokens[kind])
        expected = 200 if kind == 'allowed' else 403
        assert status == expected, (kind, status, body)
        if kind == 'allowed':
            assert body['data']['total'] == 23
        receipts.append({'case': kind, 'capability': capability, 'status': status})
    # A read capability must not imply module management or role administration.
    status, _ = request('/api/system/roles', tokens['allowed'])
    assert status == 403
    receipts.append({'case': 'reader-cannot-administer-roles', 'status': status})
    role_id, role = roles['allowed']
    role['menuIds'] = [ids['monitor:overview:view']]
    status, body = request(f'/api/system/roles/{role_id}', owner_token, role, method='PUT')
    assert status == 200, body
    status, _ = request(path, tokens['allowed'])
    assert status == 403, status
    receipts.append({'case': 'same-session-after-capability-revocation', 'status': status})
    role['menuIds'] = [ids['monitor:node:view']]
    status, body = request(f'/api/system/roles/{role_id}', owner_token, role, method='PUT')
    assert status == 200, body
    status, body = request(path, tokens['allowed'])
    assert status == 200 and body['data']['total'] == 23, body
    receipts.append({'case': 'same-session-after-capability-restoration', 'status': status})
    status, body = request(f"/api/system/users/{users['allowed']}/status", owner_token, {'status': 2}, method='PUT')
    assert status == 200, body
    status, _ = request(path, tokens['allowed'])
    assert status == 401, status
    receipts.append({'case': 'disabled-user-existing-session', 'status': status})
    status, body = request(path, owner_token)
    assert status == 200 and body['data']['total'] == 23
    receipts.append({'case': 'unrelated-owner-remains-authorized', 'status': status})
    return receipts
