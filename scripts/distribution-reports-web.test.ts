import { expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import { join, resolve } from "node:path";

import { resolveSelection } from "../distribution/resolver";

const root = resolve(import.meta.dir, "..");
const fixture = "distribution/fixtures/reports.json";
const selection = resolveSelection(require(join(root, fixture)));
const output = join(root, "target/distributions", selection.compositionId, "web");
const run = (script: string) =>
    Bun.spawnSync([process.execPath, join(root, "scripts", script), "--selection", fixture], {
        cwd: root,
        stdout: "pipe",
        stderr: "pipe",
    });

test("Reports selected Web contains only access and Reports owners", async () => {
    expect(run("distribution-build-web.ts").exitCode).toBe(0);
    expect(run("distribution-verify-web.ts").exitCode).toBe(0);
    const inventory = JSON.parse(await readFile(join(output, "inventory.json"), "utf8")) as {
        selectedRoutes: string[];
        moduleIds: string[];
        emittedFiles: string[];
    };
    expect(inventory.selectedRoutes).toContain("reports/templates.tsx");
    expect(inventory.selectedRoutes).toContain("reports/runs.tsx");
    expect(inventory.selectedRoutes).toContain("reports/-templates/templates-content.tsx");
    expect(
        inventory.moduleIds.some((id) => id.split("?", 1)[0] === "apps/web/src/api/reports/core-api.ts"),
    ).toBe(true);
    expect(
        inventory.moduleIds.some((id) => /api\/(?:monitor|insights|notifications)\//.test(id)),
    ).toBe(false);
    expect(
        inventory.moduleIds.some((id) => id.split("?", 1)[0] === "apps/web/src/api/reports/api.ts"),
    ).toBe(false);
    const text = (
        await Promise.all(
            inventory.emittedFiles
                .filter((file) => /\.(?:html|js)$/.test(file))
                .map((file) => readFile(join(output, "dist", file), "utf8")),
        )
    ).join("\n");
    expect(text).toContain("/reports/templates");
    expect(text).toContain("/api/reports/runs");
    expect(text).not.toContain("/api/reports/notification-delivery");
    expect(text).not.toContain("notification-delivery-card");
    expect(text).not.toContain("/api/monitor");
    expect(text).not.toContain("/monitoring/");
    expect(text).not.toContain("/api/insights");
}, 120_000);
