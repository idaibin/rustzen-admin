import { constants } from "node:fs";
import { lstat, open, realpath } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { revalidateMonitorNativeRuntime } from "../distribution/monitor-native-runtime-revalidator.ts";
import { parseMonitorNativeRuntimeEvidence } from "../distribution/monitor-native-runtime-evidence.ts";
import { parseSelectedWebBootstrapBrowserReceipt } from "./selected-web-bootstrap-browser-receipt.ts";

export type Stable = { path: string; sha256: string; bytes: Uint8Array };
const encoder = new TextEncoder(),
    decoder = new TextDecoder(),
    hash = /^[a-f0-9]{64}$/;
export async function stable(
    path: string,
    limit = 2 * 1024 * 1024,
): Promise<Stable> {
    const input = resolve(path),
        handle = await open(input, constants.O_RDONLY | constants.O_NOFOLLOW);
    try {
        const named = await lstat(input, { bigint: true }),
            before = await handle.stat({ bigint: true }),
            bytes = new Uint8Array(await handle.readFile()),
            after = await handle.stat({ bigint: true }),
            link = await lstat(input, { bigint: true });
        if (
            !before.isFile() || before.isSymbolicLink() || named.isSymbolicLink() ||
            before.size > BigInt(limit) || before.uid !== BigInt(process.getuid()) ||
            before.dev !== after.dev ||
            before.ino !== after.ino ||
            before.size !== after.size ||
            link.dev !== before.dev ||
            link.ino !== before.ino || link.mode !== before.mode
        )
            throw Error(`input changed while read: ${input}`);
        return {
            path: await realpath(input),
            bytes,
            sha256: new Bun.CryptoHasher("sha256").update(bytes).digest("hex"),
        };
    } finally {
        await handle.close();
    }
}
export async function json(path: string, limit?: number) {
    const text = decoder.decode((await stable(path, limit)).bytes), value = JSON.parse(text) as Record<string, unknown>;
    if (text !== canonicalJson(value) + "\n" && text !== canonicalJson(value)) throw Error("JSON input is not canonical");
    return value;
}
export async function directory(path: string) {
    const input = resolve(path), info = await lstat(input);
    if (!info.isDirectory() || info.isSymbolicLink() || await realpath(input) !== input) throw Error(`directory input is unsafe: ${input}`);
    return input;
}
export async function credential(path: string) {
    const value = await stable(path, 4096), info = await lstat(value.path, { bigint: true });
    if (!info.isFile() || info.isSymbolicLink() || (info.mode & 0o777n) !== 0o600n || info.uid !== BigInt(process.getuid())) throw Error("credential must be current-owner mode 0600 regular file");
    return new TextDecoder().decode(value.bytes).trim();
}
/** Revalidates the complete P8e evidence set; its PID tuple is historical evidence only. */
export async function revalidatedNativeEvidence(evidencePath: string, releaseResultPath: string) {
    const root = dirname(resolve(evidencePath));
    const parsed = parseMonitorNativeRuntimeEvidence(await json(evidencePath));
    const revalidated = await revalidateMonitorNativeRuntime({
        evidencePath: resolve(evidencePath), releaseResultPath: resolve(releaseResultPath),
        admissionPath: `${root}/published-certificate.json`, factsPath: `${root}/facts.json`, loginEvidencePath: `${root}/login-evidence.json`,
        verifyPath: `${root}/verify.json`, dryRunPath: `${root}/dry-run.json`, applyPath: `${root}/apply.json`, statusPath: `${root}/install-status.json`, activatePath: `${root}/activate.json`,
        publicationMarkerPath: `${root}/publication-marker.json`, activationMarkerPath: `${root}/activation-marker.json`,
    });
    if (canonicalJson(parsed) !== canonicalJson(revalidated)) throw Error("P8e revalidated evidence differs");
    return revalidated;
}
export async function nativeEvidenceSummary(evidencePath: string, releaseResultPath: string) {
    const root = dirname(resolve(evidencePath)), paths = [evidencePath, releaseResultPath, "published-certificate.json", "facts.json", "login-evidence.json", "verify.json", "dry-run.json", "apply.json", "install-status.json", "activate.json", "publication-marker.json", "activation-marker.json"].map(path => path.includes("/") ? resolve(path) : `${root}/${path}`);
    return Object.fromEntries(await Promise.all(paths.map(async path => {
        const input = await stable(path);
        return [input.path, input.sha256];
    })));
}
export function admitted(
    native: Record<string, unknown>,
    browser: Record<string, unknown>,
) {
    parseSelectedWebBootstrapBrowserReceipt(browser);
    const nativeKeys = ["browser", "checks", "health", "installation", "kind", "load", "markers", "platform", "release", "releaseReady", "runtime", "schemaVersion", "selection", "services"];
    const browserKeys = ["adminHealth", "cases", "chromiumVersion", "digests", "integritySensitivityPassed", "release", "runtime", "schemaVersion", "sensitivity", "sourceIdentity", "status", "verifier"];
    if (JSON.stringify(Object.keys(native).sort()) !== JSON.stringify(nativeKeys) || JSON.stringify(Object.keys(browser).sort()) !== JSON.stringify(browserKeys)) throw Error("P8e/P8f schema fields differ");
    const selection = native.selection as Record<string, unknown>,
        release = native.release as Record<string, unknown>,
        b = browser.release as Record<string, unknown>,
        digest = browser.digests as Record<string, unknown>,
        source = browser.sourceIdentity as Record<string, unknown>,
        runtime = browser.runtime as Record<string, unknown>;
    if (
        native.kind !== "monitor-native-runtime-evidence" ||
        native.runtime !== true ||
        native.browser !== false ||
        native.load !== false ||
        native.releaseReady !== false ||
        browser.schemaVersion !== 1 || browser.status !== "passed" ||
        typeof browser.chromiumVersion !== "string" || !browser.chromiumVersion ||
        browser.integritySensitivityPassed !== true ||
        !object(browser.adminHealth) || !exactKeys(browser.adminHealth, ["final", "initial"]) ||
        !object(browser.sensitivity) || !exactKeys(browser.sensitivity, ["health", "marker", "requests"]) || browser.sensitivity.marker !== true || !Array.isArray(browser.sensitivity.requests) || !object(browser.verifier) || !exactKeys(browser.verifier, ["sources"]) || !Array.isArray(browser.verifier.sources) ||
        !Array.isArray(browser.cases) || !browser.cases.length || browser.cases.some(value => !object(value)) ||
        !exactKeys(digest, ["adminBinary", "binding", "buildId", "compositionId", "html", "installation", "verified"]) ||
        !object(digest.adminBinary) || !exactKeys(digest.adminBinary as Record<string, unknown>, ["after", "before"]) ||
        !["buildId", "compositionId", "html", "binding", "installation", "verified"].every(key => hash.test(String(digest[key]))) ||
        !hash.test(String((digest.adminBinary as Record<string, unknown>).before)) || !hash.test(String((digest.adminBinary as Record<string, unknown>).after)) ||
        !exactKeys(source, ["current", "expected"]) || !exactKeys(runtime, ["after", "before"]) ||
        !exactKeys(b, ["archiveSha256", "binaryDigests", "buildId", "certificateSha256", "envelopeSha256", "manifestSha256", "selection"]) ||
        !object(b.selection) || !exactKeys(b.selection as Record<string, unknown>, ["artifactClass", "compositionId", "target"]) ||
        !Array.isArray(b.binaryDigests) || !b.binaryDigests.length ||
        !same(selection.buildId, digest.buildId) ||
        !same(selection.compositionId, digest.compositionId) ||
        !same(selection.buildId, b.buildId) || !same(selection.compositionId, (b.selection as Record<string, unknown>).compositionId) ||
        source.expected !== source.current ||
        !same(runtime.before, runtime.after) ||
        ![
            "certificateSha256",
            "manifestSha256",
            "archiveSha256",
            "envelopeSha256",
        ].every(
            (key) =>
                hash.test(String(release[key])) && same(release[key], b[key]),
        )
    )
        throw Error("P8e/P8f tuple differs");
    return { selection, release, runtime: runtime.after, source };
}
function object(value: unknown): value is Record<string, unknown> { return Boolean(value) && typeof value === "object" && !Array.isArray(value); }
function exactKeys(value: Record<string, unknown>, keys: string[]) { return JSON.stringify(Object.keys(value).sort()) === JSON.stringify(keys); }
export async function freshOutput(path: string) {
    const output = resolve(path),
        parent = resolve(dirname(output));
    const root = resolve("target/rz"), ancestors = [root, parent];
    let occupied = false;
    try { await lstat(output); occupied = true; }
    catch (error) { if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error; }
    if ((parent !== root && !parent.startsWith(root + "/")) || occupied)
        throw Error("output must be fresh beneath target/rz");
    for (const ancestor of ancestors) if ((await lstat(ancestor)).isSymbolicLink() || await realpath(ancestor) !== ancestor) throw Error("output ancestor is not canonical");
    return output;
}
export async function publish(
    output: string,
    receipt: Record<string, unknown>,
    validate?: (value: unknown) => unknown,
) {
    validate?.(receipt);
    const temporary = `${output}.tmp-${crypto.randomUUID()}`, bytes = encoder.encode(canonicalJson(receipt));
    await Bun.write(temporary, bytes);
    const result = Bun.spawnSync(["mkdir", output]);
    if (result.exitCode) { await Bun.file(temporary).delete(); throw Error("cannot create fresh receipt directory"); }
    const moved = Bun.spawnSync([
        "mv",
        "-n",
        temporary,
        `${output}/monitor-load-evidence.json`,
    ]);
    if (moved.exitCode) { await Bun.file(temporary).delete(); Bun.spawnSync(["rmdir", output]); throw Error("cannot publish receipt"); }
    let reread = await stable(`${output}/monitor-load-evidence.json`);
    if (reread.sha256 !== new Bun.CryptoHasher("sha256").update(bytes).digest("hex")) { Bun.spawnSync(["rm", "-f", `${output}/monitor-load-evidence.json`]); Bun.spawnSync(["rmdir", output]); throw Error("published receipt reread differs"); }
    validate?.(JSON.parse(decoder.decode(reread.bytes)));
}
export function parseMonitorLoadEvidence(value: unknown) {
    if (!object(value)) throw Error("monitor load receipt is invalid");
    const keys = ["boundary", "boundaryInFlight", "browserRuntime", "corpus", "fault", "faultDurationMs", "initialServices", "inputs", "kind", "lanes", "load", "overallMaxima", "p8eSidecars", "phases", "pureMonitorSse", "quietBaselines", "recovery", "release", "resources", "schemaVersion", "selection", "source"];
    if (!exactKeys(value, keys) || value.schemaVersion !== 1 || value.kind !== "monitor-load-evidence" || value.load !== true || typeof value.faultDurationMs !== "number" || !Number.isFinite(value.faultDurationMs) || value.faultDurationMs < 0 || !object(value.inputs) || !object(value.p8eSidecars) || !object(value.release) || !object(value.browserRuntime) || !object(value.initialServices) || !Array.isArray(value.lanes) || value.lanes.length !== 2 || !Array.isArray(value.phases) || value.phases.length !== 8 || !Array.isArray(value.boundary) || !Array.isArray(value.boundaryInFlight) || !Array.isArray(value.fault) || !Array.isArray(value.quietBaselines) || value.quietBaselines.length !== 2 || !object(value.overallMaxima) || !object(value.resources) || !object(value.pureMonitorSse)) throw Error("monitor load receipt schema differs");
    for (const digest of Object.values(value.inputs)) if (!hash.test(String(digest))) throw Error("monitor load receipt input digest differs");
    for (const digest of Object.values(value.p8eSidecars)) if (!hash.test(String(digest))) throw Error("monitor load receipt P8e digest differs");
    for (const lane of value.lanes) if (!object(lane) || !Array.isArray(lane.snapshots) || !object(lane.maxima) || !Number.isSafeInteger(lane.durationMs) || Number(lane.durationMs) < 60000 || lane.warmupCount !== 128 || !Number.isInteger(lane.offered) || Number(lane.offered) < 3200 || lane.completed !== lane.offered || lane.failures !== 0 || !Number.isSafeInteger(lane.p95Ms) || Number(lane.p95Ms) > 500 || !Number.isSafeInteger(lane.p99Ms) || Number(lane.p99Ms) > 1000) throw Error("monitor load receipt lane differs");
    for (const row of value.boundary) if (!object(row) || !["response", "timeout", "transport"].includes(String(row.kind))) throw Error("monitor load receipt boundary differs");
    const stops = value.fault.filter(row => object(row) && row.step === "stop") as Record<string, unknown>[];
    if (stops.length !== 1 || typeof stops[0]!.at !== "number" || value.boundaryInFlight.length !== 4) throw Error("monitor load receipt stop differs");
    for (const row of value.boundaryInFlight) if (!object(row) || !exactKeys(row, ["at", "code", "end", "kind", "start", "status"]) || row.kind !== "response" || row.status !== 503 || row.code !== 40001 || typeof row.start !== "number" || typeof row.end !== "number" || row.at !== row.end || !(row.start < stops[0]!.at && stops[0]!.at < row.end)) throw Error("monitor load receipt inflight differs");
    return value;
}
function same(left: unknown, right: unknown) {
    return JSON.stringify(left) === JSON.stringify(right);
}
