import { expect, test } from "bun:test";
import { chmod, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

import { produceContainerExport } from "./container-export.ts";

const selection = { schemaVersion: 1, preset: "monitor", target: "x86_64-unknown-linux-musl" };

test("container export records one exact Monitor payload and provenance", async () => {
    const root = await populate();
    try {
        const result = await produceContainerExport({
            selection,
            outputRoot: root,
            targetTriple: "x86_64-unknown-linux-musl",
            sourceIdentity: "git:abc123 tree:def456",
            buildCommands: [["cargo", "build", "--release"], ["cargo", "build", "--bin", "rz-monitor-agent"]],
            rustcVv: "rustc 1.95.0\nhost: x86_64-unknown-linux-gnu\n",
            releaseVersion: "0.5.0",
            runtime: { platform: "linux", arch: "x64" },
        });
        expect(result.manifest.files.map((file) => file.path)).toContain("release/web/dist/index.html");
        expect(result.manifest.files.map((file) => file.path)).toContain("witness/bin/rz-monitor-agent");
        expect(result.manifest.kind).toBe("monitor-container-output");
        expect(result.provenance).toMatchObject({ kind: "monitor-container-provenance", buildPlatform: "linux/amd64" });
        expect(result.provenance.outputManifestSha256).toMatch(/^[a-f0-9]{64}$/);
        expect(await Bun.file(join(root, "release/output-manifest.json")).json()).toEqual(result.manifest);
        expect(await Bun.file(join(root, "release/container-provenance.json")).json()).toEqual(result.provenance);
        expect(result.manifest.files.map((file) => file.path)).toEqual(
            result.manifest.files.map((file) => file.path).sort((left, right) =>
                left < right ? -1 : left > right ? 1 : 0,
            ),
        );
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("container export rejects omitted payload members and wrong build environment", async () => {
    const root = await populate();
    try {
        await rm(join(root, "release/contracts/protocol/protocol.json"));
        await expect(produceContainerExport({
            selection,
            outputRoot: root,
            targetTriple: "x86_64-unknown-linux-musl",
            sourceIdentity: "source",
            buildCommands: [["cargo", "build"]],
            rustcVv: "rustc 1.95.0\n", releaseVersion: "0.5.0",
            runtime: { platform: "linux", arch: "x64" },
        })).rejects.toThrow("protocol");
        const valid = await populate();
        await expect(produceContainerExport({
            selection, outputRoot: valid, targetTriple: "x86_64-unknown-linux-musl", sourceIdentity: "source",
            buildCommands: [["cargo", "build"]], rustcVv: "rustc 1.95.0\n", releaseVersion: "0.5.0",
            runtime: { platform: "darwin", arch: "arm64" },
        })).rejects.toThrow("linux/amd64");
        await rm(valid, { recursive: true, force: true });
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("container export rejects extra contracts and non-executable binaries", async () => {
    const extraRoot = await populate();
    const modeRoot = await populate();
    try {
        await writeFile(join(extraRoot, "release/contracts/api/extra.json"), "{}");
        await expect(produce(extraRoot)).rejects.toThrow("contract inventory is not exact");
        await chmod(join(modeRoot, "release/server/bin/rz-admin"), 0o644);
        await expect(produce(modeRoot)).rejects.toThrow("invalid payload mode");
    } finally {
        await rm(extraRoot, { recursive: true, force: true });
        await rm(modeRoot, { recursive: true, force: true });
    }
});

function produce(outputRoot: string) {
    return produceContainerExport({
        selection,
        outputRoot,
        targetTriple: "x86_64-unknown-linux-musl",
        sourceIdentity: "source",
        buildCommands: [["cargo", "build"]],
        rustcVv: "rustc 1.95.0\n", releaseVersion: "0.5.0",
        runtime: { platform: "linux", arch: "x64" },
    });
}

async function populate() {
    const root = await mkdtemp(join(tmpdir(), "rz-container-export-"));
    const executable = ["release/server/bin/rz-admin", "release/server/bin/rz-monitor", "witness/bin/rz-monitor-agent"];
    const files = [
        ...executable,
        "release/web/inventory.json",
        "release/web/binding.json",
        "release/web/api.ts",
        "release/web/dist/index.html",
        "release/web/dist/assets/Z.js",
        "release/web/dist/assets/a.js",
        "release/contracts/api/api.json",
        "release/contracts/config/config.json",
        "release/contracts/schema/schema.json",
        "release/contracts/native/native-layout.json",
        "release/contracts/protocol/protocol.json",
    ];
    for (const path of files) {
        const target = join(root, path);
        await mkdir(join(target, ".."), { recursive: true });
        await writeFile(target, path);
        if (executable.includes(path)) await chmod(target, 0o755);
    }
    return root;
}
