import { describe, expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { join } from "node:path";
import {
    fakeGateFixture,
    runFakeGate,
    runSignaledFakeGate,
} from "./verify-reports-notification-runtime-linux.test-support.mjs";

const directory = new URL("./", import.meta.url);
const outerUrl = new URL("verify-reports-notification-runtime-linux.sh", directory);
const innerUrl = new URL("verify-reports-notification-runtime-linux-inner.sh", directory);
const clientUrl = new URL("reports-notification-runtime-client.py", directory);
const outer = await Bun.file(outerUrl).text();
const inner = await Bun.file(innerUrl).text();
const client = await Bun.file(clientUrl).text();
const evidenceValidator = await Bun.file(
    new URL("verify-reports-notification-runtime-evidence.jq", directory),
).text();
const verifier = await Bun.file(new URL("admin-browser-verifier.Dockerfile", directory)).text();
const justfile = await Bun.file(new URL("../justfile", directory)).text();
const validation = await Bun.file(
    new URL("../docs/product/features/composable-distribution/validation.md", directory),
).text();

function lines(text) {
    return text.trimEnd().split("\n").length;
}

describe("Reports notification Linux runtime gate", () => {
    test("is bounded, source-bound, atomically published, and compact", () => {
        expect(lines(outer)).toBeLessThan(300);
        expect(lines(inner)).toBeLessThan(300);
        expect(lines(client)).toBeLessThan(300);
        for (const value of [
            "RUSTZEN_REPORTS_NOTIFY_TIMEOUT",
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
        expect(justfile).toContain("verify-reports-notification-runtime-linux:");
        expect(justfile).toContain("scripts/verify-reports-notification-runtime-linux.sh");
        expect(evidenceValidator).toContain('length==25');
        expect(evidenceValidator).toContain('test("^[A-Za-z0-9][A-Za-z0-9._-]*$")');
    });

    test("builds positive and negative artifacts and drives real TCP plus SQLite", () => {
        for (const value of [
            "-p rustzen-admin --bin rz-admin",
            "-p rustzen-reports --bin rz-reports",
            "--features monitor-distribution",
            "rustzen-reports --no-default-features",
            "rz-admin-selected",
            "rz-reports-selected",
            "rz-admin-pure",
            "rz-reports-pure",
        ]) {
            expect(outer).toContain(value);
        }
        for (const value of [
            "notification_outbox",
            "notification_receipts",
            "selected-ingress-listener.txt",
            "FAILED STEP",
            "/api/notifications?limit=100",
            "admin-outage-and-backfill",
            "retry-initiator-immutable",
            "restart-recovery",
            "scheduled-silence",
            "current-permission-revocation",
            "dropped-response-duplicate",
            "bad-signature.json",
            "unsigned.json",
            "public-internal.json",
            "pure-absence.json",
            "pure-notification-route.json",
            "sport = :19831",
            "notificationTask:false",
        ]) {
            expect(inner).toContain(value);
        }
        expect(inner).not.toContain("mock");
        for (const value of [
            "useradd --system",
            'setpriv --reuid="$reports_user"',
            "--no-new-privs",
            'prepare_reports_layout "$reports_home"',
            'prepare_reports_layout "$pure_reports_home"',
            'capture_reports_identity "$reports_pid"',
            "reports-runtime-identity.json",
            "preserve_failure_logs",
        ]) {
            expect(inner).toContain(value);
        }
        expect(inner).not.toContain("--no-sandbox");
        expect(evidenceValidator).toContain("valid_reports_identity");
        expect(evidenceValidator).toContain("$runtimeIdentity[0].user.uid > 0");
    });

    test("uses the frozen signature domain without adding an unpinned runtime", () => {
        expect(client).toContain('DOMAIN = "rz-notification-producer-v1"');
        expect(client).toContain("9668d0b7d9dbc9f6f341554fdb1da55b51f35f145dfbd4b98ecfb88e06cf80c0");
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
            env: { ...process.env, RUSTZEN_REPORTS_NOTIFY_TIMEOUT: "0" },
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
            "duplicate-receipt", "wrong-status", "path-traversal", "lifecycle-status",
            "pure-status", "reports-root-process", "reports-dir-owner",
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
            expect(fixture.dockerCalls()).toContain("logs rz-reports-selected-runtime-");
            const failed = join(fixture.evidence, "failed-runs", fixture.failed()[0]);
            expect(existsSync(join(failed, "steps.log"))).toBeTrue();
            expect(existsSync(join(failed, "container.log"))).toBeTrue();
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
            expect(fixture.dockerCalls().match(/rm -f rz-reports-selected-build-/g)?.length).toBe(2);
        } finally {
            fixture.cleanup();
        }
    });
});
