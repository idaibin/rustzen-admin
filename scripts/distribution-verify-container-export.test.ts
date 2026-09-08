import { expect, test } from "bun:test";
import { resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const script = resolve(import.meta.dir, "distribution-verify-container-export.ts");

test("container export verifier CLI requires exactly its three explicit inputs", () => {
    const invalid = [
        [],
        ["--selection", "distribution/fixtures/monitor.json"],
        ["--selection", "distribution/fixtures/monitor.json", "--export-root", "target/export"],
        ["--selection", "distribution/fixtures/monitor.json", "--export-root", "target/export", "--unexpected", "value"],
        ["--selection", "distribution/fixtures/monitor.json", "--selection", "distribution/fixtures/monitor.json", "--export-root", "target/export", "--expected-source-identity", "source"],
    ];
    for (const args of invalid) {
        const result = Bun.spawnSync([process.execPath, script, ...args], { cwd: root, stdout: "pipe", stderr: "pipe" });
        expect(result.exitCode).not.toBe(0);
        expect(new TextDecoder().decode(result.stderr)).toContain("--expected-source-identity <source-identity>");
    }
});
