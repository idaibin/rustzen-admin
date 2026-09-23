import assert from "node:assert/strict";
import { randomUUID } from "node:crypto";

export const monitoringNavigationPaths = [
    "/monitoring/overview",
    "/monitoring/nodes",
    "/monitoring/incidents",
    "/monitoring/summaries",
];

const retiredMonitoringNavigationPaths = ["/monitoring/settings", "/monitoring/checks"];

export function assertMonitoringNavigation(paths) {
    for (const path of retiredMonitoringNavigationPaths) {
        assert.ok(!paths.includes(path), `retired Monitor page must not be navigable: ${path}`);
    }
    assert.deepEqual(paths.toSorted(), monitoringNavigationPaths.toSorted());
}

// Only called against the verifier's newly initialized local database.
export async function verifyMonitoringScenarios({ adminBase, adminToken, agentToken }) {
    async function call(path, { token = adminToken, method = "GET", body, status = 200, agent = false } = {}) {
        const response = await fetch(`${adminBase}${path}`, {
            method,
            headers: {
                ...(token ? { authorization: `Bearer ${token}` } : {}),
                ...(body ? { "content-type": "application/json" } : {}),
                ...(agent ? { "x-rustzen-monitor-agent-token": agentToken } : {}),
            },
            ...(body ? { body: JSON.stringify(body) } : {}),
        });
        assert.equal(response.status, status, `${method} ${path}`);
        if (status !== 200) { await response.arrayBuffer(); return; }
        const payload = await response.json();
        assert.equal(payload.code, 0, `${method} ${path} envelope`);
        return payload.data;
    }
    const roleOptions = await call("/api/system/roles/options?limit=500");
    const viewerRole = roleOptions.find((role) => role.code === "viewer");
    assert.ok(viewerRole, "fresh database exposes the built-in viewer role");
    await call("/api/system/users", {
        method: "POST",
        body: {
            username: "monitoring_scenario_viewer",
            email: "monitoring-scenario-viewer@example.test",
            password: "monitoring-scenario-viewer-password",
            realName: "Monitoring scenario viewer",
            status: 1,
            roleIds: [viewerRole.value],
        },
    });
    const viewer = await call("/api/auth/login", {
        token: null,
        method: "POST",
        body: {
            username: "monitoring_scenario_viewer",
            password: "monitoring-scenario-viewer-password",
        },
    });
    const viewerToken = viewer.token;
    for (const token of [adminToken, viewerToken]) {
        const nav = await call("/api/system/modules/navigation", { token });
        assertMonitoringNavigation(nav.filter((m) => m.module === "monitor").map((m) => m.path));
    }
    await call("/api/monitor/alert-settings", { token: viewerToken });

    const settings = await call("/api/monitor/alert-settings");
    const policy = { cpu: settings.cpu, memory: settings.memory, disk: settings.disk, offline: settings.offline };
    await call("/api/monitor/alert-settings", { token: viewerToken, method: "PUT", body: policy, status: 403 });
    await call("/api/monitor/alert-settings", { token: null, status: 401 });
    await call("/api/monitor/alert-settings", { method: "PUT", body: { ...policy, cpu: { enabled: true, thresholdPercent: 101 } }, status: 422 });
    assert.deepEqual((await call("/api/monitor/alert-settings")).cpu, settings.cpu, "invalid policy is atomic");

    const nodeId = `scenario-${randomUUID()}`;
    const bootId = randomUUID();
    const start = Date.now() - 10_000;
    function report(sequence, high) {
        return { nodeId, bootId, sequence, hostname: "Verification node", agentVersion: "0.5.0",
            collectedAt: new Date(start + sequence * 1000).toISOString(), cpuPercent: high ? 95 : 10,
            memory: { usedBytes: 10, totalBytes: 100 },
            disks: [{ mountPoint: "/", usedBytes: high ? 95 : 10, totalBytes: 100 }, { mountPoint: "/data", usedBytes: 20, totalBytes: 100 }] };
    }
    async function submit(value, expected) {
        const result = await call("/api/monitor/agent-reports", { token: null, method: "POST", agent: true, body: value });
        assert.equal(result.status, expected);
    }
    for (let sequence = 1; sequence <= 3; sequence++) await submit(report(sequence, true), "accepted");
    await submit(report(3, true), "duplicate");
    await submit(report(2, true), "stale");
    const active = await call(`/api/monitor/incidents?nodeId=${nodeId}&status=active&pageSize=1&current=1`, { token: viewerToken });
    assert.equal(active.total, 2, "CPU and root disk trigger independent incidents");
    assert.equal(active.data.length, 1);
    const second = await call(`/api/monitor/incidents?nodeId=${nodeId}&status=active&pageSize=1&current=2`);
    assert.notEqual(active.data[0].id, second.data[0].id);
    const detail = await call(`/api/monitor/incidents/${active.data[0].id}`, { token: viewerToken });
    assert.equal(detail.node.nodeId, nodeId);
    const beforeRecovery = await call(`/api/monitor/nodes/${nodeId}/metrics?bucket=raw`);
    assert.equal(beforeRecovery.points.length, 3, "duplicate/stale submissions add no samples");
    for (let sequence = 4; sequence <= 6; sequence++) await submit(report(sequence, false), "accepted");
    assert.equal((await call(`/api/monitor/incidents?nodeId=${nodeId}&status=active`)).total, 0);
    assert.equal((await call(`/api/monitor/incidents?nodeId=${nodeId}&status=resolved`)).total, 2);
    const nodePath = `/api/monitor/nodes/${nodeId}/alert-settings`;
    await call(nodePath, { token: viewerToken, method: "PUT", body: policy, status: 403 });
    await call(nodePath, { token: viewerToken, method: "DELETE", status: 403 });
    const custom = await call(nodePath, { method: "PUT", body: { ...policy, cpu: { enabled: true, thresholdPercent: 80 } } });
    assert.equal(custom.isCustom, true);
    await call("/api/monitor/alert-settings", { method: "PUT", body: { ...policy, cpu: { enabled: true, thresholdPercent: 85 } } });
    assert.equal((await call(nodePath)).cpu.thresholdPercent, 80);
    const reset = await call(nodePath, { method: "DELETE" });
    assert.equal(reset.isCustom, false);
    assert.equal(reset.cpu.thresholdPercent, 85);
    assert.equal((await call(nodePath, { method: "DELETE" })).isCustom, false);
    await call("/api/monitor/alert-settings", { method: "PUT", body: policy });
    for (const path of ["/api/monitor/incidents?status=acknowledged", "/api/monitor/incidents?current=0", "/api/monitor/daily-summaries?current=0", `/api/monitor/nodes/${nodeId}/metrics?bucket=invalid`]) {
        await call(path, { status: 422 });
    }
    const from = new Date(Date.now() - 31 * 86400_000).toISOString();
    const to = new Date().toISOString();
    for (const path of ["/api/monitor/incidents", "/api/monitor/daily-summaries", `/api/monitor/nodes/${nodeId}/metrics`]) {
        await call(`${path}?from=${encodeURIComponent(from)}&to=${encodeURIComponent(to)}`, { status: 422 });
    }
    const empty = await call(`/api/monitor/daily-summaries?nodeId=${nodeId}&current=2&pageSize=1`, { token: viewerToken });
    assert.equal(empty.success, true);
    assert.equal(empty.data.length, 0);
    console.log("Monitoring gateway scenarios: four menus, retired settings/checks pages rejected, viewer RBAC, report fencing, alert recovery, paging, policy inheritance and query validation passed");
}
