import { expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import { join, resolve } from "node:path";

import { resolveSelection } from "../distribution/resolver";

const root = resolve(import.meta.dir, "..");
const fixture = "distribution/fixtures/analytics.json";
const selection = resolveSelection(require(join(root, fixture)));
const output = join(root, "target/distributions", selection.compositionId, "web");
const run = (script: string) =>
    Bun.spawnSync([process.execPath, join(root, "scripts", script), "--selection", fixture], {
        cwd: root,
        stdout: "pipe",
        stderr: "pipe",
    });

test("Analytics selected Web contains only access and Insights owners", async () => {
    expect(run("distribution-build-web.ts").exitCode).toBe(0);
    expect(run("distribution-verify-web.ts").exitCode).toBe(0);
    const inventory = JSON.parse(await readFile(join(output, "inventory.json"), "utf8")) as {
        selectedRoutes: string[];
        moduleIds: string[];
        emittedFiles: string[];
    };
    expect(inventory.selectedRoutes).toContain("analytics/overview.tsx");
    expect(inventory.selectedRoutes).toContain("analytics/details.tsx");
    expect(inventory.moduleIds.some((id) => id.startsWith("apps/web/src/api/insights/"))).toBe(
        false,
    );
    expect(
        inventory.moduleIds.some((id) => /api\/(?:monitor|reports|notifications)\//.test(id)),
    ).toBe(false);
    const text = (
        await Promise.all(
            inventory.emittedFiles
                .filter((file) => /\.(?:html|js)$/.test(file))
                .map((file) => readFile(join(output, "dist", file), "utf8")),
        )
    ).join("\n");
    expect(text).toContain("/analytics/overview");
    expect(text).toContain("/api/insights/overview");
    expect(text).toContain("/api/insights/events");
    expect(text).not.toContain("/api/insights/collection-policy");
    expect(text).not.toContain("/api/insights/track");
    expect(text).not.toContain("/monitoring/");
    expect(text).not.toContain("/api/reports");
}, 120_000);
