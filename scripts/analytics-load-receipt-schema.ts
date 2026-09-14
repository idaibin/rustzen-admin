import { canonicalJson, validHash } from "../distribution/release-manifest-core.ts";
import { READ_MINIMUM, RECOVERY_MINIMUM, TRACK_MINIMUM, VISITORS, SEED_EVENTS_PER_VISITOR } from "./analytics-load-contract.ts";

export type AnalyticsLoadReceipt = {
    schemaVersion: 1;
    kind: "analytics-load-evidence";
    load: true;
    releaseReady: false;
    inputs: Record<string, string>;
    selection: { preset: "analytics"; target: string; compositionId: string; buildId: string };
    release: { keyId: string; certificateSha256: string; manifestSha256: string; archiveSha256: string; envelopeSha256: string };
    corpus: { visitors: number; seedEventsPerVisitor: number; seedOffered: number };
    lanes: Array<{ name: string; round: number; durationMs: number; warmupCount: number; offered: number; completed: number; failures: number; p95Ms: number; p99Ms: number }>;
    quietBaselines: Array<{ rss: number; pidsCurrent: number }>;
    fault: Array<{ step: string; status?: number; pid?: number; sha256?: string }>;
    faultDurationMs: number;
    recovery: { durationMs: number; warmupCount: number; offered: number; completed: number; failures: number; p95Ms: number; p99Ms: number };
    initialServices: { admin: { pid: number; sha256: string }; insights: { pid: number; sha256: string } };
    runtimeRef: { path: string; sha256: string };
    browserRef: { path: string; sha256: string };
};

const LANE_NAMES = ["overview-read", "events-read", "track-write"];

function object(value: unknown, label: string): Record<string, any> { if (!value || typeof value !== "object" || Array.isArray(value)) throw Error(`${label} must be an object`); return value as Record<string, any>; }
function only(value: Record<string, any>, allowed: string[], label: string) { for (const key of Object.keys(value)) if (!allowed.includes(key)) throw Error(`unknown ${label} field: ${key}`); }
function text(value: unknown, label: string): string { if (typeof value !== "string" || !value) throw Error(`${label} is invalid`); return value; }
function count(value: unknown, label: string): number { if (!Number.isSafeInteger(value) || (value as number) < 0) throw Error(`${label} is invalid`); return value as number; }

