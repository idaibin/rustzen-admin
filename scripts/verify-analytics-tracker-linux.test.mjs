import { describe, expect, test } from "bun:test";

const gate = await Bun.file(new URL("./verify-analytics-tracker-linux.sh", import.meta.url)).text();
const fixture = await Bun.file(new URL("./fixtures/analytics-tracker/index.html", import.meta.url)).text();
const justfile = await Bun.file(new URL("../justfile", import.meta.url)).text();
const gatePath = new URL("./verify-analytics-tracker-linux.sh", import.meta.url).pathname;

function runGate(env) {
    return Bun.spawnSync({
        cmd: ["bash", gatePath],
        env: { ...process.env, ...env },
        stdout: "pipe",
        stderr: "pipe",
    });
}

describe("Analytics tracker Linux gate contract", () => {
    test("is independent, bounded, and uses the existing verifier image and Reports route", () => {
        expect(gate).toContain("RUSTZEN_ANALYTICS_TRACKER_INNER");
        expect(gate).toContain("ensure-admin-browser-verifier-image.sh");
        expect(gate).toContain("/verify/fixture");
        expect(gate).toContain("/api/reports/systems");
        expect(gate).toContain("/api/reports/runs");
        expect(gate).toContain("RUSTZEN_ANALYTICS_TRACKER_TIMEOUT");
        expect(gate).toContain("RUSTZEN_ANALYTICS_TRACKER_DOCKER");
        expect(gate).toContain("final_source_tree_sha256");
        expect(gate).toContain('"$docker_bin" rm "$container"');
        expect(gate).toContain("http[\"507\"].status == \"not-verified\"");
        expect(gate).not.toContain("verify-admin-browser-linux-inner.sh");
    });

    test("binds the fixture evidence to source, browser, and persistence cases", () => {
        expect(gate).toContain("sourceTreeSha256");
        expect(gate).toContain("browser:{chromium:$browser}");
        expect(gate).toContain("preOptIn");
        expect(gate).toContain("optOut");
        expect(gate).toContain('pre-optin:native:no-ids');
        expect(gate).toContain('opted-in:patched:ids');
        expect(gate).toContain('opted-out:native:no-ids');
        expect(gate).toContain('"#request-count","text":"0"');
        expect(gate).toContain('"#request-count","text":"1"');
        expect(gate).toContain("optin_rows=$(event_total)");
        expect(gate).toContain("test \"$optout_rows\" = \"$optin_rows\"");
        expect(gate).not.toContain("run_flow");
        expect(gate).toContain("rowDelta:($after413-$before413)");
        expect(gate).toContain("rowDelta:($after429-$before429)");
        expect(gate).toContain("acceptedPreloadCount:$acceptedPreloadCount");
        expect(gate).toContain("/verify/evidence/manifest.json");
        expect(gate).toContain("run_bounded_capture");
        expect(gate).toContain("Docker architecture discovery failed or exceeded its timeout");
        expect(gate).toContain("RUSTZEN_ANALYTICS_TRACKER_DOCKER_INFO_TIMEOUT must be 1..60 seconds");
        expect(gate).toContain("refusing to replace unsupported Analytics tracker evidence directory");
        expect(gate).toContain("RUSTZEN_ANALYTICS_TRACKER_TEST_SETUP_FAILURE");
        expect(gate).toContain('trap cleanup EXIT');
        expect(gate).toContain('trap - EXIT INT TERM');
        expect(gate).toContain("publish_candidate");
        expect(gate).toContain("atomic_replace_symlink");
        expect(gate).toContain('readlink "$current"');
    });

    test("fixture proves consent transitions and uses only a legal pathname event", () => {
        expect(fixture).toContain('src="http://127.0.0.1:19801/api/insights/tracker.js"');
        expect(fixture).toContain('data-endpoint="http://127.0.0.1:19801/api/insights/track"');
        expect(fixture).toContain("pre-optin");
        expect(fixture).toContain("rustzenAnalytics.enable({ consent: true })");
        expect(fixture).toContain("rustzenAnalytics.optOut()");
        expect(fixture).toContain("/fixture/allowed?private=secret#fragment");
        expect(fixture).toContain("hooksAreNative");
        expect(fixture).toContain("hasIds");
        expect(fixture).toContain("__nativeSend");
        expect(fixture).toContain("XMLHttpRequest.prototype.send === window.__nativeSend");
        expect(fixture).toContain("__trackingRequestCount");
        expect(fixture).toContain("requestsBeforeOptOut");
        expect(fixture).toContain("/fixture/ordinary-fetch");
        expect(fixture).toContain("request-count-changed");
        expect(fixture).toContain('id="request-count"');
    });

    test("is exposed through the local verification entry point", () => {
        expect(justfile).toContain("verify-analytics-tracker-linux:");
        expect(justfile).toContain("scripts/verify-analytics-tracker-linux.sh");
    });

    test("rejects a hanging or bogus Docker discovery and invalid timeouts", async () => {
        const fakeDocker = `/tmp/rz-analytics-fake-docker-${crypto.randomUUID()}`;
        await Bun.write(fakeDocker, '#!/bin/sh\nexec sleep 5 >/dev/null 2>&1\n');
        Bun.spawnSync(["chmod", "+x", fakeDocker]);
        const discovery = runGate({
            RUSTZEN_ANALYTICS_TRACKER_DOCKER: fakeDocker,
            RUSTZEN_ANALYTICS_TRACKER_DOCKER_INFO_TIMEOUT: "1",
        });
        expect(discovery.exitCode).not.toBe(0);
        await Bun.write(fakeDocker, '#!/bin/sh\nprintf bogus\n');
        const bogus = runGate({ RUSTZEN_ANALYTICS_TRACKER_DOCKER: fakeDocker });
        expect(bogus.exitCode).not.toBe(0);
        const invalidTimeout = runGate({
            RUSTZEN_UI_LINUX_ARCH: "aarch64",
            RUSTZEN_ANALYTICS_TRACKER_TIMEOUT: "0",
        });
        expect(invalidTimeout.exitCode).toBe(2);
        const invalidInfoTimeout = runGate({
            RUSTZEN_UI_LINUX_ARCH: "aarch64",
            RUSTZEN_ANALYTICS_TRACKER_DOCKER_INFO_TIMEOUT: "61",
        });
        expect(invalidInfoTimeout.exitCode).toBe(2);
        Bun.spawnSync(["rm", "-f", fakeDocker]);
    });

    test("keeps the old current evidence when publication fails", () => {
        const result = runGate({ RUSTZEN_ANALYTICS_TRACKER_TEST_PUBLISH_FAILURE: "1" });
        expect(result.exitCode).toBe(0);
        expect(new TextDecoder().decode(result.stdout)).toContain(
            "publication success and failure seams passed",
        );
    });

    test("cleans the lock after an injected setup failure and preserves current", () => {
        const result = runGate({ RUSTZEN_ANALYTICS_TRACKER_TEST_SETUP_FAILURE: "1" });
        expect(result.exitCode).not.toBe(0);
        expect(new TextDecoder().decode(result.stderr)).toContain(
            "setup failure seam passed",
        );
    });

    test("runs EXIT cleanup after INT and TERM with signal exit semantics", () => {
        for (const [signal, exitCode] of [["INT", 130], ["TERM", 143]]) {
            const result = runGate({ RUSTZEN_ANALYTICS_TRACKER_TEST_SIGNAL: signal });
            expect(result.exitCode).toBe(exitCode);
            expect(new TextDecoder().decode(result.stderr)).toContain(
                `Analytics tracker ${signal} signal seam passed`,
            );
        }
        const successCleanup = gate.lastIndexOf('status=0\nrm -rf "$lock_dir"\ntrap - EXIT INT TERM');
        expect(successCleanup).toBeGreaterThan(-1);
    });
});
