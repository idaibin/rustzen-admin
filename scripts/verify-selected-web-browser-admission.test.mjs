import { expect, test } from "bun:test";
import { chmodSync, copyFileSync, mkdirSync, mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const result = Bun.spawnSync(["scripts/admin-browser-source-identity.sh"], { stdout: "pipe", stderr: "pipe" });

function run(command, args, cwd) {
    const result = Bun.spawnSync([command, ...args], { cwd, stdout: "pipe", stderr: "pipe" });
    expect(result.exitCode, new TextDecoder().decode(result.stderr)).toBe(0);
}

function identity(script, ...inputs) {
    const result = Bun.spawnSync([script, ...inputs], { stdout: "pipe", stderr: "pipe" });
    return { result, output: new TextDecoder().decode(result.stdout) };
}

function digest(output) {
    const match = /^([0-9a-f]{40})\t(clean|dirty)\t([0-9a-f]{64})\n?$/.exec(output);
    expect(match).not.toBeNull();
    return match[3];
}
test("normalizes the real Admin browser source-identity script output", () => {
    expect(result.exitCode).toBe(0);
    const match = /^([0-9a-f]{40})\t(clean|dirty)\t([0-9a-f]{64})\n?$/.exec(new TextDecoder().decode(result.stdout));
    expect(match).not.toBeNull();
    expect(`git:${match[1]} tree:${match[3]} state:${match[2]}`).toMatch(/^git:[0-9a-f]{40} tree:[0-9a-f]{64} state:(clean|dirty)$/);
});

test("binds optional ignored Web inputs without changing no-argument identity", () => {
    const root = mkdtempSync(join(tmpdir(), "rz-source-identity-"));
    try {
        mkdirSync(join(root, "scripts"), { recursive: true });
        const script = join(root, "scripts/admin-browser-source-identity.sh");
        copyFileSync("scripts/admin-browser-source-identity.sh", script);
        chmodSync(script, 0o755);
        writeFileSync(join(root, ".gitignore"), "apps/web/dist/\napps/admin/selected-web/\n");
        writeFileSync(join(root, "tracked.txt"), "tracked\n");
        run("git", ["init", "-q"], root);
        run("git", ["config", "user.email", "test@example.invalid"], root);
        run("git", ["config", "user.name", "Source Identity Test"], root);
        run("git", ["add", ".gitignore", "tracked.txt"], root);
        run("git", ["commit", "-qm", "fixture"], root);
        mkdirSync(join(root, "apps/web/dist"), { recursive: true });
        mkdirSync(join(root, "apps/admin/selected-web"), { recursive: true });
        writeFileSync(join(root, "apps/web/dist/index.html"), "dist one\n");
        writeFileSync(join(root, "apps/admin/selected-web/manifest.json"), "selected one\n");

        const plain = identity(script).output;
        const initial = digest(identity(script, "apps/web/dist", "apps/admin/selected-web").output);
        writeFileSync(join(root, "apps/web/dist/index.html"), "dist two\n");
        expect(identity(script).output).toBe(plain);
        const afterDist = digest(identity(script, "apps/web/dist", "apps/admin/selected-web").output);
        expect(afterDist).not.toBe(initial);
        writeFileSync(join(root, "apps/admin/selected-web/manifest.json"), "selected two\n");
        const afterSelected = digest(identity(script, "apps/web/dist", "apps/admin/selected-web").output);
        expect(afterSelected).not.toBe(afterDist);

        const missing = identity(script, "missing/input");
        expect(missing.result.exitCode).toBe(0);
        expect(digest(missing.output)).not.toBe(digest(plain));
        symlinkSync("../web/dist", join(root, "apps/admin/web-link"));
        for (const input of ["/tmp/outside", "apps/../tracked.txt", "apps/admin/web-link"]) {
            const invalid = identity(script, input);
            expect(invalid.result.exitCode, input).toBe(2);
        }
    } finally {
        rmSync(root, { recursive: true, force: true });
    }
});
