// Strict receipt parser for the node-agent PID1/service-restart gate.
import { canonicalJson, validHash } from "../distribution/release-manifest-core.ts";

export type MonitorAgentPid1Receipt = {
    schemaVersion: 1;
    kind: "monitor-agent-pid1-evidence";
    platform: "linux/amd64";
    source: { head: string; state: "clean" | "dirty"; tree: string };
    binaries: { cli: string; admin: string; monitor: string; agent: string };
    fixtures: { agentArchive: string; agentManifest: string; agentEnvelope: string; serverManifest: string; serverEnvelope: string };
    activation: { unit: string; enabled: boolean; active: boolean; mainPid: number; exeSha256: string; controllerEndpoint: string };
    delivery: { initialStatus: "accepted" | "duplicate"; nodesVisible: number; nodeId: string };
    restart: { pidChanged: boolean; exeSha256Same: boolean; activeAfter: boolean; deliveredAfterRestart: boolean };
    stopStart: { activeAfter: boolean };
    idempotentReactivation: boolean;
    limits: string[];
};

function object(value: unknown, label: string): Record<string, any> { if (!value || typeof value !== "object" || Array.isArray(value)) throw Error(`${label} must be an object`); return value as Record<string, any>; }
function only(value: Record<string, any>, allowed: string[], label: string) { if (canonicalJson(Object.keys(value).sort()) !== canonicalJson([...allowed].sort())) throw Error(`${label} fields differ`); }
function text(value: unknown, label: string): string { if (typeof value !== "string" || !value) throw Error(`${label} is invalid`); return value; }
function flag(value: unknown, label: string): true { if (value !== true) throw Error(`${label} must be true`); return true; }
function count(value: unknown, label: string, minimum: number): number { if (!Number.isSafeInteger(value) || (value as number) < minimum) throw Error(`${label} is invalid`); return value as number; }

export function parseMonitorAgentPid1Receipt(value: unknown): MonitorAgentPid1Receipt {
    const receipt = object(value, "agent pid1 receipt");
    only(receipt, ["schemaVersion", "kind", "platform", "source", "binaries", "fixtures", "activation", "delivery", "restart", "stopStart", "idempotentReactivation", "limits"], "agent pid1 receipt");
    if (receipt.schemaVersion !== 1 || receipt.kind !== "monitor-agent-pid1-evidence" || receipt.platform !== "linux/amd64") throw Error("agent pid1 receipt identity differs");
    const source = object(receipt.source, "agent pid1 source");
    only(source, ["head", "state", "tree"], "agent pid1 source");
    if (!/^[0-9a-f]{40}$/.test(text(source.head, "source head")) || !["clean", "dirty"].includes(source.state) || !/^[0-9a-f]{64}$/.test(text(source.tree, "source tree"))) throw Error("agent pid1 source identity differs");
    const binaries = object(receipt.binaries, "agent pid1 binaries");
    only(binaries, ["cli", "admin", "monitor", "agent"], "agent pid1 binaries");
    for (const key of ["cli", "admin", "monitor", "agent"]) validHash(text(binaries[key], `binary ${key}`));
    const fixtures = object(receipt.fixtures, "agent pid1 fixtures");
    only(fixtures, ["agentArchive", "agentManifest", "agentEnvelope", "serverManifest", "serverEnvelope"], "agent pid1 fixtures");
    for (const key of ["agentArchive", "agentManifest", "agentEnvelope", "serverManifest", "serverEnvelope"]) validHash(text(fixtures[key], `fixture ${key}`));
    const activation = object(receipt.activation, "agent pid1 activation");
    only(activation, ["unit", "enabled", "active", "mainPid", "exeSha256", "controllerEndpoint"], "agent pid1 activation");
    if (activation.unit !== "rz-monitor-agent.service" || activation.enabled !== true || activation.active !== true) throw Error("agent pid1 activation state differs");
    count(activation.mainPid, "main pid", 2);
    validHash(text(activation.exeSha256, "activation exe sha256"));
    if (!/^https:\/\/[A-Za-z0-9.-]+(:[0-9]+)?$/.test(text(activation.controllerEndpoint, "controller endpoint"))) throw Error("agent pid1 controller endpoint is not https");
    const delivery = object(receipt.delivery, "agent pid1 delivery");
    only(delivery, ["initialStatus", "nodesVisible", "nodeId"], "agent pid1 delivery");
    if (!["accepted", "duplicate"].includes(delivery.initialStatus) || count(delivery.nodesVisible, "nodes visible", 1) !== 1) throw Error("agent pid1 delivery differs");
    text(delivery.nodeId, "node id");
    const restart = object(receipt.restart, "agent pid1 restart");
    only(restart, ["pidChanged", "exeSha256Same", "activeAfter", "deliveredAfterRestart"], "agent pid1 restart");
    flag(restart.pidChanged, "restart pidChanged");
    flag(restart.exeSha256Same, "restart exeSha256Same");
    flag(restart.activeAfter, "restart activeAfter");
    flag(restart.deliveredAfterRestart, "restart deliveredAfterRestart");
    const stopStart = object(receipt.stopStart, "agent pid1 stopStart");
    only(stopStart, ["activeAfter"], "agent pid1 stopStart");
    flag(stopStart.activeAfter, "stopStart activeAfter");
    flag(receipt.idempotentReactivation, "idempotent reactivation");
    if (!Array.isArray(receipt.limits) || !receipt.limits.includes("installer-test signing keys")) throw Error("agent pid1 limits differ");
    const serialized = JSON.stringify(value);
    for (const forbidden of ["owner-password", "agent-token-", "Bearer "])
        if (serialized.includes(forbidden)) throw Error(`agent pid1 receipt leaks sensitive material: ${forbidden.trim()}`);
    return {
        schemaVersion: 1, kind: "monitor-agent-pid1-evidence", platform: "linux/amd64",
        source: source as MonitorAgentPid1Receipt["source"],
        binaries: binaries as MonitorAgentPid1Receipt["binaries"],
        fixtures: fixtures as MonitorAgentPid1Receipt["fixtures"],
        activation: { ...activation, mainPid: activation.mainPid, exeSha256: activation.exeSha256 } as MonitorAgentPid1Receipt["activation"],
        delivery: delivery as MonitorAgentPid1Receipt["delivery"],
        restart: restart as MonitorAgentPid1Receipt["restart"],
        stopStart: stopStart as MonitorAgentPid1Receipt["stopStart"],
        idempotentReactivation: true,
        limits: receipt.limits,
    };
}

if (Bun.argv.length === 3) {
    const bytes = await Bun.file(Bun.argv[2]!).bytes();
    const parsed = parseMonitorAgentPid1Receipt(JSON.parse(new TextDecoder().decode(bytes)));
    if (new TextDecoder().decode(bytes) !== canonicalJson(parsed)) throw Error("agent pid1 receipt file is not canonical");
    console.log(canonicalJson({ verified: true, unit: parsed.activation.unit, node: parsed.delivery.nodeId }));
}
