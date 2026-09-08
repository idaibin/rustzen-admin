import { expect, test } from "bun:test";
import { chmod, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";

import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import { produceContainerExport } from "./container-export.ts";
import {
    VerifiedContainerExportSnapshot,
    verifyContainerExport,
} from "./container-export-validator.ts";
import { completeSelectedApiContractForTest } from "./selected-contract-validator.ts";
import { completeSelectedConfigForTest } from "./selected-config.ts";
import { generatedNativeLayout } from "./native-layout.ts";
import { completeSelectedProtocol } from "./selected-protocol.ts";
import { produceSchemaContract } from "./schema-contract.ts";

const selection = { schemaVersion: 1, preset: "monitor", target: "x86_64-unknown-linux-musl" };
const sourceIdentity = "git:abc123 tree:def456";

test("host validator returns one verified byte snapshot without executing Linux binaries", async () => {
    const root = await createExport();
    try {
        const snapshot = await verifyContainerExport(root, selection, sourceIdentity);
        expect(snapshot.paths().length).toBeGreaterThan(10);
        expect(snapshot.file("release/server/bin/rz-admin")[0]).toBe(0x7f);
        const copy = snapshot.file("release/server/bin/rz-admin"); copy[0] = 0;
        expect(snapshot.file("release/server/bin/rz-admin")[0]).toBe(0x7f);
        const manifestCopy = snapshot.manifest(); manifestCopy.preset = "changed" as any;
        expect(snapshot.manifest().preset).toBe("monitor");
        const cli = Bun.spawnSync([
            process.execPath,
            resolve(import.meta.dir, "../scripts/distribution-verify-container-export.ts"),
            "--selection", "distribution/fixtures/monitor.json",
            "--export-root", root,
            "--expected-source-identity", sourceIdentity,
        ], { cwd: resolve(import.meta.dir, ".."), stdout: "pipe", stderr: "pipe" });
        expect(cli.exitCode).toBe(0);
        expect(JSON.parse(new TextDecoder().decode(cli.stdout))).toMatchObject({ verified: true });
        await writeFile(join(root, "release/server/bin/rz-admin"), "replaced");
        expect(snapshot.file("release/server/bin/rz-admin")[0]).toBe(0x7f);
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("files differs");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("verified snapshot construction is internal and copies validator inputs", async () => {
    expect(() => new (VerifiedContainerExportSnapshot as any)()).toThrow(
        "verified snapshot construction is internal",
    );
    expect(() => (VerifiedContainerExportSnapshot as any).createVerified(
        Symbol("external"), [], {}, {},
    )).toThrow("verified snapshot construction is internal");

    const root = await createExport();
    try {
        const snapshot = await verifyContainerExport(root, selection, sourceIdentity);
        const original = snapshot.file("release/server/bin/rz-admin");
        await writeFile(join(root, "release/server/bin/rz-admin"), "changed after verification");
        expect(snapshot.file("release/server/bin/rz-admin")).toEqual(original);
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("host validator rejects malformed fixed ELF header fields", async () => {
    const root = await createExport();
    const binaryPath = join(root, "release/server/bin/rz-monitor");
    try {
        const original = markerBinary("rz-monitor");
        for (const [label, mutate] of [
            ["EI_VERSION", (bytes: Uint8Array) => { bytes[6] = 0; }],
            ["e_version", (bytes: Uint8Array) => new DataView(bytes.buffer).setUint32(20, 0, true)],
            ["e_ehsize", (bytes: Uint8Array) => new DataView(bytes.buffer).setUint16(52, 63, true)],
            ["e_phentsize", (bytes: Uint8Array) => new DataView(bytes.buffer).setUint16(54, 55, true)],
        ] as const) {
            const bytes = new Uint8Array(original);
            mutate(bytes);
            await writeFile(binaryPath, bytes);
            await produce(root);
            await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow(label === "EI_VERSION" ? "EI_VERSION" : label === "e_version" ? "e_version" : "header sizes");
        }
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("host validator rejects provenance and ELF identity mutations", async () => {
    const root = await createExport();
    try {
        const provenancePath = join(root, "release/container-provenance.json");
        const provenance = await Bun.file(provenancePath).json();
        provenance.buildCommands = [["cargo", "build"]];
        await writeFile(provenancePath, canonicalJson(provenance));
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("build commands");
        await writeFile(provenancePath, canonicalJson({ ...provenance, buildCommands: expectedCommands() }));
        await writeFile(join(root, "release/server/bin/rz-monitor"), markerBinary("rz-monitor", { interpreted: true }));
        await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("PT_INTERP");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("host validator rejects noncanonical manifest order, rustc host ambiguity and actual modes", async () => {
    const root = await createExport();
    try {
        const manifestPath = join(root, "release/output-manifest.json");
        const provenancePath = join(root, "release/container-provenance.json");
        const manifest = await Bun.file(manifestPath).json();
        manifest.files.reverse();
        const provenance = await Bun.file(provenancePath).json();
        provenance.outputManifestSha256 = sha256(canonicalJson(manifest));
        await writeFile(manifestPath, canonicalJson(manifest));
        await writeFile(provenancePath, canonicalJson(provenance));
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("sorted and unique");
        await produce(root);
        const validProvenance = await Bun.file(provenancePath).json();
        validProvenance.rustcVv = recordedRustc() + "host: x86_64-unknown-linux-gnu\n";
        await writeFile(provenancePath, canonicalJson(validProvenance));
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("pinned Docker basis");
        await produce(root);
        await chmod(provenancePath, 0o755);
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("file mode differs");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("host validator permits only inventory and dist under the selected Web root", async () => {
    const root = await createExport();
    try {
        await writeFile(join(root, "release/web/evil"), "unexpected");
        await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("unexpected Web payload");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("host validator rejects changed selection, identity and unexpected payload paths", async () => {
    const root = await createExport();
    try {
        await expect(verifyContainerExport(root, selection, "other")).rejects.toThrow("source identity");
        await writeFile(join(root, "release/contracts/extra.json"), "{}");
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("inventory differs");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("host validator parses every selected contract from retained export bytes", async () => {
    const root = await createExport();
    try {
        await writeFile(join(root, "release/contracts/api/api.json"), "{}");
        await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("selected API artifact");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("host validator binds Web inventory identity, emitted files and excluded module policy", async () => {
    const root = await createExport();
    const inventoryPath = join(root, "release/web/inventory.json");
    try {
        await writeFile(inventoryPath, "{}"); await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("selected Web inventory");
        await createInventory(root);
        const inventory = await Bun.file(inventoryPath).json(); inventory.emittedFiles = ["missing.js"]; await writeFile(inventoryPath, canonicalJson(inventory)); await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("emitted files differs");
        await createInventory(root);
        const excluded = await Bun.file(inventoryPath).json(); excluded.moduleIds = ["apps/web/src/api/notifications/inbox.ts"]; await writeFile(inventoryPath, canonicalJson(excluded)); await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("unclassified module ID");
    } finally { await rm(root, { recursive: true, force: true }); }
});

test("host validator requires every declared public asset in the emitted snapshot", async () => {
    const root = await createExport();
    try {
        await rm(join(root, "release/web/dist/rustzen.png"));
        const inventoryPath = join(root, "release/web/inventory.json");
        const inventory = await Bun.file(inventoryPath).json();
        inventory.emittedFiles = ["index.html"];
        await writeFile(inventoryPath, canonicalJson(inventory));
        await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("public asset is absent");
    } finally { await rm(root, { recursive: true, force: true }); }
});

test("host validator binds selected Web routes, public assets, and output text", async () => {
    const root = await createExport();
    const inventoryPath = join(root, "release/web/inventory.json");
    try {
        const routes = await Bun.file(inventoryPath).json();
        routes.selectedRoutes = [];
        await writeFile(inventoryPath, canonicalJson(routes));
        await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("selectedRoutes mismatch");

        await createInventory(root);
        const assets = await Bun.file(inventoryPath).json();
        assets.publicAssets = [];
        await writeFile(inventoryPath, canonicalJson(assets));
        await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("publicAssets mismatch");

        await createInventory(root);
        await writeFile(join(root, "release/web/dist/index.html"), "no selected Web API text");
        await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity)).rejects.toThrow("missing required text");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

async function createExport() {
    const root = await mkdtemp(join(tmpdir(), "rz-container-validator-"));
    const binaries = ["release/server/bin/rz-admin", "release/server/bin/rz-monitor", "witness/bin/rz-monitor-agent"];
    const files = [...binaries, "release/web/inventory.json", "release/web/dist/index.html", "release/web/dist/rustzen.png", "release/contracts/api/api.json", "release/contracts/config/config.json", "release/contracts/schema/schema.json", "release/contracts/native/native-layout.json", "release/contracts/protocol/protocol.json"];
    for (const path of files) {
        const full = join(root, path);
        await mkdir(join(full, ".."), { recursive: true });
        const binary = path.endsWith("rz-admin") ? markerBinary("rz-admin") : path.endsWith("rz-monitor") && !path.endsWith("rz-monitor-agent") ? markerBinary("rz-monitor") : path.endsWith("rz-monitor-agent") ? markerBinary("rz-monitor-agent") : path;
        await writeFile(full, binary);
        if (binaries.includes(path)) await chmod(full, 0o755);
    }
    await writeFile(join(root, "release/contracts/api/api.json"), canonicalJson(completeSelectedApiContractForTest(selection)));
    await createInventory(root);
    await writeFile(join(root, "release/contracts/config/config.json"), canonicalJson(completeSelectedConfigForTest(selection)));
    await writeFile(join(root, "release/contracts/native/native-layout.json"), canonicalJson(generatedNativeLayout(selection)));
    await writeFile(join(root, "release/contracts/protocol/protocol.json"), canonicalJson(completeSelectedProtocol(selection)));
    await produceSchemaContract(selection, resolve(import.meta.dir, ".."), join(root, "release/contracts/schema"));
    await produce(root);
    return root;
}
async function createInventory(root: string) {
    const compositionId = completeSelectedApiContractForTest(selection).compositionId;
    const routes = [
        "index.tsx", "__root.tsx", "403.tsx", "404.tsx", "login.tsx", "monitoring.tsx",
        "monitoring/incidents.tsx", "monitoring/nodes.tsx", "monitoring/overview.tsx",
        "monitoring/summaries.tsx", "profile.tsx", "system/role.tsx", "system/user.tsx",
        "monitoring/-global-alert-settings.tsx", "monitoring/-incident-drawer.tsx",
        "monitoring/-node-details.tsx", "monitoring/-node-onboarding.tsx", "monitoring/-save-state.ts",
        "system/-role-actions.tsx", "system/-role-delete-state.ts", "system/-role-dialog.tsx",
        "system/-role-permission-picker.tsx", "system/-user-actions.tsx", "system/-user-dialog.tsx",
    ].sort();
    await writeFile(
        join(root, "release/web/dist/index.html"),
        "/api/auth/login /api/auth/me /api/monitor/ /api/system/users /api/system/roles /api/system/menus/options /monitoring/overview",
    );
    await writeFile(join(root, "release/web/inventory.json"), canonicalJson({
        schemaVersion: 1,
        preset: "monitor",
        compositionId,
        generatedRoot: `apps/web/.selected-web/${compositionId}`,
        outputDirectory: `target/distributions/${compositionId}/web/dist`,
        selectedRoutes: routes,
        publicAssets: ["rustzen.png"],
        emittedFiles: ["index.html", "rustzen.png"],
        moduleIds: [`apps/web/.selected-web/${compositionId}/index.tsx`],
    }));
}
async function produce(root: string) {
    await produceContainerExport({ selection, outputRoot: root, targetTriple: "x86_64-unknown-linux-musl", sourceIdentity, buildCommands: expectedCommands(), rustcVv: recordedRustc(), runtime: { platform: "linux", arch: "x64" } });
}
function recordedRustc() { return "rustc 1.95.0 (59807616e 2026-04-14)\nbinary: rustc\ncommit-hash: 59807616e1fa2540724bfbac14d7976d7e4a3860\ncommit-date: 2026-04-14\nhost: x86_64-unknown-linux-gnu\nrelease: 1.95.0\nLLVM version: 22.1.2\n"; }
function expectedCommands() {
    return [
        ["env", "RUSTFLAGS=-C target-feature=+crt-static", "cargo", "build", "--release", "--target", "x86_64-unknown-linux-musl", "-p", "rustzen-admin", "--no-default-features", "--features", "monitor-distribution"],
        ["env", "RUSTFLAGS=-C target-feature=+crt-static", "cargo", "build", "--release", "--target", "x86_64-unknown-linux-musl", "-p", "rustzen-monitor", "--no-default-features", "--features", "controller", "--bin", "rz-monitor"],
        ["env", "RUSTFLAGS=-C target-feature=+crt-static", "cargo", "build", "--release", "--target", "x86_64-unknown-linux-musl", "-p", "rustzen-monitor", "--no-default-features", "--features", "agent", "--bin", "rz-monitor-agent"],
    ];
}
function markerBinary(name: string, options: { interpreted?: boolean } = {}) {
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
