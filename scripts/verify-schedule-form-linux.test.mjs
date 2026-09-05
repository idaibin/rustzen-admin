import { describe, expect, test } from "bun:test";

const gate = await Bun.file(new URL("./verify-schedule-form-linux.sh", import.meta.url)).text();
const inner = await Bun.file(new URL("./verify-schedule-form-linux-inner.sh", import.meta.url)).text();
const driver = await Bun.file(new URL("./schedule-form-browser-steps.mjs", import.meta.url)).text();
const justfile = await Bun.file(new URL("../justfile", import.meta.url)).text();
const gatePath = new URL("./verify-schedule-form-linux.sh", import.meta.url).pathname;
const runGate = (env) => Bun.spawnSync({ cmd: ["bash", gatePath], env: { ...process.env, ...env }, stdout: "pipe", stderr: "pipe" });

test("SR-UI-002 gate keeps browser actions, evidence, and no-request proof scoped", () => {
    for (const value of ["admin-browser-source-identity.sh", "atomic_replace_symlink", "browser-steps.json", "RUSTZEN_SCHEDULE_FORM_TIMEOUT"]) expect(gate).toContain(value);
    for (const value of ["localValidation", "proxyPostCount", "secretPolicy", "assertFocus", "managementVisible"]) expect(`${inner}\n${driver}`).toContain(value);
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
    expect(gate).not.toContain("verify-admin-browser-linux-inner.sh");
    expect(justfile).toContain("verify-schedule-form-linux:");
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
