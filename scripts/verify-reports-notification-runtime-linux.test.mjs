import { describe, expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { join } from "node:path";
import {
    fakeGateFixture,
    runFakeGate,
    runSignaledFakeGate,
} from "./verify-reports-notification-runtime-linux.test-support.mjs";

const directory = new URL("./", import.meta.url);
const outerUrl = new URL("verify-reports-notification-runtime-linux.sh", directory);
const innerUrl = new URL("verify-reports-notification-runtime-linux-inner.sh", directory);
const clientUrl = new URL("reports-notification-runtime-client.py", directory);
const outer = await Bun.file(outerUrl).text();
const inner = await Bun.file(innerUrl).text();
const client = await Bun.file(clientUrl).text();
const evidenceValidator = await Bun.file(
    new URL("verify-reports-notification-runtime-evidence.jq", directory),
).text();
const verifier = await Bun.file(new URL("admin-browser-verifier.Dockerfile", directory)).text();
const justfile = await Bun.file(new URL("../justfile", directory)).text();
const validation = await Bun.file(
    new URL("../docs/product/features/composable-distribution/validation.md", directory),
).text();

function lines(text) {
    return text.trimEnd().split("\n").length;
}

function hasBlockingBuildTargetLock(script) {
    const build = script.slice(
        script.indexOf('if run_bounded "$build_timeout"'),
        script.indexOf('\n  "\nthen :; else'),
    );
    const locks = [...build.matchAll(/exec\s+\d+>(\/cargo-target\/\S+)/g)].map((match) => match[1]);
    const flockLines = build.match(/^\s*flock\b.*$/gm) ?? [];
    const lock = build.indexOf("exec 9>/cargo-target/.gate.lock");
    const flock = build.indexOf("flock -x 9");
    const builds = build.match(/cargo build --release --target \$target_triple/g) ?? [];
    const installs = build.match(/install -m 0755/g) ?? [];
    const firstBuild = build.indexOf("cargo build --release --target $target_triple");
    const finalInstall = build.lastIndexOf("rz-reports-pure");
    return locks.length === 1
        && locks[0] === "/cargo-target/.gate.lock"
        && flockLines.length === 1
        && flockLines[0].trim() === "flock -x 9"
        && !build.includes("flock -n")
        && lock > -1
        && flock > lock
        && flock < firstBuild
        && builds.length === 4
        && installs.length === 4
        && finalInstall > firstBuild;
}

function hasFeatureIsolatedTargets(script) {
    const build = script.slice(
        script.indexOf('if run_bounded "$build_timeout"'),
        script.indexOf('\n  "\nthen :; else'),
    );
    const selected = '--target-dir \\"\\$selected_target\\"';
    const pure = '--target-dir \\"\\$pure_target\\"';
    const pairs = [
        'cargo build --release --target $target_triple --target-dir \\"\\$selected_target\\" -p rustzen-admin --bin rz-admin\n    install -m 0755 \\"\\$selected_target/$target_triple/release/rz-admin\\" \\"\\$out/rz-admin-selected\\"',
        'cargo build --release --target $target_triple --target-dir \\"\\$selected_target\\" -p rustzen-reports --bin rz-reports\n    install -m 0755 \\"\\$selected_target/$target_triple/release/rz-reports\\" \\"\\$out/rz-reports-selected\\"',
        'cargo build --release --target $target_triple --target-dir \\"\\$pure_target\\" -p rustzen-admin --no-default-features --features monitor-distribution --bin rz-admin\n    install -m 0755 \\"\\$pure_target/$target_triple/release/rz-admin\\" \\"\\$out/rz-admin-pure\\"',
        'cargo build --release --target $target_triple --target-dir \\"\\$pure_target\\" -p rustzen-reports --no-default-features --bin rz-reports\n    install -m 0755 \\"\\$pure_target/$target_triple/release/rz-reports\\" \\"\\$out/rz-reports-pure\\"',
    ];
    return build.includes("selected_target=/cargo-target")
        && build.includes("pure_target=/cargo-target/pure-v1")
        && build.split(selected).length === 3
        && build.split(pure).length === 3
        && build.indexOf(selected) < build.indexOf(pure)
        && pairs.every((pair) => build.includes(pair));
}

function hasStagedBuildSource(script) {
    const build = script.slice(
        script.indexOf('if run_bounded "$build_timeout"'),
        script.indexOf('\n  "\nthen :; else'),
    );
    const aptUpdate = build.indexOf("apt-get update");
    const install = build.indexOf("apt-get install -y --no-install-recommends musl-tools util-linux");
    const rustup = build.indexOf("rustup target add $target_triple");
    const lock = build.indexOf("flock -x 9");
    const copy = build.indexOf("tar -C /source");
    const workdir = build.indexOf("cd /work");
    const cargo = build.indexOf("cargo build --release --target $target_triple");
    return build.includes('--mount "type=bind,src=$root,dst=/source,readonly"')
        && build.includes("--tmpfs /work:rw,size=512m")
        && !build.includes('--mount "type=bind,src=$root,dst=/work"')
        && build.includes("-cf - Cargo.toml Cargo.lock rust-toolchain.toml apps crates | tar -C /work -xf -")
        && build.includes("--exclude='target'")
        && build.includes("--exclude='.git'")
        && build.includes("--exclude='apps/server'")
        && build.includes("--exclude='apps/web/node_modules'")
        && build.includes("--exclude='apps/web/.selected-web'")
        && build.includes("--exclude='apps/web/.vite'")
        && build.includes("--exclude='apps/web/dist-*.zip'")
        && !build.includes("--exclude='apps/web/dist'")
        && !build.includes("--exclude='apps/admin/selected-web'")
        && aptUpdate > -1
        && install > aptUpdate
        && rustup > install
        && lock > rustup
        && lock > -1
        && copy > lock
        && workdir > copy
        && cargo > workdir;
}

describe("Reports notification Linux runtime gate", () => {
    test("is bounded, source-bound, atomically published, and compact", () => {
        expect(lines(outer)).toBeLessThan(300);
        expect(lines(inner)).toBeLessThan(300);
        expect(lines(client)).toBeLessThan(300);
        for (const value of [
            "RUSTZEN_REPORTS_NOTIFY_BUILD_TIMEOUT",
            "RUSTZEN_REPORTS_NOTIFY_RUNTIME_TIMEOUT",
            "admin-browser-source-identity.sh",
            "sourceTreeSha256",
            "atomic_replace_symlink",
            "failed-runs",
            "stop_tree",
            "receipt_allowlist",
            "remove_container",
            "container inspect",
        ]) {
            expect(outer).toContain(value);
        }
        expect(justfile).toContain("verify-reports-notification-runtime-linux:");
        expect(justfile).toContain("scripts/verify-reports-notification-runtime-linux.sh");
        expect(evidenceValidator).toContain('length==25');
        expect(evidenceValidator).toContain('test("^[A-Za-z0-9][A-Za-z0-9._-]*$")');
        expect(outer).not.toContain("RUSTZEN_REPORTS_NOTIFY_TIMEOUT");
        expect(outer).toContain('validate_seconds "$build_timeout" RUSTZEN_REPORTS_NOTIFY_BUILD_TIMEOUT 7200');
        expect(outer).toContain('validate_seconds "$runtime_timeout" RUSTZEN_REPORTS_NOTIFY_RUNTIME_TIMEOUT 1800');
        expect(outer).toContain('source_inputs=(apps/web/dist apps/admin/selected-web)');
        expect(outer.match(/"\$source_identity" "\$\{source_inputs\[@\]\}"/g)).toHaveLength(2);
        expect(outer).toContain('if run_bounded "$build_timeout" "$docker_bin" run --name "$build_container"');
        expect(outer).toContain('if run_bounded "$runtime_timeout" "$docker_bin" run --name "$runtime_container"');
        expect(outer).toContain('target_cache_schema=v1');
        expect(outer).toContain('rustzen-reports-notify-target-${target_cache_schema}-${architecture}-${target_triple}-rust195-crtstatic');
        expect(outer).toContain('--mount "type=volume,src=$target_cache,dst=/cargo-target"');
        expect(outer).toContain('--mount "type=bind,src=$staged,dst=/out"');
        expect(outer).toContain('out=/out');
        expect(outer).toContain('selected_target=/cargo-target');
        expect(outer).toContain('pure_target=/cargo-target/pure-v1');
        expect(outer).not.toContain('target/reports-notification-runtime/cargo');
        expect(outer).toContain('apt-get install -y --no-install-recommends musl-tools util-linux');
        expect(hasBlockingBuildTargetLock(outer)).toBeTrue();
        expect(hasFeatureIsolatedTargets(outer)).toBeTrue();
        expect(hasStagedBuildSource(outer)).toBeTrue();
        expect(outer).toContain('binary="$staged/$name"');
        expect(outer).toContain('shasum -a 256 "$staged/$name"');
    });

    test("rejects missing, nonblocking, or redirected build-target locks", () => {
        for (const mutated of [
            outer.replace("flock -x 9", ""),
            outer.replace("flock -x 9", "flock -n -x 9"),
            outer.replace("/cargo-target/.gate.lock", "/cargo-target/other.lock"),
        ]) expect(hasBlockingBuildTargetLock(mutated)).toBeFalse();
    });

    test("rejects shared selected/pure targets or split pure targets", () => {
        for (const mutated of [
            outer.replace("pure_target=/cargo-target/pure-v1", "pure_target=/cargo-target"),
            outer.replace('\\"\\$pure_target\\" -p rustzen-reports --no-default-features', '\\"\\$selected_target\\" -p rustzen-reports --no-default-features'),
            outer.replace('\\"\\$selected_target\\" -p rustzen-reports --bin rz-reports', '\\"\\$pure_target\\" -p rustzen-reports --bin rz-reports'),
            outer.replace('\\"\\$pure_target\\" -p rustzen-admin --no-default-features', '\\"\\$selected_target\\" -p rustzen-admin --no-default-features'),
        ]) expect(hasFeatureIsolatedTargets(mutated)).toBeFalse();
    });

    test("rejects host-source builds or excluded required web artifacts", () => {
        for (const mutated of [
            outer.replace("dst=/source,readonly", "dst=/work"),
            outer.replace("--tmpfs /work:rw,size=512m", ""),
            outer.replace("--exclude='apps/web/.selected-web'", "--exclude='apps/web/dist'"),
            outer.replace("--exclude='apps/server'", ""),
            outer.replace("--exclude='apps/web/.vite'", ""),
            outer.replace("--exclude='apps/web/dist-*.zip'", ""),
        ]) expect(hasStagedBuildSource(mutated)).toBeFalse();
    });

    test("builds positive and negative artifacts and drives real TCP plus SQLite", () => {
        for (const value of [
            "-p rustzen-admin --bin rz-admin",
            "-p rustzen-reports --bin rz-reports",
            "--features monitor-distribution",
            "rustzen-reports --no-default-features",
            "rz-admin-selected",
            "rz-reports-selected",
            "rz-admin-pure",
            "rz-reports-pure",
        ]) {
            expect(outer).toContain(value);
        }
        for (const value of [
            "notification_outbox",
            "notification_receipts",
            "selected-ingress-listener.txt",
            "FAILED STEP",
            "/api/notifications?limit=100",
            "admin-outage-and-backfill",
            "retry-initiator-immutable",
            "restart-recovery",
            "scheduled-silence",
            "current-permission-revocation",
            "dropped-response-duplicate",
            "bad-signature.json",
            "unsigned.json",
            "public-internal.json",
            "pure-absence.json",
            "pure-notification-route.json",
            "sport = :19831",
            "notificationTask:false",
        ]) {
            expect(inner).toContain(value);
        }
        expect(inner).not.toContain("mock");
        for (const value of [
            "useradd --system",
            'setpriv --reuid="$reports_user"',
            "--no-new-privs",
            'prepare_reports_layout "$reports_home"',
            'prepare_reports_layout "$pure_reports_home"',
            'capture_reports_identity "$reports_pid"',
            "reports-runtime-identity.json",
            "preserve_failure_logs",
        ]) {
            expect(inner).toContain(value);
        }
        expect(inner).not.toContain("--no-sandbox");
        expect(evidenceValidator).toContain("valid_reports_identity");
        expect(evidenceValidator).toContain("$runtimeIdentity[0].user.uid > 0");
    });

    test("keeps every runtime-gate pause within the saved FlowStep limit", () => {
        expect(inner).toContain('long_flow=$(flow long \'[{"action":"pause","durationMs":30000}]\')');
        const durations = [...inner.matchAll(/"durationMs":(\d+)/g)].map((match) => Number(match[1]));
        expect(durations.length).toBeGreaterThan(0);
        expect(durations.every((duration) => duration <= 30_000)).toBeTrue();
    });

    test("uses the frozen signature domain without adding an unpinned runtime", () => {
        expect(client).toContain('DOMAIN = "rz-notification-producer-v1"');
        expect(client).toContain("9668d0b7d9dbc9f6f341554fdb1da55b51f35f145dfbd4b98ecfb88e06cf80c0");
        expect(verifier).toContain("python3");
        expect(verifier).not.toMatch(/\bbun\b/i);
        expect(validation).toContain("already includes Python but intentionally contains no Bun");
        const result = Bun.spawnSync(["python3", clientUrl.pathname, "--self-test"]);
        expect(result.exitCode).toBe(0);
    });

    test("rejects invalid limits before contacting Docker and has valid shell syntax", () => {
        for (const script of [outerUrl.pathname, innerUrl.pathname]) {
            expect(Bun.spawnSync(["bash", "-n", script]).exitCode).toBe(0);
        }
        for (const [name, value, limit] of [
            ["RUSTZEN_REPORTS_NOTIFY_BUILD_TIMEOUT", "0", "7200"],
            ["RUSTZEN_REPORTS_NOTIFY_RUNTIME_TIMEOUT", "1801", "1800"],
        ]) {
            const result = Bun.spawnSync({
                cmd: ["bash", outerUrl.pathname],
                env: { ...process.env, [name]: value },
                stdout: "pipe",
                stderr: "pipe",
            });
            expect(result.exitCode).toBe(2);
            expect(new TextDecoder().decode(result.stderr)).toContain(`${name} must be 1..${limit} seconds`);
        }
    });

    test("publishes only after both fake Docker containers are absent", () => {
        const fixture = fakeGateFixture(outerUrl.pathname);
        try {
            const result = runFakeGate(fixture);
            expect(result.exitCode).toBe(0);
            expect(fixture.current()).toStartWith("runs/");
            expect(fixture.current()).not.toBe("runs/previous");
            expect(fixture.active()).toEqual([]);
            expect(fixture.dockerCalls().match(/container inspect/g)?.length).toBe(2);
            expect(fixture.dockerCalls()).toContain(
                "type=volume,src=rustzen-reports-notify-target-v1-aarch64-aarch64-unknown-linux-musl-rust195-crtstatic,dst=/cargo-target",
            );
            expect(fixture.dockerCalls()).toContain("dst=/out");
            expect(fixture.dockerCalls()).toContain("dst=/source,readonly");
            expect(fixture.dockerCalls()).toContain("--tmpfs /work:rw,size=512m");
            expect(fixture.dockerCalls()).not.toContain("dst=/work");
            expect(existsSync(join(fixture.evidence, fixture.current(), "manifest.json"))).toBeTrue();
        } finally {
            fixture.cleanup();
        }
    });

    test("rejects receipt and semantic tampering without replacing current", () => {
        for (const tamper of [
            "duplicate-receipt", "wrong-status", "path-traversal", "lifecycle-status",
            "pure-status", "reports-root-process", "reports-dir-owner",
        ]) {
            const fixture = fakeGateFixture(outerUrl.pathname, { tamper });
            try {
                const result = runFakeGate(fixture);
                expect(result.exitCode, tamper).not.toBe(0);
                expect(fixture.current(), tamper).toBe("runs/previous");
                expect(fixture.failed().length, tamper).toBe(1);
                expect(fixture.active(), tamper).toEqual([]);
            } finally {
                fixture.cleanup();
            }
        }
    }, 30_000);

    test("distinguishes watchdog timeout from a stage exit of 124", () => {
        for (const [stage, options, message] of [
            ["build", { block: "build", buildTimeout: "1", runtimeTimeout: "5" }, "build stage timed out after 1s"],
            ["runtime", { block: "runtime", buildTimeout: "5", runtimeTimeout: "1" }, "runtime stage timed out after 1s"],
            ["build", { buildExit: "124" }, "build stage failed (status 124)"],
            ["runtime", { runtimeExit: "124" }, "runtime stage failed (status 124)"],
        ]) {
            const fixture = fakeGateFixture(outerUrl.pathname, options);
            try {
                const result = runFakeGate(fixture);
                expect(result.exitCode, stage).toBe(124);
                expect(new TextDecoder().decode(result.stderr), stage).toContain(message);
                expect(fixture.current(), stage).toBe("runs/previous");
                expect(fixture.failed().length, stage).toBe(1);
                expect(fixture.active(), stage).toEqual([]);
                expect(fixture.locked(), stage).toBeFalse();
            } finally {
                fixture.cleanup();
            }
        }
    }, 20_000);

    test("cleans the outer gate after a build timeout before retry", () => {
        const fixture = fakeGateFixture(outerUrl.pathname, { block: "build", buildTimeout: "1" });
        try {
            expect(runFakeGate(fixture).exitCode).toBe(124);
            expect(fixture.current()).toBe("runs/previous");
            expect(fixture.active()).toEqual([]);
            expect(fixture.locked()).toBeFalse();
            fixture.env.FAKE_BLOCK = "";
            fixture.env.RUSTZEN_REPORTS_NOTIFY_BUILD_TIMEOUT = "5";
            expect(runFakeGate(fixture).exitCode).toBe(0);
            expect(fixture.current()).toStartWith("runs/");
            expect(fixture.active()).toEqual([]);
            expect(fixture.locked()).toBeFalse();
        } finally {
            fixture.cleanup();
        }
    }, 15_000);

    for (const [signal, exitCode] of [["SIGINT", 130], ["SIGTERM", 143]]) {
        test(`returns ${exitCode}, cleans resources, and preserves current on ${signal}`, async () => {
            const fixture = fakeGateFixture(outerUrl.pathname, { block: "build", buildTimeout: "30" });
            try {
                const result = await runSignaledFakeGate(fixture, signal);
                expect(result.exitCode).toBe(exitCode);
                expect(fixture.current()).toBe("runs/previous");
                expect(fixture.failed().length).toBe(1);
                expect(fixture.active()).toEqual([]);
                expect(fixture.locked()).toBeFalse();
                fixture.env.FAKE_BLOCK = "";
                expect(runFakeGate(fixture).exitCode).toBe(0);
                expect(fixture.current()).toStartWith("runs/");
                expect(fixture.active()).toEqual([]);
                expect(fixture.locked()).toBeFalse();
            } finally {
                fixture.cleanup();
            }
        }, 15_000);
    }

    test("bounds failure-log collection and still removes the runtime container", () => {
        const fixture = fakeGateFixture(outerUrl.pathname, { runtimeExit: "7", block: "logs" });
        try {
            const result = runFakeGate(fixture);
            expect(result.exitCode).toBe(7);
            expect(fixture.current()).toBe("runs/previous");
            expect(fixture.failed().length).toBe(1);
            expect(fixture.active()).toEqual([]);
            expect(new TextDecoder().decode(result.stderr)).toContain("runtime stage failed (status 7)");
            expect(fixture.dockerCalls()).toContain("logs rz-reports-selected-runtime-");
            const failed = join(fixture.evidence, "failed-runs", fixture.failed()[0]);
            expect(existsSync(join(failed, "steps.log"))).toBeTrue();
            expect(existsSync(join(failed, "container.log"))).toBeTrue();
        } finally {
            fixture.cleanup();
        }
    });

    test("a blocked removal is bounded and cannot publish current", () => {
        const fixture = fakeGateFixture(outerUrl.pathname, { block: "rm" });
        try {
            const result = runFakeGate(fixture);
            expect(result.exitCode).not.toBe(0);
            expect(fixture.current()).toBe("runs/previous");
            expect(fixture.failed().length).toBe(1);
            expect(fixture.dockerCalls().match(/rm -f rz-reports-selected-build-/g)?.length).toBe(2);
        } finally {
            fixture.cleanup();
        }
    });
});
