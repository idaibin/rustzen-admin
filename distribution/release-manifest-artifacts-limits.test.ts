import { expect, test } from "bun:test";
import { mkdir, mkdtemp, rm, truncate, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

import { readArtifactFileTree } from "./release-manifest-artifacts.ts";

async function withTree(
    build: (root: string) => Promise<void>,
    verify: (root: string) => Promise<void>,
) {
    const root = await mkdtemp(join(tmpdir(), "rz-artifact-limits-"));
    try {
        await build(root);
        await verify(root);
    } finally {
        await rm(root, { recursive: true, force: true });
    }
}

test("artifact reader rejects an oversized sparse file before accepting its payload", () =>
    withTree(
        async (root) => {
            const file = join(root, "sparse.bin");
            await writeFile(file, "x");
            await truncate(file, 1024 * 1024);
        },
        async (root) => {
            await expect(readArtifactFileTree(root, { maxFileBytes: 1024 }))
                .rejects.toThrow("artifact file size limit exceeded");
        },
    ));

test("artifact reader enforces cumulative payload bytes", () =>
    withTree(
        async (root) => {
            await writeFile(join(root, "first"), "1234567890");
            await writeFile(join(root, "second"), "abcdefghij");
        },
        async (root) => {
            await expect(readArtifactFileTree(root, { maxTotalBytes: 15 }))
                .rejects.toThrow("artifact total size limit exceeded");
        },
    ));

test("artifact reader streams directory entries through per-directory and total budgets", async () => {
    await withTree(
        async (root) => {
            await writeFile(join(root, "first"), "1");
            await writeFile(join(root, "second"), "2");
        },
        async (root) => {
            await expect(readArtifactFileTree(root, { maxDirectoryEntries: 1 }))
                .rejects.toThrow("artifact directory entry limit exceeded");
        },
    );
    await withTree(
        async (root) => {
            await mkdir(join(root, "one"));
            await mkdir(join(root, "two"));
            await writeFile(join(root, "one", "file"), "1");
            await writeFile(join(root, "two", "file"), "2");
        },
        async (root) => {
            await expect(readArtifactFileTree(root, { maxTotalEntries: 3 }))
                .rejects.toThrow("artifact total entry limit exceeded");
        },
    );
});

test("artifact reader rejects a tree deeper than its traversal budget", () =>
    withTree(
        async (root) => {
            await mkdir(join(root, "one", "two"), { recursive: true });
            await writeFile(join(root, "one", "two", "file"), "1");
        },
        async (root) => {
            await expect(readArtifactFileTree(root, { maxDepth: 1 }))
                .rejects.toThrow("artifact depth limit exceeded");
        },
    ));
