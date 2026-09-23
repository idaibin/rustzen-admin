import { expect, test } from "bun:test";

const script = "scripts/verify-monitor-notify-business-browser-workflow-linux.sh";
const complete = ["--export-root", "x", "--release-result", "x", "--certificate", "x", "--public-key", "x", "--expected-source-identity", "x", "--native-output", "x", "--bootstrap-output", "x", "--output", "x"];
const exit = args => Bun.spawnSync(["bash", script, ...args], { stdout: "pipe", stderr: "pipe" }).exitCode;

test("P8f-B workflow rejects missing, duplicate, and unknown arguments before execution", () => {
    expect(exit(complete.slice(0, -2))).toBe(2);
    expect(exit([...complete.slice(0, -2), "--export-root", "other"])).toBe(2);
    expect(exit([...complete.slice(0, -2), "--unknown", "other"])).toBe(2);
    expect(exit([...complete, "--chromium", "a", "--chromium", "b"])).toBe(2);
});
