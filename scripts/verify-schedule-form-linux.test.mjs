import { describe, expect, test } from "bun:test";

const gate = await Bun.file(new URL("./verify-schedule-form-linux.sh", import.meta.url)).text();
const inner = await Bun.file(new URL("./verify-schedule-form-linux-inner.sh", import.meta.url)).text();
const driver = await Bun.file(new URL("./schedule-form-browser-steps.mjs", import.meta.url)).text();
const schedulePanel = await Bun.file(new URL("../apps/web/src/routes/reports/-templates/schedule-panel.tsx", import.meta.url)).text();
const layout = await Bun.file(new URL("../apps/web/src/components/layout/index.tsx", import.meta.url)).text();
const scheduleColumns = await Bun.file(new URL("../apps/web/src/routes/reports/-templates/schedule-columns.tsx", import.meta.url)).text();
const justfile = await Bun.file(new URL("../justfile", import.meta.url)).text();
const gatePath = new URL("./verify-schedule-form-linux.sh", import.meta.url).pathname;
const runGate = (env) => Bun.spawnSync({ cmd: ["bash", gatePath], env: { ...process.env, ...env }, stdout: "pipe", stderr: "pipe" });
const viewerSteps = driver.slice(driver.indexOf("viewer: viewer(["), driver.indexOf("\n    ]),", driver.indexOf("viewer: viewer([")));

test("SR-UI-002 gate keeps browser actions, evidence, and no-request proof scoped", () => {
    for (const value of ["admin-browser-source-identity.sh", "atomic_replace_symlink", "browser-steps.json", "RUSTZEN_SCHEDULE_FORM_TIMEOUT"]) expect(gate).toContain(value);
    for (const value of ["localValidation", "proxyPostCount", "secretPolicy", "assertFocus", "managementVisible", "10:16 · UTC", "schedule-timezone-"]) expect(`${inner}\n${driver}`).toContain(value);
    expect(inner).toContain("local case_name=$1 system_id=$2 case_steps=$3 body flow_id run_id case_status");
    expect(inner).toContain("--argjson steps \"$case_steps\"");
    for (const value of ["case_diagnostics", "create-daily", "/steps", "/artifacts", "tail -n 40"]) expect(inner).toContain(value);
    expect(gate).toContain("failed-runs/$run_id");
    expect(gate).toContain('RUSTZEN_VERIFY_PLATFORM="$platform"');
    expect(inner).toContain('--arg platform "$RUSTZEN_VERIFY_PLATFORM"');
    expect(inner).not.toContain('"linux/$RUSTZEN_VERIFY_ARCHITECTURE"');
    expect(driver).toContain("sr-ui-002-secret-marker");
    expect(driver).toContain('assertValue", selector: "textarea"');
    expect(inner).toContain('local_posts" = 0');
    expect(inner).toContain('secret_posts" = 1');
    expect(driver).toContain('schedule-create');
    expect(driver).toContain('schedule-edit');
    expect(driver).toContain('assertElementLayout", selector: "[data-testid^=schedule-timezone-]", visibleCount: 1, withinViewportRight: true');
    expect(gate).toContain("verify_schedule_form_manifest");
    expect(gate).toContain("verify_schedule_form_receipts");
    expect(gate).toContain("verify_schedule_form_artifacts");
    expect(inner).toContain("save_schedule_form_run_steps");
    const dailyJourney = driver.slice(driver.indexOf("createDaily:"), driver.indexOf("editWeekly:"));
    const weeklyJourney = driver.slice(driver.indexOf("editWeekly:"), driver.indexOf("viewer:"));
    expect(dailyJourney).toContain("...englishScheduleMenu");
    expect(weeklyJourney).toContain("...englishScheduleMenu");
    expect(driver).toContain("[data-testid='navigation-reports-schedules'][data-label='Scheduled reports']");
    expect(driver).toContain("[data-testid='navigation-reports-schedules'][data-label='定时报表']");
    expect(driver).not.toContain("xpath=//*[@aria-label='Main navigation']");
    expect(layout).toContain('data-testid="navigation-reports-schedules"');
    expect(layout).toContain('data-label={item.name}');
    expect(schedulePanel).toContain("state.checkPermissions(REPORTS_SCHEDULE_MANAGE)");
    expect(schedulePanel).toContain("createScheduleColumns({ flowOptions, canManageSchedules, onSaved: refresh })");
    expect(scheduleColumns).toContain("if (canManageSchedules)");
    expect(scheduleColumns).toContain("columns.push({");
    expect(scheduleColumns).toContain("width: 230");
    expect(scheduleColumns).toContain("data-testid={`schedule-timezone-${row.id}`}");
    expect(scheduleColumns).toContain('data-testid="schedule-actions-column"');
    expect(gate).not.toContain("verify-admin-browser-linux-inner.sh");
    expect(justfile).toContain("verify-schedule-form-linux:");
});

