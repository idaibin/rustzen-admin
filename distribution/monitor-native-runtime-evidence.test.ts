import { expect, test } from "bun:test";
import { monitorNativeRuntimeEvidenceBytes, parseMonitorNativeRuntimeEvidence, type MonitorNativeRuntimeEvidence } from "./monitor-native-runtime-evidence.ts";
import { canonicalJson } from "./release-manifest-core.ts";

const hash = "a".repeat(64);
const evidence: MonitorNativeRuntimeEvidence = {
    schemaVersion: 1, kind: "monitor-native-runtime-evidence", platform: "linux/amd64",
    selection: { preset: "monitor", target: "x86_64-unknown-linux-musl", artifactClass: "server", compositionId: hash, buildId: hash },
    release: { keyId: "test", certificateSha256: hash, manifestSha256: hash, archiveSha256: hash, envelopeSha256: hash, binaryDigests: [{ path: "bin/rz-admin", sha256: hash }, { path: "bin/rz-monitor", sha256: hash }] },
    installation: { verify: true, dryRun: true, apply: true, installStatus: true },
    markers: { publicationSha256: hash, activationSha256: hash },
    services: [
        { unit: "rz-admin.service", mainPid: 1, executable: { dev: "1", ino: "2", sha256: hash } },
        { unit: "rz-monitor.service", mainPid: 2, executable: { dev: "1", ino: "3", sha256: hash } },
    ],
    health: [{ service: "admin", buildId: hash, compositionId: hash }, { service: "monitor", buildId: hash, compositionId: hash }],
    checks: { ownerLogin: true, defaultPasswordsRejected: true, insightsAbsent: true, reportsAbsent: true, restart: true, adminThenMonitor: true, monitorThenAdmin: true, notificationIngress: "absent" },
    runtime: true, browser: false, load: false, releaseReady: false,
};
test("runtime evidence is closed and binds both health records", () => {
    expect(parseMonitorNativeRuntimeEvidence(evidence)).toEqual(evidence);
    expect(new TextDecoder().decode(monitorNativeRuntimeEvidenceBytes(evidence as unknown))).toBe(canonicalJson(evidence));
    expect(() => parseMonitorNativeRuntimeEvidence({ ...evidence, later: true })).toThrow("unknown");
    expect(() => parseMonitorNativeRuntimeEvidence({ ...evidence, platform: "linux/arm64" })).toThrow("platform");
    expect(() => parseMonitorNativeRuntimeEvidence({ ...evidence, health: [{ ...evidence.health[0] }, { ...evidence.health[1], buildId: "b".repeat(64) }] })).toThrow("bindings");
});

test("runtime evidence requires the Monitor notification-ingress result", () => {
    expect(new TextDecoder().decode(monitorNativeRuntimeEvidenceBytes(evidence))).toBe(canonicalJson(evidence));
    const monitor = structuredClone(evidence); delete monitor.checks.notificationIngress;
    expect(() => parseMonitorNativeRuntimeEvidence(monitor)).toThrow("required check");
    const notify = structuredClone(evidence);
    notify.selection.preset = "monitor-notify";
    notify.checks.notificationIngress = "unauthorized";
    expect(parseMonitorNativeRuntimeEvidence(notify)).toEqual(notify);
    notify.checks.notificationIngress = "absent";
    expect(() => parseMonitorNativeRuntimeEvidence(notify)).toThrow("required check");
});

const analyticsEvidence: MonitorNativeRuntimeEvidence = {
    ...evidence,
    kind: "analytics-native-runtime-evidence",
    selection: { ...evidence.selection, preset: "analytics" },
    release: { ...evidence.release, binaryDigests: [{ path: "bin/rz-admin", sha256: hash }, { path: "bin/rz-insights", sha256: hash }] },
    services: [
        evidence.services[0],
        { unit: "rz-insights.service", mainPid: 2, executable: { dev: "1", ino: "3", sha256: hash } },
    ],
    health: [evidence.health[0], { service: "insights", buildId: hash, compositionId: hash }],
    checks: { ownerLogin: true, defaultPasswordsRejected: true, monitorAbsent: true, reportsAbsent: true, restart: true, adminThenInsights: true, insightsThenAdmin: true },
};
test("analytics runtime evidence binds its family and rejects cross-family shapes", () => {
    expect(parseMonitorNativeRuntimeEvidence(analyticsEvidence)).toEqual(analyticsEvidence);
    expect(() => parseMonitorNativeRuntimeEvidence({ ...analyticsEvidence, selection: { ...analyticsEvidence.selection, preset: "monitor" } })).toThrow("required check");
    expect(() => parseMonitorNativeRuntimeEvidence({ ...evidence, selection: { ...evidence.selection, preset: "analytics" } })).toThrow("required check");
    expect(() => parseMonitorNativeRuntimeEvidence({ ...analyticsEvidence, checks: { ...analyticsEvidence.checks, notificationIngress: "absent" } })).toThrow("unknown");
});
