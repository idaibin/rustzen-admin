import { expect, test } from "bun:test";

const fixturePath = new URL("./monitoring-ui-state-fixture.py", import.meta.url).pathname;
const keys = (value) => Object.keys(value).sort();
const expectKeys = (value, expected) => {
    expect(keys(value)).toEqual([...expected].sort());
};

test("four Monitoring success payloads match Web types and Monitor handlers", async () => {
    const port = 22000 + Math.floor(Math.random() * 1000);
    const fixture = Bun.spawn(["python3", "-B", fixturePath], {
        env: {
            ...process.env,
            RUSTZEN_MONITORING_FIXTURE_PORT: String(port),
        },
        stdout: "ignore",
        stderr: "pipe",
    });
    const base = "http://127.0.0.1:" + port;
    try {
        for (let attempt = 0; attempt < 80; attempt += 1) {
            const health = await fetch(base + "/__monitoring_fixture/health").catch(() => null);
            if (health?.ok) break;
            await Bun.sleep(25);
        }
        expect((await fetch(base + "/__monitoring_fixture/mode", {
            method: "PATCH",
            headers: { "content-type": "application/json" },
            body: JSON.stringify({ overview: "slow" }),
        })).ok).toBe(true);
        const slowStarted = performance.now();
        expect((await fetch(base + "/api/monitor/overview")).status).toBe(200);
        expect(performance.now() - slowStarted).toBeGreaterThanOrEqual(4_500);
        expect((await fetch(base + "/__monitoring_fixture/mode", {
            method: "PATCH",
            headers: { "content-type": "application/json" },
            body: JSON.stringify({ overview: "success" }),
        })).ok).toBe(true);
        const overview = await (await fetch(base + "/api/monitor/overview")).json();
        expectKeys(overview, ["code", "message", "data"]);
        expectKeys(overview.data, [
            "registeredNodes",
            "onlineNodes",
            "offlineNodes",
            "activeIncidents",
            "latestResource",
        ]);
        expectKeys(overview.data.latestResource, [
            "nodeId",
            "collectedAt",
            "lastReceivedAt",
            "cpuPercent",
            "memoryPercent",
            "disks",
        ]);
        expectKeys(overview.data.latestResource.disks[0], [
            "mountPoint",
            "usedBytes",
            "totalBytes",
            "usagePercent",
        ]);

        const nodes = await (await fetch(base + "/api/monitor/nodes")).json();
        expectKeys(nodes, ["code", "message", "data"]);
        expectKeys(nodes.data[0], [
            "nodeId",
            "hostname",
            "agentVersion",
            "bootId",
            "sequence",
            "lastReportAt",
            "lastReceivedAt",
            "status",
            "alertPolicySource",
            "cpuPercent",
            "memory",
            "disks",
            "createdAt",
            "updatedAt",
        ]);
        expectKeys(nodes.data[0].memory, ["usedBytes", "totalBytes", "usagePercent"]);
        expectKeys(nodes.data[0].disks[0], [
            "mountPoint",
            "collectedAt",
            "usedBytes",
            "totalBytes",
            "usagePercent",
        ]);

        const incidentUrl = base + "/api/monitor/incidents?current=1&pageSize=20";
        const incidents = await (await fetch(incidentUrl)).json();
        expectKeys(incidents, ["code", "message", "data"]);
        expectKeys(incidents.data, ["data", "total", "success"]);
        expectKeys(incidents.data.data[0], [
            "id",
            "nodeId",
            "kind",
            "target",
            "status",
            "title",
            "thresholdPercent",
            "observedPercent",
            "openedAt",
            "lastObservedAt",
            "resolvedAt",
            "resolutionReason",
            "details",
        ]);

        const summaryUrl = base + "/api/monitor/daily-summaries?current=1&pageSize=20";
        const summaries = await (await fetch(summaryUrl)).json();
        expectKeys(summaries, ["code", "message", "data"]);
        expectKeys(summaries.data, ["data", "total", "success"]);
        expectKeys(summaries.data.data[0], [
            "nodeId",
            "date",
            "sampleCount",
            "coverage",
            "coveragePercent",
            "cpu",
            "memory",
            "diskSummary",
            "offlineSeconds",
            "incidentCount",
        ]);
        expectKeys(summaries.data.data[0].cpu, ["min", "avg", "max"]);
        expectKeys(summaries.data.data[0].memory, ["min", "avg", "max"]);
        expectKeys(summaries.data.data[0].diskSummary["/"], ["min", "avg", "max"]);
    } finally {
        fixture.kill();
        await fixture.exited;
    }
}, 10_000);
