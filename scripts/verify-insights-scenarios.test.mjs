import { expect, test } from "bun:test";

import { verifyInsightsScenarios } from "./verify-insights-scenarios.mjs";

function response(status, data, headers = {}) {
    return { status, data, headers: new Headers(headers) };
}

async function runScenario(details = { success: true, total: 3, data: [] }) {
    let enabled = false;
    const calls = [];
    await verifyInsightsScenarios({
        insightsBase: "http://insights.test",
        directRequest: async (_base, module, path, access, init = {}) => {
            calls.push({ module, path, access, method: init.method ?? "GET" });
            const body = init.body ? JSON.parse(init.body) : undefined;
            if (path === "/api/insights/collection-policy") {
                enabled = body.collectionEnabled;
                return response(200, { collectionEnabled: enabled, projectConfigured: true });
            }
            if (path === "/api/insights/track" && init.method === "OPTIONS") {
                const allowed = init.headers.origin === "HTTPS://APP.EXAMPLE:443/";
                return response(allowed ? 204 : 403, null, allowed
                    ? { "access-control-allow-origin": "https://app.example", "access-control-allow-methods": "POST", "access-control-allow-headers": "content-type, x-rustzen-project-key", vary: "Origin" }
                    : { vary: "Origin" });
            }
            if (path === "/api/insights/track") {
                const origin = init.headers.origin;
                if (!enabled || origin === "https://not-allowed.example") return response(403, null, { vary: "Origin" });
                if (Array.isArray(body)) return response(200, { accepted: 3 }, { "access-control-allow-origin": origin });
                return response(422, null, { "access-control-allow-origin": origin });
            }
            if (path === "/api/insights/overview") return response(200, { pv: 1, uv: 2, eventCount: 3, requestCount: 1, errorCount: 1, p95DurationMs: 42 });
            if (path === "/api/insights/events") return response(200, details);
            if (path === "/api/insights/tracker.js") return response(200, null, { "content-type": "application/javascript; charset=utf-8" });
            throw new Error(`unexpected request ${path}`);
        },
        expectStatus: async (value, expected, label) => {
            if (value.status !== expected) throw new Error(`${label}: expected ${expected}, got ${value.status}`);
            return value;
        },
        responseData: async (value) => value.data,
    });
    return calls;
}

test("Insights verifier preserves policy, CORS, ingestion, query, and tracker scenarios", async () => {
    const calls = await runScenario();
    expect(calls).toContainEqual({ module: "insights", path: "/api/insights/collection-policy", access: "insights:manage", method: "PUT" });
    expect(calls.filter((call) => call.path === "/api/insights/track")).toHaveLength(6);
    expect(calls).toContainEqual({ module: "insights", path: "/api/insights/overview", access: "insights:overview:view", method: "GET" });
    expect(calls).toContainEqual({ module: "insights", path: "/api/insights/events", access: "insights:event:view", method: "GET" });
    expect(calls).toContainEqual({ module: "insights", path: "/api/insights/tracker.js", access: "public", method: "GET" });
});

test("Insights verifier rejects unsafe detail pathname fields", async () => {
    await expect(runScenario({ success: true, total: 3, data: [{ pagePath: "/safe?secret" }] })).rejects.toThrow(
        "Insights details exposed an unsafe pagePath",
    );
});
