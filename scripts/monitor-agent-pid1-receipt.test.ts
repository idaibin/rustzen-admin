import { expect, test } from "bun:test";
import { parseMonitorAgentPid1Receipt } from "./monitor-agent-pid1-receipt.ts";

const hash = "a".repeat(64);
const receipt: any = {
    schemaVersion: 1,
    kind: "monitor-agent-pid1-evidence",
    platform: "linux/amd64",
    source: { head: "b".repeat(40), state: "dirty", tree: hash },
    binaries: { cli: hash, admin: hash, monitor: hash, agent: hash },
    fixtures: { agentArchive: hash, agentManifest: hash, agentEnvelope: hash, serverManifest: hash, serverEnvelope: hash },
    activation: { unit: "rz-monitor-agent.service", enabled: true, active: true, mainPid: 4242, exeSha256: hash, controllerEndpoint: "https://monitor.internal" },
    delivery: { initialStatus: "accepted", nodesVisible: 1, nodeId: "pid1-agent-node" },
    restart: { pidChanged: true, exeSha256Same: true, activeAfter: true, deliveredAfterRestart: true },
    stopStart: { activeAfter: true },
    idempotentReactivation: true,
    limits: ["installer-test signing keys", "single host", "socat TLS front"],
};

test("agent pid1 receipt accepts the full closure", () => {
    expect(parseMonitorAgentPid1Receipt(receipt)).toEqual(receipt);
});
test("agent pid1 receipt rejects weak, leaking, or misattributed evidence", () => {
    for (const mutate of [
        (v: any) => { v.activation.active = false; },
        (v: any) => { v.activation.enabled = false; },
        (v: any) => { v.activation.unit = "other.service"; },
        (v: any) => { v.activation.mainPid = 1; },
        (v: any) => { v.activation.controllerEndpoint = "http://monitor.internal"; },
        (v: any) => { v.delivery.nodesVisible = 0; },
        (v: any) => { v.delivery.initialStatus = "stale"; },
        (v: any) => { v.restart.pidChanged = false; },
        (v: any) => { v.restart.deliveredAfterRestart = false; },
        (v: any) => { v.stopStart.activeAfter = false; },
        (v: any) => { v.idempotentReactivation = false; },
        (v: any) => { v.limits = ["single host"]; },
        (v: any) => { v.limits.push("owner-password"); },
        (v: any) => { v.platform = "linux/arm64"; },
        (v: any) => { v.source.head = "xyz"; },
        (v: any) => { v.binaries.agent = "not-a-hash"; },
        (v: any) => { delete v.fixtures.serverEnvelope; },
        (v: any) => { v.extra = true; },
    ]) {
        const mutated = structuredClone(receipt);
        mutate(mutated);
        expect(() => parseMonitorAgentPid1Receipt(mutated)).toThrow();
    }
});
