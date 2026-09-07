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
const receiptActions = {
    managerProcessing: ["setUiPreferences", "setViewport", "goto", "waitFor", "fill", "fill", "click", "waitFor", "goto", "waitFor", "click", "waitFor", "assertText", "assertText", "assertNoHorizontalOverflow", "screenshotViewport"],
    managerRuntimeFailure: ["setUiPreferences", "setViewport", "goto", "waitFor", "fill", "fill", "click", "waitFor", "goto", "waitFor", "assertText", "click", "waitFor", "assertText", "assertText", "assertNoHorizontalOverflow", "screenshotViewport", "click", "waitFor"],
    managerPartial: ["setUiPreferences", "setViewport", "goto", "waitFor", "fill", "fill", "click", "waitFor", "goto", "waitFor", "assertText", "assertText", "assertText", "assertAbsent", "waitFor", "assertNoHorizontalOverflow", "screenshotViewport"],
    viewOnlyMobile: ["setUiPreferences", "setViewport", "goto", "waitFor", "fill", "fill", "click", "waitFor", "goto", "waitFor", "assertAbsent", "assertAbsent", "assertAbsent", "assertAbsent", "goto", "waitFor", "assertAbsent", "assertAbsent", "assertAbsent", "click", "waitFor", "assertAbsent", "assertNoHorizontalOverflow", "screenshotViewport"],
};

const sha = async (path) => {
    const result = Bun.spawnSync(["shasum", "-a", "256", path], { stdout: "pipe" });
    return new TextDecoder().decode(result.stdout).split(" ")[0];
};

test("Reports state gate keeps the declared closure bounded and source-bound", async () => {
    const [outer, inner, evidence, justfile] = await Promise.all([
        read("./verify-reports-ui-state-linux.sh"),
        read("./verify-reports-ui-state-linux-inner.sh"),
        read("./reports-ui-state-evidence-lib.sh"),
        Bun.file(`${root}/justfile`).text(),
    ]);

    expect(justfile).toContain("verify-reports-ui-state-linux:");
    expect(outer).toContain("monitoring-ui-state-gate-lib.sh");
    expect(outer).toContain("verify_reports_ui_state_receipts");
    expect(outer).toContain("verify_reports_ui_state_source_evidence");
    expect(inner).toContain("RUSTZEN_REPORTS_MAX_CONCURRENCY=2");
    expect(inner).toContain('"durationMs":30000');
    expect(inner).toContain("processing-run-steps.json");
    expect(inner).toContain('text:"1. goto"');
    expect(inner).not.toContain('text:"Waiting for step results"');
    expect(inner).not.toContain('text:"2. pause"');
    expect(inner).toContain('selector:"tr:has([data-testid=run-retry-list-\\($run)])"');
    expect(inner).not.toContain('selector:".ant-table-row",text:"assertText did not match"');
    const failureCase = inner.slice(inner.indexOf("failure_steps="), inner.indexOf("failure_browser_run="));
    expect(failureCase).toContain('selector:"[data-testid=run-audit][data-run-id=\\"\\($run)\\"]"');
    expect(failureCase).toContain('selector:"[data-testid=run-audit]:not([data-run-id=\\"\\($run)\\"])"');
    expect(failureCase).not.toContain('{action:"waitFor",selector:"[data-testid=run-audit]"}');
    expect(inner).toContain("capture_failed_browser_run");
    expect(inner).toContain("failed-browser-runs/${case_name}-${run_id}");
    expect(inner).toContain('"$admin/api/reports/runs/$run_id/steps"');
    expect(inner).toContain('"$admin/api/reports/runs/$run_id/artifacts"');
    expect(inner).toContain("assertText did not match");
    expect(inner).toContain("source-artifacts.before.json");
    expect(inner).toContain("source-artifacts.after.json");
    expect(inner).toContain('python3 -B - "$reports_db" "$enqueued_schedule" "$skipped_schedule" "$failure_run"');
    expect(inner).toContain("import sqlite3");
    expect(inner).toContain("sqlite3.connect(database, timeout=5.0)");
    expect(inner).toContain('connection.execute("PRAGMA foreign_keys = ON")');
    expect(inner).toContain('connection.execute("PRAGMA foreign_keys").fetchone() != (1,)');
    expect(inner).toContain("connection.execute(");
    expect(inner).toContain("VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)");
    expect(inner).not.toContain('sqlite3 "$reports_db"');
    expect(inner).toContain("schedule-delete");
    expect(inner).toContain("run-create");
    expect(inner).toContain("run-cancel-");
    expect(evidence).toContain("pauseDurationMs == 30000");
    expect(evidence).toContain("$source.before[$kind].sha256 == $source.after[$kind].sha256");
    expect(evidence).toContain("all(.data[]; .runId == $run)");
    expect(receiptActions.managerProcessing.filter((action) => action === "assertText")).toHaveLength(2);
    expect(inner).not.toContain("cron");
    expect(inner).not.toContain("webhook");
});

test("state gate uses the existing four-service browser harness without Docker execution", async () => {
    const outer = await read("./verify-reports-ui-state-linux.sh");
    const inner = await read("./verify-reports-ui-state-linux-inner.sh");
    expect(outer).toContain("monitoring-ui-state-inner-lib.sh");
    expect(outer).toContain("monitoring-ui-state-fixture.py");
    expect(inner).toContain(". /verify/inner-lib.sh");
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
                '. "$1"; verify_reports_ui_state_manifest "$2" head dirty sha linux/arm64 && verify_reports_ui_state_receipts "$3" "$2" && verify_reports_ui_state_source_evidence "$3" "$2"',
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
            artifacts,
        };
        await writeManifest(manifest);
        const firstVerification = verify();
        expect(firstVerification.exitCode, new TextDecoder().decode(firstVerification.stderr)).toBe(0);

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