test("viewer steps directly reject every schedule action control", () => {
    for (const selector of [
        "schedule-actions-column",
        "schedule-create",
        "schedule-dialog",
        "schedule-edit",
        "schedule-toggle",
        "schedule-delete",
    ]) {
        expect(viewerSteps).toContain(`{ action: "assertAbsent", selector: "[data-testid=${selector}]" }`);
    }
    expect(viewerSteps).toContain('text: "10:16 · UTC"');
    expect(viewerSteps).toContain('{ action: "assertNoHorizontalOverflow" }');
});

test("focus assertions wait for Ant Modal restoration", () => {
    for (const name of ["cancelCreate", "createDaily", "editWeekly"]) {
        const start = driver.indexOf(`${name}:`);
        const end = driver.indexOf("assertFocus", start);
        expect(driver.slice(start, end)).toContain('{ action: "pause", durationMs: 300 }');
    }
    expect(inner).toContain("tail -n 40");
});

test("publication seam retains the published current manifest on a forced failure", () => {
    const output = runGate({ RUSTZEN_SCHEDULE_FORM_TEST_PUBLISH_FAILURE: "1" });
    expect(output.exitCode).toBe(0);
    expect(new TextDecoder().decode(output.stdout)).toContain("publication seams passed");
});

test("invalid timeouts, Docker discovery, setup, and signals clean their isolated roots", async () => {
    expect(runGate({ RUSTZEN_UI_LINUX_ARCH: "aarch64", RUSTZEN_SCHEDULE_FORM_TIMEOUT: "0" }).exitCode).toBe(2);
    const fakeDocker = `/tmp/rz-schedule-form-fake-${crypto.randomUUID()}`;
    await Bun.write(fakeDocker, "#!/bin/sh\nexec sleep 5\n"); Bun.spawnSync(["chmod", "+x", fakeDocker]);
    expect(runGate({ RUSTZEN_SCHEDULE_FORM_DOCKER: fakeDocker, RUSTZEN_SCHEDULE_FORM_DOCKER_INFO_TIMEOUT: "1" }).exitCode).not.toBe(0);
    for (const [signal, code] of [["INT", 130], ["TERM", 143]]) {
        const root = `/tmp/rz-schedule-form-signal-${crypto.randomUUID()}`; Bun.spawnSync(["mkdir", "-p", root]);
        expect(runGate({ RUSTZEN_SCHEDULE_FORM_TEST_SIGNAL: signal, RUSTZEN_SCHEDULE_FORM_TEST_ROOT: root }).exitCode).toBe(code);
        expect(Bun.spawnSync(["readlink", `${root}/current`]).stdout.toString().trim()).toBe("runs/old");
        expect(await Bun.file(`${root}/.verify.lock`).exists()).toBeFalse(); expect(await Bun.file(`${root}/.candidate`).exists()).toBeFalse(); Bun.spawnSync(["rm", "-rf", root]);
    }
    const root = `/tmp/rz-schedule-form-setup-${crypto.randomUUID()}`; Bun.spawnSync(["mkdir", "-p", root]);
    const setup = runGate({ RUSTZEN_SCHEDULE_FORM_TEST_SETUP_FAILURE: "1", RUSTZEN_SCHEDULE_FORM_TEST_ROOT: root });
    expect(setup.exitCode).not.toBe(0); expect(Bun.spawnSync(["readlink", `${root}/current`]).stdout.toString().trim()).toBe("runs/old");
    expect(await Bun.file(`${root}/.verify.lock`).exists()).toBeFalse(); expect(await Bun.file(`${root}/.candidate`).exists()).toBeFalse(); Bun.spawnSync(["rm", "-rf", root, fakeDocker]);
});

const scheduleEvidence = new URL("./schedule-form-evidence-lib.sh", import.meta.url).pathname;
const scheduleSteps = JSON.parse(new TextDecoder().decode(Bun.spawnSync(["bun", new URL("./schedule-form-browser-steps.mjs", import.meta.url).pathname], { stdout: "pipe" }).stdout));
const scheduleCases = {
    cancel: "cancelCreate",
    malformedInput: "malformedInput",
    missingTime: "missingTime",
    secretRejected: "secretRejected",
    daily: "createDaily",
    weekly: "editWeekly",
    viewer: "viewer",
};

