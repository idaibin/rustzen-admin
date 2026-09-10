import { expect, test } from "bun:test";
import { chmodSync, cpSync, mkdtempSync, mkdirSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { tmpdir } from "node:os";
import { preparedContext } from "./prepared-monitor-load-context.ts";

const prepare = "scripts/prepare-monitor-load-runtime.sh", cleanup = "scripts/cleanup-monitor-load-runtime.sh", owner = "a".repeat(32);
test("prepare canonicalizes relative context credentials before preflight and owned cleanup", async () => {
    const root = realpathSync(mkdtempSync(join(tmpdir(), "rz-p8g-producer-"))), scripts = join(root, "scripts"), bin = join(root, "bin"), output = join(root, "output"), context = join(output, "context.json"), state = join(root, "state"), log = join(root, "docker.log");
    try {
        mkdirSync(scripts); mkdirSync(bin); mkdirSync(output); mkdirSync(join(root, "target", "rz"), { recursive: true }); mkdirSync(join(root, "export", "release", "server", "bin"), { recursive: true });
        cpSync(prepare, join(scripts, "prepare-monitor-load-runtime.sh")); cpSync("distribution", join(root, "distribution"), { recursive: true }); writeFileSync(join(root, "export", "release", "server", "bin", "rz-admin"), "admin");
        writeFileSync(join(scripts, "verify-monitor-native-runtime-linux-amd64.sh"), `#!/bin/sh\nwhile [ "$#" -gt 0 ]; do [ "$1" = --retain-container-output ] && { out="$2"; break; }; shift; done\nprintf '{"containerId":"container-id","containerName":"container","containerPort":19801,"hostPort":23123,"imageId":"image","nativeEvidence":"%s.native","ownerToken":"${owner}"}' "$out" > "$out"\nprintf '{"selection":{"buildId":"build","compositionId":"composition"}}' > "$out.native"\n`); chmodSync(join(scripts, "verify-monitor-native-runtime-linux-amd64.sh"), 0o755);
        writeFileSync(join(scripts, "verify-selected-web-bootstrap-browser.py"), `#!/bin/sh\nwhile [ "$#" -gt 0 ]; do [ "$1" = --output ] && { mkdir "$2"; printf '{}' > "$2/manifest.json"; exit 0; }; shift; done\nexit 1\n`); chmodSync(join(scripts, "verify-selected-web-bootstrap-browser.py"), 0o755);
        writeFileSync(join(bin, "curl"), '#!/bin/sh\nprintf \'{"status":"ok","selectedBinding":{"buildId":"build","compositionId":"composition"}}\'\n'); chmodSync(join(bin, "curl"), 0o755);
        writeFileSync(join(bin, "docker"), `#!/bin/sh\nprintf '%s\\n' "$*" >> ${JSON.stringify(log)}\nif [ "$1" = inspect ] && [ -s ${JSON.stringify(state)} ]; then [ "$3" = '{{.Id}}' ] && echo container-id || echo ${owner}; elif [ "$1" = rm ]; then : > ${JSON.stringify(state)}; fi\n`); chmodSync(join(bin, "docker"), 0o755);
        for (const path of ["release", "certificate", "public-key"]) writeFileSync(join(root, path), "x");
        const result = Bun.spawnSync(["bash", join(scripts, "prepare-monitor-load-runtime.sh"), "--export-root", "export", "--release-result", "release", "--certificate", "certificate", "--public-key", "public-key", "--expected-source-identity", "source", "--native-output", "native", "--browser-output", "browser", "--context-output", relative(root, context), "--chromium", "chromium"], { cwd: root, env: { ...process.env, PATH: `${bin}:${process.env.PATH}` }, stdout: "pipe", stderr: "pipe" });
        expect(result.exitCode).toBe(0); const value = await preparedContext(context); expect(value.passwordFile).toBe(`${resolve(context)}.password`); expect(value.agentTokenFile).toBe(`${resolve(context)}.agent-token`);
        writeFileSync(state, "active"); const removed = Bun.spawnSync(["bash", cleanup, context], { env: { ...process.env, PATH: `${bin}:${process.env.PATH}` }, stdout: "pipe", stderr: "pipe" });
        expect(removed.exitCode).toBe(0); expect(readFileSync(state, "utf8")).toBe(""); expect(readFileSync(log, "utf8").split("\n").filter(line => line.startsWith("rm -f container"))).toHaveLength(1);
        expect(Bun.file(context).size).toBe(0); expect(Bun.file(`${context}.password`).size).toBe(0); expect(Bun.file(`${context}.agent-token`).size).toBe(0);
    } finally { rmSync(root, { recursive: true, force: true }); }
});
