import { afterEach, describe, expect, test } from "bun:test";
import { stat } from "node:fs/promises";

const url = (name) => new URL("./" + name, import.meta.url);
const path = (name) => url(name).pathname;
const read = (name) => Bun.file(url(name)).text();
const driverPath = path("monitoring-ui-state-browser-steps.mjs");
const fixturePath = path("monitoring-ui-state-fixture.py");
const gatePath = path("verify-monitoring-ui-state-linux.sh");
const innerPath = path("verify-monitoring-ui-state-linux-inner.sh");
const innerLibPath = path("monitoring-ui-state-inner-lib.sh");
const evidenceLibPath = path("monitoring-ui-state-evidence-lib.sh");
const names = [
    "driver",
    "fixture",
    "gate",
    "gateLib",
    "evidenceLib",
    "seams",
    "inner",
    "innerLib",
];
const files = [
    "monitoring-ui-state-browser-steps.mjs",
    "monitoring-ui-state-fixture.py",
    "verify-monitoring-ui-state-linux.sh",
    "monitoring-ui-state-gate-lib.sh",
    "monitoring-ui-state-evidence-lib.sh",
    "monitoring-ui-state-seams.sh",
    "verify-monitoring-ui-state-linux-inner.sh",
    "monitoring-ui-state-inner-lib.sh",
];
const values = await Promise.all(files.map(read));
const source = Object.fromEntries(names.map((name, index) => [name, values[index]]));
const justfile = await Bun.file(new URL("../justfile", import.meta.url)).text();
const guide = await Bun.file(
    new URL("../docs/guides/local-verification.md", import.meta.url),
).text();
const processes = [];
const decode = (value) => new TextDecoder().decode(value);
const spawn = (command, env = {}) =>
    Bun.spawnSync({
        cmd: command,
        env: { ...process.env, ...env },
        stdout: "pipe",
        stderr: "pipe",
    });
const runGate = (env) => spawn(["bash", gatePath], env);
const runInner = (signal) =>
    spawn(["bash", innerPath], {
        RUSTZEN_MONITORING_UI_STATE_INNER_LIB: innerLibPath,
        RUSTZEN_MONITORING_UI_STATE_INNER_TEST_STUBBORN: signal,
    });
const generateSteps = () => {
    const result = spawn([process.execPath, driverPath]);
    expect(result.exitCode).toBe(0);
    return JSON.parse(decode(result.stdout));
};

afterEach(async () => {
    await Promise.all(
        processes.splice(0).map(async (handle) => {
            handle.kill();
            await handle.exited;
        }),
    );
});

