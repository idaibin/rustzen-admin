import { expect, test } from "bun:test";
import { chmod, mkdir, mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const script = await Bun.file(
    resolve(import.meta.dir, "verify-monitor-native-runtime-linux-amd64.sh"),
).text();
const justfile = await Bun.file(resolve(import.meta.dir, "..", "justfile")).text();

test("just recipe forwards the explicit runtime tuple", () => {
    expect(justfile).toContain(
        "verify-monitor-native-runtime-amd64 selection export_root release_result certificate public_key expected_source_identity output:",
    );
    for (const value of [
        '--selection "{{selection}}"',
        '--export-root "{{export_root}}"',
        '--release-result "{{release_result}}"',
        '--certificate "{{certificate}}"',
        '--public-key "{{public_key}}"',
        '--expected-source-identity "{{expected_source_identity}}"',
        '--output "{{output}}"',
    ])
        expect(justfile).toContain(value);
});

test("P8e gate requires one explicit current tuple and fresh output", () => {
    for (const value of [
        "--selection",
        "--export-root",
        "--release-result",
        "--certificate",
        "--public-key",
        "--expected-source-identity",
        "--output",
        'test "$#" -eq 14',
        'test ! -e "$output"',
        'mkdir "$output"',
        "distribution-verify-published-source-build-certificate.ts",
        "--platform linux/amd64",
        "docker version --format",
        "uname -m",
        "cargo build --release -p rustzen-cli",
        "test ! -e /root/rz-p8e-dry",
        "activate-monitor-server",
        "notification-ingress.check",
        "RUSTZEN_NOTIFICATION_EVENT_KEY",
        "content-type: application/json",
        "x-rustzen-notify-version: 1",
        "x-rustzen-notify-key-id: p8e-unknown-v1",
        "x-rustzen-notify-producer: monitor",
        "x-rustzen-notify-created:",
        "x-rustzen-notify-expires:",
        "x-rustzen-notify-nonce: p8e-notify-probe-1",
        "x-rustzen-notify-signature: 0000000000000000000000000000000000000000000000000000000000000000",
        'test "$(cat /root/rz-activation/notification-ingress.status)" = 401',
        "notification-ingress.status",
        "notification-ingress.json",
        "monitor-native-runtime-evidence.json",
    ])
        expect(script).toContain(value);
    for (const forbidden of [
        "p8d-postcommit",
        'rm -rf "$output"',
        'mkdir -p "$output"',
        '"$root/$output:/output"',
        "distribution-installer-fixture.ts",
        "RUSTZEN_MONITOR_SERVER_PLATFORM",
        "RUSTZEN_P8E_CLI",
        "linux/arm64",
        "RUSTZEN_INSTALLER_",
        "tail -n 1",
    ])
        expect(script).not.toContain(forbidden);
});

test("P8e gate rejects reusable or out-of-bound evidence output", () => {
    expect(script).toContain('case "$output_parent" in "$root/target/rz"|"$root/target/rz"/*)');
    expect(script).toContain("--output must not exist");
    expect(script).toContain('test ! -L "$output"');
    expect(script.match(/--export-root/g)?.length).toBeGreaterThanOrEqual(2);
    for (const flag of ["--release-result", "--certificate", "--public-key", "--output"])
        expect(script.match(new RegExp(flag, "g"))?.length).toBeGreaterThanOrEqual(2);
    expect(
        script.indexOf("distribution-verify-published-source-build-certificate.ts"),
    ).toBeLessThan(script.indexOf('mkdir "$output"'));
    expect(script.indexOf('mkdir "$output"')).toBeLessThan(script.indexOf("docker version"));
});

test("overlap rejection is pre-Docker and leaves the input inventory unchanged", async () => {
    const target = resolve(import.meta.dir, "..", "target", "rz");
    await mkdir(target, { recursive: true });
    const input = await mkdtemp(resolve(target, "p8e-input-"));
    const bin = await mkdtemp(resolve(target, "p8e-bin-"));
    const marker = resolve(target, `p8e-docker-${process.pid}`);
    try {
        await mkdir(resolve(input, "certificate"));
        await writeFile(resolve(input, "release.json"), "{}\n");
        await writeFile(resolve(input, "certificate", "source-build-manifest.json"), "{}\n");
        await writeFile(resolve(input, "public.pem"), "key\n");
        await writeFile(
            resolve(bin, "docker"),
            `#!/bin/sh\ntouch ${JSON.stringify(marker)}\nexit 99\n`,
        );
        await chmod(resolve(bin, "docker"), 0o755);
        const before = await inventory(input);
        const result = Bun.spawnSync(
            [
                "bash",
                resolve(import.meta.dir, "verify-monitor-native-runtime-linux-amd64.sh"),
                "--export-root",
                input,
                "--release-result",
                resolve(input, "release.json"),
                "--certificate",
                resolve(input, "certificate", "source-build-manifest.json"),
                "--public-key",
                resolve(input, "public.pem"),
                "--expected-source-identity",
                "git:independent",
                "--output",
                resolve(input, "evidence"),
            ],
            {
                env: { ...process.env, PATH: `${bin}:${process.env.PATH}` },
                stdout: "pipe",
                stderr: "pipe",
            },
        );
        expect(result.exitCode).not.toBe(0);
        expect(await Bun.file(marker).exists()).toBeFalse();
        expect(await Bun.file(resolve(input, "evidence")).exists()).toBeFalse();
        expect(await inventory(input)).toEqual(before);
    } finally {
        await rm(input, { recursive: true, force: true });
        await rm(bin, { recursive: true, force: true });
        await rm(marker, { force: true });
    }
});

async function inventory(root: string) {
    const paths = (await readdir(root, { recursive: true })).sort();
    return Promise.all(
        paths.map(async (path) => {
            const file = resolve(root, path);
            return [
                path,
                (await Bun.file(file).exists()) && !path.endsWith("certificate")
                    ? new Bun.CryptoHasher("sha256").update(await readFile(file)).digest("hex")
                    : "directory",
            ];
        }),
    );
}

test("P8e gate uses the independent complete runtime revalidator", () =>
    expect(script).toContain("revalidateMonitorNativeRuntime"));

test("P8e gate proves the required positive and negative runtime checks", () => {
    for (const value of [
        "--json verify",
        "--json apply --dry-run",
        "--json apply --destination /opt/rz",
        "--json install-status",
        "publication-marker.json",
        "monitor-server-activation.json",
        "binaryDigests",
        "MainPID",
        "rustzen@123",
        "rz-insights.service",
        "rz-reports.service",
        "systemctl restart",
        "wait_health",
        "systemctl start rz-admin.service",
        "systemctl start rz-monitor.service",
    ])
        expect(script).toContain(value);
});
