import { expect, test } from "bun:test";

import { gatewayLatencyResult } from "./gateway-latency-contract.mjs";

test("release enforces the fixed two millisecond gateway p95 budget", () => {
    expect(gatewayLatencyResult("release", 1.777)).toEqual({
        buildProfile: "release", p95BudgetMs: 2, budgetEnforced: true, budgetPassed: true,
    });
    expect(gatewayLatencyResult("release", 2.001)).toMatchObject({
        budgetEnforced: true, budgetPassed: false,
    });
});

test("debug records the release budget without enforcing it", () => {
    expect(gatewayLatencyResult("debug", 2.920)).toEqual({
        buildProfile: "debug", p95BudgetMs: 2, budgetEnforced: false, budgetPassed: false,
    });
});

test("gateway latency profiles fail closed", () => {
    expect(() => gatewayLatencyResult("profiling", 1)).toThrow(
        "RUSTZEN_VERIFY_BUILD_PROFILE must be debug or release",
    );
    expect(() => gatewayLatencyResult("release", Number.NaN)).toThrow("gateway p95");
});
