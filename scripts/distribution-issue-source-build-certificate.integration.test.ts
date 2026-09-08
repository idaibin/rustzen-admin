import { generateKeyPairSync } from "node:crypto";
import { cp, mkdir, mkdtemp, readdir, rm, stat, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { expect, test } from "bun:test";
import {
    createExport,
    releaseVersion,
    selection,
    sourceIdentity,
} from "../distribution/container-export-test-fixture.ts";
import { verifyContainerExport } from "../distribution/container-export-validator.ts";
import { publishMonitorContainerRelease } from "../distribution/container-export-release.ts";
import { parseSourceBuildCertificate } from "../distribution/source-build-certificate.ts";

test("source-build certificate CLI publishes one canonical no-replace certificate", async () => {
    const repository = resolve(import.meta.dir, "..");
    await mkdir(join(repository, "target"), { recursive: true });
    const root = await mkdtemp(join(repository, "target", "source-build-cli-"));
    const fixtureExport = await createExport();
    const exportRoot = join(root, "export");
    const keys = generateKeyPairSync("ed25519");
    const privateKey = keys.privateKey
        .export({ type: "pkcs8", format: "pem" })
        .toString();
    const publicKey = keys.publicKey
        .export({ type: "spki", format: "pem" })
        .toString();
    const trusted = { keyId: "source-build-cli", publicKey };
    try {
        await cp(fixtureExport, exportRoot, { recursive: true });
        await writeFile(join(root, "public.pem"), publicKey, { mode: 0o644 });
        const snapshot = await verifyContainerExport(
            exportRoot,
            selection,
            sourceIdentity,
            releaseVersion,
        );
        const release = await publishMonitorContainerRelease({
            snapshot,
            outputParent: join(root, "output"),
            trustedRoot: repository,
            privateKey,
            trusted,
        });
        const relativePath = (path: string) => path.slice(repository.length + 1);
        const command = [
            process.execPath,
            "scripts/distribution-issue-source-build-certificate.ts",
            "--selection",
            "distribution/fixtures/monitor.json",
            "--export-root",
            relativePath(exportRoot),
            "--expected-source-identity",
            sourceIdentity,
            "--release-root",
            relativePath(release.root),
            "--public-key",
            join(root, "public.pem"),
            "--key-id",
            trusted.keyId,
        ];
        const result = Bun.spawnSync(command, {
            cwd: repository,
            stdout: "pipe",
            stderr: "pipe",
        });
        if (result.exitCode !== 0)
            throw new Error(new TextDecoder().decode(result.stderr));
        const output = JSON.parse(new TextDecoder().decode(result.stdout));
        expect(output.path).toEndWith(
            "/source-build-certificate/source-build-manifest.json",
        );
        expect((await stat(output.path)).mode & 0o777).toBe(0o644);
        expect(await readdir(resolve(output.path, ".."))).toEqual([
            "source-build-manifest.json",
        ]);
        expect(
            parseSourceBuildCertificate(
                await Bun.file(output.path).json(),
                selection,
            ).certifiedLayers.build.buildId,
        ).toBe(release.buildId);
        expect(Bun.spawnSync(command, { cwd: repository }).exitCode).not.toBe(0);
        for (const args of [
            [],
            ["--tree-sha256", "a".repeat(64)],
            ["--selection", "distribution/fixtures/monitor.json", "position"],
        ]) {
            const invalid = Bun.spawnSync(
                [process.execPath, command[1], ...args],
                { cwd: repository, stdout: "pipe", stderr: "pipe" },
            );
            expect(invalid.exitCode).not.toBe(0);
        }
    } finally {
        await rm(fixtureExport, { recursive: true, force: true });
        await rm(root, { recursive: true, force: true });
    }
});
