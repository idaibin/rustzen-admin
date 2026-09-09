import { createHash } from "node:crypto";
import { existsSync, mkdtempSync, mkdirSync, readFileSync, renameSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, test } from "bun:test";

const root = new URL("..", import.meta.url).pathname;
const outer = new URL("./verify-monitor-agent-multi-node-ui-linux.sh", import.meta.url).pathname;
const evidence = new URL("./verify-monitor-agent-multi-node-ui-evidence.mjs", import.meta.url).pathname;
const stepsDriver = new URL("./monitor-agent-multi-node-ui-browser-steps.py", import.meta.url).pathname;
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const run = (command, env = {}) => Bun.spawnSync({ cmd: command, env: { ...process.env, ...env }, stdout: "pipe", stderr: "pipe" });

function fixture() {
    const directory = mkdtempSync(join(tmpdir(), "rz-dual-agent-ui-"));
    const staged = join(directory, "staged");
    mkdirSync(staged);
    const hashes = {};
    for (const [key, name] of Object.entries({ admin: "rz-admin", monitor: "rz-monitor", reports: "rz-reports", agent: "rz-monitor-agent" })) {
        const bytes = Buffer.from(key); writeFileSync(join(staged, name), bytes); hashes[key] = digest(bytes);
    }
    const expected = JSON.parse(new TextDecoder().decode(run(["python3", stepsDriver, "linux-agent-a", "linux-agent-b"], { RUSTZEN_VERIFY_BOOT_A: "boot-a", RUSTZEN_VERIFY_BOOT_B: "boot-b" }).stdout));
    writeFileSync(join(directory, "browser-steps.json"), JSON.stringify(expected));
    const flow = Buffer.from(JSON.stringify({ data: [{ id: "flow-1", steps: expected }] }));
    writeFileSync(join(directory, "browser-flow.json"), flow);
    const receipt = Buffer.from(JSON.stringify({ data: expected.map((step, stepIndex) => ({ ...step, stepIndex, runId: "run-1", status: "succeeded" })) }));
    writeFileSync(join(directory, "browser-steps-receipt.json"), receipt);
    const artifacts = ["a", "b"].map((name) => {
        const bytes = Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82]), Buffer.from([0, 0, 5, 160, 0, 0, 3, 132, 8, 6, 0, 0, 0, 0, 0, 0, 0]), Buffer.from([0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130])]); const file = `nodes-linux-agent-${name}.png`; writeFileSync(join(directory, file), bytes);
        return { file, sha256: digest(bytes), bytes: bytes.length, dimensions: "1440 x 900", viewport: { width: 1440, height: 900 } };
    });
    const descriptor = (file, bytes) => ({ file, sha256: digest(bytes), bytes: bytes.length });
    const manifest = { schemaVersion: 1, status: "passed", sourceTreeSha256: "s".repeat(64), binaries: hashes, nodeBootIds: [{ nodeId: "linux-agent-a", bootId: "boot-a" }, { nodeId: "linux-agent-b", bootId: "boot-b" }], browser: { runId: "run-1", steps: descriptor("browser-steps.json", Buffer.from(JSON.stringify(expected))), flow: { id: "flow-1", ...descriptor("browser-flow.json", flow) }, receipts: [{ ...descriptor("browser-steps-receipt.json", receipt), runId: "run-1" }], artifacts } };
    writeFileSync(join(directory, "manifest.json"), JSON.stringify(manifest));
    return { directory, staged };
}

