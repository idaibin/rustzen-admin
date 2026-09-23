import { isExactSupportedPlan, type SourceBuildPlan } from "./source-build-plan.ts";

/** The selected route graph is currently materialized for these server closures. */
export const supportsSelectedWeb = (plan: SourceBuildPlan): boolean =>
    isExactSupportedPlan(plan, ["monitor", "monitor-notify", "analytics", "reports"]);
