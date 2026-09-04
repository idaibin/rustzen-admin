import { describe, expect, test } from "bun:test";
import { rename, rm, symlink } from "node:fs/promises";
import { join, resolve } from "node:path";

import { resolveSelection } from "../distribution/resolver.ts";

const repositoryRoot = resolve(import.meta.dir, "..");
const webRoot = join(repositoryRoot, "apps/web");
const fixture = join(repositoryRoot, "distribution/fixtures/monitor.json");
const selection = resolveSelection(await Bun.file(fixture).json());
const distributionRoot = join(repositoryRoot, "target/distributions", selection.compositionId);
const outputRoot = join(distributionRoot, "web");
const inventoryPath = join(outputRoot, "inventory.json");
const generatedRoot = join(webRoot, ".selected-web", selection.compositionId);
const selectedApiSource = join(webRoot, "src/distribution/monitor-api.ts");
const verifier = join(repositoryRoot, "scripts/distribution-verify-web.ts");
const producer = join(repositoryRoot, "scripts/distribution-build-web.ts");

const run = (script: string) =>
    Bun.spawnSync([process.execPath, script, "--selection", fixture], { cwd: repositoryRoot });
const verify = () => run(verifier);

describe("selected monitor Web inventory", () => {
    test("accepts a clean full-then-monitor build and rejects producer/inventory mutations", async () => {
        const fullBuild = Bun.spawnSync([process.execPath, "run", "build"], { cwd: webRoot });
        expect(fullBuild.exitCode).toBe(0);
        expect(run(producer).exitCode).toBe(0);
        expect(verify().exitCode).toBe(0);
        const cleanInventory = await Bun.file(inventoryPath).json() as { moduleIds: string[] };
        expect(cleanInventory.moduleIds).toContain("\0vite/preload-helper.js");
        expect(cleanInventory.moduleIds.some((id) => id.startsWith("apps/web/node_modules/react/"))).toBe(true);
        expect(cleanInventory.moduleIds.some((id) => id.split("?", 1)[0] === "apps/web/src/api/request.ts")).toBe(true);
        expect(cleanInventory.moduleIds.some((id) => id.split("?", 1)[0] === "apps/web/src/components/theme-provider.tsx")).toBe(true);
        expect(cleanInventory.moduleIds.some((id) => id.startsWith("apps/web/src/api/monitor/"))).toBe(true);

        await mutateInventory("excluded module", (inventory) => {
            inventory.moduleIds.push("apps/web/src/api/insights/api.ts");
        });
        await mutateInventory("exact source filename prefix bypass", (inventory) => {
            inventory.moduleIds.push("apps/web/src/api/request.ts-malicious.ts");
        });
        await mutateInventory("exact component filename prefix bypass", (inventory) => {
            inventory.moduleIds.push("apps/web/src/components/theme-provider.tsx-backdoor.ts");
        });
        await mutateInventory("another composition module", (inventory) => {
            inventory.moduleIds.push("apps/web/.selected-web/other-composition/api.ts");
        });
        await mutateInventory("absolute module escape", (inventory) => {
            inventory.moduleIds.push("/tmp/unselected.ts");
        });
        await mutateInventory("relative module escape", (inventory) => {
            inventory.moduleIds.push("../apps/web/src/api/reports/api.ts");
        });
        await mutateInventory("unknown node modules package", (inventory) => {
            inventory.moduleIds.push("apps/web/node_modules/unknown-selected-package/index.js");
        });
        await mutateInventory("unknown virtual module", (inventory) => {
            inventory.moduleIds.push("\0vite/unselected-helper.js");
        });
        await mutateInventory("preset", (inventory) => {
            inventory.preset = "reports";
        });
        await mutateInventory("composition", (inventory) => {
            inventory.compositionId = "tampered";
        });
        await mutateInventory("generated root path escape", (inventory) => {
            inventory.generatedRoot = "../escape";
        });
        await mutateInventory("output directory", (inventory) => {
            inventory.outputDirectory = "target/distributions/other/web/dist";
        });
        await mutateInventory("routes", (inventory) => {
            inventory.selectedRoutes.push("analytics/overview.tsx");
        });
        await mutateInventory("assets", (inventory) => {
            inventory.publicAssets.push("__rustzen_admin_marker__.json");
        });
        await mutateInventory("emitted files", (inventory) => {
            inventory.emittedFiles.push("assets/stale-full.js");
        });

        const apiPath = join(generatedRoot, "api.ts");
        const originalApi = await Bun.file(apiPath).text();
        try {
            await Bun.write(apiPath, `${originalApi}\nconst leaked = "/api/" + "reports";\n`);
            expect(verify().exitCode).not.toBe(0);
        } finally {
            await Bun.write(apiPath, originalApi);
        }

        const extraAsset = join(outputRoot, "dist", "extra-static.js");
        try {
            await Bun.write(extraAsset, "unexpected");
            expect(verify().exitCode).not.toBe(0);
        } finally {
            await rm(extraAsset, { force: true });
        }

        const symlinkAsset = join(outputRoot, "dist", "linked-static.js");
        try {
            await symlink("rustzen.png", symlinkAsset);
            expect(verify().exitCode).not.toBe(0);
        } finally {
            await rm(symlinkAsset, { force: true });
        }
        await rejectPathSymlink(join(outputRoot, "dist"), "dist root");
        await rejectPathSymlink(outputRoot, "web output root");
        await rejectPathSymlink(distributionRoot, "distribution root");
        await rejectPathSymlink(generatedRoot, "generated root");
        await rejectPathSymlink(apiPath, "generated API source");
        await rejectPathSymlink(selectedApiSource, "selected API source");
        expect(verify().exitCode).toBe(0);
    });
});

async function mutateInventory(
    _name: string,
    mutate: (inventory: {
        preset: string;
        compositionId: string;
        generatedRoot: string;
        outputDirectory: string;
        selectedRoutes: string[];
        publicAssets: string[];
        emittedFiles: string[];
        moduleIds: string[];
    }) => void,
) {
    const original = await Bun.file(inventoryPath).text();
    try {
        const inventory = JSON.parse(original) as Parameters<typeof mutate>[0];
        mutate(inventory);
        await Bun.write(inventoryPath, `${JSON.stringify(inventory, null, 2)}\n`);
        expect(verify().exitCode).not.toBe(0);
    } finally {
        await Bun.write(inventoryPath, original);
    }
}

async function rejectPathSymlink(path: string, _name: string) {
    const backup = `${path}.selected-web-test-real`;
    try {
        await rename(path, backup);
        await symlink(backup, path);
        expect(verify().exitCode).not.toBe(0);
    } finally {
        await rm(path, { force: true });
        await rename(backup, path);
    }
}
