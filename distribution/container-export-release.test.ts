import { generateKeyPairSync } from "node:crypto";
import { mkdtemp, rename, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, test } from "bun:test";
import {
    publishMonitorContainerRelease,
    setContainerReleaseBeforeRereadHookForTest,
} from "./container-export-release.ts";
import { verifyContainerExport } from "./container-export-validator.ts";
import { verifyReleaseSnapshot } from "./release-publisher.ts";
import { publishSelectedRelease } from "./release-publisher.ts";
import { serverManifestFixture } from "./release-manifest-fixtures.ts";
import {
    createExport,
    releaseVersion,
    selection,
    sourceIdentity,
} from "./container-export-test-fixture.ts";

test("captured Monitor snapshot publishes and rereads an immutable signed triplet", async () => {
    const exportRoot = await createExport();
    const trustedRoot = await mkdtemp(join(tmpdir(), "rz-captured-release-"));
    const keys = generateKeyPairSync("ed25519");
    const trusted = {
        keyId: "captured-test",
        publicKey: keys.publicKey
            .export({ type: "spki", format: "pem" })
            .toString(),
    };
    try {
        const snapshot = await verifyContainerExport(
            exportRoot,
            selection,
            sourceIdentity,
            releaseVersion,
        );
        const result = await publishMonitorContainerRelease({
            snapshot,
            outputParent: join(trustedRoot, "output"),
            trustedRoot,
            privateKey: keys.privateKey
                .export({ type: "pkcs8", format: "pem" })
                .toString(),
            trusted,
        });
        expect(result.manifest.buildId).toBe(result.buildId);
        expect(
            (await verifyReleaseSnapshot(result.root, selection, trusted))
                .manifest,
        ).toEqual(result.manifest);
        expect(
            (await Bun.file(join(result.root, "archive.tar")).stat()).mode &
                0o777,
        ).toBe(0o644);
        await expect(
            publishMonitorContainerRelease({
                snapshot,
                outputParent: join(trustedRoot, "output"),
                trustedRoot,
                privateKey: keys.privateKey
                    .export({ type: "pkcs8", format: "pem" })
                    .toString(),
                trusted,
            }),
        ).rejects.toThrow("output");
    } finally {
        await rm(exportRoot, { recursive: true, force: true });
        await rm(trustedRoot, { recursive: true, force: true });
    }
});

test("captured release rejects a valid replacement after publication", async () => {
    const exportRoot = await createExport();
    const trustedRoot = await mkdtemp(join(tmpdir(), "rz-captured-reread-"));
    const keys = generateKeyPairSync("ed25519");
    const privateKey = keys.privateKey
        .export({ type: "pkcs8", format: "pem" })
        .toString();
    const trusted = {
        keyId: "reread-test",
        publicKey: keys.publicKey
            .export({ type: "spki", format: "pem" })
            .toString(),
    };
    const fixture = await serverManifestFixture();
    try {
        const alternate = await publishSelectedRelease({
            selection,
            staging: fixture.staging,
            manifest: fixture.manifest,
            privateKey,
            trusted,
        });
        const snapshot = await verifyContainerExport(
            exportRoot,
            selection,
            sourceIdentity,
            releaseVersion,
        );
        setContainerReleaseBeforeRereadHookForTest(async (root) => {
            await rm(root, { recursive: true, force: true });
            await rename(alternate.root, root);
        });
        await expect(
            publishMonitorContainerRelease({
                snapshot,
                outputParent: join(trustedRoot, "output"),
                trustedRoot,
                privateKey,
                trusted,
            }),
        ).rejects.toThrow("publication tuple");
    } finally {
        setContainerReleaseBeforeRereadHookForTest();
        await rm(exportRoot, { recursive: true, force: true });
        await rm(trustedRoot, { recursive: true, force: true });
        await rm(fixture.root, { recursive: true, force: true });
    }
});

test("invalid key material fails before captured staging can create output", async () => {
    const exportRoot = await createExport();
    const trustedRoot = await mkdtemp(
        join(tmpdir(), "rz-captured-invalid-key-"),
    );
    const first = generateKeyPairSync("ed25519");
    const second = generateKeyPairSync("ed25519");
    const rsa = generateKeyPairSync("rsa", { modulusLength: 2048 });
    const snapshot = await verifyContainerExport(
        exportRoot,
        selection,
        sourceIdentity,
        releaseVersion,
    );
    const outputParent = join(trustedRoot, "output");
    const privateKey = first.privateKey
        .export({ type: "pkcs8", format: "pem" })
        .toString();
    try {
        for (const trusted of [
            {
                keyId: "bad key",
                publicKey: first.publicKey
                    .export({ type: "spki", format: "pem" })
                    .toString(),
            },
            {
                keyId: "mismatch",
                publicKey: second.publicKey
                    .export({ type: "spki", format: "pem" })
                    .toString(),
            },
            {
                keyId: "rsa",
                publicKey: rsa.publicKey
                    .export({ type: "spki", format: "pem" })
                    .toString(),
            },
        ]) {
            await expect(
                publishMonitorContainerRelease({
                    snapshot,
                    outputParent,
                    trustedRoot,
                    privateKey,
                    trusted,
                }),
            ).rejects.toThrow();
            expect(
                await Bun.file(join(outputParent, ".native-staging")).exists(),
            ).toBeFalse();
        }
    } finally {
        await rm(exportRoot, { recursive: true, force: true });
        await rm(trustedRoot, { recursive: true, force: true });
    }
});

test("monitor-notify publishes and rereads an exact signed triplet", async () => {
    const notify = {
        schemaVersion: 1,
        preset: "monitor-notify",
        target: "x86_64-unknown-linux-musl",
    };
    const root = await createExport(notify);
    const trustedRoot = await mkdtemp(join(tmpdir(), "rz-notify-release-"));
    const keys = generateKeyPairSync("ed25519");
    const privateKey = keys.privateKey
        .export({ type: "pkcs8", format: "pem" })
        .toString();
    const trusted = {
        keyId: "notify-triplet",
        publicKey: keys.publicKey
            .export({ type: "spki", format: "pem" })
            .toString(),
    };
    try {
        const snapshot = await verifyContainerExport(
            root,
            notify,
            sourceIdentity,
            releaseVersion,
        );
        const release = await publishMonitorContainerRelease({
            snapshot,
            outputParent: join(trustedRoot, "out"),
            trustedRoot,
            privateKey,
            trusted,
        });
        expect(release.manifest.preset).toBe("monitor-notify");
        expect(
            await verifyReleaseSnapshot(release.root, notify, trusted),
        ).toMatchObject({ manifest: { preset: "monitor-notify" } });
    } finally {
        await rm(root, { recursive: true, force: true });
        await rm(trustedRoot, { recursive: true, force: true });
    }
});
