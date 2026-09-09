import { createHash } from "node:crypto";
import { chmodSync, existsSync, mkdtempSync, mkdirSync, readFileSync, readdirSync, readlinkSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, test } from "bun:test";

const root = new URL("..", import.meta.url).pathname;
const gatePath = new URL("./verify-task-console-linux.sh", import.meta.url).pathname;
const text = (file) => readFileSync(new URL(file, import.meta.url), "utf8");
const digest = (value) => createHash("sha256").update(value).digest("hex");
const run = (env) => Bun.spawnSync({ cmd: ["bash", gatePath], env: { ...process.env, ...env }, stdout: "pipe", stderr: "pipe" });

test("task-console gate entrypoint is executable", () => {
    expect(statSync(gatePath).mode & 0o111).not.toBe(0);
});

function fixture({ badProvenance = false, changedSource = false } = {}) {
    const directory = mkdtempSync(join(tmpdir(), "rz-task-console-gate-"));
    const bin = join(directory, "bin"); mkdirSync(bin);
    const lines = ["schemaVersion\t1", "gitHead\thead", "sourceTreeState\tdirty", `sourceTreeSha256\t${"a".repeat(64)}`, "architecture\tx86_64", "targetTriple\tx86_64-unknown-linux-musl", "platform\tlinux/amd64", "distribution\tfull"];
    for (const name of ["rz-admin", "rz-monitor", "rz-insights", "rz-reports"]) { const value = `${name}-original`; writeFileSync(join(bin, name), value); chmodSync(join(bin, name), 0o755); lines.push(`${name}\t${digest(value)}`); }
    if (badProvenance) lines[1] = "gitHead\told";
    writeFileSync(join(bin, "build-provenance.txt"), `${lines.join("\n")}\n`);
    const identity = join(directory, "identity.sh");
    writeFileSync(identity, `#!/bin/sh\ncount=${directory}/identity-count\nvalue=0; test -f "$count" && value=$(cat "$count"); value=$((value + 1)); printf '%s' "$value" > "$count"\nif test "$value" -gt 1 && test ${changedSource ? 1 : 0} = 1; then printf 'head dirty %s\\n' '${"b".repeat(64)}'; else printf 'head dirty %s\\n' '${"a".repeat(64)}'; fi\n`); chmodSync(identity, 0o755);
    const docker = join(directory, "docker.py");
    writeFileSync(docker, String.raw`#!/usr/bin/env python3
import hashlib, json, os, struct, sys, zlib
args = sys.argv[1:]
if args[0] == "logs": print("safe-container-log")
elif args[0] == "rm": sys.exit(1 if os.environ.get("RUSTZEN_TASK_CONSOLE_TEST_DOCKER_MODE") == "rmfail" else 0)
elif args[0] == "ps":
    if os.environ.get("RUSTZEN_TASK_CONSOLE_TEST_DOCKER_MODE") == "psfail": sys.exit(9)
    if os.environ.get("RUSTZEN_TASK_CONSOLE_TEST_DOCKER_MODE") == "stubborn": print("rz-task-console-stubborn")
elif args[0] == "run":
    env = {}; candidate = None
    for index, value in enumerate(args):
        if value == "--env":
            key, content = args[index + 1].split("=", 1); env[key] = content
        if value == "--mount" and ",dst=/verify/evidence" in args[index + 1]: candidate = args[index + 1].split("src=", 1)[1].split(",dst=", 1)[0]
    def write(name, value): open(os.path.join(candidate, name), "wb").write(value)
    receipts = ["tasks.json", "list-only-tasks.json", "list-only-runs-before.json", "list-only-post.json", "list-only-status.txt", "list-only-runs-after.json", "owner-runs-after.json", "empty-receipt.json", "empty-response.json", "error-receipt.json", "browser-owner-run.json", "browser-owner-steps.json", "browser-owner-artifacts.json", "browser-owner-record-run.json", "browser-owner-record-steps.json", "browser-owner-record-artifacts.json", "browser-viewer-run.json", "browser-viewer-steps.json", "browser-viewer-artifacts.json", "browser-empty-run.json", "browser-empty-steps.json", "browser-empty-artifacts.json", "browser-error-run.json", "browser-error-steps.json", "browser-error-artifacts.json"]
    payloads = {name: b"{}" for name in receipts}
    payloads.update({"list-only-runs-before.json": b'{"data":[],"total":0}', "list-only-runs-after.json": b'{"data":[],"total":0}', "empty-receipt.json": b'{"method":"GET","mode":"empty","route":"/api/manage/tasks","hitCount":1}', "empty-response.json": b'{"code":0,"message":"Success","data":[],"total":0}', "browser-owner-run.json": b'{"data":{"status":"succeeded"}}', "browser-owner-record-run.json": b'{"data":{"status":"succeeded"}}'})
    for name, value in payloads.items(): write(name, value)
    def png(width, height):
        raw = b"\0" + b"\0\0\0" * width
        def chunk(kind, value): return struct.pack(">I", len(value)) + kind + value + struct.pack(">I", zlib.crc32(kind + value) & 0xffffffff)
        return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b"")
    write("task-console-desktop.png", png(1440, 900)); write("task-console-mobile.png", png(390, 844))
    def descriptor(name):
        value = open(os.path.join(candidate, name), "rb").read(); return {"file": name, "sha256": hashlib.sha256(value).hexdigest(), "bytes": len(value)}
    manifest = {"schemaVersion": 2, "status": "passed", "gitHead": env["RUSTZEN_VERIFY_HEAD"], "sourceTreeState": env["RUSTZEN_VERIFY_SOURCE_TREE_STATE"], "sourceTreeSha256": env["RUSTZEN_VERIFY_SOURCE_TREE_SHA256"], "platform": env["RUSTZEN_VERIFY_PLATFORM"], "buildProvenanceSha256": env["RUSTZEN_VERIFY_BUILD_PROVENANCE_SHA256"], "binaries": [{"name": name, "sha256": sha} for name, sha in json.loads(env["RUSTZEN_VERIFY_BINARY_HASHES"]).items()], "api": {"taskCount": 3, "manualRun": {"id": "manual", "status": "success"}, "listOnlyPost": 403}, "browser": {"ownerRun": "owner", "recordRun": "record", "viewerRun": "viewer", "emptyRun": "empty", "errorRun": "error"}, "receipts": [descriptor(name) for name in receipts], "screenshots": [{**descriptor("task-console-desktop.png"), "dimensions": "1440 x 900", "viewport": {"width": 1440, "height": 900}}, {**descriptor("task-console-mobile.png"), "dimensions": "390 x 844", "viewport": {"width": 390, "height": 844}}]}
    tamper = os.environ.get("RUSTZEN_TASK_CONSOLE_TEST_TAMPER")
    if tamper == "binary": manifest["binaries"][0]["sha256"] = "0" * 64
    if tamper == "viewport": manifest["screenshots"][0]["viewport"]["width"] = 1439
    if tamper == "screenshot": write("task-console-desktop.png", b"tampered")
    if tamper == "receipt": write("tasks.json", b"tampered")
    write("manifest.json", json.dumps(manifest).encode())
`); chmodSync(docker, 0o755);
    const verifier = join(directory, "verifier.sh"); writeFileSync(verifier, "#!/bin/sh\nprintf 'fake-image fake-key fake-provenance\\n'\n"); chmodSync(verifier, 0o755);
    const evidence = join(directory, "evidence");
    return { directory, bin, identity, docker, verifier, evidence, cleanup: () => rmSync(directory, { recursive: true, force: true }) };
}

