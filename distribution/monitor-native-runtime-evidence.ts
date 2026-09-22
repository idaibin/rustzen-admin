import { canonicalJson, validHash } from "./release-manifest-core.ts";

export type MonitorNativeRuntimeEvidence = {
    schemaVersion: 1;
    kind: "monitor-native-runtime-evidence" | "analytics-native-runtime-evidence";
    platform: "linux/amd64";
    selection: { preset: "monitor" | "monitor-notify" | "analytics"; target: string; artifactClass: "server"; compositionId: string; buildId: string };
    release: { keyId: string; certificateSha256: string; manifestSha256: string; archiveSha256: string; envelopeSha256: string; binaryDigests: Array<{ path: string; sha256: string }> };
    installation: { verify: true; dryRun: true; apply: true; installStatus: true };
    markers: { publicationSha256: string; activationSha256: string };
    services: Array<{ unit: "rz-admin.service" | "rz-monitor.service" | "rz-insights.service"; mainPid: number; executable: { dev: string; ino: string; sha256: string } }>;
    health: Array<{ service: "admin" | "monitor" | "insights"; buildId: string; compositionId: string }>;
    checks: { ownerLogin: true; defaultPasswordsRejected: true; insightsAbsent?: true; monitorAbsent?: true; reportsAbsent: true; restart: true; adminThenMonitor?: true; monitorThenAdmin?: true; adminThenInsights?: true; insightsThenAdmin?: true; notificationIngress?: "absent" | "unauthorized" };
    runtime: true;
    browser: false;
    load: false;
    releaseReady: false;
};

export function monitorNativeRuntimeEvidenceBytes(value: unknown): Uint8Array {
    return new TextEncoder().encode(canonicalJson(parseMonitorNativeRuntimeEvidence(value)));
}

