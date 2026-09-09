import { expect, test } from "bun:test";

const root = new URL("..", import.meta.url).pathname;
const read = (path) => Bun.file(new URL(path, import.meta.url)).text();
const cases = ["managerProcessing", "managerRuntimeFailure", "managerPartial", "viewOnlyMobile"];
const screenshotCases = {
    managerProcessing: ["reports-processing-desktop-dark-en.png", 1440, 900],
    managerRuntimeFailure: ["reports-runtime-failure-desktop-dark-en.png", 1440, 900],
    managerPartial: ["reports-partial-desktop-dark-en.png", 1440, 900],
    viewOnlyMobile: ["reports-view-only-mobile-light-zh.png", 390, 844],
};
const receiptActions = Object.fromEntries(cases.map((name) => {
    const result = Bun.spawnSync(["bash", "-c", '. "$1"; reports_ui_state_expected_actions "$2"', "actions", `${root}/scripts/reports-ui-state-evidence-lib.sh`, name]);
    return [name, JSON.parse(new TextDecoder().decode(result.stdout))];
}));

const sha = async (path) => {
    const result = Bun.spawnSync(["shasum", "-a", "256", path], { stdout: "pipe" });
    return new TextDecoder().decode(result.stdout).split(" ")[0];
};

test("delivery helper exports executable status and database contracts", async () => {
    const helper = new URL("./reports-ui-state-delivery-lib.sh", import.meta.url).pathname;
    const database = `/tmp/rz-delivery-${crypto.randomUUID()}.db`;
    const result = Bun.spawnSync([
        "bash", "-c", `
            . "$1"
            test "$(type -t reports_ui_delivery_expected_jq)" = function
            test "$(type -t reports_ui_delivery_db_json)" = function
            python3 -B - "$2" <<'PY'
import sqlite3, sys
with sqlite3.connect(sys.argv[1]) as c:
    c.execute("CREATE TABLE notification_delivery_status (id INTEGER PRIMARY KEY,pending_count INTEGER,pending_bytes INTEGER,quarantine_count INTEGER,quarantine_bytes INTEGER,omitted_count INTEGER,expired_count INTEGER,unconfirmed_count INTEGER,quarantined_count INTEGER,quarantine_evicted_count INTEGER,first_gap_at TEXT,last_gap_at TEXT,last_success_at TEXT)")
    c.execute("INSERT INTO notification_delivery_status (id) VALUES (1)")
PY
            seed_reports_ui_delivery_status "$2"
            reports_ui_delivery_db_json "$2" | jq -e '.["pendingCount"] == 2 and .["lastSuccessAt"] == "2026-09-10T03:04:05Z"' >/dev/null
            reports_ui_delivery_db_json "$2" | jq -c '{data:.}' | jq -e "$(reports_ui_delivery_expected_jq)" >/dev/null
        `, "delivery-helper", helper, database,
    ], { stdout: "pipe", stderr: "pipe" });
    await Bun.spawn(["rm", "-f", database]).exited;
    expect(result.exitCode, new TextDecoder().decode(result.stderr)).toBe(0);
});

test("every browser case jq step array compiles and parses", async () => {
    const inner = await read("./verify-reports-ui-state-linux-inner.sh");
    const cases = [
        ["processing_steps=", "processing_browser_run="],
        ["failure_steps=", "failure_browser_run="],
        ["partial_steps=", "partial_browser_run="],
        ["viewer_steps=", "viewer_browser_run="],
    ];
    for (const [start, end] of cases) {
        const block = inner.slice(inner.indexOf(start), inner.indexOf(end));
        const filter = block.match(/jq -nc[^\n]* '\n([\s\S]*?)'\)/)?.[1];
        expect(filter, `${start} jq filter`).toBeDefined();
        const result = Bun.spawnSync([
            "jq", "-nc", "--arg", "run", "run", "--arg", "enqueued", "enqueued", "--arg", "skipped", "skipped", filter ?? "error(\"missing filter\")",
        ], { stdout: "pipe", stderr: "pipe" });
        expect(result.exitCode, `${start}: ${new TextDecoder().decode(result.stderr)}`).toBe(0);
        const steps = JSON.parse(new TextDecoder().decode(result.stdout));
        expect(Array.isArray(steps)).toBeTrue();
        expect(steps.length).toBeGreaterThan(0);
    }
});

