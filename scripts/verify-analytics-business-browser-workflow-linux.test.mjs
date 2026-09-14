import { expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const workflow = await readFile(resolve(import.meta.dir, "verify-analytics-business-browser-workflow-linux.sh"), "utf8");
const wrapper = await readFile(resolve(import.meta.dir, "verify-analytics-business-browser-linux.sh"), "utf8");
const driver = await readFile(resolve(import.meta.dir, "analytics-business-browser-driver.ts"), "utf8");
const justfile = await readFile(resolve(import.meta.dir, "..", "justfile"), "utf8");

test("just recipe forwards the explicit P8f workflow tuple", () => {
    expect(justfile).toContain("verify-analytics-business-browser-linux export_root release_result certificate public_key expected_source_identity runtime_evidence native_output output:");
    for (const value of [
        '--export-root "{{export_root}}"',
        '--release-result "{{release_result}}"',
        '--certificate "{{certificate}}"',
        '--public-key "{{public_key}}"',
        '--expected-source-identity "{{expected_source_identity}}"',
        '--runtime-evidence "{{runtime_evidence}}"',
        '--native-output "{{native_output}}"',
        '--output "{{output}}"',
    ])
        expect(justfile).toContain(value);
});

test("workflow reuses one retained P8e container and always cleans it up", () => {
    for (const value of [
        "verify-analytics-native-runtime-linux-amd64.sh",
        "--retain-container-output",
        "--retain-owner-token",
        "cleanup-retained-p8e-container.sh --owner-token",
        "trap cleanup EXIT INT TERM",
        "verify-analytics-business-browser-linux.sh",
        "runtimeEvidenceSha256",
    ])
        expect(workflow).toContain(value);
    for (const forbidden of ["docker rm -f $container", "rm -rf \"$output\""])
        expect(workflow).not.toContain(forbidden);
});

test("workflow restart watcher only restarts the selected analytics services", () => {
    expect(workflow).toContain("systemctl restart rz-insights.service rz-admin.service");
    expect(workflow).toContain("restart-requested");
    expect(workflow).toContain("restart-done");
    expect(workflow).not.toContain("rz-monitor.service");
});

test("wrapper pins the evidence allowlist and verifies the receipt bytes", () => {
    for (const value of [
        "analytics-business-browser-driver.ts",
        "analytics-business-browser-receipt.ts \"$candidate/receipt.json\"",
        "overview-populated.png receipt.json",
        "P8f output allowlist differs",
        "--remote-debugging-port=0",
        "systemctl is-active --quiet rz-insights.service",
    ])
        expect(wrapper).toContain(value);
});

test("driver keeps the deployment identity, redacts secrets, and fails closed", () => {
    for (const value of [
        "selectedBinding",
        "collectionEnabled: true",
        "x-rustzen-project-key",
        "Unable to read analytics data",
        "rustzen-admin-theme",
        "rustzen-admin-locale",
        "throw Error(",
    ])
        expect(driver).toContain(value);
    // The owner credential must arrive only through the workflow's private
    // password file; the shipped default password is a public probe input.
    expect(driver).not.toContain("p8e-owner-password");
    if (!driver.includes("context.passwordFile")) throw Error("driver lost its credential input");
});
