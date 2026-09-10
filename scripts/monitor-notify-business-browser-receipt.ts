import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { requiredBusinessProvenance } from "./monitor-notify-business-provenance.ts";

const hash = /^[a-f0-9]{64}$/;
const sidecarNames = ["native-runtime-evidence", "release-result", "published-certificate.json", "facts.json", "login-evidence.json", "verify.json", "dry-run.json", "apply.json", "install-status.json", "activate.json", "publication-marker.json", "activation-marker.json", "notification-ingress.check"];
const screenshotNames = ["message-center.png", "message-detail.png"];
const object = (value: unknown): value is Record<string, unknown> => Boolean(value) && typeof value === "object" && !Array.isArray(value);
const exact = (value: unknown, keys: readonly string[]) => object(value) && canonicalJson(Object.keys(value).sort()) === canonicalJson([...keys].sort());
const text = (value: unknown) => typeof value === "string" && value.length > 0;
const tuple = (value: any) => exact(value, ["dev", "ino", "pid", "sha256"]) && Number.isSafeInteger(value.pid) && value.pid > 1 && ["dev", "ino", "sha256"].every(key => typeof value[key] === "string") && hash.test(value.sha256);
const screenshot = (value: any) => exact(value, ["bytes", "dimensions", "file", "sha256"]) && screenshotNames.includes(value.file) && Number.isSafeInteger(value.bytes) && value.bytes > 24 && exact(value.dimensions, ["height", "width"]) && value.dimensions.width === 1440 && value.dimensions.height === 900 && hash.test(value.sha256);
const source = (value: any) => exact(value, ["path", "sha256"]) && text(value.path) && hash.test(value.sha256);

function validP8e(value: any) {
    return exact(value, ["release", "selection", "services", "sidecars", "witness"])
        && exact(value.selection, ["artifactClass", "buildId", "compositionId", "preset", "target"])
        && value.selection.preset === "monitor-notify" && value.selection.artifactClass === "server" && value.selection.target === "x86_64-unknown-linux-musl" && hash.test(value.selection.buildId) && hash.test(value.selection.compositionId)
        && exact(value.release, ["archiveSha256", "certificateSha256", "envelopeSha256", "manifestSha256"]) && Object.values(value.release).every(item => hash.test(String(item)))
        && exact(value.sidecars, sidecarNames) && Object.values(value.sidecars).every(item => hash.test(String(item)))
        && exact(value.witness, ["outputManifestSha256", "provenanceSha256", "sha256"]) && Object.values(value.witness).every(item => hash.test(String(item)))
        && exact(value.services, ["after", "before"]) && tuple(value.services.before.admin) && tuple(value.services.before.monitor) && canonicalJson(value.services.before) === canonicalJson(value.services.after);
}

