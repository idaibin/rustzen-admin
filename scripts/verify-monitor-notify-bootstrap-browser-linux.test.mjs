import { expect, test } from "bun:test";

const script = await Bun.file(new URL("./verify-monitor-notify-bootstrap-browser-linux.sh", import.meta.url)).text();

test("P8f notify bootstrap invokes only the reviewed notify tuple and cleans its retained container", () => {
    for (const value of [
        "distribution/fixtures/monitor-notify.json",
        "verify-monitor-native-runtime-linux-amd64.sh",
        "verify-selected-web-bootstrap-browser.py",
        "--selection distribution/fixtures/monitor-notify.json",
        "--retain-container-output",
        "--runtime-container",
        "cleanup-retained-p8e-container.sh --owner-token",
        "--native-output",
        "--browser-output",
    ]) expect(script).toContain(value);
    expect(script).not.toContain("prepare-monitor-load-runtime.sh");
    expect(script).not.toContain("verify-monitor-load-certification");
});
