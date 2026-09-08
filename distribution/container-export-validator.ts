import { canonicalJson, sha256, validHash } from "./release-manifest-core.ts";
import { readArtifactFileTree, type ArtifactFile } from "./release-manifest-artifacts.ts";
import { resolveSelection } from "./resolver.ts";
import { verifyMonitorExportElf } from "./container-export-elf.ts";
import { parseSelectedApiBytes } from "./selected-contract-validator.ts";
import { parseSelectedConfigBytes } from "./selected-config.ts";
import { parseNativeLayoutBytes } from "./native-layout.ts";
import { parseSelectedProtocolBytes } from "./selected-protocol.ts";
import { parseSchemaArtifactBytes } from "./schema-contract.ts";
import { compareContainerExportPath } from "./container-export-path.ts";
import { assertSelectedWebSnapshot, parseInventory } from "../scripts/distribution-web-inventory-policy.ts";

type ExportFile = { path: string; mode: "0644" | "0755"; size: number; sha256: string };
type OutputManifest = {
    schemaVersion: 1; kind: "monitor-container-output"; preset: "monitor";
    artifactClass: "server"; compositionId: string; target: "x86_64-unknown-linux-musl";
    files: ExportFile[];
};
type Provenance = {
    schemaVersion: 1; kind: "monitor-container-provenance"; buildPlatform: "linux/amd64";
    targetTriple: "x86_64-unknown-linux-musl"; selection: unknown; selectionSha256: string;
    sourceIdentityInput: string; rustcVv: string; buildCommands: string[][]; outputManifestSha256: string;
};
const snapshotConstructionToken = Symbol("verified container export snapshot");
export class VerifiedContainerExportSnapshot {
    #files: Map<string, ArtifactFile>; #manifest: OutputManifest; #provenance: Provenance;
    private constructor(
        token: symbol,
        files: ArtifactFile[],
        manifest: OutputManifest,
        provenance: Provenance,
    ) {
        if (token !== snapshotConstructionToken)
            throw new Error("verified snapshot construction is internal");
        this.#files = new Map(files.map((file) => [file.entry.path, {
            entry: structuredClone(file.entry),
            bytes: new Uint8Array(file.bytes),
        }]));
        this.#manifest = structuredClone(manifest);
        this.#provenance = structuredClone(provenance);
    }
    static createVerified(
        token: symbol,
        files: ArtifactFile[],
        manifest: OutputManifest,
        provenance: Provenance,
    ) {
        return new VerifiedContainerExportSnapshot(token, files, manifest, provenance);
    }
    manifest() { return structuredClone(this.#manifest); }
    recordedProvenance() { return structuredClone(this.#provenance); }
    file(path: string) { const file = required(this.#files, path); if (sha256(file.bytes) !== file.entry.sha256) throw new Error(`verified snapshot digest differs: ${path}`); return new Uint8Array(file.bytes); }
    paths() { return [...this.#files.keys()].sort(compareContainerExportPath); }
}

const serverPaths = ["release/server/bin/rz-admin", "release/server/bin/rz-monitor"];
const witnessPath = "witness/bin/rz-monitor-agent";
const contractPaths = [
    "release/contracts/api/api.json", "release/contracts/config/config.json",
    "release/contracts/native/native-layout.json", "release/contracts/protocol/protocol.json",
    "release/contracts/schema/schema.json",
];
const metadataPaths = ["release/container-provenance.json", "release/output-manifest.json"];
const commands = [
    ["env", "RUSTFLAGS=-C target-feature=+crt-static", "cargo", "build", "--release", "--target", "x86_64-unknown-linux-musl", "-p", "rustzen-admin", "--no-default-features", "--features", "monitor-distribution"],
    ["env", "RUSTFLAGS=-C target-feature=+crt-static", "cargo", "build", "--release", "--target", "x86_64-unknown-linux-musl", "-p", "rustzen-monitor", "--no-default-features", "--features", "controller", "--bin", "rz-monitor"],
    ["env", "RUSTFLAGS=-C target-feature=+crt-static", "cargo", "build", "--release", "--target", "x86_64-unknown-linux-musl", "-p", "rustzen-monitor", "--no-default-features", "--features", "agent", "--bin", "rz-monitor-agent"],
];

/** Reads and validates one immutable Monitor container export without executing Linux binaries. */
export async function verifyContainerExport(
    root: string, selectionInput: unknown, expectedSourceIdentity: string,
): Promise<VerifiedContainerExportSnapshot> {
    const plan = resolveSelection(selectionInput);
    if (plan.preset !== "monitor" || plan.artifactClass !== "server" || plan.target !== "x86_64-unknown-linux-musl")
        throw new Error("container export validator supports only Monitor server selection");
    if (typeof expectedSourceIdentity !== "string" || !expectedSourceIdentity || /[\r\n]/.test(expectedSourceIdentity))
        throw new Error("expected source identity must be nonempty single-line text");
    const files = await readArtifactFileTree(root, { maxFiles: 512, maxDirectoryEntries: 256, maxTotalEntries: 768, maxDepth: 8, maxFileBytes: 64 * 1024 * 1024, maxTotalBytes: 128 * 1024 * 1024, metadataPaths, maxMetadataBytes: 256 * 1024 });
    const byPath = new Map(files.map((file) => [file.entry.path, file]));
    const manifestFile = required(byPath, "release/output-manifest.json");
    const provenanceFile = required(byPath, "release/container-provenance.json");
    const manifest = parseManifest(manifestFile.bytes);
    const provenance = parseProvenance(provenanceFile.bytes);
    const payload = files.filter((file) => !metadataPaths.includes(file.entry.path));
    verifyModes(files);
    exact(paths(files), [...manifest.files.map((file) => file.path), ...metadataPaths].sort(), "container export inventory");
    exact(manifest.files, payload.map(({ entry }) => ({ path: entry.path, mode: entry.mode, size: entry.size, sha256: entry.sha256 })), "container output manifest files");
    exact(manifestIdentity(manifest), { preset: plan.preset, artifactClass: plan.artifactClass, compositionId: plan.compositionId, target: plan.target }, "container output selection");
    if (canonicalJson(provenance.selection) !== canonicalJson(selectionInput) || provenance.selectionSha256 !== sha256(canonicalJson(selectionInput)))
        throw new Error("container provenance selection differs from input");
    if (provenance.sourceIdentityInput !== expectedSourceIdentity)
        throw new Error("container provenance source identity differs from expected input");
    if (provenance.targetTriple !== plan.target || provenance.buildPlatform !== "linux/amd64")
        throw new Error("container provenance target or build platform differs");
    verifyRustcHost(provenance.rustcVv);
    exact(provenance.buildCommands, commands, "container provenance build commands");
    if (provenance.outputManifestSha256 !== sha256(canonicalJson(manifest)))
        throw new Error("container provenance output manifest digest differs");
    exactPayload(payload);
    verifyWeb(byPath, payload, plan);
    verifyContracts(byPath, selectionInput);
    const admin = required(byPath, "release/server/bin/rz-admin").bytes;
    const monitor = required(byPath, "release/server/bin/rz-monitor").bytes;
    const agent = required(byPath, witnessPath).bytes;
    verifyMonitorExportElf(admin, "rz-admin");
    verifyMonitorExportElf(monitor, "rz-monitor");
    verifyMonitorExportElf(agent, "rz-monitor-agent");
    return VerifiedContainerExportSnapshot.createVerified(
        snapshotConstructionToken,
        files,
        manifest,
        provenance,
    );
}
function verifyWeb(files: Map<string, ArtifactFile>, payload: ArtifactFile[], plan: ReturnType<typeof resolveSelection>) {
    const inventory = parseInventory(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(required(files, "release/web/inventory.json").bytes)));
    const actual = payload.filter((file) => file.entry.path.startsWith("release/web/dist/")).map((file) => file.entry.path.slice("release/web/dist/".length)).sort(compareContainerExportPath);
    if (canonicalJson(inventory.emittedFiles) !== canonicalJson(actual)) throw new Error("container Web inventory emitted files differs");
    const text = payload.filter((file) => file.entry.path.startsWith("release/web/dist/") && /\.(?:html|js|css|map)$/.test(file.entry.path)).map((file) => new TextDecoder().decode(file.bytes)).join("\n");
    assertSelectedWebSnapshot(inventory, plan, actual, text);
}
function verifyContracts(files: Map<string, ArtifactFile>, selection: unknown) {
    parseSelectedApiBytes(required(files, "release/contracts/api/api.json").bytes, selection);
    parseSelectedConfigBytes(required(files, "release/contracts/config/config.json").bytes, selection);
    parseNativeLayoutBytes(required(files, "release/contracts/native/native-layout.json").bytes, selection);
    parseSelectedProtocolBytes(required(files, "release/contracts/protocol/protocol.json").bytes, selection);
    parseSchemaArtifactBytes(required(files, "release/contracts/schema/schema.json").bytes, selection);
}

function parseManifest(bytes: Uint8Array): OutputManifest {
    const value = json(bytes, "container output manifest");
    objectKeys(value, ["artifactClass", "compositionId", "files", "kind", "preset", "schemaVersion", "target"], "container output manifest");
    if (value.schemaVersion !== 1 || value.kind !== "monitor-container-output" || value.preset !== "monitor" || value.artifactClass !== "server" || value.target !== "x86_64-unknown-linux-musl")
        throw new Error("container output manifest identity is invalid");
    validHash(text(value.compositionId, "container output manifest compositionId"));
    const files = parseFiles(value.files);
    const result = { ...value, files } as OutputManifest;
    canonical(bytes, result, "container output manifest");
    return result;
}
function parseProvenance(bytes: Uint8Array): Provenance {
    const value = json(bytes, "container provenance");
    objectKeys(value, ["buildCommands", "buildPlatform", "kind", "outputManifestSha256", "rustcVv", "schemaVersion", "selection", "selectionSha256", "sourceIdentityInput", "targetTriple"], "container provenance");
    if (value.schemaVersion !== 1 || value.kind !== "monitor-container-provenance" || value.buildPlatform !== "linux/amd64" || value.targetTriple !== "x86_64-unknown-linux-musl")
        throw new Error("container provenance identity is invalid");
    validHash(text(value.selectionSha256, "container provenance selectionSha256"));
    validHash(text(value.outputManifestSha256, "container provenance outputManifestSha256"));
    const buildCommands = value.buildCommands;
    if (!Array.isArray(buildCommands) || buildCommands.some((command) => !Array.isArray(command) || command.some((part) => typeof part !== "string" || !part)))
        throw new Error("container provenance buildCommands is invalid");
    const result = { ...value, sourceIdentityInput: text(value.sourceIdentityInput, "container provenance sourceIdentityInput"), rustcVv: text(value.rustcVv, "container provenance rustcVv"), buildCommands } as Provenance;
    canonical(bytes, result, "container provenance");
    return result;
}
function parseFiles(value: unknown): ExportFile[] {
    if (!Array.isArray(value) || !value.length) throw new Error("container output manifest files is invalid");
    const parsed = value.map((entry) => {
        objectKeys(entry, ["mode", "path", "sha256", "size"], "container output manifest file");
        const record = entry as Record<string, unknown>;
        const path = text(record.path, "container output manifest file path");
        if (!path || path.startsWith("/") || path.includes("\\") || path.split("/").some((part) => !part || part === "." || part === ".."))
            throw new Error("container output manifest file path is invalid");
        const mode = record.mode;
        if (mode !== "0644" && mode !== "0755") throw new Error("container output manifest file mode is invalid");
        if (!Number.isSafeInteger(record.size) || (record.size as number) < 0) throw new Error("container output manifest file size is invalid");
        return { path, mode, size: record.size as number, sha256: validHash(text(record.sha256, "container output manifest file sha256")) } as ExportFile;
    });
    if (
        canonicalJson(parsed) !== canonicalJson(value) ||
        new Set(parsed.map((entry) => entry.path)).size !== parsed.length ||
        canonicalJson(parsed) !== canonicalJson(parsed.slice().sort((left, right) => compareContainerExportPath(left.path, right.path)))
    )
        throw new Error("container output manifest files must be sorted and unique");
    return parsed;
}
function exactPayload(payload: ArtifactFile[]) {
    const expected = [...serverPaths, witnessPath, ...contractPaths];
    for (const file of payload) {
        const path = file.entry.path;
        if (path.startsWith("release/web/")) {
            if (path !== "release/web/inventory.json" && !path.startsWith("release/web/dist/"))
                throw new Error(`container export has unexpected Web payload path: ${path}`);
        } else if (!expected.includes(path)) {
            throw new Error(`container export has unexpected payload path: ${path}`);
        }
    }
    exact(payload.filter((file) => !file.entry.path.startsWith("release/web/")).map((file) => file.entry.path), expected.sort(), "container binary and contract inventory");
    if (!payload.some((file) => file.entry.path.startsWith("release/web/dist/"))) throw new Error("container export selected Web dist is empty");
}
function verifyModes(files: ArtifactFile[]) {
    for (const file of files) {
        const path = file.entry.path;
        const executable = serverPaths.includes(path) || path === witnessPath;
        const expected = executable ? "0755" : "0644";
        if (file.entry.mode !== expected)
            throw new Error(`container export file mode differs: ${path}`);
    }
}
function verifyRustcHost(rustcVv: string) {
    const expected = ["rustc 1.95.0 (59807616e 2026-04-14)", "binary: rustc", "commit-hash: 59807616e1fa2540724bfbac14d7976d7e4a3860", "commit-date: 2026-04-14", "host: x86_64-unknown-linux-gnu", "release: 1.95.0", "LLVM version: 22.1.2"];
    if (rustcVv !== `${expected.join("\n")}\n`) throw new Error("container recorded rustc provenance differs from pinned Docker basis");
}
function manifestIdentity(value: OutputManifest) { return { preset: value.preset, artifactClass: value.artifactClass, compositionId: value.compositionId, target: value.target }; }
function required(files: Map<string, ArtifactFile>, path: string) { const file = files.get(path); if (!file) throw new Error(`container export is missing ${path}`); return file; }
function paths(files: ArtifactFile[]) { return files.map((file) => file.entry.path).sort(); }
function exact(actual: unknown, expected: unknown, label: string) { if (canonicalJson(actual) !== canonicalJson(expected)) throw new Error(`${label} differs`); }
function json(bytes: Uint8Array, label: string): Record<string, unknown> { try { const value = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes)); if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(); return value as Record<string, unknown>; } catch { throw new Error(`${label} is not JSON object`); } }
function objectKeys(value: unknown, keys: string[], label: string) { if (!value || typeof value !== "object" || Array.isArray(value) || canonicalJson(Object.keys(value as object).sort()) !== canonicalJson(keys)) throw new Error(`${label} fields are invalid`); }
function text(value: unknown, label: string): string { if (typeof value !== "string" || !value) throw new Error(`${label} is invalid`); return value; }
function canonical(bytes: Uint8Array, value: unknown, label: string) { if (new TextDecoder().decode(bytes) !== canonicalJson(value)) throw new Error(`${label} is not canonical`); }