export function parseAnalyticsLoadReceipt(value: unknown): AnalyticsLoadReceipt {
    const receipt = object(value, "analytics load receipt");
    only(receipt, ["schemaVersion", "kind", "load", "releaseReady", "inputs", "selection", "release", "corpus", "lanes", "quietBaselines", "fault", "faultDurationMs", "recovery", "initialServices", "runtimeRef", "browserRef"], "analytics load receipt");
    if (receipt.schemaVersion !== 1 || receipt.kind !== "analytics-load-evidence") throw Error("analytics load receipt identity is invalid");
    if (receipt.load !== true || receipt.releaseReady !== false) throw Error("analytics load receipt flags differ");
    const selection = object(receipt.selection, "load selection");
    only(selection, ["preset", "target", "compositionId", "buildId"], "load selection");
    if (selection.preset !== "analytics" || selection.target !== "x86_64-unknown-linux-musl") throw Error("load selection differs");
    validHash(text(selection.compositionId, "load compositionId"));
    validHash(text(selection.buildId, "load buildId"));
    const release = object(receipt.release, "load release");
    only(release, ["keyId", "certificateSha256", "manifestSha256", "archiveSha256", "envelopeSha256"], "load release");
    for (const key of ["certificateSha256", "manifestSha256", "archiveSha256", "envelopeSha256"]) validHash(text(release[key], `load ${key}`));
    text(release.keyId, "load keyId");
    const corpus = object(receipt.corpus, "load corpus");
    only(corpus, ["visitors", "seedEventsPerVisitor", "seedOffered"], "load corpus");
    if (count(corpus.visitors, "visitors") !== VISITORS || count(corpus.seedEventsPerVisitor, "seed events") !== SEED_EVENTS_PER_VISITOR || count(corpus.seedOffered, "seed offered") !== 300) throw Error("load corpus volume differs");
    count(corpus.seedOffered, "seed offered");
    const inputs = object(receipt.inputs, "load inputs");
    for (const [name, digest] of Object.entries(inputs)) {
        if (!name.startsWith("/") || typeof digest !== "string" || !/^[a-f0-9]{64}$/.test(digest)) throw Error(`load input differs: ${name}`);
    }
    if (Object.keys(inputs).length < 4) throw Error("load input inventory is incomplete");
    const lanes = Array.isArray(receipt.lanes) ? receipt.lanes.map((entry) => {
        const lane = object(entry, "load lane");
        only(lane, ["name", "round", "durationMs", "warmupCount", "offered", "completed", "failures", "p95Ms", "p99Ms"], "load lane");
        const minimum = lane.name === "track-write" ? TRACK_MINIMUM : READ_MINIMUM;
        if (!LANE_NAMES.includes(lane.name) || ![1, 2].includes(lane.round)) throw Error(`load lane identity differs: ${lane.name}`);
        if (count(lane.offered, "offered") < minimum || count(lane.completed, "completed") < minimum || lane.failures !== 0) throw Error(`load lane undershot: ${lane.name} round ${lane.round}`);
        if (count(lane.durationMs, "duration") < 30_000 || typeof lane.p95Ms !== "number" || lane.p95Ms > 500 || typeof lane.p99Ms !== "number" || lane.p99Ms > 1000) throw Error(`load lane latency differs: ${lane.name}`);
        return { name: text(lane.name, "lane name"), round: lane.round, durationMs: lane.durationMs, warmupCount: count(lane.warmupCount, "warmup"), offered: lane.offered, completed: lane.completed, failures: lane.failures, p95Ms: lane.p95Ms, p99Ms: lane.p99Ms };
    }) : (() => { throw Error("load lanes must be an array"); })();
    if (lanes.length !== 6 || LANE_NAMES.every((name) => lanes.filter((lane) => lane.name === name).length === 2) === false) throw Error("load lane inventory differs");
    const quiet = Array.isArray(receipt.quietBaselines) ? receipt.quietBaselines.map((entry) => { const baseline = object(entry, "quiet baseline"); only(baseline, ["rss", "pidsCurrent"], "quiet baseline"); return { rss: count(baseline.rss, "quiet rss"), pidsCurrent: count(baseline.pidsCurrent, "quiet pids") }; }) : [];
    if (quiet.length !== 2) throw Error("quiet baseline inventory differs");
    const fault = Array.isArray(receipt.fault) ? receipt.fault.map((entry) => { const step = object(entry, "fault step"); only(step, ["step", "status", "pid", "sha256"], "fault step"); return step; }) : [];
    if (JSON.stringify(fault.map((step) => step.step)) !== JSON.stringify(["stop", "apiFailed", "listenerReady", "registryHealthy"])) throw Error("fault boundary steps differ");
    if (typeof fault[1]!.status !== "number" || fault[1]!.status < 500) throw Error("fault outage status differs");
    if (fault[2]!.pid === receipt.initialServices.insights.pid || fault[2]!.sha256 !== receipt.initialServices.insights.sha256) throw Error("fault restart owner differs");
    count(receipt.faultDurationMs, "fault duration");
    const recovery = object(receipt.recovery, "load recovery");
    only(recovery, ["durationMs", "warmupCount", "offered", "completed", "failures", "p95Ms", "p99Ms"], "load recovery");
    if (count(recovery.completed, "recovery completed") < RECOVERY_MINIMUM || recovery.failures !== 0 || typeof recovery.p95Ms !== "number" || recovery.p95Ms > 500 || typeof recovery.p99Ms !== "number" || recovery.p99Ms > 1000) throw Error("load recovery undershot");
    const initialServices = object(receipt.initialServices, "initial services");
    only(initialServices, ["admin", "insights"], "initial services");
    for (const name of ["admin", "insights"]) {
        const service = object(initialServices[name], `initial ${name}`);
        only(service, ["pid", "sha256"], `initial ${name}`);
        count(service.pid, `${name} pid`);
        validHash(text(service.sha256, `${name} sha256`));
    }
    for (const key of ["runtimeRef", "browserRef"]) {
        const reference = object(receipt[key], key);
        only(reference, ["path", "sha256"], key);
        text(reference.path, `${key} path`);
        validHash(text(reference.sha256, `${key} sha256`));
    }
    const serialized = JSON.stringify(value);
    for (const forbidden of ["p8e-owner-password", "x-rustzen-project-key", "Bearer ", "eyJ"])
        if (serialized.includes(forbidden)) throw Error(`analytics load receipt leaks sensitive material: ${forbidden.trim()}`);
    return {
        schemaVersion: 1, kind: "analytics-load-evidence", load: true, releaseReady: false,
        inputs: inputs as Record<string, string>,
        selection: { preset: "analytics", target: selection.target, compositionId: selection.compositionId, buildId: selection.buildId },
        release: release as AnalyticsLoadReceipt["release"],
        corpus: { visitors: corpus.visitors, seedEventsPerVisitor: corpus.seedEventsPerVisitor, seedOffered: corpus.seedOffered },
        lanes, quietBaselines: quiet, fault: fault as AnalyticsLoadReceipt["fault"], faultDurationMs: receipt.faultDurationMs,
        recovery: recovery as AnalyticsLoadReceipt["recovery"],
        initialServices: initialServices as AnalyticsLoadReceipt["initialServices"],
        runtimeRef: receipt.runtimeRef as AnalyticsLoadReceipt["runtimeRef"],
        browserRef: receipt.browserRef as AnalyticsLoadReceipt["browserRef"],
    };
}

if (Bun.argv.length === 3) {
    const bytes = await Bun.file(Bun.argv[2]!).bytes();
    const parsed = parseAnalyticsLoadReceipt(JSON.parse(new TextDecoder().decode(bytes)));
    if (new TextDecoder().decode(bytes) !== canonicalJson(parsed)) throw Error("analytics load receipt file is not canonical");
    console.log(canonicalJson({ verified: true, lanes: parsed.lanes.length, offered: parsed.lanes.reduce((sum, lane) => sum + lane.offered, 0) }));
}
