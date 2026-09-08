import { lstat, mkdir, readdir, rm, writeFile } from "node:fs/promises";
import { join, relative } from "node:path";

import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import { resolveSelection } from "./resolver.ts";
import { compareContainerExportPath } from "./container-export-path.ts";

type FileEntry = { path: string; mode: "0644" | "0755"; size: number; sha256: string };
type BuildCommand = string[];

export type ContainerExportInput = {
    selection: unknown;
    outputRoot: string;
    targetTriple: string;
    sourceIdentity: string;
    buildCommands: BuildCommand[];
    rustcVv: string;
    runtime?: { platform: string; arch: string };
};

/**
 * Records the exact Monitor payload emitted by one Linux/amd64 build stage.
 * It intentionally creates evidence only; certificate issuing remains external.
 */
export async function produceContainerExport(input: ContainerExportInput) {
    const plan = resolveSelection(input.selection);
    if (
        plan.preset !== "monitor" ||
        plan.artifactClass !== "server" ||
        plan.target !== "x86_64-unknown-linux-musl" ||
        input.targetTriple !== plan.target
    )
        throw new Error("container export supports only the reviewed Monitor server target");
    const runtime = input.runtime ?? { platform: process.platform, arch: process.arch };
    if (runtime.platform !== "linux" || runtime.arch !== "x64")
        throw new Error("container export must run in a linux/amd64 build stage");
    if (!validText(input.sourceIdentity))
        throw new Error("container source identity input must be nonempty single-line text");
    if (!input.rustcVv.includes("rustc "))
        throw new Error("container provenance requires rustc -Vv output");
    if (
        !Array.isArray(input.buildCommands) ||
        input.buildCommands.length === 0 ||
        input.buildCommands.some(
            (command) => !Array.isArray(command) || command.length === 0 || command.some((part) => !validText(part)),
        )
    )
        throw new Error("container provenance requires exact nonempty build commands");

    const releaseRoot = join(input.outputRoot, "release");
    const witnessRoot = join(input.outputRoot, "witness");
    const payload = await listFiles(input.outputRoot, new Set([
        "release/container-provenance.json",
        "release/output-manifest.json",
    ]));
    const required = [
        "release/server/bin/rz-admin",
        "release/server/bin/rz-monitor",
        "witness/bin/rz-monitor-agent",
        "release/web/inventory.json",
        "release/contracts/api/api.json",
        "release/contracts/config/config.json",
        "release/contracts/schema/schema.json",
        "release/contracts/native/native-layout.json",
        "release/contracts/protocol/protocol.json",
    ];
    assertExactPaths(payload, required);
    if (!payload.some((entry) => entry.path.startsWith("release/web/dist/")))
        throw new Error("container export is missing selected Web dist files");

    const manifest = {
        schemaVersion: 1 as const,
        kind: "monitor-container-output" as const,
        preset: plan.preset,
        artifactClass: plan.artifactClass,
        compositionId: plan.compositionId,
        target: plan.target,
        files: payload,
    };
    const provenance = {
        schemaVersion: 1 as const,
        kind: "monitor-container-provenance" as const,
        buildPlatform: "linux/amd64",
        targetTriple: input.targetTriple,
        selection: input.selection,
        selectionSha256: sha256(canonicalJson(input.selection)),
        sourceIdentityInput: input.sourceIdentity,
        rustcVv: input.rustcVv,
        buildCommands: input.buildCommands,
        outputManifestSha256: sha256(canonicalJson(manifest)),
    };
    await mkdir(releaseRoot, { recursive: true });
    await rm(join(releaseRoot, "output-manifest.json"), { force: true });
    await rm(join(releaseRoot, "container-provenance.json"), { force: true });
    await writeFile(join(releaseRoot, "output-manifest.json"), canonicalJson(manifest), { mode: 0o644 });
    await writeFile(join(releaseRoot, "container-provenance.json"), canonicalJson(provenance), { mode: 0o644 });
    return { manifest, provenance, releaseRoot, witnessRoot };
}

function validText(value: unknown): value is string {
    return typeof value === "string" && value.length > 0 && value.length <= 4096 && !/[\r\n]/.test(value);
}

function assertExactPaths(entries: FileEntry[], required: string[]) {
    const actual = new Set(entries.map((entry) => entry.path));
    for (const path of required) if (!actual.has(path)) throw new Error(`container export is missing ${path}`);
    for (const entry of entries) {
        if (
            !entry.path.startsWith("release/server/bin/") &&
            !entry.path.startsWith("witness/bin/") &&
            !entry.path.startsWith("release/web/") &&
            !entry.path.startsWith("release/contracts/")
        )
            throw new Error(`container export has unexpected payload path: ${entry.path}`);
    }
    const server = entries.filter((entry) => entry.path.startsWith("release/server/bin/")).map((entry) => entry.path).sort();
    const witness = entries.filter((entry) => entry.path.startsWith("witness/bin/")).map((entry) => entry.path).sort();
    const contracts = entries.filter((entry) => entry.path.startsWith("release/contracts/")).map((entry) => entry.path).sort();
    const expectedContracts = required.filter((path) => path.startsWith("release/contracts/")).sort();
    if (canonicalJson(server) !== canonicalJson(["release/server/bin/rz-admin", "release/server/bin/rz-monitor"]))
        throw new Error("container export server binary inventory is not exact");
    if (canonicalJson(witness) !== canonicalJson(["witness/bin/rz-monitor-agent"]))
        throw new Error("container export witness binary inventory is not exact");
    if (canonicalJson(contracts) !== canonicalJson(expectedContracts))
        throw new Error("container export contract inventory is not exact");
    for (const entry of entries) {
        const executable = server.includes(entry.path) || witness.includes(entry.path);
        if (entry.mode !== (executable ? "0755" : "0644"))
            throw new Error(`container export has invalid payload mode: ${entry.path}`);
    }
}

async function listFiles(root: string, ignored: Set<string>, directory = root): Promise<FileEntry[]> {
    const entries = await readdir(directory, { withFileTypes: true });
    const nested = await Promise.all(entries.map(async (entry) => {
        const path = join(directory, entry.name);
        if (entry.isSymbolicLink()) throw new Error(`container export contains symlink: ${path}`);
        if (entry.isDirectory()) return listFiles(root, ignored, path);
        if (!entry.isFile()) throw new Error(`container export contains unsupported path: ${path}`);
        const relativePath = relative(root, path).replaceAll("\\", "/");
        if (ignored.has(relativePath)) return [];
        const state = await lstat(path);
        if (state.isSymbolicLink()) throw new Error(`container export contains symlink: ${path}`);
        const bytes = await Bun.file(path).bytes();
        const mode: FileEntry["mode"] = (state.mode & 0o777) === 0o755 ? "0755" : "0644";
        if (!([0o644, 0o755] as number[]).includes(state.mode & 0o777))
            throw new Error(`container export has unsupported mode: ${relativePath}`);
        return [{ path: relativePath, mode, size: bytes.byteLength, sha256: sha256(bytes) }];
    }));
    return nested.flat().sort((left, right) => compareContainerExportPath(left.path, right.path));
}
