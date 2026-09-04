import { chmod, lstat, mkdir, rm, symlink, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { expect, test } from "bun:test";
import { produceNativeStaging } from "./native-staging.ts";
import { setNativeStagingBeforePublishHookForTest } from "./native-staging.ts";
import {
    monitorSelection,
    releaseFixture,
} from "./release-manifest-fixtures.ts";
import { setArtifactAfterOpenHookForTest } from "./release-manifest-artifacts.ts";
import { readArtifactFileTree } from "./release-manifest-artifacts.ts";

async function assertNoFailedOutput(root: string) {
    let files: Awaited<ReturnType<typeof readArtifactFileTree>> = [];
    try {
        files = await readArtifactFileTree(root);
    } catch (error: unknown) {
        if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
    expect(
        files.some((file) =>
            new TextDecoder().decode(file.bytes).includes("SENTINEL"),
        ),
    ).toBeFalse();
    expect(
        files.some(
            (file) =>
                file.entry.path.includes(".tmp") ||
                file.entry.path.includes(".lock") ||
                file.entry.path.includes("payload"),
        ),
    ).toBeFalse();
}

async function input() {
    const fixture = await releaseFixture();
    const binaryRoot = join(fixture.root, "binary");
    await mkdir(join(binaryRoot, "bin"), { recursive: true });
    for (const name of ["rz-admin", "rz-monitor"]) {
        await writeFile(join(binaryRoot, "bin", name), name);
        await chmod(join(binaryRoot, "bin", name), 0o755);
    }
    return {
        fixture,
        binaryRoot,
        selection: monitorSelection,
        outputParent: join(fixture.root, "out"),
        trustedRoot: fixture.root,
        releaseVersion: "1.0.0",
        sourceIdentity: "source",
        toolchain: "tool",
        selectedRoutes: [],
        webRoot: fixture.webRoot,
        apiRoot: fixture.apiRoot,
        schemaRoot: fixture.schemaRoot,
        configRoot: fixture.configRoot,
        nativeRoot: fixture.nativeRoot,
        protocolRoot: fixture.protocolRoot,
    };
}

test("staging rejects linked selected source roots without publishing", async () => {
    const base = await input();
    const external = join(base.fixture.root, "external");
    await mkdir(external);
    await writeFile(join(external, "sentinel"), "SENTINEL");
    try {
        for (const key of [
            "binaryRoot",
            "webRoot",
            "apiRoot",
            "schemaRoot",
            "configRoot",
            "nativeRoot",
            "protocolRoot",
        ] as const) {
            const original = base[key];
            const held = `${original}.held`;
            await Bun.$`mv ${original} ${held}`;
            await symlink(external, original);
            await expect(produceNativeStaging(base)).rejects.toThrow();
            await assertNoFailedOutput(base.outputParent);
            await rm(original);
            await Bun.$`mv ${held} ${original}`;
        }
    } finally {
        await rm(base.fixture.root, { recursive: true, force: true });
    }
});

test("staging rejects after-open replacement for tree and single-file inputs", async () => {
    for (const shape of ["binary", "web", "contract"] as const) {
        const base = await input();
        const target =
            shape === "binary"
                ? join(base.binaryRoot, "bin/rz-admin")
                : shape === "web"
                  ? join(base.webRoot, "index.html")
                  : join(base.configRoot, "config.json");
        try {
            setArtifactAfterOpenHookForTest(async (path) => {
                if (path === target) await writeFile(path, "SENTINEL");
            });
            await expect(produceNativeStaging(base)).rejects.toThrow("changed");
            await assertNoFailedOutput(base.outputParent);
        } finally {
            setArtifactAfterOpenHookForTest();
            await rm(base.fixture.root, { recursive: true, force: true });
        }
    }
});

test("staging preserves existing final object kinds", async () => {
    const base = await input();
    try {
        const seed = await produceNativeStaging({
            ...base,
            outputParent: join(base.fixture.root, "seed"),
        });
        for (const kind of ["file", "directory", "symlink"] as const) {
            const parent = join(base.fixture.root, `out-${kind}`);
            const final = join(
                parent,
                ".native-staging",
                seed.buildId,
                monitorSelection.target,
                "server",
                "payload",
            );
            await mkdir(join(final, ".."), { recursive: true });
            if (kind === "file") await writeFile(final, "keep");
            if (kind === "directory") {
                await mkdir(final);
                await writeFile(join(final, "keep"), "keep");
            }
            if (kind === "symlink") {
                const target = join(base.fixture.root, `target-${kind}`);
                await mkdir(target);
                await writeFile(join(target, "keep"), "keep");
                await symlink(target, final);
            }
            await expect(
                produceNativeStaging({ ...base, outputParent: parent }),
            ).rejects.toThrow("output");
            expect(
                (await lstat(final)).isSymbolicLink() || kind !== "symlink",
            ).toBeTrue();
        }
    } finally {
        await rm(base.fixture.root, { recursive: true, force: true });
    }
});

test("staging rejects unsafe output parents and ancestors", async () => {
    const base = await input();
    try {
        const link = join(base.fixture.root, "out-link");
        await symlink(base.fixture.root, link);
        await expect(
            produceNativeStaging({ ...base, outputParent: link }),
        ).rejects.toThrow("output base");
        const publicParent = join(base.fixture.root, "out-public");
        await mkdir(publicParent);
        await chmod(publicParent, 0o777);
        await expect(
            produceNativeStaging({ ...base, outputParent: publicParent }),
        ).rejects.toThrow("output base");
        const seed = await produceNativeStaging({
            ...base,
            outputParent: join(base.fixture.root, "seed"),
        });
        const parent = join(base.fixture.root, "out-ancestor");
        const build = join(parent, ".native-staging", seed.buildId);
        await mkdir(parent);
        await mkdir(join(parent, ".native-staging"), { mode: 0o700 });
        await symlink(base.fixture.root, build);
        await expect(
            produceNativeStaging({ ...base, outputParent: parent }),
        ).rejects.toThrow("private");
    } finally {
        await rm(base.fixture.root, { recursive: true, force: true });
    }
});

test("staging rechecks an output-base ancestor before publish", async () => {
    const base = await input();
    const old = `${base.outputParent}.old`;
    try {
        setNativeStagingBeforePublishHookForTest(async () => {
            await Bun.$`mv ${base.outputParent} ${old}`;
            await mkdir(base.outputParent, { mode: 0o700 });
        });
        await expect(produceNativeStaging(base)).rejects.toThrow(
            "changed before publish",
        );
        await assertNoFailedOutput(base.outputParent);
    } finally {
        setNativeStagingBeforePublishHookForTest();
        await rm(base.fixture.root, { recursive: true, force: true });
    }
});

test("staging rechecks the class ancestor before publish", async () => {
    const base = await input();
    try {
        const seed = await produceNativeStaging({
            ...base,
            outputParent: join(base.fixture.root, "seed"),
        });
        const ancestor = join(
            base.outputParent,
            ".native-staging",
            seed.buildId,
            base.selection.target,
            "server",
        );
        setNativeStagingBeforePublishHookForTest(async () => {
            await Bun.$`mv ${ancestor} ${ancestor}.old`;
            await mkdir(ancestor, { mode: 0o700 });
        });
        await expect(produceNativeStaging(base)).rejects.toThrow(
            "changed before publish",
        );
        await assertNoFailedOutput(base.outputParent);
    } finally {
        setNativeStagingBeforePublishHookForTest();
        await rm(base.fixture.root, { recursive: true, force: true });
    }
});
