import { isExactSupportedPlan, type SourceBuildPlan } from "./source-build-plan.ts";

export type SelectedServerInventory = {
    binaries: ["bin/rz-admin", "bin/rz-insights"] | ["bin/rz-admin", "bin/rz-monitor"];
    hasAgentWitness: boolean;
};

/** Exact selected-server inventory; container admission remains separately scoped. */
export function selectedServerInventory(plan: SourceBuildPlan): SelectedServerInventory {
    if (isExactSupportedPlan(plan, ["analytics"]))
        return { binaries: ["bin/rz-admin", "bin/rz-insights"], hasAgentWitness: false };
    if (isExactSupportedPlan(plan, ["monitor", "monitor-notify"]))
        return { binaries: ["bin/rz-admin", "bin/rz-monitor"], hasAgentWitness: true };
    throw new Error("selected server inventory does not support this exact closure");
}
