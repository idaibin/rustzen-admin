import { generateKeyPairSync } from "node:crypto";
import { join } from "node:path";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { expect, test } from "bun:test";
import { createExport, releaseVersion, selection, sourceIdentity } from "./container-export-test-fixture.ts";
import { verifyContainerExport } from "./container-export-validator.ts";
import { publishMonitorContainerRelease } from "./container-export-release.ts";
import { produceMonitorNativeStagingManifest } from "./container-export-native-staging.ts";
import { issueSourceBuildCertificate } from "./source-build-issuer.ts";
import { publishSourceBuildCertificate } from "./source-build-publisher.ts";
import { publishSelectedRelease } from "./release-publisher.ts";

test("issuer derives and atomically publishes one certificate from verified evidence", async () => {
    const exportRoot = await createExport();
    const trustedRoot = await mkdtemp(join(tmpdir(), "rz-source-build-"));
    const keys = generateKeyPairSync("ed25519");
    const privateKey = keys.privateKey.export({ type: "pkcs8", format: "pem" }).toString();
    const trusted = { keyId: "source-build", publicKey: keys.publicKey.export({ type: "spki", format: "pem" }).toString() };
    try {
        const snapshot = await verifyContainerExport(exportRoot, selection, sourceIdentity, releaseVersion);
        const release = await publishMonitorContainerRelease({ snapshot, outputParent: join(trustedRoot, "output"), trustedRoot, privateKey, trusted });
        await rm(release.stagingRoot, { recursive: true, force: true });
        const issued = await issueSourceBuildCertificate({ snapshot, releaseRoot: release.root, trustedRoot, trusted });
        const published = await publishSourceBuildCertificate({ issued });
        expect(published.certificate.certifiedLayers.source.identity).toBe(sourceIdentity);
        await expect(publishSourceBuildCertificate({ issued })).rejects.toThrow("output");
    } finally { await rm(exportRoot, { recursive: true, force: true }); await rm(trustedRoot, { recursive: true, force: true }); }
});

test("issuer rejects a fully signed manifest that differs from retained snapshot bytes", async () => {
    const exportRoot = await createExport();
    const trustedRoot = await mkdtemp(join(tmpdir(), "rz-source-build-mismatch-"));
    const keys = generateKeyPairSync("ed25519");
    const privateKey = keys.privateKey.export({ type: "pkcs8", format: "pem" }).toString();
    const trusted = { keyId: "source-build-mismatch", publicKey: keys.publicKey.export({ type: "spki", format: "pem" }).toString() };
    try {
        const snapshot = await verifyContainerExport(exportRoot, selection, sourceIdentity, releaseVersion);
        const staged = await produceMonitorNativeStagingManifest({
            snapshot,
            outputParent: join(trustedRoot, "altered"),
            trustedRoot,
        });
        const manifest = structuredClone(staged.manifest);
        if (manifest.artifactClass !== "server") throw new Error("expected server manifest");
        manifest.schemaFingerprints.admin = "f".repeat(64);
        const release = await publishSelectedRelease({
            selection,
            staging: staged.staging,
            manifest,
            privateKey,
            trusted,
        });
        const unrelatedRoot = await mkdtemp(join(tmpdir(), "rz-source-build-unrelated-"));
        try {
            await expect(issueSourceBuildCertificate({
                snapshot,
                releaseRoot: release.root,
                trustedRoot: unrelatedRoot,
                trusted,
            })).rejects.toThrow("escaped");
        } finally {
            await rm(unrelatedRoot, { recursive: true, force: true });
        }
        await expect(issueSourceBuildCertificate({
            snapshot,
            releaseRoot: release.root,
            trustedRoot,
            trusted,
        })).rejects.toThrow("full manifest");
    } finally {
        await rm(exportRoot, { recursive: true, force: true });
        await rm(trustedRoot, { recursive: true, force: true });
    }
});

test("issuer and publisher retain monitor-notify selection", async () => {
    const notify = { schemaVersion: 1, preset: "monitor-notify", target: "x86_64-unknown-linux-musl" };
    const exportRoot = await createExport(notify);
    const trustedRoot = await mkdtemp(join(tmpdir(), "rz-source-build-notify-"));
    const keys = generateKeyPairSync("ed25519");
    const privateKey = keys.privateKey.export({ type: "pkcs8", format: "pem" }).toString();
    const trusted = { keyId: "source-build-notify", publicKey: keys.publicKey.export({ type: "spki", format: "pem" }).toString() };
    try {
        const snapshot = await verifyContainerExport(exportRoot, notify, sourceIdentity, releaseVersion);
        const release = await publishMonitorContainerRelease({ snapshot, outputParent: join(trustedRoot, "output"), trustedRoot, privateKey, trusted });
        const published = await publishSourceBuildCertificate({ issued: await issueSourceBuildCertificate({ snapshot, releaseRoot: release.root, trustedRoot, trusted }) });
        expect(published.certificate.selection.preset).toBe("monitor-notify");
        expect(published.certificate.releaseReady).toBeFalse();
    } finally { await rm(exportRoot, { recursive: true, force: true }); await rm(trustedRoot, { recursive: true, force: true }); }
});
