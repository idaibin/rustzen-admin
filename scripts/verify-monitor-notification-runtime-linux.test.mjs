import { describe, expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { join } from "node:path";
import {
    fakeGateFixture,
    runFakeGate,
    runSignaledFakeGate,
} from "./verify-monitor-notification-runtime-linux.test-support.mjs";

const directory = new URL("./", import.meta.url);
const outerUrl = new URL("verify-monitor-notification-runtime-linux.sh", directory);
const innerUrl = new URL("verify-monitor-notification-runtime-linux-inner.sh", directory);
const clientUrl = new URL("monitor-notification-runtime-client.py", directory);
const outer = await Bun.file(outerUrl).text();
const inner = await Bun.file(innerUrl).text();
const client = await Bun.file(clientUrl).text();
const evidenceValidator = await Bun.file(
    new URL("verify-monitor-notification-runtime-evidence.jq", directory),
).text();
const verifier = await Bun.file(new URL("admin-browser-verifier.Dockerfile", directory)).text();
const justfile = await Bun.file(new URL("../justfile", directory)).text();
const validation = await Bun.file(
    new URL("../docs/product/features/composable-distribution/validation.md", directory),
).text();

function lines(text) {
    return text.trimEnd().split("\n").length;
}

describe("Monitor notification Linux runtime gate", () => {
    test("is bounded, source-bound, atomically published, and compact", () => {
        expect(lines(outer)).toBeLessThan(300);
        expect(lines(inner)).toBeLessThan(300);
        expect(lines(client)).toBeLessThan(300);
        for (const value of [
            "RUSTZEN_MONITOR_NOTIFY_TIMEOUT",
            "admin-browser-source-identity.sh",
            "sourceTreeSha256",
            "atomic_replace_symlink",
            "failed-runs",
            "stop_tree",
            "receipt_allowlist",
            "remove_container",
            "container inspect",
        ]) {
            expect(outer).toContain(value);
        }
        expect(justfile).toContain("verify-monitor-notification-runtime-linux:");
        expect(justfile).toContain("scripts/verify-monitor-notification-runtime-linux.sh");
        expect(evidenceValidator).toContain('length == 18');
        expect(evidenceValidator).toContain('test("^[A-Za-z0-9][A-Za-z0-9._-]*$")');
    });

    test("builds positive and negative artifacts and drives real TCP plus SQLite", () => {
        for (const value of [
            "monitor-distribution,notifications",
            "--features notifications",
            "--features monitor-distribution",
            "--features controller",
            "rz-admin-notify",
            "rz-monitor-notify",
            "rz-admin-pure",
            "rz-monitor-pure",
        ]) {
            expect(outer).toContain(value);
        }
        for (const value of [
            "notification_outbox",
            "notification_receipts",
            "selected-ingress-listener.txt",
            "FAILED STEP",
            "/api/notifications?limit=100",
            "monitor.incident.opened",
            "monitor.incident.resolved",
            "bad-signature.json",
            "unsigned.json",
            "public-internal.json",
            "plain-absence.json",
            "sport = :19822",
            "relayTask:false",
        ]) {
            expect(inner).toContain(value);
        }
        expect(inner).not.toContain("mock");
    });

    test("uses the frozen signature domain without adding an unpinned runtime", () => {
        expect(client).toContain('DOMAIN = "rz-notification-producer-v1"');
        expect(client).toContain("4af5d59a23f3d472ed96b7fb4c3839f7e8cf5b06ff83b82c036ea5ebde13f850");
        expect(verifier).toContain("python3");
        expect(verifier).not.toMatch(/\bbun\b/i);
        expect(validation).toContain("already includes Python but intentionally contains no Bun");
        const result = Bun.spawnSync(["python3", clientUrl.pathname, "--self-test"]);
        expect(result.exitCode).toBe(0);
    });

    test("rejects invalid limits before contacting Docker and has valid shell syntax", () => {
        for (const script of [outerUrl.pathname, innerUrl.pathname]) {
            expect(Bun.spawnSync(["bash", "-n", script]).exitCode).toBe(0);
        }
        const result = Bun.spawnSync({
            cmd: ["bash", outerUrl.pathname],
            env: { ...process.env, RUSTZEN_MONITOR_NOTIFY_TIMEOUT: "0" },
            stdout: "pipe",
            stderr: "pipe",
        });
        expect(result.exitCode).toBe(2);
        expect(new TextDecoder().decode(result.stderr)).toContain("must be 1..1800 seconds");
    });

    test("publishes only after both fake Docker containers are absent", () => {
        const fixture = fakeGateFixture(outerUrl.pathname);
        try {
            const result = runFakeGate(fixture);
            expect(result.exitCode).toBe(0);
            expect(fixture.current()).toStartWith("runs/");
            expect(fixture.current()).not.toBe("runs/previous");
            expect(fixture.active()).toEqual([]);
            expect(fixture.dockerCalls().match(/container inspect/g)?.length).toBe(2);
            expect(existsSync(join(fixture.evidence, fixture.current(), "manifest.json"))).toBeTrue();
        } finally {
            fixture.cleanup();
        }
    });

    test("rejects receipt and semantic tampering without replacing current", () => {
        for (const tamper of [
            "duplicate-receipt", "wrong-status", "path-traversal", "open-status",
            "artifact-status", "plain-status",
        ]) {
            const fixture = fakeGateFixture(outerUrl.pathname, { tamper });
            try {
                const result = runFakeGate(fixture);
                expect(result.exitCode, tamper).not.toBe(0);
                expect(fixture.current(), tamper).toBe("runs/previous");
                expect(fixture.failed().length, tamper).toBe(1);
                expect(fixture.active(), tamper).toEqual([]);
            } finally {
                fixture.cleanup();
            }
        }
    }, 30_000);

    test("returns 124 and retains failure evidence after a blocked run", () => {
        const fixture = fakeGateFixture(outerUrl.pathname, { block: "build", timeout: "1" });
        try {
            const result = runFakeGate(fixture);
            expect(result.exitCode).toBe(124);
            expect(fixture.current()).toBe("runs/previous");
            expect(fixture.failed().length).toBe(1);
            expect(fixture.active()).toEqual([]);
        } finally {
            fixture.cleanup();
        }
    });

    for (const [signal, exitCode] of [["SIGINT", 130], ["SIGTERM", 143]]) {
        test(`returns ${exitCode}, cleans resources, and preserves current on ${signal}`, async () => {
            const fixture = fakeGateFixture(outerUrl.pathname, { block: "build", timeout: "30" });
            try {
                const result = await runSignaledFakeGate(fixture, signal);
                expect(result.exitCode).toBe(exitCode);
                expect(fixture.current()).toBe("runs/previous");
                expect(fixture.failed().length).toBe(1);
                expect(fixture.active()).toEqual([]);
            } finally {
                fixture.cleanup();
            }
        }, 15_000);
    }

    test("bounds failure-log collection and still removes the runtime container", () => {
        const fixture = fakeGateFixture(outerUrl.pathname, { runtimeExit: "7", block: "logs" });
        try {
            const result = runFakeGate(fixture);
            expect(result.exitCode).toBe(7);
            expect(fixture.current()).toBe("runs/previous");
            expect(fixture.failed().length).toBe(1);
            expect(fixture.active()).toEqual([]);
            expect(fixture.dockerCalls()).toContain("logs rz-monitor-notify-runtime-");
        } finally {
            fixture.cleanup();
        }
    });

    test("a blocked removal is bounded and cannot publish current", () => {
        const fixture = fakeGateFixture(outerUrl.pathname, { block: "rm" });
        try {
            const result = runFakeGate(fixture);
            expect(result.exitCode).not.toBe(0);
            expect(fixture.current()).toBe("runs/previous");
            expect(fixture.failed().length).toBe(1);
            expect(fixture.dockerCalls().match(/rm -f rz-monitor-notify-build-/g)?.length).toBe(2);
        } finally {
            fixture.cleanup();
        }
    });
});
