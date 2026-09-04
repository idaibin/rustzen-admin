import { chmod, lstat, mkdir, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { expect, test } from "bun:test";
import { nativeUnitBytes } from "./native-layout.ts";
import {
    produceNativeStaging,
    setNativeStagingBeforePublishHookForTest,
} from "./native-staging.ts";
import {
    releaseFixture,
    monitorSelection,
} from "./release-manifest-fixtures.ts";

async function roots(kind: "server" | "agent") {
    const fixture = await releaseFixture(kind);
    const binaryRoot = join(fixture.root, "binary");
    await mkdir(join(binaryRoot, "bin"), { recursive: true });
    const names =
        kind === "server" ? ["rz-admin", "rz-monitor"] : ["rz-monitor-agent"];
    for (const name of names) {
        await writeFile(join(binaryRoot, "bin", name), name);
        await chmod(join(binaryRoot, "bin", name), 0o755);
    }
    return {
        fixture,
        binaryRoot,
        selection:
            kind === "server"
                ? monitorSelection
                : { preset: "node-agent", target: monitorSelection.target },
    };
}

test("native staging writes exact selected server and Agent trees", async () => {
    for (const kind of ["server", "agent"] as const) {
        const { fixture, binaryRoot, selection } = await roots(kind);
        const outputParent = join(fixture.root, "staging");
        try {
            const result = await produceNativeStaging({
                selection,
                outputParent,
                trustedRoot: fixture.root,
                releaseVersion: "1.0.0",
                sourceIdentity: "test-source",
                toolchain: "test-toolchain",
                selectedRoutes: [],
                binaryRoot,
                webRoot: fixture.webRoot,
                apiRoot: fixture.apiRoot,
                schemaRoot: fixture.schemaRoot,
                configRoot: fixture.configRoot,
                nativeRoot: fixture.nativeRoot,
                protocolRoot: fixture.protocolRoot,
            });
            const expected =
                kind === "server"
                    ? [
                          "bin/rz-admin",
                          "bin/rz-monitor",
                          "contracts/api/api.json",
                          "contracts/config/config.json",
                          "contracts/native/native-layout.json",
                          "contracts/protocol/protocol.json",
                          "contracts/schema/schema.json",
                          ...Object.keys(nativeUnitBytes(selection)),
                          "web/assets/main.js",
                          "web/index.html",
                      ]
                    : [
                          "bin/rz-monitor-agent",
                          "contracts/config/config.json",
                          "contracts/native/native-layout.json",
                          "contracts/protocol/protocol.json",
                          ...Object.keys(nativeUnitBytes(selection)),
                      ];
            expect(result.files.map((file) => file.path)).toEqual(
                expected.sort(),
            );
        } finally {
            await rm(fixture.root, { recursive: true, force: true });
        }
    }
});

test("native staging rejects polluted and missing selected inputs", async () => {
    const { fixture, binaryRoot } = await roots("agent");
    try {
        await writeFile(join(binaryRoot, "bin", "rz"), "forbidden");
        await chmod(join(binaryRoot, "bin", "rz"), 0o755);
        await expect(
            produceNativeStaging({
                selection: {
                    preset: "node-agent",
                    target: monitorSelection.target,
                },
                outputParent: join(fixture.root, "out"),
                trustedRoot: fixture.root,
                releaseVersion: "1.0.0",
                sourceIdentity: "test-source",
                toolchain: "test-toolchain",
                selectedRoutes: [],
                binaryRoot,
                configRoot: fixture.configRoot,
                nativeRoot: fixture.nativeRoot,
                protocolRoot: fixture.protocolRoot,
            }),
        ).rejects.toThrow("inventory");
    } finally {
        await rm(fixture.root, { recursive: true, force: true });
    }
});

test("native staging never replaces an existing build-qualified payload", async () => {
    const { fixture, binaryRoot, selection } = await roots("agent");
    const input = {
        selection,
        outputParent: join(fixture.root, "out"),
        trustedRoot: fixture.root,
        releaseVersion: "1.0.0",
        sourceIdentity: "test-source",
        toolchain: "test-toolchain",
        selectedRoutes: [],
        binaryRoot,
        configRoot: fixture.configRoot,
        nativeRoot: fixture.nativeRoot,
        protocolRoot: fixture.protocolRoot,
    };
    try {
        const first = await produceNativeStaging(input);
        await expect(produceNativeStaging(input)).rejects.toThrow("output");
        expect(
            await Bun.file(join(first.root, "bin/rz-monitor-agent")).text(),
        ).toBe("rz-monitor-agent");
    } finally {
        await rm(fixture.root, { recursive: true, force: true });
    }
});

test("native staging uses one lock and removes only its failed publish state", async () => {
    const { fixture, binaryRoot, selection } = await roots("agent");
    const input = {
        selection,
        outputParent: join(fixture.root, "out"),
        trustedRoot: fixture.root,
        releaseVersion: "1.0.0",
        sourceIdentity: "test-source",
        toolchain: "test-toolchain",
        selectedRoutes: [],
        binaryRoot,
        configRoot: fixture.configRoot,
        nativeRoot: fixture.nativeRoot,
        protocolRoot: fixture.protocolRoot,
    };
    try {
        const both = await Promise.allSettled([
            produceNativeStaging(input),
            produceNativeStaging(input),
        ]);
        expect(
            both.filter((result) => result.status === "fulfilled"),
        ).toHaveLength(1);
        expect(
            both.filter((result) => result.status === "rejected"),
        ).toHaveLength(1);
        const sibling = join(input.outputParent, "keep");
        await mkdir(sibling, { recursive: true });
        await writeFile(join(sibling, "sentinel"), "keep");
        const changed = { ...input, releaseVersion: "1.0.1" };
        const raceSeed = await produceNativeStaging({
            ...changed,
            outputParent: join(fixture.root, "race-seed"),
        });
        const racedFinal = join(
            input.outputParent,
            ".native-staging",
            raceSeed.buildId,
            selection.target,
            "node-agent",
            "payload",
        );
        setNativeStagingBeforePublishHookForTest(async () => {
            await mkdir(racedFinal);
        });
        await expect(produceNativeStaging(changed)).rejects.toThrow("output");
        expect((await lstat(racedFinal)).isDirectory()).toBeTrue();
        expect(await Bun.file(join(sibling, "sentinel")).text()).toBe("keep");
        expect(
            (
                await Bun.$`find ${input.outputParent} -name '*.tmp' -o -name '*.lock'`.text()
            ).trim(),
        ).toBe("");
    } finally {
        setNativeStagingBeforePublishHookForTest();
        await rm(fixture.root, { recursive: true, force: true });
    }
});
