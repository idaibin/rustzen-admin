import { expect, test } from "bun:test";
import { parseAnalyticsLoadReceipt } from "./analytics-load-receipt-schema.ts";

const hash = "a".repeat(64);
const lane = (name: string, round: number, offered = 3500) => ({
    name, round, durationMs: 60_000, warmupCount: name === "track-write" ? 0 : 128, offered, completed: offered, failures: 0, p95Ms: 120.5, p99Ms: 300.25,
});
const receipt: any = {
    schemaVersion: 1,
    kind: "analytics-load-evidence",
    load: true,
    releaseReady: false,
    inputs: { "/release.json": hash, "/certificate": hash, "/public.pem": hash, "/password": hash },
    selection: { preset: "analytics", target: "x86_64-unknown-linux-musl", compositionId: hash, buildId: hash },
    release: { keyId: "p8g-analytics", certificateSha256: hash, manifestSha256: hash, archiveSha256: hash, envelopeSha256: hash },
    corpus: { visitors: 100, seedEventsPerVisitor: 3, seedOffered: 300 },
    lanes: [lane("overview-read", 1), lane("events-read", 1), lane("track-write", 1, 15), lane("overview-read", 2), lane("events-read", 2), lane("track-write", 2, 15)],
    quietBaselines: [{ rss: 1, pidsCurrent: 2 }, { rss: 1, pidsCurrent: 2 }],
    fault: [{ step: "stop" }, { step: "apiFailed", status: 502 }, { step: "listenerReady", pid: 999, sha256: hash }, { step: "registryHealthy" }],
    faultDurationMs: 5300,
    recovery: { durationMs: 10_000, warmupCount: 128, offered: 600, completed: 600, failures: 0, p95Ms: 90, p99Ms: 210 },
    initialServices: { admin: { pid: 311, sha256: hash }, insights: { pid: 312, sha256: hash } },
    runtimeRef: { path: "/p8e/analytics-native-runtime-evidence.json", sha256: hash },
    browserRef: { path: "/p8f/receipt.json", sha256: hash },
};

test("analytics load receipt accepts the full closure", () => {
    expect(parseAnalyticsLoadReceipt(receipt)).toEqual(receipt);
});
test("analytics load receipt rejects weak or leaking evidence", () => {
    for (const mutate of [
        (v: any) => { v.load = false; },
        (v: any) => { v.releaseReady = true; },
        (v: any) => { v.lanes[0].offered = 3000; },
        (v: any) => { v.lanes[2].offered = 13; },
        (v: any) => { v.lanes[0].p95Ms = 600; },
        (v: any) => { v.lanes.pop(); },
        (v: any) => { v.lanes.push({ ...lane("overview-read", 3) }); },
        (v: any) => { v.lanes[0].failures = 1; },
        (v: any) => { v.fault[1].status = 200; },
        (v: any) => { v.fault[2].pid = v.initialServices.insights.pid; },
        (v: any) => { v.recovery.completed = 511; },
        (v: any) => { v.recovery.p99Ms = 1200; },
        (v: any) => { v.lanes[0].durationMs = 29_000; },
        (v: any) => { v.lanes[3].name = "unknown-lane"; },
        (v: any) => { v.inputs = { "/a": hash, "/b": hash, "/c": hash }; },
        (v: any) => { v.inputs["/leak"] = "Bearer eyJabc"; },
        (v: any) => { v.corpus.seedOffered = 299; },
        (v: any) => { v.quietBaselines.pop(); },
        (v: any) => { v.inputs = { "relative": hash }; },
        (v: any) => { delete v.browserRef; },
        (v: any) => { v.inputs["/x"] = "p8e-owner-password"; },
        (v: any) => { v.selection.preset = "monitor"; },
        (v: any) => { v.extra = true; },
    ]) {
        const mutated = structuredClone(receipt);
        mutate(mutated);
        expect(() => parseAnalyticsLoadReceipt(mutated)).toThrow();
    }
});
