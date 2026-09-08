import { expect, test } from "bun:test";

const result = Bun.spawnSync(["scripts/admin-browser-source-identity.sh"], { stdout: "pipe", stderr: "pipe" });
test("normalizes the real Admin browser source-identity script output", () => {
    expect(result.exitCode).toBe(0);
    const match = /^([0-9a-f]{40})\t(clean|dirty)\t([0-9a-f]{64})\n?$/.exec(new TextDecoder().decode(result.stdout));
    expect(match).not.toBeNull();
    expect(`git:${match[1]} tree:${match[3]} state:${match[2]}`).toMatch(/^git:[0-9a-f]{40} tree:[0-9a-f]{64} state:(clean|dirty)$/);
});