export function parseMonitorNativeRuntimeEvidence(value: unknown): MonitorNativeRuntimeEvidence {
    const record = object(value, "runtime evidence");
    only(record, ["schemaVersion", "kind", "platform", "selection", "release", "installation", "markers", "services", "health", "checks", "runtime", "browser", "load", "releaseReady"]);
    if (record.schemaVersion !== 1 || (record.kind !== "monitor-native-runtime-evidence" && record.kind !== "analytics-native-runtime-evidence") || record.platform !== "linux/amd64")
        throw new Error("runtime evidence version, kind or platform is invalid");
    const analytics = record.kind === "analytics-native-runtime-evidence";
    const binaryPaths = analytics ? ["bin/rz-admin", "bin/rz-insights"] : ["bin/rz-admin", "bin/rz-monitor"];
    const unitNames = analytics ? ["rz-admin.service", "rz-insights.service"] : ["rz-admin.service", "rz-monitor.service"];
    const serviceNames = analytics ? ["admin", "insights"] : ["admin", "monitor"];
    const selection = object(record.selection, "runtime evidence selection");
    const release = object(record.release, "runtime evidence release");
    const installation = object(record.installation, "runtime evidence installation");
    const markers = object(record.markers, "runtime evidence markers");
    only(selection, ["preset", "target", "artifactClass", "compositionId", "buildId"]);
    only(release, ["keyId", "certificateSha256", "manifestSha256", "archiveSha256", "envelopeSha256", "binaryDigests"]);
    only(installation, ["verify", "dryRun", "apply", "installStatus"]);
    only(markers, ["publicationSha256", "activationSha256"]);
    const binaryDigests = list(release.binaryDigests, "certificate binary digests").map((item) => {
        const entry = object(item, "certificate binary digest"); only(entry, ["path", "sha256"]);
        return { path: string(entry.path), sha256: validHash(string(entry.sha256)) };
    });
    if (canonicalJson(binaryDigests.map((x) => x.path)) !== canonicalJson(binaryPaths)) throw new Error("runtime evidence certificate inventory is invalid");
    const services = list(record.services, "services").map((item) => {
        const service = object(item, "runtime service"); only(service, ["unit", "mainPid", "executable"]);
        const executable = object(service.executable, "runtime executable"); only(executable, ["dev", "ino", "sha256"]);
        if (!unitNames.includes(service.unit) || !Number.isSafeInteger(service.mainPid) || service.mainPid < 1)
            throw new Error("runtime service is invalid");
        return { unit: service.unit, mainPid: service.mainPid, executable: { dev: decimal(executable.dev), ino: decimal(executable.ino), sha256: validHash(string(executable.sha256)) } };
    });
    const health = list(record.health, "health").map((item) => {
        const entry = object(item, "runtime health"); only(entry, ["service", "buildId", "compositionId"]);
        if (!serviceNames.includes(entry.service)) throw new Error("runtime health service is invalid");
        return { service: entry.service, buildId: validHash(string(entry.buildId)), compositionId: validHash(string(entry.compositionId)) };
    });
    if (canonicalJson(services.map((x) => x.unit)) !== canonicalJson(unitNames) || canonicalJson(health.map((x) => x.service)) !== canonicalJson(serviceNames))
        throw new Error("runtime evidence service inventory is invalid");
    const checks = object(record.checks, "runtime evidence checks"); only(checks, analytics ? ["ownerLogin", "defaultPasswordsRejected", "monitorAbsent", "reportsAbsent", "restart", "adminThenInsights", "insightsThenAdmin"] : ["ownerLogin", "defaultPasswordsRejected", "insightsAbsent", "reportsAbsent", "restart", "adminThenMonitor", "monitorThenAdmin", "notificationIngress"]);
    if (record.runtime !== true || record.browser !== false || record.load !== false || record.releaseReady !== false) throw new Error("runtime evidence later-layer flags are invalid");
    const required = analytics
        ? [installation.verify, installation.dryRun, installation.apply, installation.installStatus, checks.ownerLogin, checks.defaultPasswordsRejected, checks.monitorAbsent, checks.reportsAbsent, checks.restart, checks.adminThenInsights, checks.insightsThenAdmin]
        : [installation.verify, installation.dryRun, installation.apply, installation.installStatus, checks.ownerLogin, checks.defaultPasswordsRejected, checks.insightsAbsent, checks.reportsAbsent, checks.restart, checks.adminThenMonitor, checks.monitorThenAdmin];
    if (!required.every((value) => value === true)
        || (analytics && (selection.preset !== "analytics" || checks.notificationIngress !== undefined || checks.insightsAbsent !== undefined || checks.adminThenMonitor !== undefined))
        || (!analytics && selection.preset === "analytics")
        || (!analytics && (selection.preset === "monitor" && checks.notificationIngress !== "absent"))
        || (!analytics && selection.preset === "monitor-notify" && checks.notificationIngress !== "unauthorized")) throw new Error("runtime evidence required check is false");
    const result = { schemaVersion: 1 as const, kind: (analytics ? "analytics-native-runtime-evidence" : "monitor-native-runtime-evidence") as MonitorNativeRuntimeEvidence["kind"], platform: "linux/amd64" as const,
        selection: { preset: selection.preset === "monitor" || selection.preset === "monitor-notify" || selection.preset === "analytics" ? selection.preset : (() => { throw new Error("runtime evidence preset is invalid"); })(), target: string(selection.target), artifactClass: selection.artifactClass === "server" ? "server" as const : (() => { throw new Error("runtime evidence artifact class is invalid"); })(), compositionId: validHash(string(selection.compositionId)), buildId: validHash(string(selection.buildId)) },
        release: { keyId: string(release.keyId), certificateSha256: validHash(string(release.certificateSha256)), manifestSha256: validHash(string(release.manifestSha256)), archiveSha256: validHash(string(release.archiveSha256)), envelopeSha256: validHash(string(release.envelopeSha256)), binaryDigests },
        installation: { verify: true as const, dryRun: true as const, apply: true as const, installStatus: true as const },
        markers: { publicationSha256: validHash(string(markers.publicationSha256)), activationSha256: validHash(string(markers.activationSha256)) }, services, health,
        checks: analytics
            ? { ownerLogin: true as const, defaultPasswordsRejected: true as const, monitorAbsent: true as const, reportsAbsent: true as const, restart: true as const, adminThenInsights: true as const, insightsThenAdmin: true as const }
            : { ownerLogin: true as const, defaultPasswordsRejected: true as const, insightsAbsent: true as const, reportsAbsent: true as const, restart: true as const, adminThenMonitor: true as const, monitorThenAdmin: true as const, notificationIngress: checks.notificationIngress === "absent" || checks.notificationIngress === "unauthorized" ? checks.notificationIngress : (() => { throw new Error("runtime evidence notification check is invalid"); })() }, runtime: true as const, browser: false as const, load: false as const, releaseReady: false as const };
    if (result.selection.target !== "x86_64-unknown-linux-musl" || health.some((entry) => entry.buildId !== result.selection.buildId || entry.compositionId !== result.selection.compositionId))
        throw new Error("runtime evidence health bindings differ");
    const serviceDigests = new Map(result.services.map((service) => [service.unit === "rz-admin.service" ? "bin/rz-admin" : service.unit === "rz-monitor.service" ? "bin/rz-monitor" : "bin/rz-insights", service.executable.sha256]));
    if (binaryDigests.some((entry) => serviceDigests.get(entry.path) !== entry.sha256)) throw new Error("runtime evidence executable differs from certificate");
    return result;
}
function object(value: unknown, label: string): Record<string, any> { if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`${label} must be an object`); return value as Record<string, any>; }
function list(value: unknown, label: string): unknown[] { if (!Array.isArray(value)) throw new Error(`${label} must be an array`); return value; }
function only(value: Record<string, any>, allowed: string[]) { for (const key of Object.keys(value)) if (!allowed.includes(key)) throw new Error(`unknown runtime evidence field: ${key}`); }
function string(value: unknown): string { if (typeof value !== "string" || !value) throw new Error("runtime evidence string is invalid"); return value; }
function decimal(value: unknown): string { const result = string(value); if (!/^[0-9]+$/.test(result)) throw new Error("runtime evidence inode is invalid"); return result; }
