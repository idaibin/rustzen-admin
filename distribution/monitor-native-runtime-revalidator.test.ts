import { expect, test } from "bun:test";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import { revalidateMonitorNativeRuntime } from "./monitor-native-runtime-revalidator.ts";

test("revalidator fails closed for certificate, process, marker and encoding changes", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-p8e-revalidate-"));
    const hash = "a".repeat(64), build = "b".repeat(64), composition = "c".repeat(64);
    const binary = [{ path: "bin/rz-admin", sha256: hash }, { path: "bin/rz-monitor", sha256: hash }];
    const publication = { journal: { archiveSha256: hash, artifactClass: "server", buildId: build, compositionId: composition, envelopeSha256: hash, keyId: "key", manifestSha256: hash, phase: "payload-publishing", target: "x86_64-unknown-linux-musl", trustedKeySha256: hash, version: 1, workRootNonce: "d".repeat(32) }, state: "payload-published", version: 1 };
    const activation = { adminConfigSha256: hash, buildId: build, monitorConfigSha256: hash, schemaFingerprint: { admin: hash, monitor: hash }, dataContractId: { admin: hash, monitor: hash }, state: "ready", unitSha256: { "rz-admin.service": hash, "rz-monitor.service": hash, "rz.target": hash }, version: 2 };
    const evidence: any = { schemaVersion: 1, kind: "monitor-native-runtime-evidence", platform: "linux/amd64", selection: { preset: "monitor", target: "x86_64-unknown-linux-musl", artifactClass: "server", compositionId: composition, buildId: build }, release: { keyId: "key", certificateSha256: hash, manifestSha256: hash, archiveSha256: hash, envelopeSha256: hash, binaryDigests: binary }, installation: { verify: true, dryRun: true, apply: true, installStatus: true }, markers: { publicationSha256: sha256(canonicalJson(publication)), activationSha256: sha256(canonicalJson(activation)) }, services: [{ unit: "rz-admin.service", mainPid: 1, executable: { dev: "1", ino: "2", sha256: hash } }, { unit: "rz-monitor.service", mainPid: 2, executable: { dev: "1", ino: "3", sha256: hash } }], health: [{ service: "admin", buildId: build, compositionId: composition }, { service: "monitor", buildId: build, compositionId: composition }], checks: { ownerLogin: true, defaultPasswordsRejected: true, insightsAbsent: true, reportsAbsent: true, restart: true, adminThenMonitor: true, monitorThenAdmin: true }, runtime: true, browser: false, load: false, releaseReady: false };
    const files: Record<string, unknown> = {
        evidence, admission: { archiveSha256: hash, binaryDigests: binary, buildId: build, certificateSha256: hash, envelopeSha256: hash, manifestSha256: hash, selection: { artifactClass: "server", compositionId: composition, target: "x86_64-unknown-linux-musl" } }, release: { archiveSha256: hash, buildId: build, envelopeSha256: hash, manifestSha256: hash, root },
        facts: { keyId: "key", buildId: build, compositionId: composition, certificateSha256: hash, manifestSha256: hash, archiveSha256: hash, envelopeSha256: hash, publicationSha256: evidence.markers.publicationSha256, activationSha256: evidence.markers.activationSha256, adminPid: 1, adminDev: "1", adminIno: "2", adminSha256: hash, monitorPid: 2, monitorDev: "1", monitorIno: "3", monitorSha256: hash },
        logins: { owner: { status: "200", body: { data: { token: "token" } } }, defaults: ["owner", "admin", "viewer"].map((account) => ({ account, status: "401", body: { code: 10101 } })) },
        verify: envelope("verify", { build_id: build, artifact_class: "server", target: "x86_64-unknown-linux-musl", files: 1 }), dry: envelope("apply", { dry_run: true, release: releaseData(build) }), apply: envelope("apply", { dry_run: false, release: releaseData(build) }), status: envelope("install-status", { present: true, markerPresent: true, runnable: false, destination: "/opt/rz", current: `releases/${build}/payload` }), activate: envelope("activate-monitor-server", { unit: "rz.target", config: ["/opt/rz/config/rz-admin.env", "/opt/rz/config/rz-monitor.env"] }), publication, activation,
    };
    const input: any = Object.fromEntries(Object.keys(files).map((name) => [`${name === "dry" ? "dryRun" : name === "status" ? "status" : name}Path`, join(root, `${name}.json`)]));
    input.releaseResultPath = input.releasePath; delete input.releasePath; input.loginEvidencePath = input.loginsPath; delete input.loginsPath; input.publicationMarkerPath = input.publicationPath; delete input.publicationPath; input.activationMarkerPath = input.activationPath; delete input.activationPath;
    const write = async () => { for (const [name, value] of Object.entries(files)) await writeFile(join(root, `${name}.json`), canonicalJson(value)); };
    try {
        await write(); await expect(revalidateMonitorNativeRuntime(input)).resolves.toEqual(evidence);
        (files.admission as any).certificateSha256 = build; await write(); await expect(revalidateMonitorNativeRuntime(input)).rejects.toThrow("evidence selection differs");
        (files.admission as any).certificateSha256 = hash; (files.facts as any).adminPid = 9; await write(); await expect(revalidateMonitorNativeRuntime(input)).rejects.toThrow("runtime facts differ");
        (files.facts as any).adminPid = 1; (files.publication as any).extra = true; await write(); await expect(revalidateMonitorNativeRuntime(input)).rejects.toThrow("marker bytes differ");
        delete (files.publication as any).extra; delete (files.activation as any).state; await write(); await expect(revalidateMonitorNativeRuntime(input)).rejects.toThrow("marker bytes differ");
        files.activation = activation; await write(); await writeFile(input.evidencePath, canonicalJson(evidence) + "\n"); await expect(revalidateMonitorNativeRuntime(input)).rejects.toThrow("not canonical");
    } finally { await rm(root, { recursive: true, force: true }); }
});
function envelope(command: string, data: unknown) { return { schema_version: 1, ok: true, command, data }; }
function releaseData(build: string) { return { build_id: build, artifact_class: "server", target: "x86_64-unknown-linux-musl", files: 1 }; }
