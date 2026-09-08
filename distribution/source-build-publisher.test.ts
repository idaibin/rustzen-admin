import { generateKeyPairSync } from "node:crypto";
import { mkdtemp, readdir, rm, stat, symlink, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { tmpdir } from "node:os";
import { expect, test } from "bun:test";
import {
    createExport,
    releaseVersion,
    selection,
    sourceIdentity,
} from "./container-export-test-fixture.ts";
import { publishMonitorContainerRelease } from "./container-export-release.ts";
import { verifyContainerExport } from "./container-export-validator.ts";
import { issueSourceBuildCertificate } from "./source-build-issuer.ts";
import {
    publishSourceBuildCertificate,
    setSourceBuildBeforeRenameHookForTest,
    setSourceBuildDirectorySyncHookForTest,
} from "./source-build-publisher.ts";

test("certificate publisher accepts only issued evidence and preserves its final", async () => {
    const exportRoot = await createExport();
    const trustedRoot = await mkdtemp(join(tmpdir(), "rz-cert-publish-"));
    const keys = generateKeyPairSync("ed25519");
    const privateKey = keys.privateKey
        .export({ type: "pkcs8", format: "pem" })
        .toString();
    const trusted = {
        keyId: "certificate-publisher",
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
        const release = await publishMonitorContainerRelease({
            snapshot,
            outputParent: join(trustedRoot, "output"),
            trustedRoot,
            privateKey,
            trusted,
        });
        const issued = await issueSourceBuildCertificate({
            snapshot,
            releaseRoot: release.root,
            trustedRoot,
            trusted,
        });
        await expect(publishSourceBuildCertificate({
            issued: {} as typeof issued,
        })).rejects.toThrow("capability");
        await expect(publishSourceBuildCertificate({
            issued,
            extra: true,
        } as unknown as { issued: typeof issued })).rejects.toThrow("fields");
        const certificateRoot = join(dirname(release.root), "source-build-certificate");
        const raced = join(certificateRoot, "raced");
        const synced: string[] = [];
        setSourceBuildDirectorySyncHookForTest((path) => { synced.push(path); });
        setSourceBuildBeforeRenameHookForTest(async () => {
            await writeFile(raced, "preserve", { mode: 0o600 });
        });
        await expect(publishSourceBuildCertificate({ issued })).rejects.toThrow("inventory");
        expect(synced).toContain(dirname(release.root));
        expect(await Bun.file(raced).text()).toBe("preserve");
        expect(await Bun.file(join(certificateRoot, "source-build-manifest.json")).exists()).toBeFalse();
        await rm(raced);
        setSourceBuildBeforeRenameHookForTest();
        setSourceBuildDirectorySyncHookForTest();
        const foreign = join(certificateRoot, "notes");
        await writeFile(foreign, "preserve", { mode: 0o600 });
        await expect(publishSourceBuildCertificate({ issued })).rejects.toThrow("inventory");
        expect(await Bun.file(foreign).text()).toBe("preserve");
        expect(await Bun.file(join(certificateRoot, "source-build-manifest.json")).exists()).toBeFalse();
        await rm(foreign);
        const finalPath = join(certificateRoot, "source-build-manifest.json");
        const lockPath = join(certificateRoot, ".source-build.lock");
        let durableCleanSync = false;
        setSourceBuildDirectorySyncHookForTest(async (path) => {
            if (
                path === certificateRoot &&
                await Bun.file(finalPath).exists() &&
                !await Bun.file(lockPath).exists()
            ) durableCleanSync = true;
        });
        const published = await publishSourceBuildCertificate({ issued });
        expect(durableCleanSync).toBeTrue();
        setSourceBuildDirectorySyncHookForTest();
        expect((await stat(published.path)).mode & 0o777).toBe(0o644);
        expect(await readdir(dirname(published.path))).toEqual([
            "source-build-manifest.json",
        ]);
        await expect(publishSourceBuildCertificate({ issued })).rejects.toThrow("output");
        await rm(published.path, { force: true });
        await symlink("other", published.path);
        await expect(publishSourceBuildCertificate({ issued })).rejects.toThrow("output");
    } finally {
        setSourceBuildBeforeRenameHookForTest();
        setSourceBuildDirectorySyncHookForTest();
        await rm(exportRoot, { recursive: true, force: true });
        await rm(trustedRoot, { recursive: true, force: true });
    }
});
