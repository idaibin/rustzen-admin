import { lstatSync, mkdirSync } from "node:fs";
import { lstat, readdir, rm } from "node:fs/promises";
import { join } from "node:path";
import { expect, test } from "bun:test";
import { setAtomicRenameBeforeCallHookForTest } from "./atomic-rename.ts";
import { produceNativeStaging } from "./native-staging.ts";
import { nativeStagingRoots } from "./native-staging-test-fixture.ts";

test("native staging preserves a final created at the atomic rename boundary", async () => {
    const { fixture, binaryRoot, selection } = await nativeStagingRoots("agent");
    const input = {
        selection,
        outputParent: join(fixture.root, "atomic-out"),
        trustedRoot: fixture.root,
        releaseVersion: "1.0.2",
        sourceIdentity: "test-source",
        toolchain: "test-toolchain",
        selectedRoutes: [],
        binaryRoot,
        configRoot: fixture.configRoot,
        nativeRoot: fixture.nativeRoot,
        protocolRoot: fixture.protocolRoot,
    };
    const seed = await produceNativeStaging({
        ...input,
        outputParent: join(fixture.root, "seed"),
    });
    const final = join(
        input.outputParent,
        ".native-staging",
        seed.buildId,
        selection.target,
        "node-agent",
        "payload",
    );
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