test("Reports state gate keeps the declared closure bounded and source-bound", async () => {
    const [outer, inner, browser, delivery, evidence, justfile] = await Promise.all([
        read("./verify-reports-ui-state-linux.sh"),
        read("./verify-reports-ui-state-linux-inner.sh"),
        read("./reports-ui-state-browser-lib.sh"),
        read("./reports-ui-state-delivery-lib.sh"),
        read("./reports-ui-state-evidence-lib.sh"),
        Bun.file(`${root}/justfile`).text(),
    ]);

    expect(justfile).toContain("verify-reports-ui-state-linux:");
    expect(justfile).toContain("scripts/reports-ui-state-browser-lib.test.mjs");
    expect(outer).toContain("monitoring-ui-state-gate-lib.sh");
    expect(outer).toContain("verify_reports_ui_state_receipts");
    expect(outer).toContain("verify_reports_ui_state_source_evidence");
    expect(outer).toContain("reports-ui-state-delivery-lib.sh");
    expect(outer).toContain("verify_reports_ui_state_delivery_receipts");
    expect(inner).toContain("RUSTZEN_REPORTS_MAX_CONCURRENCY=2");
    const activeFlow = inner.slice(inner.indexOf("active_steps="), inner.indexOf("active_run="));
    const activeFilter = activeFlow.match(/jq -nc '([^']+)'/)?.[1];
    const activeSteps = Bun.spawnSync(["jq", "-nc", activeFilter ?? "error(\"missing active flow\")"]);
    expect(activeSteps.exitCode).toBe(0);
    const activePauses = JSON.parse(new TextDecoder().decode(activeSteps.stdout))
        .filter((step) => step.action === "pause")
        .map((step) => step.durationMs);
    expect(activePauses).toHaveLength(10);
    expect(activePauses.every((duration) => duration <= 30_000)).toBeTrue();
    expect(inner).toContain("processing-run-steps.json");
    expect(inner).toContain('text:"1. goto"');
    expect(inner).not.toContain('text:"Waiting for step results"');
    expect(inner).not.toContain('text:"2. pause"');
    const processingCase = inner.slice(inner.indexOf("processing_steps="), inner.indexOf("processing_browser_run="));
    const screenshot = processingCase.indexOf('screenshotViewport",name:"reports-processing-desktop-dark-en"');
    const closeAudit = processingCase.indexOf('selector:"button.ant-modal-close"');
    const settleAuditClose = processingCase.indexOf('{action:"pause",durationMs:300}', closeAudit);
    const cancel = processingCase.indexOf('selector:"[data-testid=run-cancel-\\($run)]"');
    expect(screenshot).toBeGreaterThan(-1);
    expect(closeAudit).toBeGreaterThan(screenshot);
    expect(settleAuditClose).toBeGreaterThan(closeAudit);
    expect(cancel).toBeGreaterThan(settleAuditClose);
    expect(processingCase).toContain('selector:"[data-testid=run-cancel-\\($run)][disabled]"');
    expect(inner).not.toContain('"$admin/api/reports/runs/$active_run/cancel"');
    expect(inner).toContain('selector:"tr:has([data-testid=run-retry-list-\\($run)])"');
    expect(inner).not.toContain('selector:".ant-table-row",text:"assertText did not match"');
    const failureCase = inner.slice(inner.indexOf("failure_steps="), inner.indexOf("failure_browser_run="));
    expect(failureCase).toContain('selector:"[data-testid=run-audit][data-run-id=\\"\\($run)\\"]"');
    expect(failureCase).toContain('selector:"[data-testid=run-audit]:not([data-run-id=\\"\\($run)\\"])"');
    expect(failureCase).not.toContain('{action:"waitFor",selector:"[data-testid=run-audit]"}');
    expect(browser).toContain('"$admin/api/reports/runs/$1/steps"');
    expect(inner).toContain("assertText did not match");
    expect(inner).toContain("source-artifacts.before.json");
    expect(inner).toContain("source-artifacts.after.json");
    expect(delivery).toContain("seed_partial_schedule_fixture");
    expect(delivery).toContain("wait_reports_ui_outbox_settled");
    expect(inner).not.toContain('sqlite3 "$reports_db"');
    expect(inner).toContain("schedule-delete");
    expect(inner).toContain("run-create");
    expect(inner).toContain("run-cancel-");
    expect(evidence).toContain("pauseDurationMs == 30000");
    expect(evidence).toContain("$source.before[$kind].sha256 == $source.after[$kind].sha256");
    expect(evidence).toContain("all(.data[]; .runId == $run)");
    expect(receiptActions.managerProcessing.filter((action) => action === "assertText")).toHaveLength(8);
    expect(inner).toContain("seed_reports_ui_delivery_status");
    expect(inner).toContain("15 irreversible notification delivery gaps");
    expect(inner).toContain("通知投递存在 15 个不可恢复缺口");
    expect(outer).toContain("--env TZ=UTC");
    expect(inner).toContain('test "$(date +%Z)" = UTC');
    expect(inner).toContain("First gap 09/10/2026, 01:02:03 AM");
    expect(inner).toContain("首个缺口 2026/09/10 01:02:03");
    const finalSettle = inner.lastIndexOf("wait_reports_ui_outbox_settled");
    const finalSeed = inner.lastIndexOf("seed_reports_ui_delivery_status");
    const viewerSeed = inner.indexOf("seed_reports_ui_delivery_status", inner.indexOf("viewer_retry_status"));
    expect(inner.indexOf("partial_browser_run=$(run_browser_case")).toBeLessThan(viewerSeed);
    expect(viewerSeed).toBeLessThan(inner.indexOf("viewer_steps="));
    expect(inner.indexOf("viewer_browser_run=$(run_browser_case")).toBeLessThan(finalSettle);
    expect(finalSettle).toBeLessThan(finalSeed);
    expect(finalSeed).toBeLessThan(inner.indexOf("reports-delivery-owner.json", finalSeed));
    expect(inner.slice(finalSeed)).not.toContain("run_browser_case");
    expect(inner.slice(finalSeed)).toContain("reports_ui_delivery_db_json");
    expect(browser).toContain("wait_for_status \"$run\" succeeded || return 1");
    expect(browser).toContain("test \"$dimensions\" = \"$5 x $6\" || return 1");
    expect(inner).not.toContain("cron");
    expect(inner).not.toContain("webhook");
});

