import { describe, expect, test } from "bun:test";
import { stat } from "node:fs/promises";

const outer = await Bun.file(new URL("./verify-analytics-ui-state-linux.sh", import.meta.url)).text();
const inner = await Bun.file(new URL("./verify-analytics-ui-state-linux-inner.sh", import.meta.url)).text();
const fixture = await Bun.file(new URL("./analytics-ui-state-fixture.py", import.meta.url)).text();
const fixturePath = new URL("./analytics-ui-state-fixture.py", import.meta.url).pathname;
const driver = await Bun.file(new URL("./analytics-ui-state-browser-steps.mjs", import.meta.url)).text();
const justfile = await Bun.file(new URL("../justfile", import.meta.url)).text();
const gatePath = new URL("./verify-analytics-ui-state-linux.sh", import.meta.url).pathname;
const runGate = (env) =>
    Bun.spawnSync({ cmd: ["bash", gatePath], env: { ...process.env, ...env }, stdout: "pipe", stderr: "pipe" });
const executableScripts = [
    new URL("./verify-analytics-ui-state-linux.sh", import.meta.url),
    new URL("./verify-analytics-ui-state-linux-inner.sh", import.meta.url),
    new URL("./analytics-ui-state-browser-steps.mjs", import.meta.url),
];

describe("Analytics UI state-matrix Linux gate contract", () => {
    test("uses an independent bounded and source-bound evidence pipeline", () => {
        expect(outer).toContain("RUSTZEN_ANALYTICS_UI_STATE_TIMEOUT");
        expect(outer).toContain("RUSTZEN_ANALYTICS_UI_STATE_TIMEOUT:-900");
        expect(outer).toContain("ensure-admin-browser-verifier-image.sh");
        expect(outer).toContain("admin-browser-source-identity.sh");
        expect(outer).toContain("sourceTreeSha256");
        expect(outer).toContain("failed-runs/$run_id");
        expect(outer).toContain("atomic_replace_symlink");
        expect(outer).toContain("refusing to replace non-link Analytics UI evidence path");
        expect(outer).not.toContain("verify-admin-browser-linux-inner.sh");
    });

    test("fixture intercepts only the two Analytics reads and records exact modes", () => {
        expect(fixture).toContain('"/api/insights/overview", "/api/insights/events"');
        expect(fixture).toContain('{"slow", "success", "empty", "403", "500"}');
        expect(fixture).toContain("__analytics_fixture/receipt");
        expect(fixture).toContain("self.proxy()");
        expect(fixture).toContain("state[\"requests\"].append");
        expect(fixture).not.toContain('state["requests"] = []');
        expect(fixture).toContain('payload.get(f"{route}FailAfterFirstStatus")');
        expect(fixture).toContain('{"403", "500"}');
        expect(fixture).toContain('RUSTZEN_ANALYTICS_FIXTURE_SLOW_SECONDS", "5"');
    });

    test("mode changes accumulate exact success-to-500 and success-to-403 background receipts", async () => {
        const port = 21000 + Math.floor(Math.random() * 1000);
        const fixtureProcess = Bun.spawn(["python3", "-B", fixturePath], {
            env: {
                ...process.env,
                RUSTZEN_ANALYTICS_FIXTURE_PORT: `${port}`,
                RUSTZEN_ANALYTICS_FIXTURE_SLOW_SECONDS: "0.01",
            },
            stdout: "ignore",
            stderr: "pipe",
        });
        const base = `http://127.0.0.1:${port}`;
        try {
            for (let attempt = 0; attempt < 40; attempt += 1) {
                if ((await fetch(`${base}/__analytics_fixture/health`).catch(() => null))?.ok) break;
                await Bun.sleep(25);
            }
            const setMode = (body) =>
                fetch(`${base}/__analytics_fixture/mode`, {
                    method: "PATCH",
                    headers: { "content-type": "application/json" },
                    body: JSON.stringify(body),
                });
            expect((await setMode({ overview: "slow" })).ok).toBe(true);
            expect((await fetch(`${base}/api/insights/overview`)).status).toBe(200);
            expect((await setMode({ events: "empty" })).ok).toBe(true);
            expect((await fetch(`${base}/api/insights/events`)).status).toBe(200);
            expect((await setMode({ events: "success", eventsFailAfterFirstStatus: "500" })).ok).toBe(true);
            expect((await fetch(`${base}/api/insights/events`)).status).toBe(200);
            expect((await fetch(`${base}/api/insights/events`)).status).toBe(500);
            expect((await setMode({ overview: "success", overviewFailAfterFirstStatus: "403" })).ok).toBe(true);
            expect((await fetch(`${base}/api/insights/overview`)).status).toBe(200);
            expect((await fetch(`${base}/api/insights/overview`)).status).toBe(403);
            expect((await setMode({ events: "success", eventsFailAfterFirstStatus: "401" })).status).toBe(400);
            const receipt = await (await fetch(`${base}/__analytics_fixture/receipt`)).json();
            expect(receipt.requests.map((request) => request.mode)).toEqual([
                "slow",
                "empty",
                "success",
                "500",
                "success",
                "403",
            ]);
        } finally {
            fixtureProcess.kill();
            await fixtureProcess.exited;
        }
    });

    test("browser matrix covers loading, empty, permission, error, and the two fixed viewports", () => {
        expect(driver).toContain("overviewLoading");
        expect(driver).toContain("detailsEmpty");
        expect(driver).toContain("overview403");
        expect(driver).toContain("details500");
        expect(driver).toContain("detailsBackgroundRefresh");
        expect(driver).toContain("overviewBackground403");
        expect(driver).toContain("detailsBackground403");
        expect(driver).toContain("detailsFilterResetsPage");
        expect(driver).toContain(".ant-pagination-item-2");
        expect(driver).toContain("/fixture-filter");
        const pauseDurations = [...driver.matchAll(/action: "pause", durationMs: ([0-9_]+)/g)].map(
            ([, value]) => Number(value.replaceAll("_", "")),
        );
        expect(pauseDurations.length).toBeGreaterThan(0);
        expect(pauseDurations.every((duration) => duration <= 30_000)).toBe(true);
        const backgroundStart = driver.indexOf("detailsBackgroundRefresh: desktop([");
        const backgroundEnd = driver.indexOf("\n    ]),", backgroundStart);
        const backgroundSteps = driver.slice(backgroundStart, backgroundEnd);
        expect(backgroundSteps).toContain('{ action: "pause", durationMs: 30_000 }');
        expect(backgroundSteps).toContain('{ action: "waitFor", selector: "[role=alert]" }');
        expect(backgroundSteps).toContain('{ action: "pause", durationMs: 300 }');
        expect(backgroundSteps.indexOf('durationMs: 30_000')).toBeLessThan(
            backgroundSteps.indexOf('selector: "[role=alert]"'),
        );
        expect(backgroundSteps.indexOf('selector: "[role=alert]"')).toBeLessThan(
            backgroundSteps.indexOf('durationMs: 300'),
        );
        expect(driver).toContain("assertNoHorizontalOverflow");
        expect(driver).toContain("width: 1440, height: 900");
        expect(driver).toContain("width: 390, height: 844");
        for (const name of ["overview403", "details403", "overview500", "details500"]) {
            const start = driver.indexOf(`${name}: desktop([`);
            const end = driver.indexOf("\n    ]),", start + name.length + 1);
            expect(driver.slice(start, end)).toContain(
                '{ action: "waitFor", selector: "[role=alert]" }',
            );
        }
        expect(inner).toContain("fixtureReceipt:$receipt");
        expect(inner).toContain("overview-loading");
        expect(inner).toContain("details-empty");
        expect(inner).toContain("overview-403");
        expect(inner).toContain("details-500");
        expect(inner).toContain("details-background-refresh");
        expect(inner).toContain("overview-background-403");
        expect(inner).toContain("details-background-403");
        expect(inner).toContain("refresh_auth");
        expect(inner.indexOf("refresh_auth\n  body=")).toBeGreaterThan(-1);
        expect(inner.indexOf("refresh_auth\n  listing=")).toBeGreaterThan(-1);
        expect(inner).toContain('test -s "$tmp"');
        expect(inner).toContain("details-filter-resets-page");
        expect(inner).toContain('.query.current[0] == "2"');
        expect(inner).toContain("eventsFailAfterFirstStatus");
        expect(inner).toContain("overviewFailAfterFirstStatus");
        expect(inner).toContain(']) as $overview | ([.requests[] | select(.route == "/api/insights/events") | .mode]) as $events |');
        expect(inner).toContain('any(range(0; ($overview | length) - 1); . as $i | $overview[$i] == "success" and $overview[$i + 1] == "403")');
        expect(inner).toContain('select(.route == "/api/insights/events" and (.mode == "empty" or .mode == "403" or .mode == "500"))');
        expect(inner).toContain("fixture-receipt.json");
        expect(inner).toContain("fixture-receipt-on-failure.json");
        expect(inner).toContain("failure-run-artifacts.json");
    });

    test("cleans after-lock signals, stages immutable binaries, and rejects incomplete evidence", () => {
        expect(runGate({ RUSTZEN_ANALYTICS_UI_STATE_TEST_TIMEOUT: "1" }).exitCode).toBe(0);
        for (const [signal, exitCode] of [["INT", 130], ["TERM", 143]]) {
            const root = `/tmp/rz-analytics-ui-state-signal-${crypto.randomUUID()}`;
            const result = runGate({
                RUSTZEN_ANALYTICS_UI_STATE_TEST_SIGNAL: signal,
                RUSTZEN_ANALYTICS_UI_STATE_TEST_ROOT: root,
            });
            expect(result.exitCode).toBe(exitCode);
            expect(new TextDecoder().decode(result.stderr)).toContain(`${signal} signal seam passed`);
            Bun.spawnSync(["rm", "-rf", root]);
        }
        for (const phase of ["before-move", "after-move", "after-link", "replace", "after-replace"]) {
            expect(runGate({ RUSTZEN_ANALYTICS_UI_STATE_TEST_PUBLISH_FAILURE: phase }).exitCode).toBe(0);
        }
        for (const [signal, exitCode] of [["INT", 130], ["TERM", 143]]) {
            const result = runGate({ RUSTZEN_ANALYTICS_UI_STATE_TEST_AFTER_REPLACE_SIGNAL: signal });
            expect(result.exitCode).toBe(exitCode);
            expect(new TextDecoder().decode(result.stderr)).toContain("after-replace signal seam passed");
        }
        for (const [signal, exitCode] of [["INT", 130], ["TERM", 143]]) {
            const innerResult = Bun.spawnSync({
                cmd: ["bash", new URL("./verify-analytics-ui-state-linux-inner.sh", import.meta.url).pathname],
                env: { ...process.env, RUSTZEN_ANALYTICS_UI_STATE_INNER_TEST_STUBBORN: signal },
                stdout: "pipe",
                stderr: "pipe",
            });
            expect(innerResult.exitCode).toBe(exitCode);
            expect(new TextDecoder().decode(innerResult.stderr)).toContain("stubborn cleanup seam passed");
        }
        expect(runGate({ RUSTZEN_ANALYTICS_UI_STATE_TEST_STAGING_REPLACE: "1" }).exitCode).toBe(0);
        expect(runGate({ RUSTZEN_ANALYTICS_UI_STATE_TEST_ARTIFACT_TAMPER: "1" }).exitCode).toBe(0);
        expect(runGate({ RUSTZEN_ANALYTICS_UI_STATE_TEST_STEP_RECEIPT_TAMPER: "1" }).exitCode).toBe(0);
    });

    test("executes the outer manifest and expected-step validator", async () => {
        const root = `/tmp/rz-analytics-ui-state-manifest-${crypto.randomUUID()}`;
        const names = [
            "overviewLoading",
            "detailsEmpty",
            "overview403",
            "details403",
            "overview500",
            "details500",
            "detailsFilterResetsPage",
            "detailsBackgroundRefresh",
            "overviewBackground403",
            "detailsBackground403",
        ];
        const expected = Object.fromEntries(names.map((name) => [name, [{ action: "goto" }]]));
        const manifest = {
            schemaVersion: 1,
            status: "passed",
            gitHead: "head",
            sourceTreeState: "dirty",
            sourceTreeSha256: "sha",
            platform: "linux/arm64",
            runs: Object.fromEntries(names.map((name) => [name, `${name}-run`])),
            runSteps: Object.fromEntries(
                names.map((name) => [name, [{ action: "goto", status: "succeeded", message: null }]]),
            ),
        };
        Bun.spawnSync(["mkdir", "-p", root]);
        await Bun.write(`${root}/manifest.json`, JSON.stringify(manifest));
        await Bun.write(`${root}/browser-steps.json`, JSON.stringify(expected));
        const env = {
            RUSTZEN_ANALYTICS_UI_STATE_TEST_MANIFEST: `${root}/manifest.json`,
            RUSTZEN_ANALYTICS_UI_STATE_TEST_BROWSER_STEPS: `${root}/browser-steps.json`,
        };
        expect(runGate(env).exitCode).toBe(0);
        manifest.runSteps.detailsBackground403[0].status = "failed";
        await Bun.write(`${root}/manifest.json`, JSON.stringify(manifest));
        expect(runGate(env).exitCode).not.toBe(0);
        Bun.spawnSync(["rm", "-rf", root]);
    });

    test("is reachable from the project's local verification entry point", () => {
        expect(justfile).toContain("verify-analytics-ui-state-linux:");
        expect(justfile).toContain("verify-analytics-ui-state-linux.test.mjs");
    });

    test("keeps every invoked gate script executable", async () => {
        for (const script of executableScripts) {
            expect((await stat(script)).mode & 0o111).not.toBe(0);
        }
    });
});
