import { describe, expect, test } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const script = `${import.meta.dir}/distribution-resolve.ts`;
const fixture = `${import.meta.dir}/../distribution/fixtures`;
const decode = (value: Uint8Array) => new TextDecoder().decode(value);

describe("distribution resolver CLI", () => {
    test("rejects invalid native targets", () => {
        const directory = mkdtempSync(join(tmpdir(), "rustzen-selection-"));
        try {
            const input = join(directory, "selection.json");
            for (const target of ["../bad", "not-a-rust-target"]) {
                writeFileSync(input, JSON.stringify({ preset: "monitor", target }));
                const result = Bun.spawnSync([
                    process.execPath,
                    script,
                    "validate",
                    "--selection",
                    input,
                ]);
                expect(result.exitCode).toBe(1);
                expect(JSON.parse(decode(result.stderr)).error).toContain(
                    "supported native target",
                );
            }
        } finally {
            rmSync(directory, { recursive: true });
        }
    });
    test("resolves JSON and rejects unknown input fields", () => {
        const resolved = Bun.spawnSync([
            "bun",
            script,
            "resolve",
            "--selection",
            `${fixture}/monitor.json`,
        ]);
        expect(resolved.exitCode).toBe(0);
        expect(JSON.parse(decode(resolved.stdout)).capabilities).toEqual(["access", "monitor"]);

        const rejected = Bun.spawnSync([
            "bun",
            script,
            "validate",
            "--selection",
            `${fixture}/unknown-field.json`,
        ]);
        expect(rejected.exitCode).toBe(1);
        expect(JSON.parse(decode(rejected.stderr)).error).toContain("unknown selection field");
    });

    test("fails closed at the release gate", () => {
        const result = Bun.spawnSync([
            "bun",
            script,
            "release-gate",
            "--selection",
            `${fixture}/monitor.json`,
        ]);
        expect(result.exitCode).toBe(1);
        expect(JSON.parse(decode(result.stderr)).error).toContain("release gate closed");
    });

    test("production release gate rejects the test-only regression fixture", () => {
        const result = Bun.spawnSync([
            "bun",
            script,
            "release-gate",
            "--selection",
            `${fixture}/current-full-regression.json`,
        ]);
        expect(result.exitCode).toBe(1);
        expect(JSON.parse(decode(result.stderr)).error).toContain("rejects test-only");
    });

    test("rejects unknown CLI arguments as JSON errors", () => {
        const result = Bun.spawnSync([
            "bun",
            script,
            "resolve",
            "--wrong",
            `${fixture}/monitor.json`,
        ]);
        expect(result.exitCode).toBe(1);
        expect(JSON.parse(decode(result.stderr)).error).toContain("usage:");
    });
});
