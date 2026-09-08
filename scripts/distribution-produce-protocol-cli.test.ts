import { expect, test } from "bun:test";
import { resolve } from "node:path";

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
