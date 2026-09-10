import { existsSync } from "node:fs";
import { mkdir, mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { expect, test } from "bun:test";
import {
    selectedCargoBuilds,
    selectedServiceCargoBuilds,
} from "../distribution/selected-cargo-producer.ts";
import { resolveSelection } from "../distribution/resolver.ts";

const repositoryRoot = resolve(import.meta.dir, "..");

test(
    "Analytics service producer builds with only the selected Insights config owner",
    async () => {
        const scratch = await mkdtemp(join(tmpdir(), "rz-analytics-service-producer-"));
        try {
            const plan = resolveSelection(
                await Bun.file(join(repositoryRoot, "distribution/fixtures/analytics.json")).json(),
            );
            for (const command of selectedServiceCargoBuilds(plan)) {
                const build = Bun.spawnSync(command, {
                    cwd: repositoryRoot,
                    env: { ...process.env, CARGO_TARGET_DIR: join(scratch, "target") },
                    stdout: "pipe",
                    stderr: "pipe",
                });
                expect(new TextDecoder().decode(build.stderr)).not.toContain("error:");
                expect(build.exitCode).toBe(0);
            }
            const tree = Bun.spawnSync([
                "cargo", "tree", "-p", "rustzen-insights", "--no-default-features",
                "--features", "selected-distribution", "-e", "features",
                "-i", "rustzen-config", "--prefix", "none",
            ], { cwd: repositoryRoot, stdout: "pipe", stderr: "pipe" });
            expect(tree.exitCode).toBe(0);
            const configFeatures = new TextDecoder().decode(tree.stdout)
                .split("\n")
                .filter((line) => line.startsWith('rustzen-config feature "'));
            expect(configFeatures).toEqual(['rustzen-config feature "insights"']);
        } finally {
            await rm(scratch, { recursive: true, force: true });
        }
    },
    300_000,
);

test(
    "formal monitor-notify producer builds a clean selected binary and exports real owners",
    async () => {
        const scratch = await mkdtemp(join(tmpdir(), "rz-monitor-notify-producer-"));
        try {
            const cwd = join(scratch, "cwd");
            const target = join(scratch, "target");
            const output = join(scratch, "contracts");
            await mkdir(cwd);
            expect(existsSync(target)).toBeFalse();
            const selectionPath = join(
                repositoryRoot,
                "distribution/fixtures/monitor-notify.json",
            );
            const plan = resolveSelection(await Bun.file(selectionPath).json());
            for (const command of selectedCargoBuilds(plan)) {
                const build = Bun.spawnSync(command, {
                    cwd: repositoryRoot,
                    env: { ...process.env, CARGO_TARGET_DIR: target },
                    stdout: "pipe",
                    stderr: "pipe",
                });
                expect(build.exitCode).toBe(0);
            }
            const result = Bun.spawnSync(
                [
                    process.execPath,
                    join(repositoryRoot, "scripts/distribution-produce-contracts.ts"),
                    "--selection",
                    selectionPath,
                    "--binary-root",
                    join(target, "debug"),
                ],
                {
                    cwd,
                    env: {
                        ...process.env,
                        RUSTZEN_CONTRACT_TARGET_DIR: target,
                        RUSTZEN_CONTRACT_OUTPUT_ROOT: output,
                    },
                    stdout: "pipe",
                    stderr: "pipe",
                },
            );
            expect(new TextDecoder().decode(result.stderr)).toBe("");
            expect(result.exitCode).toBe(0);
            expect(existsSync(join(target, "debug/rz-admin"))).toBeTrue();

            const configArtifact = JSON.parse(
                await readFile(join(output, "config/config.json"), "utf8"),
            );
            expect(Object.keys(configArtifact.owners)).toEqual([
                "access",
                "monitor",
                "notifications",
            ]);
            const emittedConfig = Bun.spawnSync(
                [
                    join(target, "debug/rz-admin"),
                    "contract",
                    "config",
                    "selected",
                    "notifications",
                ],
                { cwd, env: { PATH: process.env.PATH ?? "" }, stdout: "pipe" },
            );
            expect(emittedConfig.exitCode).toBe(0);
            expect(JSON.parse(new TextDecoder().decode(emittedConfig.stdout))).toEqual(
                configArtifact.owners.notifications,
            );

            const artifact = JSON.parse(
                await readFile(join(output, "api/api.json"), "utf8"),
            );
            expect(Object.keys(artifact.owners)).toEqual([
                "admin",
                "monitor",
                "notifications",
            ]);
            for (const owner of ["admin", "notifications"] as const) {
                const emitted = Bun.spawnSync(
                    [
                        join(target, "debug/rz-admin"),
                        "contract",
                        "selected",
                        owner,
                    ],
                    { cwd, env: { PATH: process.env.PATH ?? "" }, stdout: "pipe" },
                );
                expect(emitted.exitCode).toBe(0);
                expect(JSON.parse(new TextDecoder().decode(emitted.stdout))).toEqual(
                    artifact.owners[owner],
                );
            }
            expect(
                artifact.owners.admin.routes.some((route: { path: string }) =>
                    route.path.startsWith("/api/notifications"),
                ),
            ).toBeFalse();
            expect(
                artifact.owners.notifications.routes.every((route: { path: string }) =>
                    route.path.startsWith("/api/notifications"),
                ),
            ).toBeTrue();
        } finally {
            await rm(scratch, { recursive: true, force: true });
        }
    },
    300_000,
);
