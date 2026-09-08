import { generateKeyPairSync } from "node:crypto";
import { mkdtemp, rename, rm, symlink, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { tmpdir } from "node:os";
import { expect, test } from "bun:test";
import { createExport, releaseVersion, selection, sourceIdentity } from "./container-export-test-fixture.ts";
import { publishMonitorContainerRelease } from "./container-export-release.ts";
import { verifyContainerExport } from "./container-export-validator.ts";
import { issueSourceBuildCertificate } from "./source-build-issuer.ts";
import { publishSourceBuildCertificate } from "./source-build-publisher.ts";
import { verifyPublishedSourceBuildCertificate } from "./published-source-build-certificate.ts";

test("published certificate verifier accepts only issued canonical stable evidence", async () => {
    const exportRoot = await createExport();
    const trustedRoot = await mkdtemp(join(tmpdir(), "rz-published-cert-"));
    const keys = generateKeyPairSync("ed25519");
    const privateKey = keys.privateKey.export({ type: "pkcs8", format: "pem" }).toString();
    const trusted = { keyId: "published-cert", publicKey: keys.publicKey.export({ type: "spki", format: "pem" }).toString() };
    try {
        const snapshot = await verifyContainerExport(exportRoot, selection, sourceIdentity, releaseVersion);
        const release = await publishMonitorContainerRelease({ snapshot, outputParent: join(trustedRoot, "output"), trustedRoot, privateKey, trusted });
        const issued = await issueSourceBuildCertificate({ snapshot, releaseRoot: release.root, trustedRoot, trusted });
        const published = await publishSourceBuildCertificate({ issued });
        const verified = await verifyPublishedSourceBuildCertificate({ issued, certificatePath: published.path });
        expect(verified.buildId).toBe(release.buildId);
        expect(verified.selection).toEqual({ target: "x86_64-unknown-linux-musl", artifactClass: "server", compositionId: release.manifest.compositionId });
        expect(verified.binaryDigests).toEqual(release.manifest.binaryDigests.map(({ path, sha256 }) => ({ path, sha256 })));
        expect(verified.manifestSha256).toBe(release.manifestSha256);
        await expect(verifyPublishedSourceBuildCertificate({ issued: {} as typeof issued, certificatePath: published.path })).rejects.toThrow("capability");
        await expect(verifyPublishedSourceBuildCertificate({ issued, certificatePath: join(dirname(published.path), "other.json") })).rejects.toThrow("canonical");
        await rm(published.path); await symlink("other", published.path);
        await expect(verifyPublishedSourceBuildCertificate({ issued, certificatePath: published.path })).rejects.toThrow();
        await rm(published.path); await writeFile(published.path, "{}", { mode: 0o644 });
        await expect(verifyPublishedSourceBuildCertificate({ issued, certificatePath: published.path })).rejects.toThrow();
        const certificateRoot = dirname(published.path), renamed = certificateRoot + "-renamed";
        await rename(certificateRoot, renamed); await symlink(renamed, certificateRoot);
        await expect(verifyPublishedSourceBuildCertificate({ issued, certificatePath: published.path })).rejects.toThrow("unsafe");
        await rm(certificateRoot); await rename(renamed, certificateRoot);
    } finally { await rm(exportRoot, { recursive: true, force: true }); await rm(trustedRoot, { recursive: true, force: true }); }
});
