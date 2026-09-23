import { lstat, mkdir, readdir, rm, writeFile } from "node:fs/promises";
import { join, relative } from "node:path";

import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import { resolveSelection } from "./resolver.ts";
import { selectedServerSyntheticExportPlan } from "./selected-server-synthetic-export-plan.ts";
import { selectedServerInventory } from "./selected-server-inventory.ts";
import { compareContainerExportPath } from "./container-export-path.ts";

type FileEntry = {
    path: string;
    mode: "0644" | "0755";
    size: number;
    sha256: string;
};
type BuildCommand = string[];

export type ContainerExportInput = {
    selection: unknown;
    outputRoot: string;
    targetTriple: string;
    sourceIdentity: string;
    buildCommands: BuildCommand[];
    rustcVv: string;
    releaseVersion: string;
    runtime?: { platform: string; arch: string };
    evidence?: "linux-amd64-buildkit";
};

/** Records selected-server bytes; only Analytics uses the host-synthetic identity. */
export async function produceContainerExport(input: ContainerExportInput) {
    const plan = resolveSelection(input.selection);
    selectedServerSyntheticExportPlan(plan);
    if (input.targetTriple !== plan.target)
        throw new Error(
            "container export target differs from reviewed server selection",
        );
    const runtime = input.runtime ?? {
        platform: process.platform,
        arch: process.arch,
    };
    if (plan.preset === "analytics" && input.evidence !== undefined && input.evidence !== "linux-amd64-buildkit")
        throw new Error("Analytics container evidence is invalid");
    if (plan.preset === "analytics" && input.evidence !== "linux-amd64-buildkit") {
        if (
            !validRuntimeSegment(runtime.platform) ||
            !validRuntimeSegment(runtime.arch)
        )
            throw new Error(
                "Analytics synthetic export runtime identity is invalid",
            );
    } else if (runtime.platform !== "linux" || runtime.arch !== "x64") {
        throw new Error(
            "Monitor container export must run in a linux/amd64 build stage",
        );
    }
    if (!validText(input.sourceIdentity))
        throw new Error(
            "container source identity input must be nonempty single-line text",
        );
    if (!input.rustcVv.includes("rustc "))
        throw new Error("container provenance requires rustc -Vv output");
    if (!validText(input.releaseVersion) || input.releaseVersion.length > 64)
        throw new Error("container provenance releaseVersion is invalid");
    if (
        !Array.isArray(input.buildCommands) ||
        input.buildCommands.length === 0 ||
        input.buildCommands.some(
            (command) =>
                !Array.isArray(command) ||
                command.length === 0 ||
                command.some((part) => !validText(part)),
        )
    )
        throw new Error(
            "container provenance requires exact nonempty build commands",
        );

    const releaseRoot = join(input.outputRoot, "release");
    const witnessRoot = join(input.outputRoot, "witness");
    const payload = await listFiles(
        input.outputRoot,
        new Set([
            "release/container-provenance.json",
            "release/output-manifest.json",
        ]),
    );
    const selected = selectedServerInventory(plan);
    const server = selected.binaries.map((path) => `release/server/${path}`);
    const required = [
        ...server,
        ...(selected.hasAgentWitness ? ["witness/bin/rz-monitor-agent"] : []),
        "release/web/inventory.json",
        "release/web/binding.json",
        "release/web/api.ts",
        "release/contracts/api/api.json",
        "release/contracts/config/config.json",
        "release/contracts/schema/schema.json",
        "release/contracts/native/native-layout.json",
        "release/contracts/protocol/protocol.json",
    ];
    assertExactPaths(payload, required, server, selected.hasAgentWitness);
    if (!payload.some((entry) => entry.path.startsWith("release/web/dist/")))
        throw new Error("container export is missing selected Web dist files");

    const analytics = plan.preset === "analytics";
    const analyticsContainer = analytics && input.evidence === "linux-amd64-buildkit";
    const manifest = {
        schemaVersion: 1 as const,
        kind: analyticsContainer
            ? ("analytics-container-output" as const)
            : analytics
            ? ("selected-server-synthetic-output" as const)
            : ("monitor-container-output" as const),
        preset: plan.preset,
        artifactClass: plan.artifactClass,
        compositionId: plan.compositionId,
        target: plan.target,
        files: payload,
    };
    const provenance = {
        schemaVersion: 1 as const,
        kind: analyticsContainer
            ? ("analytics-container-provenance" as const)
            : analytics
            ? ("selected-server-synthetic-provenance" as const)
            : ("monitor-container-provenance" as const),
        buildPlatform: analyticsContainer
            ? "linux/amd64"
            : analytics
            ? `host/${runtime.platform}/${runtime.arch}`
            : "linux/amd64",
        targetTriple: input.targetTriple,
        releaseVersion: input.releaseVersion,
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
    await writeFile(
        join(releaseRoot, "output-manifest.json"),
        canonicalJson(manifest),
        { mode: 0o644 },
    );
    await writeFile(
        join(releaseRoot, "container-provenance.json"),
        canonicalJson(provenance),
        { mode: 0o644 },
    );
    return { manifest, provenance, releaseRoot, witnessRoot };
}

function validText(value: unknown): value is string {
    return (
        typeof value === "string" &&
        value.length > 0 &&
        value.length <= 4096 &&
        !/[\r\n]/.test(value)
    );
}

function validRuntimeSegment(value: unknown): value is string {
    return typeof value === "string" && /^[A-Za-z0-9._-]+$/.test(value);
}

function assertExactPaths(
    entries: FileEntry[],
    required: string[],
    expectedServer: string[],
    hasWitness: boolean,
) {
    const actual = new Set(entries.map((entry) => entry.path));
    for (const path of required)
        if (!actual.has(path))
            throw new Error(`container export is missing ${path}`);
    for (const entry of entries) {
        if (
            !entry.path.startsWith("release/server/bin/") &&
            !entry.path.startsWith("witness/bin/") &&
            !entry.path.startsWith("release/web/") &&
            !entry.path.startsWith("release/contracts/")
        )
            throw new Error(
                `container export has unexpected payload path: ${entry.path}`,
            );
    }
    const server = entries
        .filter((entry) => entry.path.startsWith("release/server/bin/"))
        .map((entry) => entry.path)
        .sort();
    const witness = entries
        .filter((entry) => entry.path.startsWith("witness/bin/"))
        .map((entry) => entry.path)
        .sort();
    const contracts = entries
        .filter((entry) => entry.path.startsWith("release/contracts/"))
        .map((entry) => entry.path)
        .sort();
    const expectedContracts = required
        .filter((path) => path.startsWith("release/contracts/"))
        .sort();
    if (canonicalJson(server) !== canonicalJson(expectedServer))
        throw new Error(
            "container export server binary inventory is not exact",
        );
    if (
        canonicalJson(witness) !==
        canonicalJson(hasWitness ? ["witness/bin/rz-monitor-agent"] : [])
    )
        throw new Error(
            "container export witness binary inventory is not exact",
        );
    if (canonicalJson(contracts) !== canonicalJson(expectedContracts))
        throw new Error("container export contract inventory is not exact");
    for (const entry of entries) {
        const executable =
            server.includes(entry.path) || witness.includes(entry.path);
        if (entry.mode !== (executable ? "0755" : "0644"))
            throw new Error(
                `container export has invalid payload mode: ${entry.path}`,
            );
    }
}

async function listFiles(
    root: string,
    ignored: Set<string>,
    directory = root,
): Promise<FileEntry[]> {
    const entries = await readdir(directory, { withFileTypes: true });
    const nested = await Promise.all(
        entries.map(async (entry) => {
            const path = join(directory, entry.name);
            if (entry.isSymbolicLink())
                throw new Error(`container export contains symlink: ${path}`);
            if (entry.isDirectory()) return listFiles(root, ignored, path);
            if (!entry.isFile())
                throw new Error(
                    `container export contains unsupported path: ${path}`,
                );
            const relativePath = relative(root, path).replaceAll("\\", "/");
            if (ignored.has(relativePath)) return [];
            const state = await lstat(path);
            if (state.isSymbolicLink())
                throw new Error(`container export contains symlink: ${path}`);
            const bytes = await Bun.file(path).bytes();
            const mode: FileEntry["mode"] =
                (state.mode & 0o777) === 0o755 ? "0755" : "0644";
            if (!([0o644, 0o755] as number[]).includes(state.mode & 0o777))
                throw new Error(
                    `container export has unsupported mode: ${relativePath}`,
                );
            return [
                {
                    path: relativePath,
                    mode,
                    size: bytes.byteLength,
                    sha256: sha256(bytes),
                },
            ];
        }),
    );
    return nested
        .flat()
        .sort((left, right) =>
            compareContainerExportPath(left.path, right.path),
        );
}