const scheduleSha = async (path) => new TextDecoder().decode(Bun.spawnSync(["shasum", "-a", "256", path], { stdout: "pipe" }).stdout).split(" ")[0];
const pngHeader = (width, height) => Buffer.from([
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82,
    width >>> 24, width >>> 16, width >>> 8, width, height >>> 24, height >>> 16, height >>> 8, height,
    8, 6, 0, 0, 0, 0, 0, 0, 0,
]);

test("schedule run-step saving works under nounset and returns the exact receipt descriptor", async () => {
    const directory = `/tmp/rz-schedule-form-save-${crypto.randomUUID()}`;
    const runId = "daily-run";
    const shell = [
        "set -eu",
        'source "$1"',
        "auth=()",
        "admin=https://admin.invalid",
        "curl_json() { printf '%s\\n' '{\"data\":[{\"runId\":\"daily-run\",\"action\":\"goto\",\"status\":\"succeeded\"}]}'; }",
        'descriptor=$(save_schedule_form_run_steps "daily-run" "daily")',
        'jq -e --arg run "daily-run" --arg file "run-steps/daily.json" \' .runId == $run and .file == $file and (.sha256 | test("^[0-9a-f]{64}$")) \' <<<"$descriptor" >/dev/null',
        'jq -e --arg run "daily-run" \' .data == [{runId:$run,action:"goto",status:"succeeded"}] \' "$RUSTZEN_SCHEDULE_FORM_EVIDENCE_ROOT/run-steps/daily.json" >/dev/null',
    ].join("\n");
    try {
        const result = Bun.spawnSync({
            cmd: ["bash", "-c", shell, "schedule-save-test", scheduleEvidence],
            env: { ...process.env, RUSTZEN_SCHEDULE_FORM_EVIDENCE_ROOT: directory },
            stdout: "pipe",
            stderr: "pipe",
        });
        expect(result.exitCode).toBe(0);
        expect(await Bun.file(`${directory}/run-steps/daily.json`).exists()).toBeTrue();
    } finally {
        await Bun.spawn(["rm", "-rf", directory]).exited;
    }
});

