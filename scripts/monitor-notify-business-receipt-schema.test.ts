import { expect, test } from "bun:test";
import { parseMonitorNotifyBusinessReceipt } from "./monitor-notify-business-browser-receipt.ts";
import { requiredBusinessProvenance } from "./monitor-notify-business-provenance.ts";

const hash = "a".repeat(64), nodeId = "p8fb-witness-node", bootId = "boot", tuple = { pid: 2, dev: "1", ino: "2", sha256: hash };
const sidecars = ["native-runtime-evidence", "release-result", "published-certificate.json", "facts.json", "login-evidence.json", "verify.json", "dry-run.json", "apply.json", "install-status.json", "activate.json", "publication-marker.json", "activation-marker.json", "notification-ingress.check"];
const release = { archiveSha256: hash, certificateSha256: hash, envelopeSha256: hash, manifestSha256: hash };
const receipt: any = {
    schemaVersion: 1, status: "passed", browser: true, load: false, releaseReady: false, chromium: { version: "Chrome" },
    p8e: { selection: { preset: "monitor-notify", artifactClass: "server", target: "x86_64-unknown-linux-musl", buildId: hash, compositionId: hash }, release, sidecars: Object.fromEntries(sidecars.map(name => [name, hash])), witness: { sha256: hash, outputManifestSha256: hash, provenanceSha256: hash }, services: { before: { admin: tuple, monitor: tuple }, after: { admin: tuple, monitor: tuple } } },
    p8fBootstrap: { bootstrapReceiptSha256: hash, buildId: hash, compositionId: hash, runtimeAdminSha256: hash, ...release },
    provenance: requiredBusinessProvenance.map(path => ({ path, sha256: hash })),
    journey: {
        witnessAgent: { mode: "development", nodeId, bootId, agentVersion: "agent", pid: 2, stopped: true },
        api: { deterministicReports: [2, 3, 4].map(sequence => ({ method: "POST", url: "/api/monitor/agent-reports", httpStatus: 200, status: "accepted", nodeId, bootId, sequence })), incident: { status: 200, success: true, total: 1, nodeId }, inbox: { status: 200, itemCount: 1, listFetchTimestamp: 2, revision: 1, snapshot: "snapshot" }, messageDetail: { id: "message", producer: "monitor", readAt: "time", subjectId: "incident", title: "CPU high", topic: "monitor.incident.opened" } },
        ids: { incidentId: "incident", messageId: "message" }, routes: { monitorVisible: true, insightsAbsent: true, reportsAbsent: true },
        ui: { monitorNode: { nodeId, pathname: "/monitoring/nodes", navigationSelector: "nav", selector: "row", text: nodeId }, deepLink: { incidentId: "incident", searchIncidentId: "incident", nodeId, pathname: "/monitoring/incidents", drawerCount: 1, drawerSelector: "drawer", selector: "related", title: "CPU high" }, read: { messageId: "message", readAt: "time", buttonSelector: "button", buttonText: "Mark as read", unreadCountBefore: 1, unreadCountAfter: 0, selector: "unread" }, reloaded: { messageId: "message", matchingMessageCount: 1, drawerSelector: "drawer", messageSelector: "message", unreadSelector: "unread" } },
        sse: { method: "GET", url: "/api/notifications/stream", bearer: true, tokenInUrl: false, status: 200, contentType: "text/event-stream", frameBytes: 2, frameBytesBefore: 1, readyTimestamp: 1, reportIssuedAt: 1, listFetchTimestamp: 2 },
        screenshots: ["message-center.png", "message-detail.png"].map(file => ({ file, sha256: hash, bytes: 25, dimensions: { width: 1440, height: 900 } })),
    },
};

test("P8f-B canonical receipt admits the complete bound journey", () => expect(parseMonitorNotifyBusinessReceipt(receipt)).toBe(receipt));

test("P8f-B rejects cross-subject, sequence, SSE, provenance, and extra-field tampering", () => {
    for (const mutate of [
        (value: any) => { value.journey.api.deterministicReports[1].status = "duplicate"; },
        (value: any) => { value.journey.api.deterministicReports[1].sequence = 9; },
        (value: any) => { value.journey.api.incident.nodeId = "other"; },
        (value: any) => { value.journey.api.messageDetail.subjectId = "other"; },
        (value: any) => { value.journey.sse.frameBytes = value.journey.sse.frameBytesBefore; },
        (value: any) => { value.provenance.push(value.provenance[0]); },
        (value: any) => { value.provenance[0].path = "extra"; },
        (value: any) => { value.journey.api.inbox.extra = true; },
        (value: any) => { value.journey.ui.monitorNode = {}; },
    ]) { const changed = structuredClone(receipt); mutate(changed); expect(() => parseMonitorNotifyBusinessReceipt(changed)).toThrow(); }
});
