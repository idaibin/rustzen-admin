import { expect, test } from "bun:test";

const source = await Bun.file("scripts/verify-worker-contracts.mjs").text();

test("worker verifier exercises the Monitor 2.0 report and metrics contracts", () => {
    expect(source).toContain('import { verifyMonitoringScenarios } from "./verify-monitoring-scenarios.mjs"');
    expect(source).toContain("await verifyMonitoringScenarios({ adminBase, adminToken, agentToken });");
    expect(source).toContain('nodeId: "verify-agent"');
    expect(source).toContain("bootId: randomUUID()");
    expect(source).toContain("sequence: 1");
    expect(source).toContain('memory: { usedBytes: 10, totalBytes: 20 }');
    expect(source).toContain('disks: [{ mountPoint: "/", usedBytes: 30, totalBytes: 40 }]');
    expect(source).toContain('"/api/monitor/agent-reports"');
    expect(source).toContain("node.nodeId === \"verify-agent\"");
    expect(source).toContain("${verifyNode.nodeId}/metrics?bucket=raw");
    expect(source).toContain("metrics.points?.length !== 1");
});

test("worker verifier no longer exercises retired heartbeat and checks endpoints", () => {
    for (const retiredPath of ["/api/monitor/heartbeat", "/api/monitor/checks", "monitor:check:"]) {
        expect(source).not.toContain(retiredPath);
    }
});
