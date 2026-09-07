import { afterEach, describe, expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { fixture, receipts, run, signal } from "./verify-admin-session-authority-linux.test-support.mjs";

const scripts = dirname(fileURLToPath(import.meta.url));
const outer = join(scripts, "verify-admin-session-authority-linux.sh");
const inner = join(scripts, "verify-admin-session-authority-linux-inner.sh");
const validator = join(scripts, "verify-admin-session-authority-evidence.jq");
const verifierHelper = join(scripts, "ensure-admin-browser-verifier-image.sh");
const fixtures = [];
afterEach(() => { while (fixtures.length) fixtures.pop().cleanup(); });
const make = (options) => { const item = fixture(outer, options); fixtures.push(item); return item; };

describe("Admin session authority Linux gate", () => {
    test("source fixes the real runtime cases and bounded publication contract", () => {
        const outerText = readFileSync(outer, "utf8");
        const innerText = readFileSync(inner, "utf8");
        const jqText = readFileSync(validator, "utf8");
        const helperText = readFileSync(verifierHelper, "utf8");
        for (const value of [
            "sid-a-after-logout.json", "sid-b-after-logout.json", "revoke-all.json",
            "password-old.json", "disable-old.json", "grant-removed.json", "grant-restored.json",
            "bad-jwt.json", "expired-jwt.json", "authority-db-failure.json",
        ]) expect(innerText).toContain(value);
        for (const value of ["run_helper_bounded", "RUSTZEN_ADMIN_SESSION_HELPER_TIMEOUT", "failed-runs", "atomic_replace_symlink", "remove_container", "source tree changed"]) expect(outerText).toContain(value);
        expect(jqText).toContain("($allowlist | sort)");
        expect(jqText).toContain("authorityFailure;401;401");
        expect(helperText).toContain("trap cleanup EXIT");
        expect(helperText).toContain("trap 'exit 130' INT");
        expect(helperText).toContain("cleanup_probe best-effort");
        expect(innerText.split("\n").length).toBeLessThan(300);
        expect(outerText.split("\n").length).toBeLessThan(300);
    });

    test("valid evidence publishes atomically after both owned containers are absent", () => {
        const item = make();
        const result = run(item);
        expect(result.exitCode).toBe(0);
        expect(item.current()).toMatch(/^runs\//);
        expect(item.active()).toEqual([]);
        expect(item.callsText()).toContain("container inspect rz-admin-session-build-");
        expect(item.callsText()).toContain("container inspect rz-admin-session-runtime-");
        const manifest = JSON.parse(readFileSync(join(item.evidence, item.current(), "manifest.json"), "utf8"));
        expect(manifest.receipts.map(({ file }) => file).sort()).toEqual([...receipts].sort());
    });

    for (const tamper of ["status", "state", "remove-epoch", "restore-epoch", "user-epoch", "expiry-session", "duplicate", "path", "expiry"]) {
        test(`outer validator rejects ${tamper} tamper and preserves current`, () => {
            const item = make({ tamper });
            const result = run(item);
            expect(result.exitCode).not.toBe(0);
            expect(item.current()).toBe("runs/previous");
            expect(item.failed()).toHaveLength(1);
            expect(item.active()).toEqual([]);
        });
    }

    test("runtime failure retains bounded logs without replacing current", () => {
        const item = make({ runtimeExit: "7" });
        const result = run(item);
        expect(result.exitCode).toBe(7);
        expect(item.current()).toBe("runs/previous");
        expect(item.failed()).toHaveLength(1);
        expect(existsSync(join(item.evidence, "failed-runs", item.failed()[0], "runtime-container.log"))).toBe(true);
        expect(item.active()).toEqual([]);
    });

    test("build failure retains nonempty phase and container diagnostics", () => {
        const item = make({ buildExit: "8" });
        const result = run(item);
        expect(result.exitCode).toBe(8);
        const failed = join(item.evidence, "failed-runs", item.failed()[0]);
        expect(readFileSync(join(failed, "phase-status.tsv"), "utf8")).toContain("build\t8");
        expect(readFileSync(join(failed, "build.log"), "utf8").length).toBeGreaterThan(0);
        expect(readFileSync(join(failed, "build-container.log"), "utf8").length).toBeGreaterThan(0);
        expect(item.current()).toBe("runs/previous");
        expect(item.active()).toEqual([]);
    });

    test("build timeout retains diagnostics, returns 124 and cleans the container", () => {
        const item = make({ block: "build", timeout: "1" });
        const result = run(item);
        expect(result.exitCode).toBe(124);
        const failed = join(item.evidence, "failed-runs", item.failed()[0]);
        expect(readFileSync(join(failed, "phase-status.tsv"), "utf8")).toContain("build\t124");
        expect(readFileSync(join(failed, "build-container.log"), "utf8").length).toBeGreaterThan(0);
        expect(item.current()).toBe("runs/previous");
        expect(item.active()).toEqual([]);
    });

    test("verifier helper failure retains a nonempty diagnostic", () => {
        const item = make({ helperFail: true });
        const result = run(item);
        expect(result.exitCode).toBe(9);
        const failed = join(item.evidence, "failed-runs", item.failed()[0]);
        expect(readFileSync(join(failed, "phase-status.tsv"), "utf8")).toContain("verifier-helper\t9");
        expect(readFileSync(join(failed, "verifier-helper.log"), "utf8")).toContain("fake verifier helper failure");
        expect(item.current()).toBe("runs/previous");
        expect(item.active()).toEqual([]);
    });

    test("helper lifecycle beyond the old 60 second bound uses the configured complete bound", () => {
        const item = make({ helperTimeout: "120", helperDelay: "0.2", helperLogicalSeconds: "61" });
        const result = run(item);
        expect(result.exitCode).toBe(0);
        expect(item.callsText()).toContain("helper logical seconds 61");
        expect(item.current()).toMatch(/^runs\//);
        expect(item.active()).toEqual([]);
    });

    test("helper timeout returns 124, retains its phase log and cleans every owned resource", () => {
        const item = make({ helperTimeout: "1", helperDelay: "30" });
        const result = run(item);
        expect(result.exitCode).toBe(124);
        const failed = join(item.evidence, "failed-runs", item.failed()[0]);
        expect(readFileSync(join(failed, "phase-status.tsv"), "utf8")).toContain("verifier-helper\t124");
        expect(readFileSync(join(failed, "verifier-helper.log"), "utf8")).toContain("fake verifier helper started");
        expect(item.current()).toBe("runs/previous");
        expect(item.active()).toEqual([]);
    });

    for (const [name, code] of [["SIGINT", 130], ["SIGTERM", 143]]) {
        test(`helper ${name} returns ${code}, retains diagnostics and removes its probe`, async () => {
            const item = make({ helperTimeout: "30", helperDelay: "30" });
            expect(await signal(item, name, "verifier-probe")).toBe(code);
            const failed = join(item.evidence, "failed-runs", item.failed()[0]);
            expect(readFileSync(join(failed, "phase-status.tsv"), "utf8")).toContain(`verifier-helper\t${code}`);
            expect(readFileSync(join(failed, "verifier-helper.log"), "utf8")).toContain("fake verifier helper started");
            expect(item.current()).toBe("runs/previous");
            expect(item.active()).toEqual([]);
        });
    }

    test("container removal failure cannot publish current", () => {
        const item = make({ rmFail: true });
        const result = run(item);
        expect(result.exitCode).not.toBe(0);
        expect(item.current()).toBe("runs/previous");
        expect(item.failed()).toHaveLength(1);
    });

    test("bounded runtime timeout returns 124 and releases owned resources", () => {
        const item = make({ block: "runtime", timeout: "1" });
        const result = run(item);
        expect(result.exitCode).toBe(124);
        expect(item.current()).toBe("runs/previous");
        expect(item.failed()).toHaveLength(1);
        expect(item.active()).toEqual([]);
    });

    for (const [name, code] of [["SIGINT", 130], ["SIGTERM", 143]]) {
        test(`${name} returns ${code}, retains failure evidence and cleans the child`, async () => {
            const item = make({ block: "runtime", timeout: "10" });
            expect(await signal(item, name)).toBe(code);
            expect(item.current()).toBe("runs/previous");
            expect(item.failed()).toHaveLength(1);
            expect(item.active()).toEqual([]);
        });
    }
});
