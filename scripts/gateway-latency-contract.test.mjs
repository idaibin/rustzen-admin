import { expect, test } from "bun:test";

import { gatewayLatencyResult, verifyGatewayLatency } from "./gateway-latency-contract.mjs";

test("release enforces the fixed two millisecond gateway p95 budget", () => {
    expect(gatewayLatencyResult("release", 1.777)).toEqual({ buildProfile: "release", p95BudgetMs: 2, budgetEnforced: true, budgetPassed: true });
    expect(gatewayLatencyResult("release", 2.001)).toMatchObject({ budgetEnforced: true, budgetPassed: false });
});
test("debug records the release budget without enforcing it", () => {
    expect(gatewayLatencyResult("debug", 2.920)).toEqual({ buildProfile: "debug", p95BudgetMs: 2, budgetEnforced: false, budgetPassed: false });
});
test("gateway latency profiles fail closed", () => {
    expect(() => gatewayLatencyResult("profiling", 1)).toThrow("RUSTZEN_VERIFY_BUILD_PROFILE must be debug or release");
    expect(() => gatewayLatencyResult("release", Number.NaN)).toThrow("gateway p95");
});

async function measured(profile) {
    const groups = [], writes = []; let time = 0, bodies = 0;
    const clock = { now: () => time };
    const prepare = (kind) => () => {
        groups.push(kind); time += 100;
        return async () => { time += kind === "direct" ? 1 : 5; return { status: 200, arrayBuffer: async () => { bodies += 1; time += 1; return new ArrayBuffer(0); } }; };
    };
    const invoke = () => verifyGatewayLatency({
        directPrepare: prepare("direct"), gatewayFetch: async () => {
            groups.push("gateway"); time += 100;
            time += 5;
            return { status: 200, arrayBuffer: async () => { bodies += 1; time += 1; return new ArrayBuffer(0); } };
        },
        expectStatus: async (response, status) => { expect(response.status).toBe(status); return response; },
        latencyOutput: "memory.json", latencyProfile: profile, clock,
        write: async (_path, text) => writes.push(JSON.parse(text)), now: () => new Date("2026-01-01T00:00:00.000Z"),
    });
    return { invoke, groups, writes, bodies: () => bodies };
}

test("latency prepares outside timing and preserves 4 warmup plus 10 sampled batch order", async () => {
    const run = await measured("debug");
    const result = await run.invoke();
    expect(run.groups).toHaveLength(896);
    expect(Array.from({ length: 28 }, (_, index) => run.groups.slice(index * 32, index * 32 + 32)[0])).toEqual(
        Array.from({ length: 14 }, () => ["direct", "gateway"]).flat(),
    );
    expect(run.bodies()).toBe(896);
    expect(result.direct).toEqual({ p50Ms: 1548, p95Ms: 3063, p99Ms: 3164 });
    expect(result.gateway).toEqual({ p50Ms: 1712, p95Ms: 3287, p99Ms: 3392 });
    expect(result.overhead.p95Ms).toBe(224);
    expect(run.writes).toHaveLength(1);
    expect(run.writes[0]).toMatchObject({ concurrency: 32, warmupRequestsPerPath: 128, sampleRequestsPerPath: 320, buildProfile: "debug", budgetPassed: false });
});

test("release writes failed evidence before enforcing the budget", async () => {
    const run = await measured("release");
    await expect(run.invoke()).rejects.toThrow("gateway p95 overhead 224.000 ms exceeds 2 ms");
    expect(run.writes).toHaveLength(1);
    expect(run.writes[0]).toMatchObject({ budgetEnforced: true, budgetPassed: false });
});
