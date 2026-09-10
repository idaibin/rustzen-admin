export const GATEWAY_P95_BUDGET_MS = 2;

export function gatewayLatencyBudget(buildProfile) {
    if (buildProfile !== "debug" && buildProfile !== "release") {
        throw new Error("RUSTZEN_VERIFY_BUILD_PROFILE must be debug or release");
    }
    return {
        buildProfile,
        p95BudgetMs: GATEWAY_P95_BUDGET_MS,
        budgetEnforced: buildProfile === "release",
    };
}

export function gatewayLatencyResult(buildProfile, p95Ms) {
    if (!Number.isFinite(p95Ms) || p95Ms < 0) {
        throw new Error("gateway p95 must be a non-negative finite number");
    }
    return { ...gatewayLatencyBudget(buildProfile), budgetPassed: p95Ms <= GATEWAY_P95_BUDGET_MS };
}