test("state gate uses the existing four-service browser harness without Docker execution", async () => {
    const outer = await read("./verify-reports-ui-state-linux.sh");
    const inner = await read("./verify-reports-ui-state-linux-inner.sh");
    expect(outer).toContain("monitoring-ui-state-inner-lib.sh");
    expect(outer).toContain("monitoring-ui-state-fixture.py");
    expect(inner).toContain(". /verify/inner-lib.sh");
    expect(inner).toContain(". /verify/delivery-lib.sh");
    expect(inner).toContain(". /verify/browser-lib.sh");
    expect(outer).toContain("target/rz/reports-ui-state");
});

test("manifest validator rejects retained-evidence and receipt tampering", async () => {
    const evidence = new URL("./reports-ui-state-evidence-lib.sh", import.meta.url).pathname;
    const directory = `/tmp/rz-reports-ui-state-${crypto.randomUUID()}`;
    const manifestPath = `${directory}/manifest.json`;
    const writeJson = (file, value) => Bun.write(`${directory}/${file}`, JSON.stringify(value));
    const descriptor = async (file) => ({ file, sha256: await sha(`${directory}/${file}`) });
    const writeManifest = (manifest) => Bun.write(manifestPath, JSON.stringify(manifest));
    const verify = () =>
        Bun.spawnSync(
            [
                "bash",
                "-c",
                '. "$1"; verify_reports_ui_state_manifest "$2" head dirty sha linux/arm64 && verify_reports_ui_state_receipts "$3" "$2" && verify_reports_ui_state_delivery_receipts "$3" "$2" && verify_reports_ui_state_source_evidence "$3" "$2"',
                "reports-ui-state-validator",
                evidence,
                manifestPath,
                directory,
            ],
            { stdout: "pipe", stderr: "pipe" },
        );
    try {
        await Bun.spawn(["mkdir", "-p", `${directory}/run-steps`]).exited;
        const runs = Object.fromEntries(cases.map((name) => [name, `${name}-run`]));
        const runSteps = {};
        for (const name of cases) {
            const file = `run-steps/${name.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`)}.json`;
            await writeJson(file, { data: receiptActions[name].map((action) => ({ runId: runs[name], action, status: "succeeded" })) });
            runSteps[name] = { runId: runs[name], ...(await descriptor(file)) };
        }
        await writeJson("processing-run-steps.json", {
            data: [{ runId: "active", action: "pause", status: "cancelled", message: "cancelled by user" }],
        });
        await writeJson("source-run.before.json", {
            data: { id: "source", status: "failed", error: "assertText did not match" },
        });
        await writeJson("source-steps.before.json", {
            data: [{ runId: "source", action: "assertText", status: "failed", message: "assertText did not match" }],
        });
        await writeJson("source-artifacts.before.json", { data: [{ runId: "source", fileName: "source-runtime-failure-witness.png" }] });
        for (const kind of ["run", "steps", "artifacts"]) {
            await Bun.write(`${directory}/source-${kind}.after.json`, await Bun.file(`${directory}/source-${kind}.before.json`).text());
        }
        const snapshots = { before: {}, after: {} };
        for (const phase of ["before", "after"]) {
            for (const kind of ["run", "steps", "artifacts"]) snapshots[phase][kind] = await descriptor(`source-${kind}.${phase}.json`);
        }
        const artifacts = Object.entries(screenshotCases).map(([caseName, [file, width, height]]) => ({
            case: caseName,
            file,
            sha256: "a".repeat(64),
            dimensions: `${width} x ${height}`,
            viewport: { width, height },
        }));
        const delivery = {
            pendingCount: 2, pendingBytes: 1024, quarantineCount: 3, quarantineBytes: 2048,
            omittedCount: 1, expiredCount: 2, unconfirmedCount: 3, quarantinedCount: 4,
            quarantineEvictedCount: 5, firstGapAt: "2026-09-10T01:02:03Z",
            lastGapAt: "2026-09-10T02:03:04Z", lastSuccessAt: "2026-09-10T03:04:05Z",
        };
        await writeJson("reports-delivery-owner.json", { data: delivery });
        await writeJson("reports-delivery-viewer.json", { data: delivery });
        const manifest = {
            schemaVersion: 2,
            status: "passed",
            gitHead: "head",
            sourceTreeState: "dirty",
            sourceTreeSha256: "sha",
            platform: "linux/arm64",
            chromiumVersion: "pinned",
            runs,
            runSteps,
            processing: {
                runId: "active",
                status: "cancelled",
                pauseDurationMs: 30000,
                pauseStep: { action: "pause", status: "cancelled" },
                stepReceipt: await descriptor("processing-run-steps.json"),
            },
            sourceEvidence: {
                runId: "source",
                status: "failed",
                error: "assertText did not match",
                failedStep: { action: "assertText", status: "failed", message: "assertText did not match" },
                ...snapshots,
            },
            retry: { childId: "child", sourcePreserved: true },
            partialFixture: { enqueuedRunId: "source", skippedRunLinked: false },
            viewOnly: { scheduleMutationStatus: 403, retryStatus: 403 },
            deliveryHealth: { gapTotal: 15, ownerReceipt: await descriptor("reports-delivery-owner.json"), viewerReceipt: await descriptor("reports-delivery-viewer.json") },
            artifacts,
        };
        await writeManifest(manifest);
        const firstVerification = verify();
        expect(firstVerification.exitCode, new TextDecoder().decode(firstVerification.stderr)).toBe(0);

        await writeJson("reports-delivery-owner.json", { data: { ...delivery, pendingCount: 99 } });
        expect(verify().exitCode).not.toBe(0);
        await writeJson("reports-delivery-owner.json", { data: delivery });
        manifest.deliveryHealth.ownerReceipt = await descriptor("reports-delivery-owner.json");
        await writeJson("reports-delivery-owner.json", { data: { ...delivery, pendingCount: 99 } });
        manifest.deliveryHealth.ownerReceipt = await descriptor("reports-delivery-owner.json");
        await writeManifest(manifest);
        expect(verify().exitCode).not.toBe(0);
        await writeJson("reports-delivery-owner.json", { data: delivery });
        manifest.deliveryHealth.ownerReceipt = await descriptor("reports-delivery-owner.json");
        manifest.deliveryHealth.viewerReceipt = { ...manifest.deliveryHealth.ownerReceipt };
        await writeManifest(manifest);
        expect(verify().exitCode).not.toBe(0);
        manifest.deliveryHealth.viewerReceipt = await descriptor("reports-delivery-viewer.json");

        manifest.sourceEvidence.runId = "";
        await writeManifest(manifest);
        expect(verify().exitCode).not.toBe(0);
        manifest.sourceEvidence.runId = "source";

        const truncatedReceipt = { data: receiptActions.managerProcessing.slice(0, -1).map((action) => ({ runId: runs.managerProcessing, action, status: "succeeded" })) };
        await writeJson("run-steps/manager-processing.json", truncatedReceipt);
        runSteps.managerProcessing = { runId: runs.managerProcessing, ...(await descriptor("run-steps/manager-processing.json")) };
        await writeManifest(manifest);
        expect(verify().exitCode).not.toBe(0);
        const failedReceipt = { data: receiptActions.managerProcessing.map((action, index) => ({ runId: runs.managerProcessing, action, status: index === 0 ? "failed" : "succeeded" })) };
        await writeJson("run-steps/manager-processing.json", failedReceipt);
        runSteps.managerProcessing = { runId: runs.managerProcessing, ...(await descriptor("run-steps/manager-processing.json")) };
        await writeManifest(manifest);
        expect(verify().exitCode).not.toBe(0);
        await writeJson("run-steps/manager-processing.json", { data: receiptActions.managerProcessing.map((action) => ({ runId: runs.managerProcessing, action, status: "succeeded" })) });
        runSteps.managerProcessing = { runId: runs.managerProcessing, ...(await descriptor("run-steps/manager-processing.json")) };

        const succeededRun = { data: { id: "source", status: "succeeded", error: "assertText did not match" } };
        await writeJson("source-run.before.json", succeededRun);
        await Bun.write(`${directory}/source-run.after.json`, JSON.stringify(succeededRun));
        snapshots.before.run = await descriptor("source-run.before.json");
        snapshots.after.run = { ...snapshots.before.run };
        await writeManifest(manifest);
        expect(verify().exitCode).not.toBe(0);
        const failedRun = { data: { id: "source", status: "failed", error: "assertText did not match" } };
        await writeJson("source-run.before.json", failedRun);
        await Bun.write(`${directory}/source-run.after.json`, JSON.stringify(failedRun));
        snapshots.before.run = await descriptor("source-run.before.json");
        snapshots.after.run = { ...snapshots.before.run };

        const sourceSteps = { data: [] };
        await writeJson("source-steps.before.json", sourceSteps);
        snapshots.before.steps = await descriptor("source-steps.before.json");
        snapshots.after.steps = { ...snapshots.before.steps };
        await Bun.write(`${directory}/source-steps.after.json`, JSON.stringify(sourceSteps));
        await writeManifest(manifest);
        expect(verify().exitCode).not.toBe(0);

        await writeJson("source-steps.before.json", { data: [{ runId: "source", action: "assertText", status: "succeeded", message: "assertText did not match" }] });
        await Bun.write(`${directory}/source-steps.after.json`, await Bun.file(`${directory}/source-steps.before.json`).text());
        snapshots.before.steps = await descriptor("source-steps.before.json");
        snapshots.after.steps = { ...snapshots.before.steps };
        await writeManifest(manifest);
        expect(verify().exitCode).not.toBe(0);

        await writeJson("source-steps.before.json", { data: [{ runId: "source", action: "assertText", status: "failed", message: "assertText did not match" }] });
        await Bun.write(`${directory}/source-steps.after.json`, JSON.stringify({ data: [{ runId: "source", action: "assertText", status: "failed", message: "changed" }] }));
        snapshots.before.steps = await descriptor("source-steps.before.json");
        snapshots.after.steps = await descriptor("source-steps.after.json");
        await writeManifest(manifest);
        expect(verify().exitCode).not.toBe(0);

        await Bun.write(`${directory}/source-steps.after.json`, await Bun.file(`${directory}/source-steps.before.json`).text());
        snapshots.after.steps = { ...snapshots.before.steps };
        manifest.sourceEvidence.after.steps.file = manifest.sourceEvidence.before.steps.file;
        await writeManifest(manifest);
        expect(verify().exitCode).not.toBe(0);
        manifest.sourceEvidence.after.steps.file = "source-steps.after.json";
        manifest.artifacts[0].file = manifest.artifacts[1].file;
        await writeManifest(manifest);
        expect(verify().exitCode).not.toBe(0);
        manifest.artifacts[0].file = screenshotCases.managerProcessing[0];

        await writeJson("source-artifacts.before.json", { data: [] });
        await Bun.write(`${directory}/source-artifacts.after.json`, await Bun.file(`${directory}/source-artifacts.before.json`).text());
        snapshots.before.artifacts = await descriptor("source-artifacts.before.json");
        snapshots.after.artifacts = { ...snapshots.before.artifacts };
        await writeManifest(manifest);
        expect(verify().exitCode).not.toBe(0);
        await writeJson("source-artifacts.before.json", { data: [{ runId: "wrong-run", fileName: "source-runtime-failure-witness.png" }] });
        await Bun.write(`${directory}/source-artifacts.after.json`, await Bun.file(`${directory}/source-artifacts.before.json`).text());
        snapshots.before.artifacts = await descriptor("source-artifacts.before.json");
        snapshots.after.artifacts = { ...snapshots.before.artifacts };
        await writeManifest(manifest);
        expect(verify().exitCode).not.toBe(0);

        manifest.processing.pauseDurationMs = 12000;
        await writeManifest(manifest);
        expect(verify().exitCode).not.toBe(0);
    } finally {
        await Bun.spawn(["rm", "-rf", directory]).exited;
    }
});
