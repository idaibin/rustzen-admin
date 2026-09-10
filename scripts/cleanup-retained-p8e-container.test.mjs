import { expect, test } from "bun:test";
import { chmodSync, cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
const cleanup = "scripts/cleanup-retained-p8e-container.sh";
function fake(ids, retainAfterRemove = false) {
    const dir = mkdtempSync(join(tmpdir(), "rz-clean-")), state = join(dir, "state"), log = join(dir, "log"), docker = join(dir, "docker");
    writeFileSync(state, ids);
    writeFileSync(docker, `#!/bin/sh\nprintf '%s\n' "$*" >> ${JSON.stringify(log)}\nif [ "$1" = ps ]; then cat ${JSON.stringify(state)}; elif [ "$1" = rm ] && [ ${retainAfterRemove ? 1 : 0} = 0 ]; then : > ${JSON.stringify(state)}; fi\n`); chmodSync(docker, 0o755);
    return { dir, log, run: () => Bun.spawnSync(["bash", cleanup, "--owner-token", "t"], { env: { ...process.env, PATH: `${dir}:${process.env.PATH}` }, stdout: "pipe", stderr: "pipe" }) };
}
test("retained cleanup accepts none, removes one then rereads absence, and rejects duplicates", () => {
    for (const [ids, code] of [["", 0], ["one\n", 0], ["one\ntwo\n", 1]]) { const item = fake(ids); try { expect(item.run().exitCode).toBe(code); if (ids === "one\n") { const calls=readFileSync(item.log, "utf8").trim().split("\n"); expect(calls.filter(x=>x.includes("ps -aq --filter label=io.rustzen.p8g-owner=t"))).toHaveLength(2); expect(calls.filter(x=>x.startsWith("rm -f one"))).toHaveLength(1); } } finally { rmSync(item.dir, { recursive: true, force: true }); } }
});
test("retained cleanup fails when Docker reports success but the owner container remains", () => {
    const item = fake("one\n", true);
    try { expect(item.run().exitCode).not.toBe(0); const calls=readFileSync(item.log, "utf8").trim().split("\n"); expect(calls.filter(x=>x.includes("ps -aq --filter label=io.rustzen.p8g-owner=t"))).toHaveLength(2); expect(calls.filter(x=>x.startsWith("rm -f one"))).toHaveLength(1); }
    finally { rmSync(item.dir, { recursive: true, force: true }); }
});
test("notify wrapper cleans by owner label when native verifier leaves a corrupt context", () => {
    const root = mkdtempSync(join(tmpdir(), "rz-wrapper-")), scripts = join(root, "scripts"), bin = join(root, "bin"), state = join(root, "docker-state"), log = join(root, "docker-log");
    try {
        cpSync("scripts", scripts, { recursive: true }); mkdirSync(bin); mkdirSync(join(root, "target", "rz"), { recursive: true }); cpSync(cleanup, join(scripts, "cleanup-retained-p8e-container.sh"));
        writeFileSync(join(scripts, "verify-monitor-native-runtime-linux-amd64.sh"), '#!/bin/sh\nwhile [ "$#" -gt 0 ]; do [ "$1" = --retain-container-output ] && { printf "{" > "$2"; exit 0; }; shift; done\n'); chmodSync(join(scripts, "verify-monitor-native-runtime-linux-amd64.sh"), 0o755);
        writeFileSync(join(bin, "pnpm"), '#!/bin/sh\n[ "$1" = dlx ] && { echo token; exit 0; }\n'); chmodSync(join(bin, "pnpm"), 0o755);
        writeFileSync(state, "retained\n");
        writeFileSync(join(bin, "docker"), `#!/bin/sh\nif [ "$1" = ps ]; then cat ${JSON.stringify(state)}; elif [ "$1" = rm ]; then echo "$3" >> ${JSON.stringify(log)}; : > ${JSON.stringify(state)}; fi\n`); chmodSync(join(bin, "docker"), 0o755);
        const out = Bun.spawnSync(["bash", join(scripts, "verify-monitor-notify-bootstrap-browser-linux.sh"), "--export-root", root, "--release-result", join(root,"r"), "--certificate", join(root,"c"), "--public-key", join(root,"p"), "--expected-source-identity", "x", "--native-output", join(root,"n"), "--browser-output", join(root,"b")], { cwd: root, env: { ...process.env, PATH: `${bin}:${process.env.PATH}` }, stdout: "pipe", stderr: "pipe" });
        expect(out.exitCode).not.toBe(0);
        expect(readFileSync(log, "utf8")).toBe("retained\n");
        expect(readFileSync(state, "utf8")).toBe("");
    } finally { rmSync(root, { recursive: true, force: true }); }
});
