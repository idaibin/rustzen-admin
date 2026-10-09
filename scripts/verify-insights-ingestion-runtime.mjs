import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { verifyInsightsScenarios } from './verify-insights-scenarios.mjs';

const base = process.env.RZ_ACCEPT_ADMIN_URL;
const insights = process.env.RZ_ACCEPT_INSIGHTS_URL;
const database = process.env.RZ_ACCEPT_INSIGHTS_DB;
const output = process.env.RZ_ACCEPT_RESULT;
const receipts = [];
const rejectedBatches = [];
let requests = 0;
let token;

async function request(url, init = {}) {
    assert(requests < 20, 'The finite 20-HTTP-request budget is exhausted');
    requests += 1;
    const response = await fetch(url, { ...init, signal: AbortSignal.timeout(3000) });
    const raw = await response.clone().text();
    const entry = { index: requests, path: new URL(url).pathname, method: init.method ?? 'GET', status: response.status,
        headers: Object.fromEntries(['content-type', 'access-control-allow-origin', 'access-control-allow-methods', 'access-control-allow-headers', 'vary'].map(name => [name, response.headers.get(name)])) };
    if (entry.path !== '/api/auth/login') {
        try { entry.body = JSON.parse(raw); }
        catch { entry.bodySha256 = createHash('sha256').update(raw).digest('hex'); }
    }
    receipts.push(entry);
    return response;
}

async function expectStatus(response, expected, label) {
    assert.equal(response.status, expected, `${label}: expected ${expected}, got ${response.status}`);
    return response;
}

function gatewayRequest(_base, _module, path, capability, init = {}) {
    return request(base + path, { ...init, headers: {
        'content-type': 'application/json', ...(init.headers ?? {}),
        ...(capability === 'public' ? {} : { authorization: `Bearer ${token}` }),
    } });
}

function countEvents() {
    return Number(execFileSync('python3', ['-c', 'import sqlite3,sys; c=sqlite3.connect("file:"+sys.argv[1]+"?mode=ro",uri=True); print(c.execute("SELECT COUNT(*) FROM insights_events").fetchone()[0])', database], { encoding: 'utf8' }).trim());
}

let result;
try {
    assert.equal(countEvents(), 0, 'Fixture starts with no event rows');
    const login = await expectStatus(await request(base + '/api/auth/login', {
        method: 'POST', headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ username: 'owner', password: 'rustzen@123' }),
    }), 200, 'Owned development owner login');
    token = (await login.json()).data.token;
    assert.equal(typeof token, 'string');
    let ready = false;
    for (let attempt = 0; attempt < 3; attempt++) {
        const response = await gatewayRequest(base, 'insights', '/api/insights/collection-policy', 'insights:manage');
        if (response.status === 200) { ready = true; break; }
        assert.equal(response.status, 503, 'Only asynchronous module unavailability may be retried');
        if (attempt < 2) await new Promise(resolve => setTimeout(resolve, 10000));
    }
    assert(ready, 'Insights gateway readiness exhausted the bounded attempts');
    await verifyInsightsScenarios({
        insightsBase: base, directRequest: gatewayRequest, expectStatus,
        responseData: async response => (await response.json()).data,
    });
    assert.equal(countEvents(), 3, 'Exactly the three HTTP-accepted events persisted');
    const valid = { eventName: 'page_view', visitorId: 'rejected-batch-fixture', pagePath: '/must-not-persist' };
    const cases = [
        ['mixed valid and invalid batch', 422, [valid, { ...valid, eventName: 'unknown' }]],
        ['51-event batch', 413, Array.from({ length: 51 }, () => valid)],
        ['body above 64 KiB', 413, { ...valid, properties: { feature: 'x'.repeat(70000) } }],
    ];
    for (const [label, expected, payload] of cases) {
        const before = countEvents();
        await expectStatus(await gatewayRequest(base, 'insights', '/api/insights/track', 'public', {
            method: 'POST', headers: { origin: 'https://app.example', 'x-rustzen-project-key': 'verify-project-key' },
            body: JSON.stringify(payload),
        }), expected, label);
        const after = countEvents();
        assert.equal(before, 3);
        assert.equal(after, before, `${label} must persist zero partial events`);
        rejectedBatches.push({ label, expectedStatus: expected, before, after });
    }
    await expectStatus(await request(base + '/api/insights/overview'), 401, 'Unauthenticated Admin gateway read');
    await expectStatus(await request(insights + '/api/insights/overview'), 401, 'Unsigned direct module read');
    result = { status: 'passed', requestCount: requests, acceptedEventRows: countEvents(), receipts, rejectedBatches,
        boundaries: 'Real Admin JWT/gateway/Insights/SQLite and synthetic Origin headers. No external-origin requests or SQL event inserts. Browser consent/bootstrap, page/function/E2E, other roles, load and production are not verified.' };
} catch (error) {
    result = { status: 'failed', requestCount: requests, error: String(error), receipts, rejectedBatches };
    process.exitCode = 1;
} finally {
    writeFileSync(output, JSON.stringify(result, null, 2) + '\n');
    console.log(JSON.stringify({ status: result.status, requestCount: requests, result: output }));
}
