import { describe, expect, test } from "bun:test";

const gate = await Bun.file(new URL("./verify-monitor-agent-multi-node-linux.sh", import.meta.url)).text();
const inner = await Bun.file(new URL("./verify-monitor-agent-multi-node-linux-inner.sh", import.meta.url)).text();
const readiness = await Bun.file(new URL("./monitor-agent-multi-node-readiness.py", import.meta.url)).text();
const justfile = await Bun.file(new URL("../justfile", import.meta.url)).text();
const gatePath = new URL("./verify-monitor-agent-multi-node-linux.sh", import.meta.url).pathname;

function runGate(env) {
    return Bun.spawnSync({ cmd: ["bash", gatePath], env: { ...process.env, ...env }, stdout: "pipe", stderr: "pipe" });
}

describe("Monitor dual-Agent Linux gate contract", () => {
    test("is bounded, source-bound, atomically published, and independently exposed", () => {
        expect(gate).toContain("RUSTZEN_MONITOR_MULTI_NODE_TIMEOUT");
        expect(gate).toContain("RUSTZEN_MONITOR_MULTI_NODE_BUILD_TIMEOUT");
        expect(gate).toContain("RUSTZEN_MONITOR_MULTI_NODE_LOG_TIMEOUT");
        expect(gate).toContain("RUSTZEN_MONITOR_MULTI_NODE_DOCKER_INFO_TIMEOUT");
        expect(gate).toContain("admin-browser-source-identity.sh");
        expect(gate).toContain("atomic_replace_symlink");
        expect(gate).toContain('status=0\nrm -rf "$staged_bin_dir"');
        expect(gate).toContain("refusing to replace unsupported Monitor multi-node evidence directory");
        expect(gate).toContain("RUSTZEN_MONITOR_MULTI_NODE_TEST_PUBLISH_FAILURE");
        expect(gate).toContain("RUSTZEN_MONITOR_MULTI_NODE_TEST_SETUP_FAILURE");
        expect(gate).toContain("RUSTZEN_MONITOR_MULTI_NODE_TEST_BUILD_CLEANUP");
        expect(gate).toContain('--name "$build_container"');
        expect(gate).toContain('remove_container "$build_container"');
        expect(gate).toContain('remove_container "$container"');
        expect(justfile).toContain("verify-monitor-agent-multi-node-linux:");
        expect(justfile).toContain("scripts/verify-monitor-agent-multi-node-linux.sh");
    });

    test("requires two real service identities, readiness after delivery, gateway data, and restart recovery", () => {
        for (const value of ["rz-agent-a", "rz-agent-b", "RUSTZEN_RUNTIME_ROOT", "NOTIFY_SOCKET", "test ! -s \"$readiness\"", "/api/monitor/nodes", "metrics?bucket=raw", "nodesLatestFields", "stop_central", "thirdNodeRegistered:false"]) {
            expect(inner).toContain(value);
        }
        expect(inner).toContain('all(. == "READY=1")');
        expect(inner).toContain("length == 2");
        expect(readiness).toContain("socket.AF_UNIX");
        expect(readiness).toContain('event["payload"] == "READY=1"');
        expect(inner).toContain("( exec setpriv");
        expect(inner).toContain("central port did not quiesce");
        expect(inner).toContain("central process did not stop after TERM");
        expect(inner).not.toContain("central_env /opt/rz-central/rz-monitor controller >");
        expect(inner).toContain("declare -A points_before");
        expect(inner).toContain('points_before[$identity]=$(jq -er');
        expect(inner).toContain('before_points=${points_before[$identity]}');
        expect(inner).toContain('"linux-agent-a":{before:$beforeA,after:$afterA}');
        expect(inner).not.toContain('eval "points_$identity');
    });

    test("rejects invalid or hanging Docker discovery and cleans setup and signal paths", async () => {
        const fakeDocker = `/tmp/rz-monitor-multi-fake-${crypto.randomUUID()}`;
        await Bun.write(fakeDocker, "#!/bin/sh\nexec sleep 5\n");
        Bun.spawnSync(["chmod", "+x", fakeDocker]);
        expect(runGate({ RUSTZEN_MONITOR_MULTI_NODE_DOCKER: fakeDocker, RUSTZEN_MONITOR_MULTI_NODE_DOCKER_INFO_TIMEOUT: "1" }).exitCode).not.toBe(0);
        expect(runGate({ RUSTZEN_UI_LINUX_ARCH: "aarch64", RUSTZEN_MONITOR_MULTI_NODE_TIMEOUT: "0" }).exitCode).toBe(2);
        expect(runGate({ RUSTZEN_UI_LINUX_ARCH: "aarch64", RUSTZEN_MONITOR_MULTI_NODE_BUILD_TIMEOUT: "1801" }).exitCode).toBe(2);
        expect(runGate({ RUSTZEN_UI_LINUX_ARCH: "aarch64", RUSTZEN_MONITOR_MULTI_NODE_DOCKER_INFO_TIMEOUT: "61" }).exitCode).toBe(2);
        for (const [signal, code] of [["INT", 130], ["TERM", 143]]) {
            const root = `/tmp/rz-monitor-multi-signal-${crypto.randomUUID()}`;
            Bun.spawnSync(["mkdir", "-p", root]);
            expect(runGate({ RUSTZEN_MONITOR_MULTI_NODE_TEST_SIGNAL: signal, RUSTZEN_MONITOR_MULTI_NODE_TEST_ROOT: root }).exitCode).toBe(code);
            expect(Bun.spawnSync(["readlink", `${root}/current`]).stdout.toString().trim()).toBe("runs/old");
            expect(await Bun.file(`${root}/.verify.lock`).exists()).toBeFalse();
            Bun.spawnSync(["rm", "-rf", root]);
        }
        const setupRoot = `/tmp/rz-monitor-multi-setup-${crypto.randomUUID()}`;
        Bun.spawnSync(["mkdir", "-p", setupRoot]);
        const setup = runGate({ RUSTZEN_MONITOR_MULTI_NODE_TEST_SETUP_FAILURE: "1", RUSTZEN_MONITOR_MULTI_NODE_TEST_ROOT: setupRoot });
        expect(setup.exitCode).not.toBe(0);
        expect(new TextDecoder().decode(setup.stderr)).toContain("setup failure seam passed");
        expect(Bun.spawnSync(["readlink", `${setupRoot}/current`]).stdout.toString().trim()).toBe("runs/old");
        expect(await Bun.file(`${setupRoot}/.verify.lock`).exists()).toBeFalse();
        Bun.spawnSync(["rm", "-rf", setupRoot]);
        const publishRoot = `/tmp/rz-monitor-multi-publish-${crypto.randomUUID()}`;
        Bun.spawnSync(["mkdir", "-p", publishRoot]);
        const publication = runGate({ RUSTZEN_MONITOR_MULTI_NODE_TEST_PUBLISH_FAILURE: "1", RUSTZEN_MONITOR_MULTI_NODE_TEST_ROOT: publishRoot });
        expect(publication.exitCode).toBe(0);
        expect(new TextDecoder().decode(publication.stdout)).toContain("publication seam passed");
        expect(Bun.spawnSync(["readlink", `${publishRoot}/current`]).stdout.toString().trim()).toBe("runs/test");
        expect((await Bun.file(`${publishRoot}/current/manifest.json`).text()).trim()).toBe("new");
        expect(await Bun.file(`${publishRoot}/.missing`).exists()).toBeFalse();
        Bun.spawnSync(["rm", "-rf", publishRoot]);
        const buildFake = `/tmp/rz-monitor-multi-build-fake-${crypto.randomUUID()}`;
        const buildState = `${buildFake}.state`;
        const buildLog = `${buildFake}.log`;
        await Bun.write(buildFake, `#!/bin/sh
printf '%s\\n' "$*" >>"${buildLog}"
case "$1" in
  run) touch "${buildState}"; exit 137 ;;
  rm) rm -f "${buildState}" ;;
  ps) test ! -e "${buildState}" || printf 'rz-monitor-multi-node-build-test\\n' ;;
  *) exit 2 ;;
esac
`);
        Bun.spawnSync(["chmod", "+x", buildFake]);
        const buildCleanup = runGate({
            RUSTZEN_MONITOR_MULTI_NODE_DOCKER: buildFake,
            RUSTZEN_MONITOR_MULTI_NODE_TEST_BUILD_CLEANUP: "1",
        });
        expect(buildCleanup.exitCode).toBe(0);
        expect(await Bun.file(buildState).exists()).toBeFalse();
        const buildCalls = await Bun.file(buildLog).text();
        expect(buildCalls).toContain("run --name rz-monitor-multi-node-build-test");
        expect(buildCalls).toContain("rm -f rz-monitor-multi-node-build-test");
        expect(buildCalls).toContain("ps -a --filter name=^/rz-monitor-multi-node-build-test$");
        Bun.spawnSync(["rm", "-f", buildFake, buildLog]);
        const exitRoot = `/tmp/rz-monitor-multi-exit-${crypto.randomUUID()}`;
        const exitFake = `${exitRoot}.docker`;
        const exitLog = `${exitRoot}.log`;
        const exitBuildState = `${exitRoot}.build`;
        const exitRuntimeState = `${exitRoot}.runtime`;
        await Bun.write(exitBuildState, "live");
        await Bun.write(exitRuntimeState, "live");
        await Bun.write(exitFake, `#!/bin/sh
printf '%s\\n' "$*" >>"${exitLog}"
case "$1" in
  logs) trap '' TERM; sleep 30 ;;
  rm)
    case "$3" in
      rz-monitor-multi-node-build-exit-test) rm -f "${exitBuildState}" ;;
      rz-monitor-multi-node-exit-test) rm -f "${exitRuntimeState}" ;;
    esac ;;
  ps)
    case "$*" in
      *build-exit-test*) test ! -e "${exitBuildState}" || echo rz-monitor-multi-node-build-exit-test ;;
      *multi-node-exit-test*) test ! -e "${exitRuntimeState}" || echo rz-monitor-multi-node-exit-test ;;
    esac ;;
  *) exit 2 ;;
esac
`);
        Bun.spawnSync(["chmod", "+x", exitFake]);
        const started = Date.now();
        const exitCleanup = runGate({
            RUSTZEN_MONITOR_MULTI_NODE_DOCKER: exitFake,
            RUSTZEN_MONITOR_MULTI_NODE_LOG_TIMEOUT: "1",
            RUSTZEN_MONITOR_MULTI_NODE_TEST_EXIT_CLEANUP: "1",
            RUSTZEN_MONITOR_MULTI_NODE_TEST_ROOT: exitRoot,
        });
        expect(exitCleanup.exitCode).toBe(9);
        expect(Date.now() - started).toBeLessThan(15_000);
        expect(await Bun.file(exitBuildState).exists()).toBeFalse();
        expect(await Bun.file(exitRuntimeState).exists()).toBeFalse();
        expect(await Bun.file(`${exitRoot}/.candidate`).exists()).toBeFalse();
        expect(await Bun.file(`${exitRoot}/.binaries`).exists()).toBeFalse();
        expect(await Bun.file(`${exitRoot}/.verify.lock`).exists()).toBeFalse();
        expect(Bun.spawnSync(["readlink", `${exitRoot}/current`]).stdout.toString().trim()).toBe("runs/old");
        const exitCalls = await Bun.file(exitLog).text();
        expect(exitCalls.indexOf("rm -f rz-monitor-multi-node-build-exit-test")).toBeLessThan(exitCalls.indexOf("logs rz-monitor-multi-node-exit-test"));
        expect(exitCalls.indexOf("logs rz-monitor-multi-node-exit-test")).toBeLessThan(exitCalls.indexOf("rm -f rz-monitor-multi-node-exit-test"));
        Bun.spawnSync(["rm", "-rf", exitRoot]);
        Bun.spawnSync(["rm", "-f", exitFake, exitLog]);
        Bun.spawnSync(["rm", "-f", fakeDocker]);
    }, 20_000);
});
