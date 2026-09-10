export const GATEWAY_P95_BUDGET_MS = 2;

export function gatewayLatencyBudget(buildProfile) {
    if (buildProfile !== "debug" && buildProfile !== "release") throw new Error("RUSTZEN_VERIFY_BUILD_PROFILE must be debug or release");
    return { buildProfile, p95BudgetMs: GATEWAY_P95_BUDGET_MS, budgetEnforced: buildProfile === "release" };
}

export function gatewayLatencyResult(buildProfile, p95Ms) {
    if (!Number.isFinite(p95Ms) || p95Ms < 0) throw new Error("gateway p95 must be a non-negative finite number");
    return { ...gatewayLatencyBudget(buildProfile), budgetPassed: p95Ms <= GATEWAY_P95_BUDGET_MS };
}

function percentile(values, quantile) {
    const sorted = [...values].sort((left, right) => left - right);
    return sorted[Math.min(sorted.length - 1, Math.ceil(sorted.length * quantile) - 1)];
}

export async function verifyGatewayLatency({
    directPrepare, gatewayFetch, expectStatus, latencyOutput, latencyProfile,
    clock = performance, write = Bun.write, now = () => new Date(),
}) {
    async function timedRequest(kind) {
        const send = kind === "direct" ? directPrepare() : gatewayFetch;
        const start = clock.now();
        const response = await send();
        await expectStatus(response, 200, `${kind} latency request`);
        await response.arrayBuffer();
        return clock.now() - start;
    }
    const runBatch = (kind) => Promise.all(Array.from({ length: 32 }, () => timedRequest(kind)));
    for (let batch = 0; batch < 4; batch += 1) { await runBatch("direct"); await runBatch("gateway"); }
    const directSamples = [], gatewaySamples = [];
    for (let batch = 0; batch < 10; batch += 1) {
        directSamples.push(...(await runBatch("direct")));
        gatewaySamples.push(...(await runBatch("gateway")));
    }
    const direct = { p50Ms: percentile(directSamples, .5), p95Ms: percentile(directSamples, .95), p99Ms: percentile(directSamples, .99) };
    const gateway = { p50Ms: percentile(gatewaySamples, .5), p95Ms: percentile(gatewaySamples, .95), p99Ms: percentile(gatewaySamples, .99) };
    const overhead = { p50Ms: gateway.p50Ms - direct.p50Ms, p95Ms: gateway.p95Ms - direct.p95Ms, p99Ms: gateway.p99Ms - direct.p99Ms };
    const latency = {
        measuredAt: now().toISOString(), endpoint: "GET /api/monitor/nodes",
        ...gatewayLatencyResult(latencyProfile, overhead.p95Ms), host: "127.0.0.1", concurrency: 32,
        warmupRequestsPerPath: 128, sampleRequestsPerPath: directSamples.length, direct, gateway, overhead,
    };
    await write(latencyOutput, `${JSON.stringify(latency, null, 2)}\n`);
    console.log(`Gateway latency: ${JSON.stringify(latency)}`);
    if (latency.budgetEnforced && !latency.budgetPassed) throw new Error(`gateway p95 overhead ${overhead.p95Ms.toFixed(3)} ms exceeds ${latency.p95BudgetMs} ms`);
    return latency;
}
