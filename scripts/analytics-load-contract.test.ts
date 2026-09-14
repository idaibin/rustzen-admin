import { describe, expect, test } from "bun:test";
import {
    VISITORS, READ_MINIMUM, TRACK_BATCH, TRACK_MINIMUM, RECOVERY_MINIMUM,
    rank, trackBody, seedBatches, trackBatch, overviewEnvelope, eventsEnvelope,
    analyticsFault, recoveryLane, lane, type Clock, type Sample, type Fetcher,
} from "./analytics-load-contract.ts";

const clock: Clock = { now: () => Date.now(), sleep: async () => {} };

describe("analytics load contract shaping", () => {
    test("seed batches and track batches match the admission ceiling", () => {
        const batches = seedBatches();
        expect(batches.length).toBe(6);
        expect(batches.flat().length).toBe(300);
        expect(batches.every((batch) => batch.length === 50)).toBe(true);
        expect(new Set(batches.flat().map((body) => body.visitorId)).size).toBe(VISITORS);
        expect(batches.flat().filter((body) => body.eventName === "page_view").length).toBe(150);
        const single = trackBatch(0);
        expect(single.length).toBe(TRACK_BATCH);
        expect(new Set(single.map((body) => body.visitorId)).size).toBe(TRACK_BATCH);
        expect(single.filter((body) => body.eventName === "page_view").length).toBe(TRACK_BATCH / 2);
        expect(trackBody(7, 4).pagePath).toBe("/p8g/load/7/4");
        expect(trackBody(7, 3).apiPath).toBe("/api/p8g/load/7/3");
    });
    test("read envelopes validate only the selected analytics shapes", () => {
        expect(overviewEnvelope({ code: 0, data: { pv: 1, uv: 2, eventCount: 3, requestCount: 4 } })).toBe(true);
        expect(() => overviewEnvelope({ code: 0, data: { pv: "x" } })).toThrow("overview metric differs");
        expect(eventsEnvelope({ code: 0, data: { data: [], total: 0 } })).toBe(true);
        expect(() => eventsEnvelope({ code: 0, data: { data: {} } })).toThrow("events envelope differs");
    });
    test("rank matches nearest-rank percentiles", () => {
        expect(rank([1, 2, 3, 4, 5], 0.95)).toBe(5);
        expect(rank([1, 2, 3, 4, 5], 0.5)).toBe(3);
        expect(() => rank([], 0.5)).toThrow();
    });
});

describe("analytics load fault and recovery acceptance", () => {
    const sha = "f".repeat(64);
    test("fault boundary requires failed read then same-binary restart", () => {
        const steps = [
            { step: "stop" },
            { step: "apiFailed", status: 502 },
            { step: "listenerReady", pid: 4242, sha256: sha },
            { step: "registryHealthy" },
        ];
        expect(() => analyticsFault(steps, 1111, sha)).not.toThrow();
        for (const mutated of [
            [{ step: "stop" }, { step: "apiFailed", status: 200 }, { step: "listenerReady", pid: 4242, sha256: sha }, { step: "registryHealthy" }],
            [{ step: "stop" }, { step: "apiFailed", status: 503 }, { step: "listenerReady", pid: 1111, sha256: sha }, { step: "registryHealthy" }],
            [{ step: "stop" }, { step: "apiFailed", status: 503 }, { step: "listenerReady", pid: 4242, sha256: "0".repeat(64) }, { step: "registryHealthy" }],
            [{ step: "stop" }, { step: "apiFailed", status: 503 }, { step: "listenerReady", pid: 4242, sha256: sha }],
        ] as any[][])
            expect(() => analyticsFault(mutated, 1111, sha)).toThrow();
    });
    test("recovery lane keeps the monitor-grade minimum", () => {
        expect(() => recoveryLane(RECOVERY_MINIMUM)).not.toThrow();
        expect(() => recoveryLane(RECOVERY_MINIMUM - 1)).toThrow();
    });
});

describe("analytics load lane acceptance", () => {
    const fetcher: Fetcher = async () => new Response(JSON.stringify({ code: 0, data: { data: [], total: 0 } }), { status: 200 });
    test("lane passes at threshold and rejects failures, minimums, and latency", async () => {
        let tick = 0;
        const stepping: Clock = { now: () => tick, sleep: async () => {} };
        const worker = async () => { tick += 2; return { ok: true, ms: 2 } as Sample; };
        expect((await lane(fetcher, worker, stepping, 10_000, 3200)).offered).toBeGreaterThanOrEqual(3200);
        const failing = async () => ({ ok: false, ms: 5, failure: "http" as const });
        await expect(lane(fetcher, failing, clock, 0, 10)).rejects.toThrow("warmup failed");
        let slowTick = 0;
        const slowClock: Clock = { now: () => slowTick, sleep: async () => {} };
        const slow = async () => { slowTick += 1500; return { ok: true, ms: 1500 } as Sample; };
        await expect(lane(fetcher, slow, slowClock, 10_000, 0)).rejects.toThrow("latency exceeded");
    });
    void READ_MINIMUM; void TRACK_MINIMUM;
});