function environment(item, extra = {}) { return { RUSTZEN_UI_LINUX_ARCH: "x86_64", RUSTZEN_UI_LINUX_BIN_DIR: item.bin, RUSTZEN_TASK_CONSOLE_SOURCE_IDENTITY: item.identity, RUSTZEN_TASK_CONSOLE_VERIFIER_HELPER: item.verifier, RUSTZEN_TASK_CONSOLE_DOCKER: item.docker, RUSTZEN_TASK_CONSOLE_EVIDENCE_ROOT: item.evidence, RUSTZEN_TASK_CONSOLE_TIMEOUT: "5", RUSTZEN_TASK_CONSOLE_CLEANUP_TIMEOUT: "2", ...extra }; }
function failed(item) { return readdirSync(join(item.evidence, "failed-runs")); }
function assertSafeFailure(item) { const files = readdirSync(join(item.evidence, "failed-runs", failed(item)[0])).sort(); expect(files).toContain("failure-summary.tsv"); expect(files).toContain("build-provenance.txt"); for (const name of ["browser-steps.json", "tasks.json", "runs.json", "list-only-post.json", "task-console-desktop.png", "task-console-mobile.png"]) expect(files).not.toContain(name); }

test("task-console gate binds complete provenance, exact cleanup, and atomic current publication", () => {
    const outer = text("./verify-task-console-linux.sh"), inner = text("./verify-task-console-linux-inner.sh"), justfile = readFileSync(`${root}/justfile`, "utf8");
    for (const value of ["verify_build_provenance", "verify_staged_binaries", "source tree changed during task console verification", "name=^/${container}$", "failed-runs", "atomic_replace_symlink", "cleanup failed; ownership lock retained"]) expect(outer).toContain(value);
    for (const value of ["cleanup-operation-logs-retention", "length == 3", 'text:"Clean operation logs"', 'text:"Clean task run records"', 'text:"Success"', "= 403", "task_console_viewer", "list-only-runs-before.json", "list-only-runs-after.json", "screenshotViewport", "assertNoHorizontalOverflow", "task-console-desktop.png", "task-console-mobile.png", '.code == 0 and .message == "Success" and .data == [] and .total == 0']) expect(inner).toContain(value);
    expect(text("./admin-browser-fault-proxy.py")).toContain('b\'{"code":0,"message":"Success","data":[],"total":0}\'');
    expect(text("./admin-browser-fault-proxy.py")).toContain('MODE == "task-transition"');
    expect(inner.indexOf('curl -fsS http://127.0.0.1:19805/api/manage/tasks > "/verify/evidence/$name-response.json"')).toBeLessThan(inner.indexOf('kill -TERM "$proxy"'));
    expect(inner).toContain('[data-status=running]');
    expect(inner).toContain('[data-status=success]');
    expect(inner).toContain(".transitionRunId | tostring");
    expect(outer).toContain("$candidate/task-transition-receipt.json");
    expect(inner.match(/\{action:"assertNoHorizontalOverflow"\}/g)).toHaveLength(4);
    expect(inner).toContain('run=$(run_browser "$name" "$steps") || browser_status=$?');
    expect(inner).not.toContain('runs/$run/run');
    expect(inner).toContain("(.data | type) == \"array\"");
    for (const value of ["chown -R rustzen:rustzen /opt/rz", "XDG_CONFIG_HOME=/opt/rz/.config", "XDG_CACHE_HOME=/opt/rz/.cache", "RUSTZEN_REPORTS_BROWSER_PATH=/usr/bin/chromium", "RUSTZEN_REPORTS_MAX_CONCURRENCY=1"]) expect(inner).toContain(value);
    expect(inner).not.toContain("50000");
    expect(justfile).toContain("verify-task-console-linux:");
});