describe("dual-Agent Nodes Chromium gate", () => {
    test("evidence verifier accepts a bound receipt and rejects a tampered PNG", () => {
        const item = fixture();
        try {
            expect(run([process.execPath, evidence, item.directory, "s".repeat(64), item.staged]).exitCode).toBe(0);
            writeFileSync(join(item.directory, "nodes-linux-agent-a.png"), "tampered");
            expect(run([process.execPath, evidence, item.directory, "s".repeat(64), item.staged]).exitCode).not.toBe(0);
        } finally { rmSync(item.directory, { recursive: true, force: true }); }
    });
    test("evidence verifier rejects truncated, wrong-sized, duplicate, and swapped detail PNGs", () => {
        for (const mutate of [
            (item) => writeFileSync(join(item.directory, "nodes-linux-agent-a.png"), Buffer.from([137, 80, 78, 71])),
            (item) => { const bytes = readFileSync(join(item.directory, "nodes-linux-agent-a.png")); bytes.writeUInt32BE(1439, 16); writeFileSync(join(item.directory, "nodes-linux-agent-a.png"), bytes); },
            (item) => { const path = join(item.directory, "manifest.json"); const value = JSON.parse(readFileSync(path)); value.browser.artifacts[1].file = "nodes-linux-agent-a.png"; writeFileSync(path, JSON.stringify(value)); },
            (item) => { const path = join(item.directory, "manifest.json"); const value = JSON.parse(readFileSync(path)); [value.browser.artifacts[0], value.browser.artifacts[1]] = [value.browser.artifacts[1], value.browser.artifacts[0]]; writeFileSync(path, JSON.stringify(value)); },
        ]) {
            const item = fixture();
            try { mutate(item); expect(run([process.execPath, evidence, item.directory, "s".repeat(64), item.staged]).exitCode).not.toBe(0); } finally { rmSync(item.directory, { recursive: true, force: true }); }
        }
    });
    test("evidence verifier rejects flow and canonical action mutations even with refreshed hashes", () => {
        for (const change of ["selector", "boot", "url", "name", "flow", "bytes"]) {
            const item = fixture();
            try {
                const manifestPath = join(item.directory, "manifest.json"); const manifest = JSON.parse(readFileSync(manifestPath));
                const target = change === "flow" ? "browser-flow.json" : "browser-steps.json";
                if (change === "bytes") manifest.browser.steps.bytes += 1;
                else {
                    const value = JSON.parse(readFileSync(join(item.directory, target)));
                    const steps = target === "browser-flow.json" ? value.data[0].steps : value;
                    const step = steps.find((entry) => entry.action === (change === "url" ? "goto" : change === "name" ? "screenshotViewport" : "assertText"));
                    if (change === "flow") value.data[0].id = "mutated-flow";
                    if (change === "selector") step.selector = "[data-testid=mutated]";
                    if (change === "boot") step.text = "mutated-boot";
                    if (change === "url") step.url = "/mutated";
                    if (change === "name") step.name = "mutated-name";
                    writeFileSync(join(item.directory, target), JSON.stringify(value));
                    const bytes = readFileSync(join(item.directory, target));
                    const descriptor = { sha256: digest(bytes), bytes: bytes.length };
                    Object.assign(change === "flow" ? manifest.browser.flow : manifest.browser.steps, descriptor);
                }
                writeFileSync(manifestPath, JSON.stringify(manifest));
                expect(run([process.execPath, evidence, item.directory, "s".repeat(64), item.staged]).exitCode).not.toBe(0);
            } finally { rmSync(item.directory, { recursive: true, force: true }); }
        }
    });
    test("evidence verifier rejects paths, links, receipt duplication, and moved-run traversal", () => {
        const cases = [
            (value) => { value.browser.steps.file = "../browser-steps.json"; },
            (value) => { value.browser.flow.file = "/tmp/browser-flow.json"; },
            (value) => { value.browser.receipts.push({ ...value.browser.receipts[0] }); },
            (value) => { value.browser.receipts[0].file = "nested/browser-steps-receipt.json"; },
        ];
        for (const mutate of cases) {
            const item = fixture();
            try { const path = join(item.directory, "manifest.json"); const value = JSON.parse(readFileSync(path)); mutate(value); writeFileSync(path, JSON.stringify(value)); expect(run([process.execPath, evidence, item.directory, "s".repeat(64), item.staged]).exitCode).not.toBe(0); } finally { rmSync(item.directory, { recursive: true, force: true }); }
        }
        const linked = fixture();
        try { const original = join(linked.directory, "nodes-linux-agent-a.png"); const target = join(linked.directory, "outside.png"); renameSync(original, target); symlinkSync(target, original); expect(run([process.execPath, evidence, linked.directory, "s".repeat(64), linked.staged]).exitCode).not.toBe(0); } finally { rmSync(linked.directory, { recursive: true, force: true }); }
        const moved = fixture();
        try {
            const runs = join(tmpdir(), `rz-dual-agent-ui-runs-${crypto.randomUUID()}`); mkdirSync(join(runs, "runs"), { recursive: true }); const target = join(runs, "runs", "published"); renameSync(moved.directory, target);
            const manifestPath = join(target, "manifest.json"); const value = JSON.parse(readFileSync(manifestPath)); value.browser.steps.file = "../published/browser-steps.json"; writeFileSync(manifestPath, JSON.stringify(value));
            expect(run([process.execPath, evidence, target, "s".repeat(64), join(target, "staged")]).exitCode).not.toBe(0); rmSync(runs, { recursive: true, force: true });
        } catch (error) { throw error; }
    });
    test("driver opens both details, captures both, and validates arguments", () => {
        expect(run(["python3", stepsDriver]).exitCode).not.toBe(0);
        const result = run(["python3", stepsDriver, "linux-agent-a", "linux-agent-b"], { RUSTZEN_VERIFY_BOOT_A: "boot-a", RUSTZEN_VERIFY_BOOT_B: "boot-b" });
        expect(result.exitCode).toBe(0);
        const steps = JSON.parse(new TextDecoder().decode(result.stdout));
        expect(steps.filter((step) => step.action === "screenshotViewport")).toHaveLength(2);
        expect(steps.filter((step) => step.selector?.includes("monitor-node-history-5m-")).length).toBeGreaterThanOrEqual(2);
        expect(steps.filter((step) => step.action === "waitFor" && step.selector?.endsWith(".recharts-line-dot"))).toHaveLength(2);
        expect(steps.filter((step) => step.action === "assertElementLayout" && step.elementCount === 2 && step.visibleCount === 2 && step.withinViewport === true)).toHaveLength(2);
    });
    test("outer validates timeout and atomically retains current evidence after failed publish", async () => {
        expect(run(["bash", outer], { RUSTZEN_UI_LINUX_ARCH: "x86_64", RUSTZEN_MONITOR_MULTI_NODE_UI_TIMEOUT: "901" }).exitCode).toBe(2);
        const directory = mkdtempSync(join(tmpdir(), "rz-dual-agent-ui-publish-"));
        try {
            const result = run(["bash", outer], { RUSTZEN_MONITOR_MULTI_NODE_UI_TEST_PUBLISH_FAILURE: "1", RUSTZEN_MONITOR_MULTI_NODE_UI_TEST_ROOT: directory });
            expect(result.exitCode).toBe(0);
            expect(run(["readlink", join(directory, "current")]).stdout.toString().trim()).toBe("runs/test");
            expect(await Bun.file(join(directory, "current/manifest.json")).text()).toBe("new");
        } finally { rmSync(directory, { recursive: true, force: true }); }
    });
    test("outer removes the container and retains only safe failed-run diagnostics", async () => {
        const directory = mkdtempSync(join(tmpdir(), "rz-dual-agent-ui-cleanup-")); const fake = join(directory, "docker");
        writeFileSync(fake, "#!/bin/sh\ncase \"$1:$2\" in logs:*) printf 'bounded diagnostic\\n';; rm:-f) exit 0;; ps:-a) exit 0;; *) exit 2;; esac\n"); run(["chmod", "+x", fake]);
        try {
            const result = run(["bash", outer], { RUSTZEN_MONITOR_MULTI_NODE_UI_TEST_CLEANUP: "1", RUSTZEN_MONITOR_MULTI_NODE_UI_TEST_ROOT: directory, RUSTZEN_MONITOR_MULTI_NODE_UI_DOCKER: fake, RUSTZEN_MONITOR_MULTI_NODE_UI_CLEANUP_TIMEOUT: "1" });
            expect(result.exitCode).not.toBe(0);
            expect(await Bun.file(join(directory, ".candidate")).exists()).toBeFalse();
            expect(await Bun.file(join(directory, ".binaries")).exists()).toBeFalse();
            expect(await Bun.file(join(directory, ".verify.lock")).exists()).toBeFalse();
            expect(run(["readlink", join(directory, "current")]).stdout.toString().trim()).toBe("runs/old");
            expect(await Bun.file(join(directory, "failed-runs/test/failure-summary.tsv")).text()).toContain("cleanupStatus\t0\n");
            expect(await Bun.file(join(directory, "failed-runs/test/container.log")).text()).toContain("bounded diagnostic");
            expect(await Bun.file(join(directory, "failed-runs/test/build-provenance.txt")).text()).toBe("safe");
            expect(await Bun.file(join(directory, "failed-runs/test/browser-steps.json")).exists()).toBeFalse();
        } finally { rmSync(directory, { recursive: true, force: true }); }
    });
    test("outer retains the ownership lock when Docker removal is not proven", async () => {
        for (const mode of ["rm-fail", "ps-present", "ps-error", "timeout"]) {
            const directory = mkdtempSync(join(tmpdir(), `rz-dual-agent-ui-${mode}-`)); const fake = join(directory, "docker");
            writeFileSync(fake, `#!/bin/sh
case "$1:$2" in
  logs:*) printf 'safe diagnostic\\n';;
  rm:-f) [ "$RZ_FAKE_MODE" != timeout ] || { trap '' TERM; sleep 60; }; [ "$RZ_FAKE_MODE" != rm-fail ];;
  ps:-a) case "$RZ_FAKE_MODE" in ps-present) printf 'stubborn\\n';; ps-error) echo 'daemon unavailable' >&2; exit 1;; esac;;
  *) exit 2;;
esac
`); run(["chmod", "+x", fake]);
            try {
                const result = run(["bash", outer], { RUSTZEN_MONITOR_MULTI_NODE_UI_TEST_CLEANUP: "1", RUSTZEN_MONITOR_MULTI_NODE_UI_TEST_ROOT: directory, RUSTZEN_MONITOR_MULTI_NODE_UI_DOCKER: fake, RUSTZEN_MONITOR_MULTI_NODE_UI_CLEANUP_TIMEOUT: "1", RZ_FAKE_MODE: mode });
                expect(result.exitCode).toBe(125);
                expect(existsSync(join(directory, ".verify.lock"))).toBeTrue();
                expect(run(["readlink", join(directory, "current")]).stdout.toString().trim()).toBe("runs/old");
                expect(await Bun.file(join(directory, "failed-runs/test/failure-summary.tsv")).text()).toContain("cleanupStatus\t125\n");
                expect(await Bun.file(join(directory, "failed-runs/test/browser-steps.json")).exists()).toBeFalse();
            } finally { rmSync(directory, { recursive: true, force: true }); }
        }
    });
    test("command and documentation expose the separately bounded browser gate", async () => {
        expect(await Bun.file(join(root, "justfile")).text()).toContain("verify-monitor-agent-multi-node-ui-linux:");
        expect(await Bun.file(join(root, "docs/guides/monitoring-testing.md")).text()).toContain("two 1440x900 detail PNGs");
        expect(await Bun.file(join(root, "docs/guides/monitoring-testing.md")).text()).toContain("failed-runs/<run-id>");
    });
});