describe("Monitoring UI state Linux gate", () => {
    test("driver emits exactly 23 route-specific runs and two screenshots", () => {
        const steps = generateSteps();
        expect(Object.keys(steps)).toHaveLength(23);
        for (const route of ["overview", "nodes", "incidents", "summaries"]) {
            for (const suffix of [
                "Loading",
                "403",
                "500",
                "Background403",
                "Background500",
            ]) {
                expect(steps[route + suffix]).toBeArray();
            }
            const denied = steps[route + "Background403"];
            expect(denied.some((step) => step.action === "assertAbsent")).toBe(true);
            expect(
                denied.some(
                    (step) =>
                        step.action === "assertText" &&
                        step.text.includes("You do not have permission"),
                ),
            ).toBe(true);
            const failed = steps[route + "Background500"];
            expect(
                failed.some(
                    (step) =>
                        step.action === "assertText" &&
                        step.text.includes("last successfully loaded data remains visible"),
                ),
            ).toBe(true);
            expect(failed.filter((step) => step.action === "assertText").length).toBeGreaterThan(1);
        }
        expect(steps.overviewLoading).toContainEqual({
            action: "screenshotViewport",
            name: "monitoring-overview-desktop-dark-en",
        });
        expect(steps.summariesPaging).toContainEqual({
            action: "screenshotViewport",
            name: "monitoring-summaries-mobile-light-zh",
        });
        expect(steps.overviewLoading).toContainEqual({
            action: "setViewport",
            width: 1440,
            height: 900,
        });
        expect(steps.summariesPaging).toContainEqual({
            action: "setViewport",
            width: 390,
            height: 844,
        });
    });

    test("fixture executes all transitions and strict receipt validation", async () => {
        const port = 21000 + Math.floor(Math.random() * 1000);
        const handle = Bun.spawn(["python3", "-B", fixturePath], {
            env: {
                ...process.env,
                RUSTZEN_MONITORING_FIXTURE_PORT: String(port),
            },
            stdout: "ignore",
            stderr: "pipe",
        });
        processes.push(handle);
        const base = "http://127.0.0.1:" + port;
        for (let attempt = 0; attempt < 80; attempt += 1) {
            const health = await fetch(base + "/__monitoring_fixture/health").catch(() => null);
            if (health?.ok) break;
            await Bun.sleep(25);
        }
        expect((await fetch(base + "/__monitoring_fixture/health")).ok).toBe(true);
        const patch = (body) =>
            fetch(base + "/__monitoring_fixture/mode", {
                method: "PATCH",
                headers: { "content-type": "application/json" },
                body: JSON.stringify(body),
            });
        const endpoints = {
            overview: "/api/monitor/overview",
            nodes: "/api/monitor/nodes",
            incidents: "/api/monitor/incidents?current=1&pageSize=20",
            summaries: "/api/monitor/daily-summaries?current=1&pageSize=20",
        };
        for (const [name, endpoint] of Object.entries(endpoints)) {
            expect((await patch({ [name]: "slow" })).ok).toBe(true);
            expect((await fetch(base + endpoint)).status).toBe(200);
            expect((await patch({ [name]: "403" })).ok).toBe(true);
            expect((await fetch(base + endpoint)).status).toBe(403);
            expect((await patch({ [name]: "500" })).ok).toBe(true);
            expect((await fetch(base + endpoint)).status).toBe(500);
            const after = name + "FailAfterFirstStatus";
            expect((await patch({ [name]: "success", [after]: "403" })).ok).toBe(true);
            expect((await fetch(base + endpoint)).status).toBe(200);
            expect((await fetch(base + endpoint)).status).toBe(403);
            expect((await patch({ [name]: "success", [after]: "500" })).ok).toBe(true);
            expect((await fetch(base + endpoint)).status).toBe(200);
            expect((await fetch(base + endpoint)).status).toBe(500);
        }
        expect((await patch({ incidents: "success", summaries: "success" })).ok).toBe(true);
        const incident = base + "/api/monitor/incidents?";
        await fetch(incident + "current=2&pageSize=20");
        await fetch(incident + "current=2&pageSize=20");
        await fetch(incident + "current=1&pageSize=20&status=active");
        await fetch(incident + "current=1&pageSize=20&status=active&kind=cpuHigh");
        await fetch(base + "/api/monitor/daily-summaries?current=2&pageSize=20");
        expect((await patch({ incidentsAfter: "403" })).status).toBe(400);
        const receipt = await (
            await fetch(base + "/__monitoring_fixture/receipt")
        ).json();
        for (const name of Object.keys(endpoints)) {
            const modes = receipt.requests
                .filter((request) => request.name === name)
                .slice(0, 7)
                .map((request) => request.mode);
            expect(modes).toEqual([
                "slow",
                "403",
                "500",
                "success",
                "403",
                "success",
                "500",
            ]);
        }
        const root = "/tmp/rz-monitoring-fixture-" + crypto.randomUUID();
        spawn(["mkdir", "-p", root]);
        const receiptPath = root + "/receipt.json";
        await Bun.write(receiptPath, JSON.stringify(receipt));
        const validate = () =>
            spawn([
                "bash",
                "-c",
                '. "$1"; verify_fixture_receipt "$2"',
                "fixture-validator",
                evidenceLibPath,
                receiptPath,
            ]);
        expect(validate().exitCode).toBe(0);
        receipt.requests[0].mode = "500";
        await Bun.write(receiptPath, JSON.stringify(receipt));
        expect(validate().exitCode).not.toBe(0);
        spawn(["rm", "-rf", root]);
    }, 15_000);

    test("manifest validator rejects run, step, and inventory tampering", async () => {
        const expected = generateSteps();
        const caseNames = Object.keys(expected);
        const successfulSteps = Object.fromEntries(
            caseNames.map((name) => [
                name,
                expected[name].map((step) => ({
                    action: step.action,
                    status: "succeeded",
                    message: null,
                })),
            ]),
        );
        const manifest = {
            schemaVersion: 1,
            status: "passed",
            runCount: 23,
            gitHead: "head",
            sourceTreeState: "dirty",
            sourceTreeSha256: "sha",
            platform: "linux/arm64",
            chromiumVersion: "pinned",
            runs: Object.fromEntries(caseNames.map((name) => [name, name + "-run"])),
            runSteps: successfulSteps,
            stepReceipts: caseNames.map((name) => ({
                case: name,
                file: name + ".json",
                sha256: "sha",
            })),
            fixtureReceipt: { requests: [] },
            artifacts: [{}, {}],
        };
        const root = "/tmp/rz-monitoring-manifest-" + crypto.randomUUID();
        spawn(["mkdir", "-p", root]);
        const manifestPath = root + "/manifest.json";
        const stepsPath = root + "/steps.json";
        const verify = async (value, expectedSteps = expected) => {
            await Bun.write(manifestPath, JSON.stringify(value));
            await Bun.write(stepsPath, JSON.stringify(expectedSteps));
            return runGate({
                RUSTZEN_MONITORING_UI_STATE_TEST_MANIFEST: manifestPath,
                RUSTZEN_MONITORING_UI_STATE_TEST_BROWSER_STEPS: stepsPath,
            });
        };
        expect((await verify(manifest)).exitCode).toBe(0);
        expect((await verify({ ...manifest, runCount: 22 })).exitCode).not.toBe(0);
        const missing = structuredClone(manifest);
        delete missing.runs.summariesPaging;
        expect((await verify(missing)).exitCode).not.toBe(0);
        const failed = structuredClone(manifest);
        failed.runSteps.overviewBackground403[0].status = "failed";
        expect((await verify(failed)).exitCode).not.toBe(0);
        const expectedMissing = structuredClone(expected);
        delete expectedMissing.nodes500;
        expect((await verify(manifest, expectedMissing)).exitCode).not.toBe(0);
        spawn(["rm", "-rf", root]);
    });

    test("executes timeout, signal, staging, artifact, and publication seams", () => {
        const timeout = runGate({
            RUSTZEN_MONITORING_UI_STATE_TEST_TIMEOUT: "1",
        });
        expect(timeout.exitCode).toBe(0);
        expect(decode(timeout.stdout)).toContain("natural timeout tree seam passed");
        expect(source.gateLib).toContain("return 124");
        for (const [signal, exitCode] of [["INT", 130], ["TERM", 143]]) {
            const root = "/tmp/rz-monitoring-signal-" + crypto.randomUUID();
            const result = runGate({
                RUSTZEN_MONITORING_UI_STATE_TEST_SIGNAL: signal,
                RUSTZEN_MONITORING_UI_STATE_TEST_ROOT: root,
            });
            expect(result.exitCode).toBe(exitCode);
            expect(decode(result.stderr)).toContain(signal + " signal seam passed");
            spawn(["rm", "-rf", root]);
            const committed = runGate({
                RUSTZEN_MONITORING_UI_STATE_TEST_AFTER_REPLACE_SIGNAL: signal,
            });
            expect(committed.exitCode).toBe(exitCode);
            expect(decode(committed.stderr)).toContain("after-replace signal seam passed");
            const innerResult = runInner(signal);
            expect(innerResult.exitCode).toBe(exitCode);
            expect(decode(innerResult.stderr)).toContain("stubborn cleanup seam passed");
        }
        const phases = ["before-move", "after-move", "after-link", "replace", "after-replace"];
        for (const phase of phases) {
            expect(runGate({
                RUSTZEN_MONITORING_UI_STATE_TEST_PUBLISH_FAILURE: phase,
            }).exitCode).toBe(0);
        }
        expect(runGate({
            RUSTZEN_MONITORING_UI_STATE_TEST_STAGING_REPLACE: "1",
        }).exitCode).toBe(0);
        expect(runGate({
            RUSTZEN_MONITORING_UI_STATE_TEST_ARTIFACT_TAMPER: "1",
        }).exitCode).toBe(0);
    }, 20_000);

    test("keeps bounded split orchestration and all required evidence seams", () => {
        expect(source.gate).toContain("monitoring-ui-state-gate-lib.sh");
        expect(source.gate).toContain("monitoring-ui-state-evidence-lib.sh");
        expect(source.gate).toContain("monitoring-ui-state-seams.sh");
        expect(source.gate).toContain("monitoring-ui-state-inner-lib.sh");
        expect(source.gate).toContain("admin-browser-source-identity.sh");
        expect(source.gate).toContain("verify_staged_binaries");
        expect(source.gateLib).toContain("stop_frozen_processes");
        expect(source.gateLib).toContain("atomic_replace_symlink");
        expect(source.gateLib).toContain("outer_cleanup");
        expect(source.gate).toContain("RUSTZEN_MONITORING_UI_STATE_TEST_BLOCKING_CLEANUP");
        expect(source.evidenceLib).toContain(".runCount == 23");
        expect(source.evidenceLib).toContain("verify_evidence_files");
        expect(source.seams).toContain("failed-runs/test/manifest.json");
        expect(source.inner).toContain("run_route_matrix overview overview");
        expect(source.inner).toContain("run_case incidentsPaging incidents-paging");
        expect(source.inner).toContain("run_case incidentsFilters incidents-filters");
        expect(source.inner).toContain("run_case summariesPaging summaries-paging");
        expect(source.innerLib).toContain("verification port occupied");
        expect(source.innerLib).toContain("did not become healthy");
        expect(source.innerLib).toContain("tail -n 60");
        expect(source.fixture).toContain('"/api/monitor/daily-summaries": "summaries"');
        expect(justfile).toContain("verify-monitoring-ui-state-linux:");
        expect(guide).toContain("A 23-run Linux Chromium gate candidate");
    });

    test("keeps every script readable, bounded, and executable", async () => {
        for (const name of files) {
            const lines = (await read(name)).split("\n");
            expect(lines.length - 1).toBeLessThanOrEqual(350);
            expect(Math.max(...lines.map((line) => line.length))).toBeLessThanOrEqual(120);
            expect((await stat(url(name))).mode & 0o111).not.toBe(0);
        }
    });
});