test("fault proxy replays the task transition exactly three times", () => {
    const result = Bun.spawnSync(["python3", new URL("./test-admin-browser-fault-proxy.py", import.meta.url).pathname], { stdout: "pipe", stderr: "pipe" });
    expect(result.exitCode, new TextDecoder().decode(result.stderr)).toBe(0);
    expect(new TextDecoder().decode(result.stdout)).toContain("replay guard passed");
}, 30_000);

test("rejects old build provenance and a staged binary replacement", () => {
    for (const [options, extra] of [[{ badProvenance: true }, {}], [{}, { RUSTZEN_TASK_CONSOLE_TEST_REPLACE_STAGED: "1" }]]) {
        const item = fixture(options); try { expect(run(environment(item, extra)).exitCode).not.toBe(0); expect(failed(item)).toHaveLength(1); assertSafeFailure(item); expect(existsSync(join(item.evidence, ".verify.lock"))).toBeFalse(); } finally { item.cleanup(); }
    }
});

test("rejects a source identity change after staging", () => {
    const item = fixture({ changedSource: true }); try { expect(run(environment(item, { RUSTZEN_TASK_CONSOLE_TEST_SOURCE_CHANGED: "1" })).exitCode).not.toBe(0); expect(failed(item)).toHaveLength(1); assertSafeFailure(item); } finally { item.cleanup(); }
});

