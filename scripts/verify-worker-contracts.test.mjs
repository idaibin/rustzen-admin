import { expect, test } from "bun:test";

const serviceVerifier = await Bun.file(new URL("./verify-services.sh", import.meta.url)).text();
const workerVerifier = await Bun.file(
    new URL("./verify-worker-contracts.mjs", import.meta.url),
).text();
const insightsVerifier = await Bun.file(
    new URL("./verify-insights-scenarios.mjs", import.meta.url),
).text();

test("the disposable service verifier explicitly matches the schedule fixture timezone", () => {
    expect(serviceVerifier).toContain("export RUSTZEN_TIMEZONE=UTC");
    expect(workerVerifier).toContain('scheduleSettings.timezone !== "UTC"');
    expect(workerVerifier).toContain("date.getUTCHours()");
    expect(workerVerifier).toContain("date.getUTCDay()");
});

test("the worker contract rejects unsafe pathname fields from Insights details", () => {
    expect(workerVerifier).toContain('import { verifyInsightsScenarios } from "./verify-insights-scenarios.mjs"');
    expect(workerVerifier).toContain("await verifyInsightsScenarios({ directRequest, expectStatus, responseData, insightsBase });");
    expect(insightsVerifier).toContain('for (const field of ["pagePath", "apiPath", "referrer"])');
    expect(insightsVerifier).toContain("/[?#]/.test(event[field])");
    expect(insightsVerifier).toContain("Insights details exposed an unsafe ${field}");
});

test("worker latency evidence records profile enforcement separately from the fixed budget", () => {
    expect(workerVerifier).toContain('import { gatewayLatencyResult } from "./gateway-latency-contract.mjs"');
    expect(workerVerifier).toContain('const latencyProfile = required("RUSTZEN_VERIFY_BUILD_PROFILE")');
    expect(workerVerifier).toContain('...gatewayLatencyResult(latencyProfile, overhead.p95Ms)');
    expect(workerVerifier).toContain('latency.budgetEnforced && !latency.budgetPassed');
});
