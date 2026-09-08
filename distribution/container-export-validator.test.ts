import { expect, test } from "bun:test";
import { chmod, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";

import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import { createExport, createInventory, expectedCommands, markerBinary, produce, recordedRustc, releaseVersion, selection, sourceIdentity } from "./container-export-test-fixture.ts";
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


test("host validator returns one verified byte snapshot without executing Linux binaries", async () => {
    const root = await createExport();
    try {
        const snapshot = await verifyContainerExport(root, selection, sourceIdentity, releaseVersion);
        expect(snapshot.paths().length).toBeGreaterThan(10);
        expect(snapshot.artifact("release/server/bin/rz-admin").bytes[0]).toBe(0x7f);
        const copy = snapshot.artifact("release/server/bin/rz-admin").bytes; copy[0] = 0;
        expect(snapshot.artifact("release/server/bin/rz-admin").bytes[0]).toBe(0x7f);
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
        expect(snapshot.artifact("release/server/bin/rz-admin").bytes[0]).toBe(0x7f);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("files differs");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("verified snapshot construction is internal and copies validator inputs", async () => {
    expect(() => new (VerifiedContainerExportSnapshot as any)()).toThrow(
        "verified snapshot construction is internal",
    );
    expect(Object.hasOwn(VerifiedContainerExportSnapshot, "createVerified")).toBeFalse();
    expect(Object.isFrozen(VerifiedContainerExportSnapshot)).toBeTrue();
    expect(Object.isFrozen(VerifiedContainerExportSnapshot.prototype)).toBeTrue();
    expect(() => Object.defineProperty(VerifiedContainerExportSnapshot, "createVerified", {
        value: () => undefined,
    })).toThrow();

    const root = await createExport();
    try {
        const snapshot = await verifyContainerExport(root, selection, sourceIdentity, releaseVersion);
        const original = snapshot.artifact("release/server/bin/rz-admin").bytes;
        await writeFile(join(root, "release/server/bin/rz-admin"), "changed after verification");
        expect(snapshot.artifact("release/server/bin/rz-admin").bytes).toEqual(original);
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
            await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow(label === "EI_VERSION" ? "EI_VERSION" : label === "e_version" ? "e_version" : "header sizes");
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
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("build commands");
        await writeFile(provenancePath, canonicalJson({ ...provenance, buildCommands: expectedCommands() }));
        await writeFile(join(root, "release/server/bin/rz-monitor"), markerBinary("rz-monitor", { interpreted: true }));
        await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("PT_INTERP");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("host validator rejects malformed recorded release versions and unknown provenance fields", async () => {
    const root = await createExport();
    const path = join(root, "release/container-provenance.json");
    try {
        for (const invalidVersion of ["", "future\nversion", "x".repeat(65)]) {
            const provenance = await Bun.file(path).json();
            provenance.releaseVersion = invalidVersion;
            await writeFile(path, canonicalJson(provenance));
            await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("provenance");
            await produce(root);
        }
        await expect(
            verifyContainerExport(root, selection, sourceIdentity, "0.5.1"),
        ).rejects.toThrow("workspace version");
        const provenance = await Bun.file(path).json();
        await writeFile(path, canonicalJson({ ...provenance, unknown: true }));
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("fields");
    } finally { await rm(root, { recursive: true, force: true }); }
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
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("sorted and unique");
        await produce(root);
        const validProvenance = await Bun.file(provenancePath).json();
        validProvenance.rustcVv = recordedRustc() + "host: x86_64-unknown-linux-gnu\n";
        await writeFile(provenancePath, canonicalJson(validProvenance));
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("pinned Docker basis");
        await produce(root);
        await chmod(provenancePath, 0o755);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("file mode differs");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("host validator permits only inventory and dist under the selected Web root", async () => {
    const root = await createExport();
    try {
        await writeFile(join(root, "release/web/evil"), "unexpected");
        await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("unexpected Web payload");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("host validator rejects changed selection, identity and unexpected payload paths", async () => {
    const root = await createExport();
    try {
        await expect(verifyContainerExport(root, selection, "other", releaseVersion)).rejects.toThrow("source identity");
        await writeFile(join(root, "release/contracts/extra.json"), "{}");
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("inventory differs");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("host validator parses every selected contract from retained export bytes", async () => {
    const root = await createExport();
    try {
        await writeFile(join(root, "release/contracts/api/api.json"), "{}");
        await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("selected API artifact");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("host validator binds Web inventory identity, emitted files and excluded module policy", async () => {
    const root = await createExport();
    const inventoryPath = join(root, "release/web/inventory.json");
    try {
        await writeFile(inventoryPath, "{}"); await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("selected Web inventory");
        await createInventory(root);
        const inventory = await Bun.file(inventoryPath).json(); inventory.emittedFiles = ["missing.js"]; await writeFile(inventoryPath, canonicalJson(inventory)); await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("emitted files differs");
        await createInventory(root);
        const excluded = await Bun.file(inventoryPath).json(); excluded.moduleIds = ["apps/web/src/api/notifications/inbox.ts"]; await writeFile(inventoryPath, canonicalJson(excluded)); await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("unclassified module ID");
    } finally { await rm(root, { recursive: true, force: true }); }
});

test("host validator rejects descriptor, inventory, stamped HTML, and emitted-file binding mutations", async () => {
    const root = await createExport();
    const inventoryPath = join(root, "release/web/inventory.json");
    const bindingPath = join(root, "release/web/binding.json");
    try {
        const descriptor = await Bun.file(bindingPath).json();
        descriptor.webDigest = "0".repeat(64);
        await writeFile(bindingPath, canonicalJson(descriptor)); await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("binding differs");

        await createInventory(root);
        const inventory = await Bun.file(inventoryPath).json();
        inventory.binding.webDigest = "0".repeat(64);
        await writeFile(inventoryPath, canonicalJson(inventory)); await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("binding differs");

        await createInventory(root);
        const files = await Bun.file(inventoryPath).json();
        files.fileInventory[0].sha256 = "0".repeat(64);
        await writeFile(inventoryPath, canonicalJson(files)); await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("file digest mismatch");

        await createInventory(root);
        const indexPath = join(root, "release/web/dist/index.html");
        const index = await Bun.file(indexPath).text();
        const changed = index.replace(/[a-f0-9]{64}/, "0".repeat(64));
        await writeFile(indexPath, changed);
        const stamped = await Bun.file(inventoryPath).json();
        const entry = stamped.fileInventory.find((file: { path: string }) => file.path === "index.html");
        entry.sha256 = sha256(changed); entry.size = new TextEncoder().encode(changed).length;
        await writeFile(inventoryPath, canonicalJson(stamped)); await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("binding stamp mismatch");
    } finally { await rm(root, { recursive: true, force: true }); }
});

test("host validator requires every declared public asset in the emitted snapshot", async () => {
    const root = await createExport();
    try {
        await rm(join(root, "release/web/dist/rustzen.png"));
        await createInventory(root);
        await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("public asset is absent");
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
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("selectedRoutes mismatch");

        await createInventory(root);
        const assets = await Bun.file(inventoryPath).json();
        assets.publicAssets = [];
        await writeFile(inventoryPath, canonicalJson(assets));
        await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("publicAssets mismatch");

        await createInventory(root);
        await writeFile(join(root, "release/web/dist/index.html"), "no selected Web API text");
        await produce(root);
        await expect(verifyContainerExport(root, selection, sourceIdentity, releaseVersion)).rejects.toThrow("binding marker");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});
