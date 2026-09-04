import { generateKeyPairSync } from "node:crypto";
import { mkdir, rm, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { expect, test } from "bun:test";
import {
    monitorSelection,
    serverManifestFixture,
} from "../distribution/release-manifest-fixtures.ts";

test("selected release CLI reads PEM paths without emitting key material", async () => {
    const repository = resolve(import.meta.dir, "..");
    const fixture = await serverManifestFixture();
    const root = join(repository, "target", "release-cli-test");
    const keys = generateKeyPairSync("ed25519");
    const privateKey = keys.privateKey
        .export({ type: "pkcs8", format: "pem" })
        .toString();
    try {
        await mkdir(root, { recursive: true });
        await writeFile(
            join(root, "staging.json"),
            JSON.stringify(fixture.staging),
        );
        await writeFile(
            join(root, "manifest.json"),
            JSON.stringify(fixture.manifest),
        );
        await writeFile(join(root, "private.pem"), privateKey, { mode: 0o600 });
        await writeFile(
            join(root, "public.pem"),
            keys.publicKey.export({ type: "spki", format: "pem" }),
        );
        const result = Bun.spawnSync(
            [
                process.execPath,
                "scripts/distribution-publish-selected-release.ts",
                "--selection",
                "distribution/fixtures/monitor.json",
                "--staging",
                "target/release-cli-test/staging.json",
                "--manifest",
                "target/release-cli-test/manifest.json",
                "--private-key",
                join(root, "private.pem"),
                "--public-key",
                join(root, "public.pem"),
                "--key-id",
                "cli-test",
            ],
            { cwd: repository },
        );
        expect(result.exitCode).toBe(0);
        const output = new TextDecoder().decode(result.stdout);
        expect(JSON.parse(output).root).toEndWith("/release");
        expect(output).not.toContain(privateKey);
    } finally {
        await rm(root, { recursive: true, force: true });
        await rm(fixture.root, { recursive: true, force: true });
    }
});
