import { expect, test } from "bun:test";
import { resolve } from "node:path";

const script = await Bun.file(resolve(import.meta.dir, "verify-monitor-native-runtime-linux-amd64.sh")).text();

test("P8e gate is pinned to the P8d artifacts and native amd64 PID1", () => {
    for (const value of [
        "p8d-postcommit-export-path.txt", "p8d-postcommit-release-result-path.txt",
        "p8d-postcommit-certificate-result-path.txt", "p8d-postcommit-key-path.txt",
        "distribution-verify-published-source-build-certificate.ts", "--platform linux/amd64",
        "docker version --format", "uname -m", "cargo build --release -p rustzen-cli", "test ! -e /root/rz-p8e-dry", "activate-monitor-server", "monitor-native-runtime-evidence.json",
    ]) expect(script).toContain(value);
    for (const forbidden of ["distribution-installer-fixture.ts", "RUSTZEN_MONITOR_SERVER_PLATFORM", "RUSTZEN_P8E_CLI", "linux/arm64", "RUSTZEN_INSTALLER_", "tail -n 1"])
        expect(script).not.toContain(forbidden);
});

test("P8e gate uses the independent complete runtime revalidator", () => expect(script).toContain("revalidateMonitorNativeRuntime"));

test("P8e gate proves the required positive and negative runtime checks", () => {
    for (const value of [
        "--json verify", "--json apply --dry-run", "--json apply --destination /opt/rz",
        "--json install-status", "publication-marker.json", "monitor-server-activation.json", "binaryDigests",
        "MainPID", "rustzen@123", "rz-insights.service", "rz-reports.service",
        "systemctl restart", "wait_health", "systemctl start rz-admin.service", "systemctl start rz-monitor.service",
    ]) expect(script).toContain(value);
});
