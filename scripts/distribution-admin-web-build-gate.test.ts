import { expect, test } from "bun:test";
import { stat, utimes } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const compositionId = "8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b";
const inventoryPath = resolve(root, "apps/admin/selected-web", compositionId, "inventory.json");
const cargo = [
    "cargo",
    "check",
    "-p",
    "rustzen-admin",
    "--no-default-features",
    "--features",
    "monitor-distribution",
];

test("Admin build rejects every selected Web inventory v2 mutation", async () => {
    const original = await Bun.file(inventoryPath).bytes();
    const originalState = await stat(inventoryPath);
    const clean = JSON.parse(new TextDecoder().decode(original));
    let timestamp = Date.now() + 10_000;
    const write = async (value: unknown) => {
        await Bun.write(inventoryPath, JSON.stringify(value));
        timestamp += 2_000;
        await utimes(inventoryPath, originalState.atime, new Date(timestamp));
    };
    const restore = async () => {
        await Bun.write(inventoryPath, original);
        timestamp += 2_000;
        await utimes(inventoryPath, originalState.atime, new Date(timestamp));
    };
    const check = async () => {
        const child = Bun.spawn(cargo, {
            cwd: root,
            env: { ...process.env, CARGO_TERM_COLOR: "never" },
            stdout: "pipe",
            stderr: "pipe",
        });
        const [stdout, stderr, status] = await Promise.all([
            new Response(child.stdout).text(),
            new Response(child.stderr).text(),
            child.exited,
        ]);
        return { status, output: stdout + stderr };
    };
    const reject = async (label: string, mutate: (value: any) => void, marker: string) => {
        const value = structuredClone(clean);
        mutate(value);
        await write(value);
        const result = await check();
        expect(result.status, label).not.toBe(0);
        expect(result.output, label).toContain(marker);
    };

    try {
        expect((await check()).status).toBe(0);
        await reject("unknown field", (v) => (v.unknown = true), "inventory schema mismatch");
        await reject("missing field", (v) => delete v.binding, "inventory schema mismatch");
        await reject("selected routes", (v) => v.selectedRoutes.pop(), "route set mismatch");
        await reject("public assets", (v) => v.publicAssets.pop(), "public assets mismatch");
        await reject("module IDs", (v) => v.moduleIds.pop(), "canonical inventory digest mismatch");
        await reject(
            "generated root",
            (v) => (v.generatedRoot += ".old"),
            "generated root mismatch",
        );
        await reject("output directory", (v) => (v.outputDirectory += ".old"), "output mismatch");
        await reject(
            "emitted file order",
            (v) =>
                ([v.emittedFiles[0], v.emittedFiles[1]] = [v.emittedFiles[1], v.emittedFiles[0]]),
            "emitted files must be sorted and unique",
        );
        await reject(
            "file table order",
            (v) =>
                ([v.fileInventory[0], v.fileInventory[1]] = [
                    v.fileInventory[1],
                    v.fileInventory[0],
                ]),
            "file inventory must be sorted and unique",
        );
    } finally {
        await restore();
        expect((await check()).status).toBe(0);
        await utimes(inventoryPath, originalState.atime, originalState.mtime);
    }
}, 120_000);
