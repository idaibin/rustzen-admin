import { join, resolve } from "node:path";
import { lstat, rm } from "node:fs/promises";
import { mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { expect, test } from "bun:test";
import { produceMonitorNativeStagingManifest, produceSelectedServerSyntheticNativeStagingManifest } from "./container-export-native-staging.ts";
import { verifyContainerExport } from "./container-export-validator.ts";
import { createExport, releaseVersion, selection, sourceIdentity } from "./container-export-test-fixture.ts";

test("captured export bytes stage exact Monitor payload after export removal", async () => {
    const root = await createExport();
    const trusted = await mkdtemp(join(tmpdir(), "rz-captured-stage-"));
    try {
        const snapshot = await verifyContainerExport(root, selection, sourceIdentity, releaseVersion);
        await rm(root, { recursive: true, force: true });
        await expect(lstat(root)).rejects.toMatchObject({ code: "ENOENT" });
        const result = await produceMonitorNativeStagingManifest({ snapshot, outputParent: join(trusted, "native-output"), trustedRoot: trusted });
        expect(result.staging.files.map((file) => file.path)).toEqual([
            "bin/rz-admin", "bin/rz-monitor", "contracts/api/api.json", "contracts/config/config.json", "contracts/native/native-layout.json", "contracts/protocol/protocol.json", "contracts/schema/schema.json", "contracts/web/binding.json", "systemd/rz-admin.service", "systemd/rz-monitor.service", "systemd/rz.target", "web/index.html", "web/rustzen.png",
        ]);
        expect(result.manifest.files).toEqual(result.staging.files);
        expect(result.manifest.releaseVersion).toBe(releaseVersion);
        expect(result.manifest.sourceIdentity).toBe(sourceIdentity);
        expect(result.manifest.preset).toBe("monitor");
        expect(result.manifest.buildId).toBe(result.staging.buildId);

        await expect(produceMonitorNativeStagingManifest({ snapshot, outputParent: join(trusted, "native-output"), trustedRoot: trusted })).rejects.toThrow("output");
    } finally { await rm(root, { recursive: true, force: true }); await rm(trusted, { recursive: true, force: true }); }
});

test("captured staging CLI requires exact arguments", async () => {
    const root = await createExport(); const repositoryRoot = resolve(import.meta.dir, ".."); const outputBase = "target/container-native-staging-cli-test";
    try {
        const cli = Bun.spawnSync([process.execPath, join(repositoryRoot, "scripts/distribution-produce-container-native-staging.ts"), "--selection", "distribution/fixtures/monitor.json", "--export-root", root, "--expected-source-identity", sourceIdentity, "--output-base", outputBase], { cwd: repositoryRoot, stdout: "pipe", stderr: "pipe" });
        if (cli.exitCode !== 0) throw new Error(new TextDecoder().decode(cli.stderr));
        expect(cli.exitCode).toBe(0);
        const value = JSON.parse(new TextDecoder().decode(cli.stdout));
        expect(value.files).toBe(13);
        for (const args of [
            [],
            ["--unknown", "x"],
            ["--selection", "distribution/fixtures/monitor.json", "--selection", "distribution/fixtures/monitor.json"],
            ["--release-version", "0.5.0"],
        ]) {
            const rejected = Bun.spawnSync([
                process.execPath,
                join(repositoryRoot, "scripts/distribution-produce-container-native-staging.ts"),
                ...args,
            ], { cwd: repositoryRoot, stdout: "pipe", stderr: "pipe" });
            expect(rejected.exitCode).not.toBe(0);
        }
    } finally { await rm(join(repositoryRoot, outputBase), { recursive: true, force: true }); await rm(root, { recursive: true, force: true }); }
});

test("captured export bytes stage exact monitor-notify payload", async () => {
    const notify = { schemaVersion: 1, preset: "monitor-notify", target: "x86_64-unknown-linux-musl" };
    const root = await createExport(notify);
    const trusted = await mkdtemp(join(tmpdir(), "rz-captured-notify-stage-"));
    try {
        const snapshot = await verifyContainerExport(root, notify, sourceIdentity, releaseVersion);
        const result = await produceMonitorNativeStagingManifest({ snapshot, outputParent: join(trusted, "native-output"), trustedRoot: trusted });
        expect(result.manifest.preset).toBe("monitor-notify");
        expect(result.manifest.binaryDigests.map((entry) => entry.path)).toEqual(["bin/rz-admin", "bin/rz-monitor"]);
        expect(result.staging.files.map((entry) => entry.path)).not.toContain("bin/rz-monitor-agent");
    } finally { await rm(root, { recursive: true, force: true }); await rm(trusted, { recursive: true, force: true }); }
});

test("captured synthetic Analytics bytes stage Admin and Insights without an Agent witness", async () => {
    const analytics = { schemaVersion: 1, preset: "analytics", target: "x86_64-unknown-linux-musl" };
    const root = await createExport(analytics);
    const trusted = await mkdtemp(join(tmpdir(), "rz-captured-analytics-stage-"));
    try {
        const snapshot = await verifyContainerExport(root, analytics, sourceIdentity, releaseVersion);
        await rm(root, { recursive: true, force: true });
        const result = await produceSelectedServerSyntheticNativeStagingManifest({ snapshot, outputParent: join(trusted, "native-output"), trustedRoot: trusted });
        expect(result.staging.files.map((file) => file.path)).toEqual([
            "bin/rz-admin", "bin/rz-insights", "contracts/api/api.json", "contracts/config/config.json",
            "contracts/native/native-layout.json", "contracts/protocol/protocol.json", "contracts/schema/schema.json",
            "contracts/web/binding.json", "systemd/rz-admin.service", "systemd/rz-insights.service",
            "systemd/rz.target", "web/index.html", "web/rustzen.png",
        ]);
        expect(result.manifest.files).toEqual(result.staging.files);
        expect(result.manifest.protocolArtifactDigest).toMatch(/^[a-f0-9]{64}$/);
        expect(result.manifest.agentProtocolContractId).toMatch(/^[a-f0-9]{64}$/);
    } finally { await rm(root, { recursive: true, force: true }); await rm(trusted, { recursive: true, force: true }); }
});

test("reviewed manifest entry stages an Analytics snapshot and still rejects an unsupported preset", async () => {
    const analytics = { schemaVersion: 1, preset: "analytics", target: "x86_64-unknown-linux-musl" };
    const root = await createExport(analytics);
    const trusted = await mkdtemp(join(tmpdir(), "rz-captured-analytics-monitor-gate-"));
    try {
        const snapshot = await verifyContainerExport(root, analytics, sourceIdentity, releaseVersion);
        const result = await produceMonitorNativeStagingManifest({ snapshot, outputParent: join(trusted, "native-output"), trustedRoot: trusted });
        expect(result.manifest.preset).toBe("analytics");
        await expect(produceMonitorNativeStagingManifest({
            snapshot,
            outputParent: join(trusted, "native-output"),
            trustedRoot: trusted,
        })).rejects.toThrow("already exists");
    } finally { await rm(root, { recursive: true, force: true }); await rm(trusted, { recursive: true, force: true }); }
});
