import { describe, expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { resolveSelection } from "../distribution/resolver";

const root = resolve(import.meta.dir, "..");
const producer = join(root, "scripts/distribution-build-web.ts");
const verifier = join(root, "scripts/distribution-verify-web.ts");
const build = (fixture: string) =>
    Bun.spawnSync([process.execPath, producer, "--selection", fixture], {
        cwd: root,
        stdout: "pipe",
        stderr: "pipe",
    });
const verify = (fixture: string) =>
    Bun.spawnSync([process.execPath, verifier, "--selection", fixture], {
        cwd: root,
        stdout: "pipe",
        stderr: "pipe",
    });
const output = (fixture: string) => {
    const selection = resolveSelection(require(join(root, fixture)));
    return join(root, "target/distributions", selection.compositionId, "web");
};

describe("notification-owned selected Web graph", () => {
    test("monitor-notify includes the shell owner while pure monitor contains no notification module or text", async () => {
        const notifyFixture = "distribution/fixtures/monitor-notify.json";
        const pureFixture = "distribution/fixtures/monitor.json";
        expect(build(notifyFixture).exitCode).toBe(0);
        expect(verify(notifyFixture).exitCode).toBe(0);
        const notify = JSON.parse(
            await readFile(join(output(notifyFixture), "inventory.json"), "utf8"),
        ) as { emittedFiles: string[]; moduleIds: string[]; selectedRoutes: string[] };
        const selection = resolveSelection(require(join(root, notifyFixture)));
        const generated = join(root, "apps/web/.selected-web", selection.compositionId);
        expect(await readFile(join(generated, "routes/index.tsx"), "utf8")).toContain(
            'redirect({ to: "/monitoring/overview" })',
        );
        expect(await readFile(join(generated, "auth-store.ts"), "utf8")).toContain(
            '"/": "monitor:overview:view"',
        );
        expect(await readFile(join(generated, "index.html"), "utf8")).toContain(
            '<link href="./style.css" rel="stylesheet" />',
        );
        expect(await readFile(join(generated, "style.css"), "utf8")).toContain(
            '@import "tailwindcss"',
        );
        expect(notify.emittedFiles.some((file) => file.endsWith(".css"))).toBe(true);
        expect(await readFile(join(output(notifyFixture), "dist/index.html"), "utf8")).toMatch(
            /<link rel="stylesheet" href="\/assets\/[^/]+\.css" \/>/,
        );
        expect(notify.selectedRoutes).toContain("-notifications-shell.tsx");
        expect(notify.moduleIds.some((id) => id.startsWith("apps/web/src/notifications/"))).toBe(
            true,
        );
        expect(
            notify.moduleIds.some((id) => id.startsWith("apps/web/src/api/notifications/")),
        ).toBe(true);

        expect(build(pureFixture).exitCode).toBe(0);
        expect(verify(pureFixture).exitCode).toBe(0);
        const pureRoot = output(pureFixture);
        const pure = JSON.parse(await readFile(join(pureRoot, "inventory.json"), "utf8")) as {
            moduleIds: string[];
            emittedFiles: string[];
            selectedRoutes: string[];
        };
        expect(pure.selectedRoutes).not.toContain("-notifications-shell.tsx");
        expect(
            pure.moduleIds.some(
                (id) =>
                    id.startsWith("apps/web/src/notifications/") ||
                    id.startsWith("apps/web/src/api/notifications/"),
            ),
        ).toBe(false);
        const text = (
            await Promise.all(
                pure.emittedFiles
                    .filter((file) => /\.(?:js|html)$/.test(file))
                    .map((file) => readFile(join(pureRoot, "dist", file), "utf8")),
            )
        ).join("\n");
        expect(text).not.toContain("/api/notifications");
        expect(text).not.toContain("Message center");
    }, 120_000);
});