function validJourney(value: any, provenance: unknown) {
    const api = value.api, ui = value.ui, sse = value.sse, paths = Array.isArray(provenance) ? provenance.map((item: any) => item.path) : [];
    return exact(value, ["api", "ids", "routes", "screenshots", "sse", "ui", "witnessAgent"])
        && exact(value.ids, ["incidentId", "messageId"]) && text(value.ids.incidentId) && text(value.ids.messageId)
        && exact(value.routes, ["insightsAbsent", "monitorVisible", "reportsAbsent"]) && value.routes.insightsAbsent && value.routes.monitorVisible && value.routes.reportsAbsent
        && exact(sse, ["bearer", "contentType", "frameBytes", "frameBytesBefore", "listFetchTimestamp", "method", "readyTimestamp", "reportIssuedAt", "status", "tokenInUrl", "url"]) && sse.bearer && !sse.tokenInUrl && sse.method === "GET" && sse.url === "/api/notifications/stream" && sse.status === 200 && String(sse.contentType).includes("text/event-stream") && Number.isSafeInteger(sse.readyTimestamp) && Number.isSafeInteger(sse.reportIssuedAt) && Number.isSafeInteger(sse.listFetchTimestamp) && sse.frameBytes > sse.frameBytesBefore && sse.listFetchTimestamp >= sse.reportIssuedAt
        && exact(value.witnessAgent, ["agentVersion", "bootId", "mode", "nodeId", "pid", "stopped"]) && value.witnessAgent.mode === "development" && value.witnessAgent.nodeId === "p8fb-witness-node" && text(value.witnessAgent.bootId) && text(value.witnessAgent.agentVersion) && Number.isSafeInteger(value.witnessAgent.pid) && value.witnessAgent.pid > 1 && value.witnessAgent.stopped === true
        && Array.isArray(value.screenshots) && value.screenshots.length === screenshotNames.length && value.screenshots.every(screenshot) && canonicalJson(value.screenshots.map((item: any) => item.file).sort()) === canonicalJson(screenshotNames)
        && exact(api, ["deterministicReports", "inbox", "incident", "messageDetail"]) && exact(api.incident, ["nodeId", "status", "success", "total"]) && api.incident.status === 200 && api.incident.success === true && api.incident.nodeId === value.witnessAgent.nodeId && Number.isSafeInteger(api.incident.total) && api.incident.total > 0
        && exact(api.inbox, ["itemCount", "listFetchTimestamp", "revision", "snapshot", "status"]) && api.inbox.status === 200 && Number.isSafeInteger(api.inbox.itemCount) && api.inbox.itemCount > 0 && Number.isSafeInteger(api.inbox.revision) && api.inbox.revision >= 0 && text(api.inbox.snapshot) && Number.isSafeInteger(api.inbox.listFetchTimestamp)
        && Array.isArray(api.deterministicReports) && api.deterministicReports.length === 3 && api.deterministicReports.every((item: any, index: number) => exact(item, ["bootId", "httpStatus", "method", "nodeId", "sequence", "status", "url"]) && item.method === "POST" && item.httpStatus === 200 && item.status === "accepted" && item.url === "/api/monitor/agent-reports" && item.nodeId === value.witnessAgent.nodeId && item.bootId === value.witnessAgent.bootId && Number.isSafeInteger(item.sequence) && item.sequence > 0 && (!index || item.sequence === api.deterministicReports[index - 1].sequence + 1))
        && exact(api.messageDetail, ["id", "producer", "readAt", "subjectId", "title", "topic"]) && api.messageDetail.id === value.ids.messageId && api.messageDetail.subjectId === value.ids.incidentId && api.messageDetail.producer === "monitor" && api.messageDetail.topic === "monitor.incident.opened" && text(api.messageDetail.readAt) && text(api.messageDetail.title)
        && exact(ui, ["deepLink", "monitorNode", "read", "reloaded"]) && exact(ui.monitorNode, ["navigationSelector", "nodeId", "pathname", "selector", "text"]) && ui.monitorNode.nodeId === value.witnessAgent.nodeId && ui.monitorNode.pathname === "/monitoring/nodes" && text(ui.monitorNode.navigationSelector) && text(ui.monitorNode.selector) && text(ui.monitorNode.text) && ui.monitorNode.text.includes(value.witnessAgent.nodeId)
        && exact(ui.deepLink, ["drawerCount", "drawerSelector", "incidentId", "nodeId", "pathname", "searchIncidentId", "selector", "title"]) && ui.deepLink.incidentId === value.ids.incidentId && ui.deepLink.searchIncidentId === value.ids.incidentId && ui.deepLink.nodeId === value.witnessAgent.nodeId && ui.deepLink.pathname === "/monitoring/incidents" && ui.deepLink.drawerCount === 1 && [ui.deepLink.drawerSelector, ui.deepLink.selector, ui.deepLink.title].every(text)
        && exact(ui.read, ["buttonSelector", "buttonText", "messageId", "readAt", "selector", "unreadCountAfter", "unreadCountBefore"]) && ui.read.messageId === value.ids.messageId && ui.read.readAt === api.messageDetail.readAt && ui.read.buttonText === "Mark as read" && [ui.read.buttonSelector, ui.read.selector].every(text) && Number.isSafeInteger(ui.read.unreadCountBefore) && Number.isSafeInteger(ui.read.unreadCountAfter) && ui.read.unreadCountBefore > ui.read.unreadCountAfter && ui.read.unreadCountAfter >= 0
        && exact(ui.reloaded, ["drawerSelector", "matchingMessageCount", "messageId", "messageSelector", "unreadSelector"]) && ui.reloaded.messageId === value.ids.messageId && ui.reloaded.matchingMessageCount === 1 && [ui.reloaded.drawerSelector, ui.reloaded.messageSelector, ui.reloaded.unreadSelector].every(text) && ui.reloaded.unreadSelector === ui.read.selector
        && Array.isArray(provenance) && provenance.every(source) && canonicalJson(paths) === canonicalJson(requiredBusinessProvenance);
}

export function parseMonitorNotifyBusinessReceipt(value: unknown) {
    const keys = ["browser", "chromium", "journey", "load", "p8e", "p8fBootstrap", "provenance", "releaseReady", "schemaVersion", "status"];
    if (!exact(value, keys) || (value as any).schemaVersion !== 1 || (value as any).status !== "passed" || (value as any).browser !== true || (value as any).load !== false || (value as any).releaseReady !== false) throw Error("P8f-B receipt schema differs");
    const receipt: any = value;
    if (!validP8e(receipt.p8e)) throw Error("P8f-B P8e binding differs");
    const bootstrapKeys = ["archiveSha256", "bootstrapReceiptSha256", "buildId", "certificateSha256", "compositionId", "envelopeSha256", "manifestSha256", "runtimeAdminSha256"];
    if (!exact(receipt.p8fBootstrap, bootstrapKeys) || !Object.values(receipt.p8fBootstrap).every(item => hash.test(String(item))) || receipt.p8fBootstrap.buildId !== receipt.p8e.selection.buildId || receipt.p8fBootstrap.compositionId !== receipt.p8e.selection.compositionId || receipt.p8fBootstrap.runtimeAdminSha256 !== receipt.p8e.services.before.admin.sha256 || ["archiveSha256", "certificateSha256", "envelopeSha256", "manifestSha256"].some(key => receipt.p8fBootstrap[key] !== receipt.p8e.release[key])) throw Error("P8f-B bootstrap tuple differs");
    if (!exact(receipt.chromium, ["version"]) || !text(receipt.chromium.version) || !validJourney(receipt.journey, receipt.provenance)) throw Error("P8f-B journey differs");
    return value;
}

if (import.meta.main) {
    const path = Bun.argv[2];
    if (!path || Bun.argv.length !== 3) throw Error("usage: <receipt>");
    const input = await Bun.file(path).text(), value = parseMonitorNotifyBusinessReceipt(JSON.parse(input));
    if (input !== canonicalJson(value)) throw Error("P8f-B receipt is not canonical");
}