test("publication failures preserve old current and safe failed evidence", () => {
    for (const phase of ["before-move", "after-move", "after-link"]) {
        const item = fixture(); try { expect(run(environment(item, { RUSTZEN_TASK_CONSOLE_TEST_PUBLISH_FAILURE: phase })).exitCode).not.toBe(0); expect(readlinkSync(join(item.evidence, "current"))).toBe("runs/old"); expect(failed(item)).toHaveLength(1); assertSafeFailure(item); expect(existsSync(join(item.evidence, ".verify.lock"))).toBeFalse(); } finally { item.cleanup(); }
    }
});

test("stubborn or unverifiable Docker cleanup retains the ownership lock and cannot replace current", () => {
    for (const mode of ["stubborn", "psfail", "rmfail"]) {
        const item = fixture(); try { expect(run(environment(item, { RUSTZEN_TASK_CONSOLE_TEST_CLEANUP: "1", RUSTZEN_TASK_CONSOLE_TEST_DOCKER_MODE: mode })).exitCode).toBe(125); expect(readlinkSync(join(item.evidence, "current"))).toBe("runs/old"); expect(failed(item)).toHaveLength(1); assertSafeFailure(item); expect(existsSync(join(item.evidence, ".verify.lock"))).toBeTrue(); } finally { item.cleanup(); }
    }
});

test("rejects binary, PNG, viewport, and receipt tampering without replacing current", () => {
    for (const tamper of ["binary", "screenshot", "viewport", "receipt"]) {
        const item = fixture();
        try {
            mkdirSync(join(item.evidence, "runs", "old"), { recursive: true }); writeFileSync(join(item.evidence, "runs", "old", "manifest.json"), "old"); Bun.spawnSync(["ln", "-s", "runs/old", join(item.evidence, "current")]);
            expect(run(environment(item, { RUSTZEN_TASK_CONSOLE_TEST_TAMPER: tamper })).exitCode).not.toBe(0);
            expect(readlinkSync(join(item.evidence, "current"))).toBe("runs/old"); expect(failed(item)).toHaveLength(1); assertSafeFailure(item);
        } finally { item.cleanup(); }
    }
}, 30_000);

test("failed evidence retains only bounded safe browser receipts", () => {
    const item = fixture();
    try {
        expect(run(environment(item, { RUSTZEN_TASK_CONSOLE_TEST_FAILURE_ALLOWLIST: "1" })).exitCode).not.toBe(0);
        const directory = join(item.evidence, "failed-runs", failed(item)[0]);
        expect(readdirSync(directory).sort()).toEqual(["browser-owner-run.json", "build-provenance.txt", "failure-summary.tsv"]);
        const receipt = readFileSync(join(directory, "browser-owner-run.json"), "utf8");
        expect(receipt).not.toContain("secret"); expect(receipt).not.toContain("token"); expect(receipt).not.toContain("input");
    } finally { item.cleanup(); }
});

