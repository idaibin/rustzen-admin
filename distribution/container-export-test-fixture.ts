import { chmod, mkdir, mkdtemp, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { canonicalJson } from "./release-manifest-core.ts";
import { produceContainerExport } from "./container-export.ts";
import { syntheticServerBuildCommands, selectedServerSyntheticExportPlan } from "./selected-server-synthetic-export-plan.ts";
import { selectedServerInventory } from "./selected-server-inventory.ts";
import { resolveSelection } from "./resolver.ts";
import { completeSelectedApiContractForTest } from "./selected-contract-validator.ts";
import { completeSelectedConfigForTest } from "./selected-config.ts";
import { generatedNativeLayout } from "./native-layout.ts";
import { completeSelectedProtocol } from "./selected-protocol.ts";
import { produceSchemaContract } from "./schema-contract.ts";
import { canonicalBindingBytes, createWebBinding, readWebFiles, stampIndex } from "./selected-web-binding.ts";
import { selectedWebRoutes } from "../scripts/distribution-web-inventory-policy.ts";
export const selection = { schemaVersion: 1, preset: "monitor", target: "x86_64-unknown-linux-musl" };
export const sourceIdentity = `git:${"a".repeat(40)} tree:${"b".repeat(64)} state:clean`;
export const releaseVersion = "0.5.0";
export async function createExport(selectionInput = selection, options: { evidence?: "linux-amd64-buildkit" } = {}) {
    const root = await mkdtemp(join(tmpdir(), "rz-container-validator-"));
    const plan = resolveSelection(selectionInput);
    const selected = selectedServerInventory(plan);
    const binaries = [
        ...selected.binaries.map((path) => `release/server/${path}`),
        ...(selected.hasAgentWitness ? ["witness/bin/rz-monitor-agent"] : []),
    ];
    const files = [...binaries, "release/web/inventory.json", "release/web/binding.json", "release/web/api.ts", "release/web/dist/index.html", "release/web/dist/rustzen.png", "release/contracts/api/api.json", "release/contracts/config/config.json", "release/contracts/schema/schema.json", "release/contracts/native/native-layout.json", "release/contracts/protocol/protocol.json"];
    for (const path of files) {
        const full = join(root, path);
        await mkdir(join(full, ".."), { recursive: true });
        const binary = path.endsWith("rz-admin") ? markerBinary("rz-admin") : path.endsWith("rz-insights") ? markerBinary("rz-insights") : path.endsWith("rz-monitor") && !path.endsWith("rz-monitor-agent") ? markerBinary("rz-monitor") : path.endsWith("rz-monitor-agent") ? markerBinary("rz-monitor-agent") : path;
        await writeFile(full, binary);
        if (binaries.includes(path)) await chmod(full, 0o755);
    }
    await writeFile(join(root, "release/contracts/api/api.json"), canonicalJson(completeSelectedApiContractForTest(selectionInput)));
    await createInventory(root, selectionInput);
    await writeFile(join(root, "release/contracts/config/config.json"), canonicalJson(completeSelectedConfigForTest(selectionInput)));
    await writeFile(join(root, "release/contracts/native/native-layout.json"), canonicalJson(generatedNativeLayout(selectionInput)));
    await writeFile(join(root, "release/contracts/protocol/protocol.json"), canonicalJson(completeSelectedProtocol(selectionInput)));
    await produceSchemaContract(selectionInput, resolve(import.meta.dir, ".."), join(root, "release/contracts/schema"));
    await produce(root, selectionInput, options);
    return root;
}
export async function createInventory(root: string, selectionInput = selection) {
    const plan = resolveSelection(selectionInput);
    const compositionId = plan.compositionId;
    const routes = selectedWebRoutes(plan);
    const analytics = plan.preset === "analytics";
    await writeFile(
        join(root, "release/web/dist/index.html"),
        (plan.preset === "monitor-notify" ? "/api/notifications/stream /api/notifications/unread-count Message center " : "") + "/api/auth/login /api/auth/me " + (analytics ? "/api/insights/overview /api/insights/events /analytics/overview" : "/api/monitor/ /monitoring/overview") + " /api/system/users /api/system/roles /api/system/menus/options<meta name=\"rustzen-web-binding\" content=\"__RUSTZEN_WEB_DIGEST__\" />",
    );
    await writeFile(join(root, "release/web/api.ts"), `export const selectedApi = '${analytics ? "/api/insights/" : "/api/monitor/"}';\n`);
    const before = await readWebFiles(join(root, "release/web/dist"));
    const binding = createWebBinding({
        compositionId,
        selectedApiBytes: await Bun.file(join(root, "release/web/api.ts")).bytes(),
        files: before,
    });
    const index = before.find((file) => file.path === "index.html");
    if (!index) throw new Error("fixture index is missing");
    await writeFile(join(root, "release/web/dist/index.html"), stampIndex(index.bytes, binding.webDigest));
    const files = await readWebFiles(join(root, "release/web/dist"));
    await writeFile(join(root, "release/web/inventory.json"), canonicalJson({
        schemaVersion: 2,
        preset: plan.preset,
        compositionId,
        generatedRoot: `apps/web/.selected-web/${compositionId}`,
        outputDirectory: `target/distributions/${compositionId}/web/dist`,
        selectedRoutes: routes,
        publicAssets: ["rustzen.png"],
        emittedFiles: files.map((file) => file.path),
        fileInventory: files.map(({ path, size, sha256 }) => ({ path, size, sha256 })),
        moduleIds: [
            `apps/web/.selected-web/${compositionId}/index.tsx`,
            "apps/web/src/api/installation/api.ts", analytics ? "apps/web/src/api/insights/contract.ts" : plan.preset === "monitor-notify" ? "apps/web/src/api/monitor/api.ts" : "apps/web/src/api/monitor/core-api.ts", "apps/web/src/api/request.ts",
            ...(plan.preset === "monitor-notify" ? ["apps/web/src/api/notifications/api.ts", "apps/web/src/notifications/message-center.tsx"] : []),
        ],
        binding,
    }));
    await writeFile(join(root, "release/web/binding.json"), canonicalBindingBytes(binding));
}
export async function produce(root: string, selectionInput = selection, options: { evidence?: "linux-amd64-buildkit" } = {}) {
    const plan = resolveSelection(selectionInput);
    selectedServerSyntheticExportPlan(plan);
    await produceContainerExport({ selection: selectionInput, outputRoot: root, targetTriple: "x86_64-unknown-linux-musl", sourceIdentity, buildCommands: syntheticServerBuildCommands(plan), rustcVv: recordedRustc(), releaseVersion: "0.5.0", runtime: options.evidence ? { platform: "linux", arch: "x64" } : plan.preset === "analytics" ? { platform: "darwin", arch: "arm64" } : { platform: "linux", arch: "x64" }, evidence: options.evidence });
}
export function recordedRustc() { return "rustc 1.95.0 (59807616e 2026-04-14)\nbinary: rustc\ncommit-hash: 59807616e1fa2540724bfbac14d7976d7e4a3860\ncommit-date: 2026-04-14\nhost: x86_64-unknown-linux-gnu\nrelease: 1.95.0\nLLVM version: 22.1.2\n"; }
export function expectedCommands(selectionInput = selection) {
    const plan = resolveSelection(selectionInput);
    selectedServerSyntheticExportPlan(plan);
    return syntheticServerBuildCommands(plan);
}
export function markerBinary(name: string, options: { interpreted?: boolean } = {}) {
    const bytes = new Uint8Array(256);
    bytes.set([0x7f, 0x45, 0x4c, 0x46, 2, 1, 1], 0);
    const view = new DataView(bytes.buffer);
    view.setUint16(16, 3, true); view.setUint16(18, 62, true);
    view.setUint32(20, 1, true); view.setBigUint64(24, 0x400080n, true); view.setBigUint64(32, 64n, true); view.setUint16(52, 64, true); view.setUint16(54, 56, true); view.setUint16(56, 1, true);
    view.setUint32(64, options.interpreted ? 3 : 1, true);
    view.setUint32(68, 1, true); view.setBigUint64(72, 0n, true); view.setBigUint64(80, 0x400000n, true); view.setBigUint64(96, 256n, true); view.setBigUint64(104, 256n, true);
    bytes.set(new TextEncoder().encode(`RUSTZEN_RELEASE_MARKER\nartifact=rz-bundle-member\nbinary=${name}\n`), 128);
    return bytes;
}
