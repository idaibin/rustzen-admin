import { expect, test } from "bun:test";

const serviceVerifier = await Bun.file(new URL("./verify-services.sh", import.meta.url)).text();
const workerVerifier = await Bun.file(
    new URL("./verify-worker-contracts.mjs", import.meta.url),
).text();
const insightsVerifier = await Bun.file(
    new URL("./verify-insights-scenarios.mjs", import.meta.url),
).text();
const reportsScheduleVerifier = await Bun.file(
    new URL("./verify-reports-schedule-scenarios.mjs", import.meta.url),
).text();

test("the disposable service verifier explicitly matches the schedule fixture timezone", () => {
    expect(serviceVerifier).toContain("export RUSTZEN_TIMEZONE=UTC");
    expect(reportsScheduleVerifier).toContain('scheduleSettings.timezone !== "UTC"');
    expect(reportsScheduleVerifier).toContain("date.getUTCHours()");
    expect(reportsScheduleVerifier).toContain("date.getUTCDay()");
});

test("the worker contract rejects unsafe pathname fields from Insights details", () => {
    expect(workerVerifier).toContain('import { verifyInsightsScenarios } from "./verify-insights-scenarios.mjs"');
    expect(workerVerifier).toContain("await verifyInsightsScenarios({ directRequest, expectStatus, responseData, insightsBase });");
    expect(insightsVerifier).toContain('for (const field of ["pagePath", "apiPath", "referrer"])');
    expect(insightsVerifier).toContain("/[?#]/.test(event[field])");
    expect(insightsVerifier).toContain("Insights details exposed an unsafe ${field}");
});

test("worker latency evidence records profile enforcement separately from the fixed budget", () => {
    expect(workerVerifier).toContain('import { verifyGatewayLatency } from "./gateway-latency-contract.mjs"');
    expect(workerVerifier).toContain('const latencyProfile = required("RUSTZEN_VERIFY_BUILD_PROFILE")');
    expect(workerVerifier).toContain("await verifyGatewayLatency({");
    expect(workerVerifier).toContain('latencyProfile,');
});

test("worker verifier delegates Reports flow and schedule scenarios in order", () => {
    const flow = workerVerifier.indexOf("await verifyReportsFlowScenarios({");
    const schedule = workerVerifier.indexOf("await verifyReportsScheduleScenarios({");
    expect(workerVerifier).toContain('import { verifyReportsFlowScenarios } from "./verify-reports-flow-scenarios.mjs"');
    expect(workerVerifier).toContain('import { verifyReportsScheduleScenarios } from "./verify-reports-schedule-scenarios.mjs"');
    expect(flow).toBeGreaterThan(-1);
    expect(schedule).toBeGreaterThan(flow);
    expect(workerVerifier).toContain('reportsRuntimeRoot: required("RUSTZEN_RUNTIME_ROOT")');
    expect(workerVerifier).toContain('sleep: Bun.sleep');
    expect(workerVerifier).toContain('spawnSync: Bun.spawnSync');
});

test("worker latency prepares direct headers before the timer and gateway headers after it", () => {
    expect(workerVerifier).toContain("directPrepare: () => {");
    expect(workerVerifier).toContain('const headers = delegatedHeaders("monitor", "/api/monitor/nodes", "monitor:node:view");');
    expect(workerVerifier).toContain("gatewayFetch: () => fetch(`${adminBase}/api/monitor/nodes`, {");
    expect(workerVerifier).toContain('return () => fetch(`${monitorBase}/api/monitor/nodes`, { headers });');
});