function runBrowserHarness(caseName) {
    const directory = mkdtempSync(join(tmpdir(), "rz-task-console-browser-"));
    const evidence = join(directory, "evidence"); mkdirSync(evidence);
    const fakeCurl = join(directory, "curl");
    writeFileSync(fakeCurl, `#!/usr/bin/env bash
set -euo pipefail
url="\${@: -1}"
case "$url" in
  */api/reports/flows) printf '%s\\n' '{"data":{"id":"flow"}}' ;;
  */api/reports/runs) printf '%s\\n' '{"data":{"id":"run"}}' ;;
  */api/reports/runs/run)
    count_file="$(dirname "$0")/run-reads"; count=0; [ ! -f "$count_file" ] || count=$(cat "$count_file"); count=$((count + 1)); printf '%s' "$count" > "$count_file"
    case "\${CASE_NAME:?}:$count" in failed:1) printf '%s\\n' '{"data":{"status":"failed"}}' ;; long-running:1|long-running:2) printf '%s\\n' '{"data":{"status":"running"}}' ;; *:1) printf '%s\\n' '{"data":{"status":"succeeded"}}' ;; *) printf '%s\\n' '{"data":{"id":"run","status":"succeeded"}}' ;; esac ;;
  */api/reports/runs/run/steps) if [ "\${CASE_NAME:?}" = receipt-fetch-failure ]; then exit 7; fi; if [ "\${CASE_NAME:?}" = failed-step ]; then printf '%s\\n' '{"data":[{"runId":"run","status":"failed"}]}' ; else printf '%s\\n' '{"data":[{"runId":"run","status":"succeeded"}]}' ; fi ;;
  */api/reports/runs/run/artifacts) if [ "\${CASE_NAME:?}" = artifact-invalid ]; then printf '%s\\n' '{"data":{}}'; else printf '%s\\n' '{"data":[]}' ; fi ;;
  *) exit 9 ;;
esac
`); chmodSync(fakeCurl, 0o755);
    const inner = text("./verify-task-console-linux-inner.sh");
    const start = inner.indexOf("run_browser() {");
    const end = inner.indexOf("\ndownload_png()", start);
    expect(start).toBeGreaterThanOrEqual(0); expect(end).toBeGreaterThan(start);
    const runBrowser = inner.slice(start, end).replaceAll("/verify/evidence", evidence);
    const harness = join(directory, "run-browser.sh");
    writeFileSync(harness, `#!/usr/bin/env bash
set -uo pipefail
admin=http://admin
browser_system=system
auth=()
curl_bin=${JSON.stringify(fakeCurl)}
browser_poll_attempts=2
${runBrowser}
set +e
run_browser owner '[]'
status=$?
printf 'RUN_BROWSER_STATUS=%s\\n' "$status"
exit "$status"
`); chmodSync(harness, 0o755);
    const result = Bun.spawnSync({ cmd: ["bash", harness], env: { ...process.env, CASE_NAME: caseName }, stdout: "pipe", stderr: "pipe" });
    return { directory, evidence, result, cleanup: () => rmSync(directory, { recursive: true, force: true }) };
}

test("run_browser rejects terminal, polling, step, and receipt failures while retaining available receipts", () => {
    for (const caseName of ["failed", "long-running", "failed-step", "receipt-fetch-failure", "artifact-invalid"]) {
        const item = runBrowserHarness(caseName);
        try {
            expect(item.result.exitCode, new TextDecoder().decode(item.result.stderr)).not.toBe(0);
            expect(new TextDecoder().decode(item.result.stdout)).toContain("run");
            expect(new TextDecoder().decode(item.result.stdout)).toContain("RUN_BROWSER_STATUS=1");
            expect(existsSync(join(item.evidence, "browser-owner-run.json"))).toBeTrue();
            expect(existsSync(join(item.evidence, "browser-owner-artifacts.json"))).toBeTrue();
            if (caseName === "receipt-fetch-failure") expect(statSync(join(item.evidence, "browser-owner-steps.json")).size).toBe(0);
            else expect(existsSync(join(item.evidence, "browser-owner-steps.json"))).toBeTrue();
        } finally { item.cleanup(); }
    }
}, 30_000);

test("run_browser accepts a succeeded run only after all receipts validate", () => {
    const item = runBrowserHarness("succeeded");
    try {
        expect(item.result.exitCode, new TextDecoder().decode(item.result.stderr)).toBe(0);
        expect(new TextDecoder().decode(item.result.stdout)).toContain("RUN_BROWSER_STATUS=0");
        for (const file of ["browser-owner-run.json", "browser-owner-steps.json", "browser-owner-artifacts.json"]) expect(existsSync(join(item.evidence, file))).toBeTrue();
    } finally { item.cleanup(); }
});
