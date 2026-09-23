import { chmod, mkdir, rm, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { expect, test } from "bun:test";
import { releaseFixture } from "../distribution/release-manifest-fixtures.ts";

test("native staging CLI writes a build-qualified server payload", async () => {
    const repositoryRoot = resolve(import.meta.dir, "..");
    const fixture = await releaseFixture("server");
    const testRoot = join(repositoryRoot, "target", "native-staging-cli-test");
    const binaryRoot = join(testRoot, "binary");
    const outputBase = join(testRoot, "distributions");
    try {
        await mkdir(join(binaryRoot, "bin"), { recursive: true });
        for (const name of ["rz-admin", "rz-monitor"]) {
            await writeFile(join(binaryRoot, "bin", name), name);
            await chmod(join(binaryRoot, "bin", name), 0o755);
        }
        await mkdir(outputBase, { recursive: true, mode: 0o755 });
        await chmod(outputBase, 0o755);
        const result = Bun.spawnSync(
            [
                process.execPath,
                join(
                    repositoryRoot,
                    "scripts/distribution-produce-native-staging.ts",
                ),
                "--selection",
                "distribution/fixtures/monitor.json",
                "--binary-root",
                binaryRoot,
                "--web-root",
                fixture.webRoot,
                "--api-root",
                fixture.apiRoot,
                "--schema-root",
                fixture.schemaRoot,
                "--config-root",
                fixture.configRoot,
                "--native-root",
                fixture.nativeRoot,
                "--protocol-root",
                fixture.protocolRoot,
                "--output-base",
                outputBase,
                "--release-version",
                "1.0.0",
                "--source-identity",
                "integration",
                "--toolchain",
                "bun-test",
                "--selected-routes",
                "login,monitor",
            ],
            { cwd: repositoryRoot },
        );
        expect(result.exitCode).toBe(0);
        const value = JSON.parse(new TextDecoder().decode(result.stdout));
        expect(value.root).toContain("/.native-staging/");
        expect(value.root).toEndWith("/server/payload");
        expect(
            await Bun.file(join(value.root, "bin/rz-admin")).exists(),
        ).toBeTrue();
        expect(
            await Bun.file(
                join(value.root, "contracts/config/config.json"),
            ).exists(),
        ).toBeTrue();
    } finally {
        await rm(testRoot, { recursive: true, force: true });
        await rm(fixture.root, { recursive: true, force: true });
    }
});
