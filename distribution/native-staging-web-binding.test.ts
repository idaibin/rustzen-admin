import { rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { expect, test } from "bun:test";
import { produceNativeStaging } from "./native-staging.ts";
import { nativeStagingRoots } from "./native-staging-test-fixture.ts";
import { writeFixtureWebBinding } from "./release-manifest-fixtures.ts";

test("native server staging requires the exact retained Web binding tuple", async () => {
    for (const mutation of ["missing", "malformed", "api"] as const) {
        const { fixture, binaryRoot, selection } =
            await nativeStagingRoots("server");
        const parent = join(fixture.webRoot, "..");
        try {
            if (mutation === "missing") await rm(join(parent, "binding.json"));
            if (mutation === "malformed") {
                await writeFile(join(parent, "binding.json"), "{}");
            }
            if (mutation === "api") {
                await writeFile(join(parent, "api.ts"), "changed-api");
            }
            await expect(
                produceNativeStaging({
                    selection,
                    outputParent: join(fixture.root, "binding-reject"),
                    trustedRoot: fixture.root,
                    releaseVersion: "1.0.0",
                    sourceIdentity: "test-source",
                    toolchain: "test-toolchain",
                    selectedRoutes: [],
                    binaryRoot,
                    webRoot: fixture.webRoot,
                    apiRoot: fixture.apiRoot,
                    schemaRoot: fixture.schemaRoot,
                    configRoot: fixture.configRoot,
                    nativeRoot: fixture.nativeRoot,
                    protocolRoot: fixture.protocolRoot,
                }),
            ).rejects.toThrow();
        } finally {
            await rm(fixture.root, { recursive: true, force: true });
        }
    }
});

test("native server staging enforces the complete selected Web policy", async () => {
    for (const mutation of [
        "preset",
        "composition",
        "generatedRoot",
        "outputDirectory",
        "routes",
        "assets",
        "modules",
        "forbiddenText",
    ] as const) {
        const { fixture, binaryRoot, selection } =
            await nativeStagingRoots("server");
        const parent = join(fixture.webRoot, "..");
        try {
            if (mutation === "forbiddenText") {
                await writeFile(
                    join(fixture.webRoot, "assets", "main.js"),
                    "/api/reports",
                );
                await writeFixtureWebBinding(fixture.webRoot, selection);
            } else {
                const path = join(parent, "inventory.json");
                const inventory = await Bun.file(path).json();
                if (mutation === "preset") inventory.preset = "full";
                if (mutation === "composition") {
                    inventory.compositionId = "0".repeat(64);
                }
                if (mutation === "generatedRoot") inventory.generatedRoot = "../escape";
                if (mutation === "outputDirectory") inventory.outputDirectory = "../escape";
                if (mutation === "routes") inventory.selectedRoutes = ["reports/overview.tsx"];
                if (mutation === "assets") inventory.publicAssets = [];
                if (mutation === "modules") inventory.moduleIds = ["apps/web/src/routes/reports.tsx"];
                await writeFile(path, JSON.stringify(inventory));
            }
            await expect(
                produceNativeStaging({
                    selection,
                    outputParent: join(fixture.root, "policy-reject"),
                    trustedRoot: fixture.root,
                    releaseVersion: "1.0.0",
                    sourceIdentity: "test-source",
                    toolchain: "test-toolchain",
                    selectedRoutes: ["forged-route"],
                    binaryRoot,
                    webRoot: fixture.webRoot,
                    apiRoot: fixture.apiRoot,
                    schemaRoot: fixture.schemaRoot,
                    configRoot: fixture.configRoot,
                    nativeRoot: fixture.nativeRoot,
                    protocolRoot: fixture.protocolRoot,
                }),
            ).rejects.toThrow();
        } finally {
            await rm(fixture.root, { recursive: true, force: true });
        }
    }
});

test("native server staging derives build routes from verified Web inventory", async () => {
    const { fixture, binaryRoot, selection } = await nativeStagingRoots("server");
    const input = {
        selection,
        trustedRoot: fixture.root,
        releaseVersion: "1.0.0",
        sourceIdentity: "test-source",
        toolchain: "test-toolchain",
        binaryRoot,
        webRoot: fixture.webRoot,
        apiRoot: fixture.apiRoot,
        schemaRoot: fixture.schemaRoot,
        configRoot: fixture.configRoot,
        nativeRoot: fixture.nativeRoot,
        protocolRoot: fixture.protocolRoot,
    };
    try {
        const first = await produceNativeStaging({
            ...input,
            selectedRoutes: ["forged-a"],
            outputParent: join(fixture.root, "route-a"),
        });
        const second = await produceNativeStaging({
            ...input,
            selectedRoutes: ["forged-b"],
            outputParent: join(fixture.root, "route-b"),
        });
        expect(second.buildId).toBe(first.buildId);
    } finally {
        await rm(fixture.root, { recursive: true, force: true });
    }
});
