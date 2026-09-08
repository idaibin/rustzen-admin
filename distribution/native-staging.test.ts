import { chmod, lstat, mkdir, readdir, rm, writeFile } from "node:fs/promises";
import { lstatSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "bun:test";
import { nativeUnitBytes } from "./native-layout.ts";
import {
    produceNativeStaging,
    publishNativeStagingBytes,
    setNativeStagingBeforePublishHookForTest,
} from "./native-staging.ts";
import {
    releaseFixture,
    monitorSelection,
} from "./release-manifest-fixtures.ts";
import { sha256 } from "./release-manifest-core.ts";
import {
    fromPathStagingInput,
    VerifiedNativeSource,
} from "./native-staging-source.ts";
import { setAtomicRenameBeforeCallHookForTest } from "./atomic-rename.ts";

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

test("verified-byte publisher rejects path escapes before output side effects", async () => {
    const { fixture } = await roots("server");
    const outputParent = join(fixture.root, "output");
    const planted = join(fixture.root, "review-planted");
    try {
        await expect(publishNativeStagingBytes({
            outputParent,
            trustedRoot: fixture.root,
            source: { files: [{ entry: { path: "../../../review-planted", type: "file", mode: "0644", size: 1, sha256: "0".repeat(64) }, bytes: new Uint8Array([1]) }], digests: { configDigest: "0".repeat(64), nativeLayoutDigest: "0".repeat(64), protocolArtifactDigest: "0".repeat(64) } } as any,
        })).rejects.toThrow("capability");
        expect(await Bun.file(planted).exists()).toBeFalse();
        expect(await Bun.file(join(outputParent, ".native-staging")).exists()).toBeFalse();
    } finally { await rm(fixture.root, { recursive: true, force: true }); }
});

test("verified-byte publisher rejects forged binary capability before output side effects", async () => {
    const { fixture } = await roots("server");
    const outputParent = join(fixture.root, "forged-output");
    const bytes = new TextEncoder().encode("arbitrary");
    const forged = {
        files: () => [{
            entry: {
                path: "bin/rz-admin",
                type: "file",
                mode: "0755",
                size: bytes.length,
                sha256: sha256(bytes),
            },
            bytes,
        }],
        digests: () => ({}),
    };
    try {
        expect(() => new (VerifiedNativeSource as any)(Symbol("forged"), forged))
            .toThrow("construction is internal");
        await expect(publishNativeStagingBytes({
            outputParent,
            trustedRoot: fixture.root,
            source: forged as any,
        })).rejects.toThrow("capability is invalid");
        expect(await Bun.file(join(outputParent, ".native-staging")).exists()).toBeFalse();
    } finally {
        await rm(fixture.root, { recursive: true, force: true });
    }
});

test("verified-byte publisher rejects method shadowing on a real capability", async () => {
    const { fixture, binaryRoot, selection } = await roots("server");
    const outputParent = join(fixture.root, "shadow-output");
    const input = {
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
    };
    try {
        const source = await fromPathStagingInput(input, true);
        expect(() => Object.defineProperty(source, "files", {
            value: () => [{ bytes: new TextEncoder().encode("arbitrary") }],
        })).toThrow();
        expect(await Bun.file(join(outputParent, ".native-staging")).exists()).toBeFalse();
    } finally {
        await rm(fixture.root, { recursive: true, force: true });
    }
});

test("verified-byte publisher rejects metadata substitutions before output side effects", async () => {
    const { fixture, binaryRoot, selection } = await roots("server");
    const outputParent = join(fixture.root, "metadata-output");
    const input = {
        selection, outputParent, trustedRoot: fixture.root,
        releaseVersion: "1.0.0", sourceIdentity: "test-source",
        toolchain: "test-toolchain", selectedRoutes: [], binaryRoot,
        webRoot: fixture.webRoot, apiRoot: fixture.apiRoot,
        schemaRoot: fixture.schemaRoot, configRoot: fixture.configRoot,
        nativeRoot: fixture.nativeRoot, protocolRoot: fixture.protocolRoot,
    };
    try {
        const source = await fromPathStagingInput(input, true);
        for (const [field, value] of [
            ["selection", { ...selection, extra: true }],
            ["buildInputs", { releaseVersion: "9.9.9" }],
            ["releaseVersion", "9.9.9"],
            ["sourceIdentity", "other-source"],
            ["toolchain", "other-toolchain"],
            ["selectedRoutes", ["other.tsx"]],
        ] as const) {
            await expect(publishNativeStagingBytes({
                outputParent, trustedRoot: fixture.root, source, [field]: value,
            } as any)).rejects.toThrow("inputs are invalid");
        }
        expect(await Bun.file(join(outputParent, ".native-staging")).exists()).toBeFalse();
    } finally { await rm(fixture.root, { recursive: true, force: true }); }
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

test("native staging preserves a final created at the atomic rename boundary", async () => {
    const { fixture, binaryRoot, selection } = await roots("agent");
    const input = {
        selection, outputParent: join(fixture.root, "atomic-out"), trustedRoot: fixture.root,
        releaseVersion: "1.0.2", sourceIdentity: "test-source",
        toolchain: "test-toolchain", selectedRoutes: [], binaryRoot,
        configRoot: fixture.configRoot, nativeRoot: fixture.nativeRoot,
        protocolRoot: fixture.protocolRoot,
    };
    const seed = await produceNativeStaging({ ...input, outputParent: join(fixture.root, "seed") });
    const final = join(input.outputParent, ".native-staging", seed.buildId, selection.target, "node-agent", "payload");
    let plantedInode: number | undefined;
    try {
        setAtomicRenameBeforeCallHookForTest(() => {
            mkdirSync(final);
            plantedInode = lstatSync(final).ino;
        });
        await expect(produceNativeStaging(input)).rejects.toThrow("atomic");
        const before = await lstat(final);
        expect(before.isDirectory()).toBeTrue();
        expect(before.ino).toBe(plantedInode);
        expect(await readdir(final)).toEqual([]);
    } finally {
        setAtomicRenameBeforeCallHookForTest();
        await rm(fixture.root, { recursive: true, force: true });
    }
});
