import { expect, test } from "bun:test";
import { lstat, readFile, readlink, rm } from "node:fs/promises";

const gatePath = new URL("./verify-monitoring-ui-state-linux.sh", import.meta.url).pathname;
const absent = async (path) => {
    try {
        await lstat(path);
        return false;
    } catch (error) {
        if (error.code === "ENOENT") return true;
        throw error;
    }
};
const pidAlive = (pid) => {
    try {
        process.kill(pid, 0);
        return true;
    } catch {
        return false;
    }
};

test("blocking Docker cleanup preserves timeout and signal exits", async () => {
    for (const [mode, exitCode] of [
        ["124", 124],
        ["INT", 130],
        ["TERM", 143],
    ]) {
        const root = "/tmp/rz-monitoring-cleanup-" + crypto.randomUUID();
        const startedAt = Date.now();
        try {
            const result = Bun.spawnSync(["bash", gatePath], {
                env: {
                    ...process.env,
                    RUSTZEN_MONITORING_UI_STATE_TEST_BLOCKING_CLEANUP: mode,
                    RUSTZEN_MONITORING_UI_STATE_TEST_ROOT: root,
                },
                stdout: "pipe",
                stderr: "pipe",
            });
            expect(result.exitCode).toBe(exitCode);
            expect(Date.now() - startedAt).toBeLessThan(8_000);
            expect(await absent(root + "/.staged")).toBe(true);
            expect(await absent(root + "/.current-test")).toBe(true);
            expect(await absent(root + "/.verify.lock")).toBe(true);
            expect(await readlink(root + "/current")).toBe("runs/old");
            expect(await Bun.file(root + "/failed-runs/test/manifest.json").text()).toBe(
                "candidate",
            );
            expect(await Bun.file(root + "/fake-docker-logs.started").text()).toBe("started");
            expect(await Bun.file(root + "/fake-docker-rm.started").text()).toBe("started");
            const pids = (await readFile(root + "/fake-docker-pids", "utf8"))
                .trim()
                .split("\n")
                .map(Number);
            expect(pids.length).toBeGreaterThanOrEqual(1);
            expect(pids.every((pid) => !pidAlive(pid))).toBe(true);
        } finally {
            await rm(root, { recursive: true, force: true });
        }
    }
}, 30_000);