test("schedule form manifest requires every run receipt and its exact successful action sequence", async () => {
    const directory = `/tmp/rz-schedule-form-${crypto.randomUUID()}`;
    const manifestPath = `${directory}/manifest.json`;
    const writeJson = (file, value) => Bun.write(`${directory}/${file}`, JSON.stringify(value));
    const descriptor = async (file, runId) => ({ runId, file, sha256: await scheduleSha(`${directory}/${file}`) });
    const verify = () => Bun.spawnSync([
        "bash", "-c",
        '. "$1"; verify_schedule_form_manifest "$2" head dirty sha linux/amd64 "$3" && verify_schedule_form_receipts "$4" "$2" "$3" && verify_schedule_form_artifacts "$4" "$2"',
        "schedule-form-validator", scheduleEvidence, manifestPath, `${directory}/browser-steps.json`, directory,
    ], { stdout: "pipe", stderr: "pipe" });
    try {
        await Bun.spawn(["mkdir", "-p", `${directory}/run-steps`]).exited;
        await Bun.write(`${directory}/browser-steps.json`, JSON.stringify(scheduleSteps));
        const runs = Object.fromEntries(Object.keys(scheduleCases).map((name) => [name, `${name}-run`]));
        const runSteps = {};
        for (const [name, source] of Object.entries(scheduleCases)) {
            const file = `run-steps/${name}.json`;
            await writeJson(file, { data: scheduleSteps[source].map((step) => ({ runId: runs[name], action: step.action, status: "succeeded" })) });
            runSteps[name] = await descriptor(file, runs[name]);
        }
        await Bun.write(`${directory}/schedule-form-desktop-dark-en.png`, pngHeader(1440, 900));
        await Bun.write(`${directory}/schedule-form-mobile-light-zh.png`, pngHeader(390, 844));
        const artifactDescriptor = async (file, dimensions) => ({ file, dimensions, sha256: await scheduleSha(`${directory}/${file}`) });
        const manifest = {
            schemaVersion: 2, status: "passed", gitHead: "head", sourceTreeState: "dirty", sourceTreeSha256: "sha", platform: "linux/amd64", chromiumVersion: "pinned",
            runs, runSteps,
            localValidation: { rowsBefore: 0, rowsAfter: 0, proxyPostCount: 0 },
            secretPolicy: { rowsBefore: 0, rowsAfter: 0, rowDelta: 0, proxyPostCount: 1 },
            schedule: { cadence: "weekly", weekday: 0, dueTime: "10:16", timezone: "UTC" },
            viewOnly: { managementVisible: false, dueTime: "10:16 · UTC" },
            artifacts: [
                await artifactDescriptor("schedule-form-desktop-dark-en.png", "1440 x 900"),
                await artifactDescriptor("schedule-form-mobile-light-zh.png", "390 x 844"),
            ],
        };
        await writeJson("manifest.json", manifest);
        expect(verify().exitCode).toBe(0);

        const desktop = `${directory}/schedule-form-desktop-dark-en.png`;
        const desktopBytes = await Bun.file(desktop).arrayBuffer();
        await Bun.spawn(["mv", desktop, `${desktop}.missing`]).exited;
        expect(verify().exitCode).not.toBe(0);
        await Bun.spawn(["mv", `${desktop}.missing`, desktop]).exited;

        await Bun.write(desktop, Buffer.concat([Buffer.from(desktopBytes), Buffer.from([0]) ]));
        expect(verify().exitCode).not.toBe(0);
        await Bun.write(desktop, desktopBytes);

        manifest.artifacts[0].sha256 = "c".repeat(64);
        await writeJson("manifest.json", manifest);
        expect(verify().exitCode).not.toBe(0);
        manifest.artifacts[0] = await artifactDescriptor("schedule-form-desktop-dark-en.png", "1440 x 900");

        await Bun.write(desktop, pngHeader(1439, 900));
        manifest.artifacts[0] = await artifactDescriptor("schedule-form-desktop-dark-en.png", "1440 x 900");
        await writeJson("manifest.json", manifest);
        expect(verify().exitCode).not.toBe(0);
        await Bun.write(desktop, desktopBytes);
        manifest.artifacts[0] = await artifactDescriptor("schedule-form-desktop-dark-en.png", "1440 x 900");

        manifest.localValidation.rowsAfter = 1;
        await writeJson("manifest.json", manifest);
        expect(verify().exitCode).not.toBe(0);
        manifest.localValidation.rowsAfter = 0;
        manifest.secretPolicy.rowDelta = 1;
        await writeJson("manifest.json", manifest);
        expect(verify().exitCode).not.toBe(0);
        manifest.secretPolicy.rowDelta = 0;
        manifest.localValidation.rowsBefore = "0";
        await writeJson("manifest.json", manifest);
        expect(verify().exitCode).not.toBe(0);
        manifest.localValidation.rowsBefore = 0;

        delete manifest.runs.daily;
        await writeJson("manifest.json", manifest);
        expect(verify().exitCode).not.toBe(0);
        manifest.runs.daily = "daily-run";

        delete manifest.runSteps.daily;
        await writeJson("manifest.json", manifest);
        expect(verify().exitCode).not.toBe(0);
        manifest.runSteps.daily = await descriptor("run-steps/daily.json", "daily-run");

        manifest.runSteps.daily.runId = "weekly-run";
        await writeJson("manifest.json", manifest);
        expect(verify().exitCode).not.toBe(0);
        manifest.runSteps.daily = await descriptor("run-steps/daily.json", "daily-run");

        manifest.runSteps.daily.sha256 = "c".repeat(64);
        await writeJson("manifest.json", manifest);
        expect(verify().exitCode).not.toBe(0);
        manifest.runSteps.daily = await descriptor("run-steps/daily.json", "daily-run");

        await writeJson("run-steps/daily.json", { data: scheduleSteps.createDaily.map((step, index) => ({ runId: "daily-run", action: index === 0 ? "click" : step.action, status: "succeeded" })) });
        manifest.runSteps.daily = await descriptor("run-steps/daily.json", "daily-run");
        await writeJson("manifest.json", manifest);
        expect(verify().exitCode).not.toBe(0);

        await writeJson("run-steps/daily.json", { data: scheduleSteps.createDaily.map((step, index) => ({ runId: "daily-run", action: step.action, status: index === 0 ? "failed" : "succeeded" })) });
        manifest.runSteps.daily = await descriptor("run-steps/daily.json", "daily-run");
        await writeJson("manifest.json", manifest);
        expect(verify().exitCode).not.toBe(0);
    } finally {
        await Bun.spawn(["rm", "-rf", directory]).exited;
    }
});
