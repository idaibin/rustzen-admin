import { test, expect } from "bun:test";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const source = new URL("./local-acceptance-launch.sh", import.meta.url);
function fixture() {
    const dir = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "rz-launch-test-")));
    const repo = path.join(dir, "repo");
    const temp = path.join(dir, "temp");
    fs.mkdirSync(repo + "/scripts", { recursive: true });
    fs.mkdirSync(temp);
    const script = repo + "/scripts/launch.sh";
    fs.copyFileSync(source, script);
    const env = { PATH: "/usr/bin:/bin", HOME: dir, TMPDIR: temp, LANG: "C", TZ: "UTC" };
    const args = (root, command = ["/usr/bin/true"], options = []) =>
        [script, "--isolation-root", root, ...options, "--", ...command];
    return {
        dir, repo, temp, script, env, args,
        run(root, command, options, overrides = {}) {
            return spawnSync("/bin/bash", args(root, command, options), {
                env: { ...env, ...overrides }, cwd: dir, encoding: "utf8",
            });
        },
        close() { fs.rmSync(dir, { recursive: true, force: true }); },
    };
}

test("temp root works without repository build outputs and child receives isolated values", () => {
    const f = fixture();
    try {
        const root = f.temp + "/run";
        const r = f.run(root, [process.execPath, "-e", "console.log(JSON.stringify(process.env))"], ["--port", "19801"]);
        expect(r.status).toBe(0);
        const env = JSON.parse(r.stdout);
        expect(env.RUSTZEN_RUNTIME_ROOT).toBe(root);
        expect(env.RUSTZEN_SKIP_DOTENV).toBe("1");
        expect(env.RUSTZEN_ADMIN_PORT).toBe("19801");
        expect(env.RUSTZEN_ADMIN_SQLITE_PATH).toBeUndefined();
        expect(fs.existsSync(f.repo + "/target")).toBe(false);
    } finally { f.close(); }
});

test("existing repo base works even when the unrelated temp base is absent", () => {
    const f = fixture();
    try {
        fs.mkdirSync(f.repo + "/target/rz", { recursive: true });
        expect(f.run(f.repo + "/target/rz/run", undefined, undefined, { TMPDIR: f.dir + "/absent" }).status).toBe(0);
    } finally { f.close(); }
});

test("path and environment rejections do not create roots or change existing data", () => {
    const f = fixture();
    try {
        const existing = f.temp + "/existing";
        fs.mkdirSync(existing);
        fs.writeFileSync(existing + "/sentinel", "keep");
        fs.symlinkSync(f.temp, f.temp + "/link");
        const cases = [
            [""], ["relative"], [f.dir + "/outside"], [f.temp + "/../escape"],
            [f.temp + "/.rustzen-admin"], [f.temp + "/link/run"], [existing],
            [f.temp + "/bad-port", undefined, ["--port", "80"]],
        ];
        for (const name of ["RUSTZEN_RUNTIME_ROOT", "RUSTZEN_ADMIN_SQLITE_PATH", "RUSTZEN_MONITOR_SQLITE_PATH", "RUSTZEN_INSIGHTS_SQLITE_PATH", "RUSTZEN_REPORTS_SQLITE_PATH"]) {
            cases.push([f.temp + "/env-" + name, undefined, undefined, { [name]: "" }]);
        }
        for (const input of cases) {
            expect(f.run(...input).status).toBe(2);
            if (input[0] !== existing) expect(fs.existsSync(input[0])).toBe(false);
        }
        expect(fs.readFileSync(existing + "/sentinel", "utf8")).toBe("keep");
    } finally { f.close(); }
});

test("child failure and exec failure retain owned evidence and forbid reuse", () => {
    const f = fixture();
    try {
        for (const [name, cmd, status] of [["failure", ["/bin/sh", "-c", "exit 7"], 7], ["missing", [f.dir + "/missing-command"], 127]]) {
            const root = f.temp + "/" + name;
            expect(f.run(root, cmd).status).toBe(status);
            const before = fs.readFileSync(root + "/LAUNCH-RECEIPT.txt", "utf8");
            expect(f.run(root).status).toBe(2);
            expect(fs.readFileSync(root + "/LAUNCH-RECEIPT.txt", "utf8")).toBe(before);
        }
    } finally { f.close(); }
});

test("partial directory creation failure retains root and never executes child", () => {
    const f = fixture();
    try {
        const bin = f.dir + "/bin";
        fs.mkdirSync(bin);
        fs.writeFileSync(bin + "/mkdir", '#!/bin/sh\nif [ "$#" -gt 1 ]; then exit 9; fi\nexec /bin/mkdir "$@"\n', { mode: 0o755 });
        const root = f.temp + "/partial";
        const child = ["/bin/sh", "-c", 'echo child > "$RUSTZEN_RUNTIME_ROOT/child"'];
        expect(f.run(root, child, undefined, { PATH: bin + ":/usr/bin:/bin" }).status).toBe(9);
        expect(fs.existsSync(root)).toBe(true);
        expect(fs.existsSync(root + "/child")).toBe(false);
        expect(fs.existsSync(root + "/LAUNCH-RECEIPT.txt")).toBe(false);
        expect(f.run(root).status).toBe(2);
    } finally { f.close(); }
});

function completion(child) {
    return new Promise((resolve, reject) => {
        child.once("error", reject);
        child.once("close", (code, signal) => resolve({ code, signal }));
    });
}

test("cancellation retains the owned root and concurrent admission has one winner", async () => {
    const f = fixture();
    const children = [];
    try {
        const root = f.temp + "/cancel";
        const command = [process.execPath, "-e", 'require("fs").writeFileSync(process.env.RUSTZEN_RUNTIME_ROOT+"/ready", "ready"); setInterval(()=>{},1000)'];
        const child = spawn("/bin/bash", f.args(root, command), { env: f.env, cwd: f.dir, stdio: "ignore" });
        children.push(child);
        const done = completion(child);
        for (let i = 0; i < 200 && !fs.existsSync(root + "/ready"); i++) await Bun.sleep(10);
        expect(fs.existsSync(root + "/ready")).toBe(true);
        child.kill("SIGTERM");
        expect((await done).signal).toBe("SIGTERM");
        expect(fs.existsSync(root + "/LAUNCH-RECEIPT.txt")).toBe(true);
        expect(f.run(root).status).toBe(2);
        const shared = f.temp + "/concurrent";
        const jobs = [1, 2].map(() => {
            const c = spawn("/bin/bash", f.args(shared), { env: f.env, cwd: f.dir, stdio: "ignore" });
            children.push(c);
            return completion(c);
        });
        const results = await Promise.all(jobs);
        expect(results.filter(r => r.code === 0).length).toBe(1);
        expect(results.filter(r => r.code !== 0).length).toBe(1);
    } finally {
        for (const c of children) if (c.exitCode === null && c.signalCode === null) c.kill("SIGKILL");
        f.close();
    }
});
