import { chmod, mkdir, mkdtemp, readdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { expect, test } from "bun:test";
import { reviewedProtocolOutput } from "../distribution/selected-protocol.ts";

const root = resolve(import.meta.dir, "..");
const script = resolve(import.meta.dir, "distribution-produce-protocol.ts");

test("protocol producer requires every explicit input exactly once", () => {
    const invalid = [
        ["--selection", "distribution/fixtures/monitor.json"],
        ["--selection", "distribution/fixtures/monitor.json", "--binary-root", "target/debug"],
        ["--selection", "distribution/fixtures/monitor.json", "--output-root", "target/out"],
        ["--selection", "distribution/fixtures/monitor.json", "--unknown", "value"],
        [
            "--selection",
            "distribution/fixtures/monitor.json",
            "--selection",
            "distribution/fixtures/node-agent.json",
            "--binary-root",
            "target/debug",
            "--output-root",
            "target/out",
        ],
    ];
    for (const args of invalid) {
        const result = Bun.spawnSync([process.execPath, script, ...args], {
            cwd: root,
            stdout: "pipe",
            stderr: "pipe",
        });
        expect(result.exitCode).not.toBe(0);
        expect(new TextDecoder().decode(result.stderr)).toContain(
            "--binary-root <release-binary-root> --output-root <output-root>",
        );
    }
});

test("Analytics CLI reads only Admin and Insights and writes protocol.json", async () => {
    const scratch = await mkdtemp(join(tmpdir(), "rz-analytics-protocol-cli-"));
    try {
        const binaries = join(scratch, "bin");
        const output = join(scratch, "out");
        await mkdir(binaries);
        const descriptor = reviewedProtocolOutput({ preset: "analytics" });
        for (const binary of ["rz-admin", "rz-insights"]) {
            const path = join(binaries, binary);
            await writeFile(path, `#!/bin/sh\ncat <<'EOF'\n${descriptor}EOF\n`);
            await chmod(path, 0o755);
        }
        const result = Bun.spawnSync([
            process.execPath,
            script,
            "--selection", join(root, "distribution/fixtures/analytics.json"),
            "--binary-root", binaries,
            "--output-root", output,
        ], { cwd: "/tmp", stdout: "pipe", stderr: "pipe" });
        expect(result.exitCode).toBe(0);
        expect(new TextDecoder().decode(result.stderr)).toBe("");
        expect(await readdir(output)).toEqual(["protocol.json"]);
    } finally {
        await rm(scratch, { recursive: true, force: true });
    }
});

test("protocol CLI rejects an unsupported selection before running peers", async () => {
    const scratch = await mkdtemp(join(tmpdir(), "rz-unsupported-protocol-cli-"));
    try {
        const binaries = join(scratch, "bin");
        const output = join(scratch, "out");
        const marker = join(scratch, "ran");
        await mkdir(binaries);
        for (const binary of ["rz-monitor", "rz-monitor-agent"]) {
            const path = join(binaries, binary);
            await writeFile(path, `#!/bin/sh\ntouch '${marker}'\nexit 1\n`);
            await chmod(path, 0o755);
        }
        const result = Bun.spawnSync([
            process.execPath,
            script,
            "--selection", join(root, "distribution/fixtures/reports.json"),
            "--binary-root", binaries,
            "--output-root", output,
        ], { cwd: "/tmp", stdout: "pipe", stderr: "pipe" });
        expect(result.exitCode).not.toBe(0);
        expect(new TextDecoder().decode(result.stderr)).toContain(
            "unavailable for this selection",
        );
        expect(await Bun.file(marker).exists()).toBeFalse();
        expect(await Bun.file(output).exists()).toBeFalse();
    } finally {
        await rm(scratch, { recursive: true, force: true });
    }
});
