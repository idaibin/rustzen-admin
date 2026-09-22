import { afterEach, describe, expect, test } from "bun:test";
import { stat, unlink } from "node:fs/promises";

const url = (name) => new URL("./" + name, import.meta.url);
const path = (name) => url(name).pathname;
const read = (name) => Bun.file(url(name)).text();
const driverPath = path("monitoring-ui-state-browser-steps.mjs");
const fixturePath = path("monitoring-ui-state-fixture.py");
const gatePath = path("verify-monitoring-ui-state-linux.sh");
const innerPath = path("verify-monitoring-ui-state-linux-inner.sh");
const innerLibPath = path("monitoring-ui-state-inner-lib.sh");
const evidenceLibPath = path("monitoring-ui-state-evidence-lib.sh");
const deliveryEvidenceLibPath = path("monitoring-ui-state-delivery-evidence-lib.sh");
const names = [
    "driver",
    "fixture",
    "gate",
    "gateLib",
    "evidenceLib",
    "seams",
    "inner",
    "innerLib",
    "retryLib",
    "deliveryLib",
    "deliveryEvidenceLib",
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
    "monitoring-ui-state-retry-lib.sh",
    "monitoring-ui-state-delivery-lib.sh",
    "monitoring-ui-state-delivery-evidence-lib.sh",
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
        const errorAlert = "[role=alert]:not([data-testid^=notification-delivery])";
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
            for (const suffix of ["Background403", "Background500"]) {
                const pauses = steps[route + suffix]
                    .filter((step) => step.action === "pause")
                    .map((step) => step.durationMs);
                expect(pauses).toEqual([15_000, 16_000]);
                expect(pauses.every((duration) => duration <= 30_000)).toBe(true);
                expect(pauses.reduce((total, duration) => total + duration, 0)).toBeGreaterThan(30_000);
            }
            for (const suffix of ["403", "500", "Background403", "Background500"]) {
                const errorSteps = steps[route + suffix].filter((step) =>
                    step.action === "waitFor" || step.action === "assertText",
                ).filter((step) => step.selector?.includes("[role=alert]"));
                expect(errorSteps).toHaveLength(2);
                expect(errorSteps.every((step) => step.selector === errorAlert)).toBe(true);
                expect(errorSteps.every((step) => step.selector.includes(":not([data-testid^=notification-delivery])"))).toBe(true);
            }
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
    }, 35_000);

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
            retryReceipts: { file: "retry-receipts.json", sha256: "9".repeat(64), bytes: 3 },
            artifacts: [
                {}, {},
                { case: "monitor-delivery-owner", file: "monitor-delivery-owner-desktop-dark-en.png",
                  sha256: "f".repeat(64), bytes: 1, dimensions: "1440 x 900" },
                { case: "monitor-delivery-viewer", file: "monitor-delivery-viewer-mobile-light-zh.png",
                  sha256: "0".repeat(64), bytes: 1, dimensions: "390 x 844" },
            ],
            deliveryHealth: {
                gapTotal: 15,
                ownerRun: "owner-run",
                viewerRun: "viewer-run",
                ownerApi: { file: "monitor-delivery-owner.json", sha256: "a".repeat(64), bytes: 1 },
                viewerApi: { file: "monitor-delivery-viewer.json", sha256: "b".repeat(64), bytes: 1 },
                database: { file: "monitor-delivery-db.json", sha256: "c".repeat(64), bytes: 1 },
                ownerSteps: { file: "delivery-monitor-delivery-owner-steps.json", sha256: "d".repeat(64), bytes: 1 },
                viewerSteps: { file: "delivery-monitor-delivery-viewer-steps.json", sha256: "e".repeat(64), bytes: 1 },
                ownerFlow: { file: "delivery-monitor-delivery-owner-flow.json", sha256: "1".repeat(64), bytes: 1 },
                viewerFlow: { file: "delivery-monitor-delivery-viewer-flow.json", sha256: "2".repeat(64), bytes: 1 },
                ownerRunReceipt: { file: "delivery-monitor-delivery-owner-run.json", sha256: "3".repeat(64), bytes: 1 },
                viewerRunReceipt: { file: "delivery-monitor-delivery-viewer-run.json", sha256: "4".repeat(64), bytes: 1 },
            },
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
        for (const mutate of [
            (value) => { value.deliveryHealth.ownerApi = value.deliveryHealth.viewerApi; },
            (value) => { value.deliveryHealth.ownerSteps = value.deliveryHealth.viewerSteps; },
            (value) => { value.deliveryHealth.ownerRun = value.deliveryHealth.viewerRun; },
            (value) => { value.deliveryHealth.database = value.deliveryHealth.ownerApi; },
            (value) => { value.artifacts[2] = value.artifacts[3]; },
        ]) {
            const tampered = structuredClone(manifest);
            mutate(tampered);
            expect((await verify(tampered)).exitCode).not.toBe(0);
        }
        spawn(["rm", "-rf", root]);
    });

    test("retry receipt permits one public pre-CDP failure and rejects semantic mutation", async () => {
        const root = "/tmp/rz-monitoring-retry-" + crypto.randomUUID();
        spawn(["mkdir", "-p", root]);
        const receiptPath = root + "/retry-receipts.json";
        const manifestPath = root + "/manifest.json";
        const record = {
            case: "overviewLoading", sourceRun: "source", childRun: "child",
            fixtureReads: { before: 7, after: 7 },
            sourceRunApi: { data: { id: "source", flowId: "flow", status: "failed", error: "reports service operation failed" } },
            sourceStepsApi: { data: [{ runId: "source", stepIndex: 0, action: "setUiPreferences", status: "succeeded" }] },
            database: {
                sourceRun: "source", childRun: "child", sourceFlowId: "flow", childFlowId: "flow",
                childRetrySourceRunId: "source", sourceChildCount: 1,
            },
        };
        const descriptor = () => {
            const hash = decode(spawn(["sha256sum", receiptPath]).stdout).split(" ")[0];
            const bytes = Number(decode(spawn(["wc", "-c", receiptPath]).stdout).trim().split(/\s+/)[0]);
            return { file: "retry-receipts.json", sha256: hash, bytes };
        };
        const verify = () => spawn([
            "bash", "-c", '. "$1"; verify_retry_receipts "$2" "$3"',
            "retry-receipt-validator", evidenceLibPath, root, manifestPath,
        ]);
        const write = async (value) => {
            await Bun.write(receiptPath, JSON.stringify(value));
            await Bun.write(manifestPath, JSON.stringify({ runs: { overviewLoading: "child" }, retryReceipts: descriptor() }));
        };
        await write([]);
        expect(verify().exitCode).toBe(0);
        const zeroStep = structuredClone(record);
        zeroStep.sourceStepsApi.data = [];
        await write([zeroStep]);
        expect(verify().exitCode).toBe(0);
        await write([record]);
        expect(verify().exitCode).toBe(0);
        const errors = structuredClone(record);
        errors.sourceRunApi.data.error = "internal detail";
        await write([errors]);
        expect(verify().exitCode).not.toBe(0);
        const fixtureGrowth = structuredClone(record);
        fixtureGrowth.fixtureReads.after += 1;
        await write([fixtureGrowth]);
        expect(verify().exitCode).not.toBe(0);
        const nonStepZero = structuredClone(record);
        nonStepZero.sourceStepsApi.data[0].stepIndex = 1;
        await write([nonStepZero]);
        expect(verify().exitCode).not.toBe(0);
        const wrongAction = structuredClone(record);
        wrongAction.sourceStepsApi.data[0].action = "goto";
        await write([wrongAction]);
        expect(verify().exitCode).not.toBe(0);
        const failedPreference = structuredClone(record);
        failedPreference.sourceStepsApi.data[0].status = "failed";
        await write([failedPreference]);
        expect(verify().exitCode).not.toBe(0);
        const multipleSteps = structuredClone(record);
        multipleSteps.sourceStepsApi.data.push({
            runId: "source", stepIndex: 1, action: "setViewport", status: "succeeded",
        });
        await write([multipleSteps]);
        expect(verify().exitCode).not.toBe(0);
        const missingRunId = structuredClone(record);
        delete missingRunId.sourceStepsApi.data[0].runId;
        await write([missingRunId]);
        expect(verify().exitCode).not.toBe(0);
        const foreignRun = structuredClone(record);
        foreignRun.sourceStepsApi.data[0].runId = "foreign";
        await write([foreignRun]);
        expect(verify().exitCode).not.toBe(0);
        const invalidBinding = structuredClone(record);
        invalidBinding.database.sourceChildCount = 2;
        await write([invalidBinding]);
        expect(verify().exitCode).not.toBe(0);
        await write([record, structuredClone(record)]);
        expect(verify().exitCode).not.toBe(0);
        spawn(["rm", "-rf", root]);
    });

    test("delivery evidence rejects API and action-receipt tampering", async () => {
        const root = "/tmp/rz-monitoring-delivery-" + crypto.randomUUID();
        spawn(["mkdir", "-p", root]);
        const delivery = {
            pendingCount: 2, pendingBytes: 1024, quarantineCount: 3, quarantineBytes: 2048,
            omittedCount: 1, expiredCount: 2, unconfirmedCount: 3, quarantinedCount: 4,
            quarantineEvictedCount: 5, firstGapAt: "2026-09-10T01:02:03Z",
            lastGapAt: "2026-09-10T02:03:04Z", lastSuccessAt: "2026-09-10T03:04:05Z",
        };
        const names = ["monitor-delivery-owner.json", "monitor-delivery-viewer.json"];
        const actions = [
            "delivery-monitor-delivery-owner-steps.json",
            "delivery-monitor-delivery-viewer-steps.json",
        ];
        for (const name of names) await Bun.write(`${root}/${name}`, JSON.stringify({ data: delivery }));
        const steps = (runId, locale, screenshot, text) => ({ data: [
            { action: "setUiPreferences", locale, runId, status: "succeeded" },
            { action: "setViewport", runId, status: "succeeded" },
            { action: "goto", runId, status: "succeeded" }, { action: "waitFor", runId, status: "succeeded" },
            { action: "fill", runId, status: "succeeded" }, { action: "fill", runId, status: "succeeded" },
            { action: "click", runId, status: "succeeded" }, { action: "waitFor", runId, status: "succeeded" },
            { action: "goto", url: "/monitoring/incidents", runId, status: "succeeded" },
            { action: "waitFor", runId, status: "succeeded" },
            { action: "assertText", text: text[0], runId, status: "succeeded" },
            { action: "click", runId, status: "succeeded" },
            { action: "waitFor", runId, status: "succeeded" },
            ...text.slice(1).map((value) => ({ action: "assertText", text: value, runId, status: "succeeded" })),
            ...(locale === "zh-CN" ? [
                { action: "waitFor", runId, status: "succeeded" },
                { action: "assertText", runId, status: "succeeded" },
                ...Array.from({ length: 6 }, () => ({ action: "assertElementLayout", runId, status: "succeeded" })),
            ] : []),
            { action: "assertNoHorizontalOverflow", runId, status: "succeeded" },
            { action: "screenshotViewport", name: screenshot, runId, status: "succeeded" },
        ].map((step, stepIndex) => ({ id: stepIndex + 1, runId, stepIndex, action: step.action, status: "succeeded", durationMs: null, message: null, createdAt: "2026-09-10T00:00:00Z" })) });
        await Bun.write(`${root}/${actions[0]}`, JSON.stringify(steps("owner", "en-US", "monitor-delivery-owner", [
            "15 irreversible delivery gaps", "Pending 2 (1024 B)", "Quarantine 3 (2048 B)",
            "First gap 09/10/2026, 01:02:03 AM", "Last gap 09/10/2026, 02:03:04 AM", "Last success 09/10/2026, 03:04:05 AM",
        ])));
        await Bun.write(`${root}/${actions[1]}`, JSON.stringify(steps("viewer", "zh-CN", "monitor-delivery-viewer", [
            "通知投递存在 15 个不可恢复缺口", "待投递 2（1024 B）", "隔离 3（2048 B）",
            "首个缺口 2026/09/10 01:02:03", "最后缺口 2026/09/10 02:03:04", "最后成功 2026/09/10 03:04:05",
        ])));
        const flow = (actor, id, theme, locale, width, height, text) => ({ data: {
            id, systemId: "system", name: `Monitoring delivery: monitor-delivery-${actor}`,
            createdAt: "2026-09-10T00:00:00Z", updatedAt: "2026-09-10T00:00:00Z", steps: [
                { action: "setUiPreferences", theme, locale }, { action: "setViewport", width, height },
                { action: "goto", url: "/login" }, { action: "waitFor", selector: "#login_username" },
                { action: "fill", selector: "#login_username", value: actor === "owner" ? "owner" : "monitor_incident_viewer" },
                { action: "fill", selector: "#login_password", value: actor === "owner" ? "rustzen@123" : "monitor-incident-viewer-password" },
                { action: "click", selector: "button[type=submit]" }, { action: "waitFor", selector: ".shell-content" },
                { action: "goto", url: "/monitoring/incidents" },
                { action: "waitFor", selector: "[data-testid=notification-delivery-card]" },
                { action: "assertText", selector: "[data-testid=notification-delivery-card]", text: text[0] },
                { action: "click", selector: "[data-testid=notification-delivery-gap]" },
                { action: "waitFor", selector: ".ant-popover:not(.ant-popover-hidden)" },
                ...text.slice(1).map((value) => ({ action: "assertText", selector: ".ant-popover:not(.ant-popover-hidden)", text: value })),
                ...(actor === "viewer" ? [
                    { action: "waitFor", selector: ".ant-table-row" },
                    { action: "assertText", selector: ".ant-table-tbody > tr.ant-table-row .monitoring-incident-primary-column", text: "Fixture incident" },
                    { action: "assertElementLayout", selector: ".ant-table-thead th:not(.ant-table-cell-scrollbar)", elementCount: null, visibleCount: 3, maxHeight: null, withinViewportRight: false, withinViewport: false },
                    { action: "assertElementLayout", selector: ".ant-table-thead .monitoring-incident-detail-column", elementCount: null, visibleCount: 0, maxHeight: null, withinViewportRight: false, withinViewport: false },
                    { action: "assertElementLayout", selector: ".ant-table-thead .monitoring-incident-primary-column", elementCount: null, visibleCount: 1, maxHeight: 64, withinViewportRight: true, withinViewport: false },
                    { action: "assertElementLayout", selector: ".ant-table-tbody > tr.ant-table-row", elementCount: 20, visibleCount: 20, maxHeight: 72, withinViewportRight: true, withinViewport: false },
                    { action: "assertElementLayout", selector: ".ant-table-body > table", elementCount: null, visibleCount: 1, maxHeight: null, withinViewportRight: true, withinViewport: false },
                    { action: "assertElementLayout", selector: "[data-testid=incidents-pagination] .ant-pagination", elementCount: null, visibleCount: 1, maxHeight: null, withinViewportRight: true, withinViewport: false },
                ] : []),
                { action: "assertNoHorizontalOverflow" }, { action: "screenshotViewport", name: `monitor-delivery-${actor}` },
            ],
        } });
        const ownerText = ["15 irreversible delivery gaps", "Pending 2 (1024 B)", "Quarantine 3 (2048 B)", "First gap 09/10/2026, 01:02:03 AM", "Last gap 09/10/2026, 02:03:04 AM", "Last success 09/10/2026, 03:04:05 AM"];
        const viewerText = ["通知投递存在 15 个不可恢复缺口", "待投递 2（1024 B）", "隔离 3（2048 B）", "首个缺口 2026/09/10 01:02:03", "最后缺口 2026/09/10 02:03:04", "最后成功 2026/09/10 03:04:05"];
        await Bun.write(`${root}/delivery-monitor-delivery-owner-flow.json`, JSON.stringify(flow("owner", "owner-flow", "dark", "en-US", 1440, 900, ownerText)));
        await Bun.write(`${root}/delivery-monitor-delivery-viewer-flow.json`, JSON.stringify(flow("viewer", "viewer-flow", "light", "zh-CN", 390, 844, viewerText)));
        const runReceipt = (id, flowId) => ({ data: { id, flowId, status: "succeeded", error: null, createdAt: "2026-09-10T00:00:00Z", startedAt: "2026-09-10T00:00:00Z", finishedAt: "2026-09-10T00:00:01Z" } });
        await Bun.write(`${root}/delivery-monitor-delivery-owner-run.json`, JSON.stringify(runReceipt("owner", "owner-flow")));
        await Bun.write(`${root}/delivery-monitor-delivery-viewer-run.json`, JSON.stringify(runReceipt("viewer", "viewer-flow")));
        const database = Object.fromEntries(Object.entries(delivery).sort());
        await Bun.write(`${root}/monitor-delivery-db.json`, JSON.stringify(database) + "\n");
        const descriptor = (name) => {
            const result = spawn(["sha256sum", `${root}/${name}`]);
            const bytes = Number(decode(spawn(["wc", "-c", `${root}/${name}`]).stdout).trim().split(/\s+/)[0]);
            return { file: name, sha256: decode(result.stdout).split(" ")[0], bytes };
        };
        const health = {
            ownerRun: "owner", viewerRun: "viewer",
            ownerApi: descriptor(names[0]), viewerApi: descriptor(names[1]),
            database: descriptor("monitor-delivery-db.json"),
            ownerSteps: descriptor(actions[0]), viewerSteps: descriptor(actions[1]),
            ownerFlow: descriptor("delivery-monitor-delivery-owner-flow.json"),
            viewerFlow: descriptor("delivery-monitor-delivery-viewer-flow.json"),
            ownerRunReceipt: descriptor("delivery-monitor-delivery-owner-run.json"),
            viewerRunReceipt: descriptor("delivery-monitor-delivery-viewer-run.json"),
        };
        const manifest = `${root}/manifest.json`;
        const verify = () => spawn([
            "bash", "-c", '. "$1"; verify_delivery_health "$2" "$3"',
            "delivery", evidenceLibPath, root, manifest,
        ]);
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).toBe(0);
        const ownerActionSteps = JSON.parse(await Bun.file(`${root}/${actions[0]}`).text());
        ownerActionSteps.data[11].action = "assertText";
        await Bun.write(`${root}/${actions[0]}`, JSON.stringify(ownerActionSteps));
        health.ownerSteps = descriptor(actions[0]);
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        await Bun.write(`${root}/${actions[0]}`, JSON.stringify(steps("owner", "en-US", "monitor-delivery-owner", ownerText)));
        health.ownerSteps = descriptor(actions[0]);
        const artifact = async (caseName, file, dimensions) => {
            await Bun.write(`${root}/${file}`, caseName);
            return { case: caseName, ...descriptor(file), dimensions };
        };
        const artifacts = [
            await artifact("monitor-delivery-owner", "monitor-delivery-owner-desktop-dark-en.png", "1440 x 900"),
            await artifact("monitor-delivery-viewer", "monitor-delivery-viewer-mobile-light-zh.png", "390 x 844"),
        ];
        const verifyArtifacts = () => spawn([
            "bash", "-c", '. "$1"; verify_delivery_artifacts "$2" "$3"',
            "artifacts", evidenceLibPath, root, manifest,
        ]);
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health, artifacts }));
        expect(verifyArtifacts().exitCode).toBe(0);
        artifacts[0].bytes += 1;
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health, artifacts }));
        expect(verifyArtifacts().exitCode).not.toBe(0);
        artifacts[0].bytes -= 1;
        health.ownerApi = { ...descriptor(names[0]), bytes: 1 };
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        health.ownerApi = descriptor(names[0]);
        health.ownerApi = descriptor(names[1]);
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        health.ownerApi = descriptor(names[0]);
        health.ownerSteps = descriptor(actions[1]);
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        health.ownerSteps = descriptor(actions[0]);
        health.ownerRun = "viewer";
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        health.ownerRun = "owner";
        health.database = descriptor(names[0]);
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        health.database = descriptor("monitor-delivery-db.json");
        const ownerFlowPath = `${root}/delivery-monitor-delivery-owner-flow.json`;
        const ownerFlowValue = await Bun.file(ownerFlowPath).text();
        await unlink(ownerFlowPath);
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        await Bun.write(ownerFlowPath, ownerFlowValue);
        health.ownerFlow = descriptor("delivery-monitor-delivery-owner-flow.json");
        const changedViewer = flow("viewer", "viewer-flow", "light", "zh-CN", 390, 844, viewerText);
        changedViewer.data.steps[4].value = "owner";
        await Bun.write(`${root}/delivery-monitor-delivery-viewer-flow.json`, JSON.stringify(changedViewer));
        health.viewerFlow = descriptor("delivery-monitor-delivery-viewer-flow.json");
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        const changedSelector = flow("viewer", "viewer-flow", "light", "zh-CN", 390, 844, viewerText);
        changedSelector.data.steps[10].selector = "body";
        await Bun.write(`${root}/delivery-monitor-delivery-viewer-flow.json`, JSON.stringify(changedSelector));
        health.viewerFlow = descriptor("delivery-monitor-delivery-viewer-flow.json");
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        await Bun.write(`${root}/delivery-monitor-delivery-viewer-flow.json`, JSON.stringify(flow("viewer", "viewer-flow", "light", "zh-CN", 390, 844, viewerText)));
        health.viewerFlow = descriptor("delivery-monitor-delivery-viewer-flow.json");
        const headerCapableSelector = flow("viewer", "viewer-flow", "light", "zh-CN", 390, 844, viewerText);
        headerCapableSelector.data.steps[19].selector = ".monitoring-incident-primary-column";
        await Bun.write(`${root}/delivery-monitor-delivery-viewer-flow.json`, JSON.stringify(headerCapableSelector));
        health.viewerFlow = descriptor("delivery-monitor-delivery-viewer-flow.json");
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        await Bun.write(`${root}/delivery-monitor-delivery-viewer-flow.json`, JSON.stringify(flow("viewer", "viewer-flow", "light", "zh-CN", 390, 844, viewerText)));
        health.viewerFlow = descriptor("delivery-monitor-delivery-viewer-flow.json");
        const missingDefault = flow("viewer", "viewer-flow", "light", "zh-CN", 390, 844, viewerText);
        delete missingDefault.data.steps[20].elementCount;
        await Bun.write(`${root}/delivery-monitor-delivery-viewer-flow.json`, JSON.stringify(missingDefault));
        health.viewerFlow = descriptor("delivery-monitor-delivery-viewer-flow.json");
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        const changedDefault = flow("viewer", "viewer-flow", "light", "zh-CN", 390, 844, viewerText);
        changedDefault.data.steps[20].withinViewportRight = true;
        await Bun.write(`${root}/delivery-monitor-delivery-viewer-flow.json`, JSON.stringify(changedDefault));
        health.viewerFlow = descriptor("delivery-monitor-delivery-viewer-flow.json");
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        await Bun.write(`${root}/delivery-monitor-delivery-viewer-flow.json`, JSON.stringify(flow("viewer", "viewer-flow", "light", "zh-CN", 390, 844, viewerText)));
        health.viewerFlow = descriptor("delivery-monitor-delivery-viewer-flow.json");
        for (const [elementCount, visibleCount] of [[19, 20], [20, 1], [1, 1]]) {
            const changedRowCount = flow("viewer", "viewer-flow", "light", "zh-CN", 390, 844, viewerText);
            changedRowCount.data.steps[23].elementCount = elementCount;
            changedRowCount.data.steps[23].visibleCount = visibleCount;
            await Bun.write(`${root}/delivery-monitor-delivery-viewer-flow.json`, JSON.stringify(changedRowCount));
            health.viewerFlow = descriptor("delivery-monitor-delivery-viewer-flow.json");
            await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
            expect(verify().exitCode).not.toBe(0);
        }
        await Bun.write(`${root}/delivery-monitor-delivery-viewer-flow.json`, JSON.stringify(flow("viewer", "viewer-flow", "light", "zh-CN", 390, 844, viewerText)));
        health.viewerFlow = descriptor("delivery-monitor-delivery-viewer-flow.json");
        health.ownerFlow = descriptor("delivery-monitor-delivery-viewer-flow.json");
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        health.ownerFlow = descriptor("delivery-monitor-delivery-owner-flow.json");
        const changedFlow = flow("owner", "owner-flow", "dark", "en-US", 1440, 900, ownerText);
        changedFlow.data.steps.splice(10, 1);
        await Bun.write(`${root}/delivery-monitor-delivery-owner-flow.json`, JSON.stringify(changedFlow));
        health.ownerFlow = descriptor("delivery-monitor-delivery-owner-flow.json");
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        await Bun.write(`${root}/delivery-monitor-delivery-owner-flow.json`, JSON.stringify(flow("owner", "owner-flow", "dark", "en-US", 1440, 900, ownerText)));
        health.ownerFlow = descriptor("delivery-monitor-delivery-owner-flow.json");
        await Bun.write(`${root}/delivery-monitor-delivery-owner-run.json`, JSON.stringify(runReceipt("owner", "viewer-flow")));
        health.ownerRunReceipt = descriptor("delivery-monitor-delivery-owner-run.json");
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        await Bun.write(`${root}/delivery-monitor-delivery-owner-run.json`, JSON.stringify(runReceipt("owner", "owner-flow")));
        health.ownerRunReceipt = descriptor("delivery-monitor-delivery-owner-run.json");
        const ownerSteps = steps("owner", "en-US", "monitor-delivery-owner", [
            "15 irreversible delivery gaps", "Pending 2 (1024 B)", "Quarantine 3 (2048 B)",
            "First gap 09/10/2026, 01:02:03 AM", "Last gap 09/10/2026, 02:03:04 AM", "Last success 09/10/2026, 03:04:05 AM",
        ]);
        ownerSteps.data.splice(11, 1);
        await Bun.write(`${root}/${actions[0]}`, JSON.stringify(ownerSteps));
        health.ownerSteps = descriptor(actions[0]);
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
        await Bun.write(`${root}/${names[0]}`, JSON.stringify({ data: { ...delivery, pendingCount: 99 } }));
        health.ownerApi = descriptor(names[0]);
        await Bun.write(manifest, JSON.stringify({ deliveryHealth: health }));
        expect(verify().exitCode).not.toBe(0);
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

    test("sets a bounded total budget for the declared Monitoring UI scenario", () => {
        const readBudget = (env = {}) => spawn([
            "bash", "-c", '. "$1"; validate_timeout "$timeout" RUSTZEN_MONITORING_UI_STATE_TIMEOUT 2400; printf "%s" "$timeout"',
            "monitoring-ui-timeout", path("monitoring-ui-state-gate-lib.sh"),
        ], env);
        const defaultBudget = Number(decode(readBudget().stdout));
        const routeRuns = 23;
        const deliveryRuns = 2;
        const observedRunSeconds = 69;
        const declaredWaitSeconds = 8 * 31 + 4 * 5;
        const scenarioBudget = Math.max(
            (routeRuns + deliveryRuns) * observedRunSeconds,
            declaredWaitSeconds,
        );
        expect(defaultBudget).toBeGreaterThanOrEqual(1_800);
        expect(defaultBudget).toBeGreaterThanOrEqual(scenarioBudget);
        expect(defaultBudget).toBeLessThanOrEqual(2_400);
        expect(decode(readBudget({ RUSTZEN_MONITORING_UI_STATE_TIMEOUT: "1800" }).stdout)).toBe("1800");
        expect(decode(readBudget({ RUSTZEN_MONITORING_UI_STATE_TIMEOUT: "2400" }).stdout)).toBe("2400");
        for (const value of ["0", "2401", "invalid"]) {
            expect(readBudget({ RUSTZEN_MONITORING_UI_STATE_TIMEOUT: value }).exitCode).toBe(2);
        }
        const timeout = runGate({ RUSTZEN_MONITORING_UI_STATE_TEST_TIMEOUT: "1" });
        expect(timeout.exitCode).toBe(0);
        expect(decode(timeout.stdout)).toContain("natural timeout tree seam passed");
    });

    test("keeps bounded split orchestration and all required evidence seams", () => {
        expect(source.inner).toContain("refresh_owner_auth || return 1");
        expect(source.inner).toContain("refresh_owner_auth || exit 1");
        expect(source.deliveryLib).toContain("owner_auth=(-H \"authorization: Bearer $token\")");
        expect(source.deliveryLib).toContain("monitor_delivery_create_viewer() {\n    local menu role login\n    refresh_owner_auth || return 1");
        expect(source.deliveryLib).toContain("monitor_delivery_capture() {\n    local monitor_db=/opt/rz/data/db/monitor.db evidence\n    refresh_owner_auth || return 1");
        expect(source.inner).toContain("fixture_reads=$(monitor_fixture_reads)");
        expect(source.inner).toContain("retry_run=$(monitor_retry_prepage_cdp \"$case_name\" \"$run\" \"$fixture_reads\")");
        expect(source.inner).toContain("status=$(monitor_wait_run \"$run\")");
        expect(source.retryLib).toContain("$before == $after and $run.status == \"failed\"");
        expect(source.retryLib).toContain('$run.error == "reports service operation failed"');
        expect(source.retryLib).toContain("$steps | length == 0");
        expect(source.retryLib).toContain("$steps | length == 1 and .[0].stepIndex == 0");
        expect(source.retryLib).toContain("-X POST \"$admin/api/reports/runs/$source_run/retry\"");
        expect(source.retryLib).toContain("SELECT COUNT(*) FROM automation_runs WHERE retry_source_run_id=?");
        expect(source.retryLib).toContain("retry run binding is not one source child on one flow");
        const retryFunction = source.retryLib.slice(source.retryLib.indexOf("monitor_retry_prepage_cdp()"));
        expect(retryFunction.indexOf("child=$(curl_json")).toBeLessThan(
            retryFunction.indexOf("binding=$(monitor_retry_db_binding"),
        );
        expect(retryFunction.indexOf("binding=$(monitor_retry_db_binding")).toBeLessThan(
            retryFunction.indexOf("monitor_retry_append_receipt"),
        );
        expect(source.inner).toContain("monitor_retry_receipts=/verify/evidence/retry-receipts.json");
        expect(source.inner).toContain("printf '[]\\n' >\"$monitor_retry_receipts\"");
        expect(source.evidenceLib).toContain("verify_retry_receipts");
        expect(source.evidenceLib).toContain("reports service operation failed");
        expect(source.evidenceLib).toContain("sourceChildCount:1");
        expect(source.inner).toContain("monitor_retry_budget=0");
        expect(source.inner).toContain("[ \"$monitor_retry_budget\" -eq 0 ]");
        expect(source.inner).toContain("monitor_retry_budget=1");
        expect(source.gate).toContain("monitoring-ui-state-gate-lib.sh");
        expect(source.gate).toContain("monitoring-ui-state-evidence-lib.sh");
        expect(source.gate).toContain("monitoring-ui-state-seams.sh");
        expect(source.gate).toContain("monitoring-ui-state-inner-lib.sh");
        expect(source.gate).toContain("monitoring-ui-state-retry-lib.sh");
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
        expect(source.inner).toContain("monitor_delivery_capture");
        expect(source.deliveryLib).toContain("monitor:incident:view");
        expect(source.deliveryLib).toContain("monitor_delivery_prepare");
        expect(source.deliveryLib).toContain("assertNoHorizontalOverflow");
        expect(source.innerLib).toContain("verification port occupied");
        expect(source.innerLib).toContain("did not become healthy");
        expect(source.innerLib).toContain("tail -n 60");
        expect(source.fixture).toContain('"/api/monitor/daily-summaries": "summaries"');
        expect(justfile).toContain("verify-monitoring-ui-state-linux:");
        expect(guide).toContain("target/rz/monitoring-ui-state/current/manifest.json");
    });

    test("keeps every script readable, bounded, and executable", async () => {
        for (const name of files) {
            const lines = (await read(name)).split("\n");
            expect(lines.length - 1).toBeLessThanOrEqual(300);
            expect(Math.max(...lines.map((line) => line.length))).toBeLessThanOrEqual(120);
            expect((await stat(url(name))).mode & 0o111).not.toBe(0);
        }
    });
});
