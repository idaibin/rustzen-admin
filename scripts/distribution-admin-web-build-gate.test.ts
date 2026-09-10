import { expect, test } from "bun:test";
import { createHash } from "node:crypto";
import { stat, utimes } from "node:fs/promises";
import { resolve } from "node:path";

import { allowedWebPackages } from "./distribution-web-allowed-packages.ts";

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

const checkFeatures = (features: string) =>
    Bun.spawnSync(
        ["cargo", "check", "-p", "rustzen-admin", "--no-default-features", "--features", features],
        {
            cwd: root,
            env: { ...process.env, CARGO_TERM_COLOR: "never" },
            stdout: "pipe",
            stderr: "pipe",
        },
    );

test("Analytics Admin binds the reviewed API authority digest", async () => {
    const analyticsId = "62d09d09b3b0e94f88329179bf9a8c1fa84a984ba87df341911dab0e7fcf0a40";
    const embedRoot = resolve(root, "apps/admin/selected-web", analyticsId);
    const expected = "3f1799bcf33b1d18a479486401c35bc43820b8e89eb1381b9cc8ec9e1464e85d";
    const source = await Bun.file(
        resolve(root, "apps/web/src/distribution/analytics-api.ts"),
    ).bytes();
    const policy = await Bun.file(resolve(root, "apps/admin/build_support/selected_api.rs")).text();
    const binding = await Bun.file(resolve(embedRoot, "binding.json")).json();
    const inventory = await Bun.file(resolve(embedRoot, "inventory.json")).json();

    expect(createHash("sha256").update(source).digest("hex")).toBe(expected);
    expect(policy).toContain(expected);
    expect(binding.selectedApiDigest).toBe(expected);
    expect(inventory.binding.selectedApiDigest).toBe(expected);
});

test("Admin accepts only reviewed composition feature unions", () => {
    for (const features of [
        "full",
        "monitor-distribution",
        "monitor-distribution,notifications",
        "analytics-distribution",
    ])
        expect(checkFeatures(features).exitCode, features).toBe(0);

    for (const [features, marker] of [
        ["selected-distribution", "select exactly one reviewed Admin composition feature"],
        [
            "selected-distribution,notifications",
            "select exactly one reviewed Admin composition feature",
        ],
        [
            "monitor-distribution,reports-notifications",
            "Reports notifications require the full Admin composition",
        ],
        ["full,monitor-distribution", "select exactly one Admin configuration profile"],
        [
            "analytics-distribution,notifications",
            "admin-insights cannot include another process configuration",
        ],
        [
            "monitor-distribution,analytics-distribution",
            "select exactly one Admin configuration profile",
        ],
        ["full,analytics-distribution", "select exactly one Admin configuration profile"],
        [
            "analytics-distribution,rustzen-config/admin",
            "select exactly one Admin configuration profile",
        ],
        ...["insights", "monitor-controller", "monitor-agent", "reports", "notifications"].map(
            (feature) => [
                `analytics-distribution,rustzen-config/${feature}`,
                "admin-insights cannot include another process configuration",
            ],
        ),
    ]) {
        const result = checkFeatures(features);
        expect(result.exitCode, features).not.toBe(0);
        expect(new TextDecoder().decode(result.stderr), features).toContain(marker);
    }
}, 120_000);

test("Rust and producer package owner policies stay identical", async () => {
    const source = await Bun.file(
        resolve(root, "apps/admin/build_support/module_policy.rs"),
    ).text();
    const values = (name: string) => {
        const block = source.match(new RegExp(`const ${name}: &\\[&str\\] = &\\[([\\s\\S]*?)\\];`));
        if (!block?.[1]) throw new Error(`missing Rust package policy ${name}`);
        return [...block[1].matchAll(/"([^"]+)"/g)].map((match) => match[1]);
    };
    const rust = new Set([
        ...values("PACKAGES"),
        ...values("RC_COMPONENTS").map((name) => `@rc-component/${name}`),
    ]);
    expect([...rust].sort()).toEqual([...allowedWebPackages()].sort());
});

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
        const portable = structuredClone(clean);
        portable.moduleIds.splice(
            portable.moduleIds.findIndex((id: string) =>
                id.startsWith("apps/web/node_modules/dayjs/"),
            ),
            1,
        );
        await write(portable);
        expect((await check()).status, "portable allowed module graph").toBe(0);
        await reject("unknown field", (v) => (v.unknown = true), "inventory schema mismatch");
        await reject("missing field", (v) => delete v.binding, "inventory schema mismatch");
        await reject("selected routes", (v) => v.selectedRoutes.pop(), "route set mismatch");
        await reject("public assets", (v) => v.publicAssets.pop(), "public assets mismatch");
        await reject(
            "required module removed",
            (v) =>
                (v.moduleIds = v.moduleIds.filter(
                    (id: string) => id !== "apps/web/src/api/installation/api.ts",
                )),
            "missing required owner",
        );
        await reject(
            "forbidden module owner",
            (v) => {
                v.moduleIds.push("apps/web/src/api/insights/api.ts");
                v.moduleIds.sort();
            },
            "unclassified owner",
        );
        await reject(
            "unsafe module path",
            (v) => {
                v.moduleIds.push("../apps/web/src/api/reports/api.ts");
                v.moduleIds.sort();
            },
            "unsafe module ID",
        );
        await reject(
            "another selected root",
            (v) => {
                v.moduleIds.push("apps/web/.selected-web/other-composition/api.ts");
                v.moduleIds.sort();
            },
            "another composition source",
        );
        await reject(
            "selected root removed",
            (v) =>
                (v.moduleIds = v.moduleIds.filter(
                    (id: string) => !id.startsWith(`apps/web/.selected-web/${compositionId}/`),
                )),
            "no generated route source",
        );
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
