import { expect, test } from "bun:test";

const script = await Bun.file(new URL("./verify-reports-linux.sh", import.meta.url)).text();

test("Reports Linux verifier keeps production event, IPC, and credential keys separate", () => {
    const eventKey = script.match(/^  EVENT_KEY=(\S+)$/m)?.[1];
    const launches = [...script.matchAll(/^  setpriv .*\/opt\/rz\/current\/bin\/rz-reports serve.*$/gm)];
    const launch = launches[0]?.[0] ?? "";
    const ipcKey = launch.match(/RUSTZEN_IPC_TOKEN=(\S+)/)?.[1];
    const credentialKey = launch.match(/RUSTZEN_REPORTS_CREDENTIAL_KEY=(\S+)/)?.[1];

    expect(launches).toHaveLength(1);
    expect(eventKey).toBeDefined();
    expect(eventKey.length).toBeGreaterThanOrEqual(32);
    expect(ipcKey).toBeDefined();
    expect(credentialKey).toBeDefined();
    expect(ipcKey).not.toBe(credentialKey);
    expect(eventKey).not.toBe(ipcKey);
    expect(eventKey).not.toBe(credentialKey);
    expect(launch).toContain("RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY_ID=reports-linux-v1");
    expect(launch).toContain('RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY="$EVENT_KEY"');
});

test("Reports Linux verifier installs CA roots through its sole package command", () => {
    const installs = [...script.matchAll(/^  apt-get install -y --no-install-recommends (.*)$/gm)];

    expect(installs).toHaveLength(1);
    expect(installs[0]?.[1].split(" ")).toContain("ca-certificates");
});
